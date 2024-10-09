use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use std::sync::{Arc, Mutex};
use esp_idf_hal::ledc::*;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::prelude::*;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::adc::oneshot::AdcChannelDriver;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;

use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::i2c::I2cDriver;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{
    prelude::*,
    I2CDisplayInterface,
    Ssd1306
};

mod key_matrix;
use key_matrix::KeyMatrix;
use key_matrix::KeyMatrixPins;

mod led_driver;
use led_driver::LedDriver;
use led_driver::LedPins;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;

    let led_driver = Arc::new(Mutex::new(LedDriver::new()));
    {
        let led_pins = LedPins {
            seg_a: peripherals.pins.gpio21.downgrade_output(),
            seg_b: peripherals.pins.gpio22.downgrade_output(),
            seg_c: peripherals.pins.gpio23.downgrade_output(),
            seg_d: peripherals.pins.gpio15.downgrade_output(),
            seg_e: peripherals.pins.gpio17.downgrade_output(),
            seg_f: peripherals.pins.gpio16.downgrade_output(),
            seg_g: peripherals.pins.gpio3.downgrade_output(),
            seg_dot: peripherals.pins.gpio2.downgrade_output(),
            seg_digit1: peripherals.pins.gpio11.downgrade_output(),
            seg_digit2: peripherals.pins.gpio18.downgrade_output(),
            seg_digit3: peripherals.pins.gpio19.downgrade_output(),
            seg_digit4: peripherals.pins.gpio20.downgrade_output(),
        };
        let led_driver_clone = Arc::clone(&led_driver);
        led_driver_clone.lock().unwrap().start_dynamic_lighting(led_pins)?;
    }

    let key_matrix = Arc::new(Mutex::new(KeyMatrix::new()));
    {
        let key_matrix_pins = KeyMatrixPins {
            key_in1: peripherals.pins.gpio9.downgrade_input(),
            key_in2: peripherals.pins.gpio10.downgrade_input(),
            key_in3: peripherals.pins.gpio8.downgrade_input(),
            key_out1: peripherals.pins.gpio1.downgrade_output(),
            key_out2: peripherals.pins.gpio0.downgrade_output(),
        };
        let key_matrix_clone = Arc::clone(&key_matrix);
        key_matrix_clone.lock().unwrap().start_scan(key_matrix_pins)?;
    }
    
    let i2c0 = peripherals.i2c0;
    let sda = peripherals.pins.gpio6;
    let scl = peripherals.pins.gpio7;

    let i2c_config = I2cConfig::new().baudrate(400.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let i2c = I2cDriver::new(i2c0, sda, scl, &i2c_config)?;

    let i2c_interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let _ = std::thread::spawn(move || {
        loop {
            // ESP 環境では std::time::Instant::now() で起動してからの時刻を取得できなかった
            let uptime_us = unsafe { esp_idf_sys::esp_timer_get_time() };
            let text = format!("{} [ms]", uptime_us / 1000);

            println!("{}", text);

            Text::with_baseline(&text, Point::new(0, 0), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        
            display.flush().unwrap();
            
            FreeRtos::delay_ms(30);
            display.clear(BinaryColor::Off).unwrap();
        }
    });

    let buzzer_pin = peripherals.pins.gpio4;
    let channel0 = peripherals.ledc.channel0;
    let timer0 = peripherals.ledc.timer0;
    
    let timer_config = &TimerConfig::new().resolution(Resolution::Bits10).frequency(1.kHz().into());
    let mut timer = LedcTimerDriver::new(timer0, timer_config)?;

    let mut channel = LedcDriver::new(
        channel0,
        &timer,
        buzzer_pin,
    )?;

    let max_duty = channel.get_max_duty();

    // ADC 関連
    let adc = AdcDriver::new(peripherals.adc1)?;
    let adc_config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio5, &adc_config)?;

    // メインスレッド
    loop {
        // ボタン情報取得
        let status = key_matrix.lock().unwrap().get_status();
        log::info!("[btn] {:02x}", status);

        let adc_value = adc.read(&mut adc_pin)?;
        log::info!("[adc] {}", adc_value);
        
        let octave =
            if      adc_value < 1000 { 0.5 }
            else if adc_value < 2000 { 1.0 }
            else if adc_value < 3000 { 2.0 }
            else                     { 4.0 };
        
        let frequency_base = match status {
            // 同時押しなので優先的に判定
            0x30 => Some(988),  // B
            // 以降は単押し判定
            0x01 => Some(523),  // C
            0x02 => Some(587),  // D
            0x04 => Some(659),  // E
            0x08 => Some(698),  // F
            0x10 => Some(783),  // G
            0x20 => Some(880),  // A
            _ => None,
        };

        const NUMBER_SEGMENT_TABLE: [u8; 10] = [
            0xFC,   // 0
            0x60,   // 1
            0xDA,   // 2
            0xF2,   // 3
            0x66,   // 4
            0xB6,   // 5
            0xBE,   // 6
            0xE4,   // 7
            0xFE,   // 8
            0xF6,   // 9
        ];
        let mut display_data = [
            NUMBER_SEGMENT_TABLE[(adc_value / 1000 % 10) as usize],
            NUMBER_SEGMENT_TABLE[(adc_value / 100  % 10) as usize],
            NUMBER_SEGMENT_TABLE[(adc_value / 10   % 10) as usize],
            NUMBER_SEGMENT_TABLE[(adc_value / 1    % 10) as usize],
        ];
        if      adc_value < 10   { display_data[0..3].fill(0); }
        else if adc_value < 100  { display_data[0..2].fill(0); }
        else if adc_value < 1000 { display_data[0..1].fill(0); }

        led_driver.lock().unwrap().write(display_data);

        match frequency_base {
            Some(value) => {
                let frequency = (value as f32 * octave) as u32;
                timer.set_frequency(frequency.Hz())?;
                channel.set_duty(max_duty / 2)?;    // これが音出力のトリガーとなる
                timer.resume()?;
            },
            None => {
                timer.pause()?;
            }
        }
        
        FreeRtos::delay_ms(20);
    }
}
