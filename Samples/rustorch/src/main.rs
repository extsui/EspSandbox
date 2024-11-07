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
use key_matrix::Button;

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
/*
    // OLED 関連

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
*/
/*
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
*/
/*
    // TODO: モード選択を実装して ToyPiano モードで↓が実行されるようにする

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
        
        // ボタンが 6 個しかないのでシを AB 同時押しで表現する
        let frequency_base = match status {
            // 同時押しなので優先的に判定
            _ if status == Button::B | Button::A => Some(988),  // B
            // 以降は単押し判定
            _ if status == Button::UP    => Some(523),  // C
            _ if status == Button::LEFT  => Some(587),  // D
            _ if status == Button::DOWN  => Some(659),  // E
            _ if status == Button::RIGHT => Some(698),  // F
            _ if status == Button::B     => Some(783),  // G
            _ if status == Button::A     => Some(880),  // A
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
*/

    // フレームの概念を導入する
    const MICRO_SECONDS_PER_FRAME : i64 = 16667;
    let mut next_frame_time_us = unsafe { esp_idf_sys::esp_timer_get_time() } + MICRO_SECONDS_PER_FRAME;

    let mut frame_count = 0u64;

    enum State {
        // [---] 起動状態
        Startup,
        // [***] 回転中
        Rolling,
        // [nnn] 全桁確定 (再開待ち)
        Fixed,
    }

    // TODO: 最終的にモジュール化が必要 (pub を要削除)
    struct SlotMachine {
        pub state: State,
        // 何桁目まで確定したか
        pub fixed_digit_count: u8,
        // 各桁の数字の内部カウンタ、7セグの回転表示に使用
        pub internal_number: [u32; 3],
        // 確定された数字の格納先
        pub fixed_number: [u8; 3],
    }

    impl SlotMachine {
        pub fn new() -> Self {
            SlotMachine {
                state: State::Startup,
                fixed_digit_count: 0,
                internal_number: Default::default(),
                fixed_number: Default::default(),
            }
        }

        /*
        // TODO: 最終的にモジュール化が必要

        // フレーム毎に呼び出される処理
        pub fn update(&self, ) {

        }
        */
    }

    let mut slot_machine = SlotMachine::new();

    let mut animation_delay_param: u32 = 5; // TORIAEZU: 初期値は適当

    loop {
        //let adc_value = adc.read(&mut adc_pin)?;

        let released_button = key_matrix.lock().unwrap().was_released(Button::MASK);
        // スロットマシン制御ボタン
        let was_rolling_started = released_button & Button::A != 0x00;
        let was_number_selected = released_button & Button::B != 0x00;
        // パラメータ調整ボタン
        let was_button_up_pressed   = released_button & Button::UP   != 0x00;
        let was_button_down_pressed = released_button & Button::DOWN != 0x00;

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

        const NUMBER_SEGMENT_SLOT_TABLE: [[u8; 6]; 10] = [
            [ 0x10, 0x18, 0x3C, 0x7C, 0xFD, 0x00 ],
            [ 0x00, 0x20, 0x20, 0x60, 0x61, 0x00 ],
            [ 0x00, 0x18, 0x1A, 0x5A, 0xDB, 0x00 ],
            [ 0x10, 0x30, 0x32, 0x72, 0xF3, 0x00 ],
            [ 0x00, 0x20, 0x22, 0x26, 0x67, 0x00 ],
            [ 0x10, 0x30, 0x32, 0x36, 0xB7, 0x00 ],
            [ 0x10, 0x38, 0x3A, 0x3E, 0xBF, 0x00 ],
            [ 0x00, 0x20, 0x60, 0xE0, 0xE5, 0x00 ],
            [ 0x10, 0x38, 0x3A, 0x7E, 0xFF, 0x00 ],
            [ 0x10, 0x30, 0x60, 0xE6, 0xF7, 0x00 ],
        ];

        match slot_machine.state {
            State::Startup => {
                if was_rolling_started {
                    slot_machine.state = State::Rolling;
                    slot_machine.fixed_digit_count = 0;
                    slot_machine.fixed_number = Default::default();
                    slot_machine.internal_number = Default::default();
                    println!("-> Rolling");
                }
            },
            State::Rolling => {
                // アニメーションの速度を動的に変更 (主にデバッグ用)
                if was_button_up_pressed {
                    if animation_delay_param > 1 {
                        animation_delay_param -= 1;
                    }
                }
                if was_button_down_pressed {
                    animation_delay_param += 1;
                }
        
                // 桁確定判定
                if was_number_selected {
                    let index = slot_machine.fixed_digit_count as usize;
                    // TODO: 内部数値から確定数値への変換処理を仕上げる (演出関連)
                    slot_machine.fixed_number[index] = (slot_machine.internal_number[index] / animation_delay_param / NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as u8;
                    slot_machine.fixed_digit_count += 1;

                    println!("fixed_digit_count : {} -> {}", index, index + 1);
                }

                // 定常処理

                // 内部数値のカウントアップ処理
                // TODO: 要パラメータ調整
                for value in slot_machine.internal_number.iter_mut() {
                    *value += 1;
                    if *value >= animation_delay_param * NUMBER_SEGMENT_TABLE.len() as u32 * NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32 {
                        *value = 0;
                    }
                }

                // 確定済み桁はその数字を表示、未確定の桁は遷移中のパターンを表示
                let mut display_data = [0u8; 4];
                for i in 0..display_data.len()-1 {
                    display_data[i] = if i < (slot_machine.fixed_digit_count as usize) {
                        let number_index = slot_machine.fixed_number[i] as usize;
                        NUMBER_SEGMENT_TABLE[number_index]
                    } else {
                        let number_index    = (slot_machine.internal_number[i] / animation_delay_param / NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as usize;
                        let animation_index = (slot_machine.internal_number[i] / animation_delay_param % NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as usize;
                        NUMBER_SEGMENT_SLOT_TABLE[number_index][animation_index]
                    };
                }

                led_driver.lock().unwrap().write(display_data);

                if slot_machine.fixed_digit_count == 3 {
                    slot_machine.state = State::Fixed;
                    println!("-> Fixed");
                }
            },
            State::Fixed => {
                // TODO: 結果に対して何かしらのアニメーションさせる?
                // TORIAEZU: 現状は NOP で遷移
                slot_machine.state = State::Startup;
                println!("-> Startup");
            },
        }

        // 次のフレームまで待つ
        loop {
            let current_time_us = unsafe { esp_idf_sys::esp_timer_get_time() };
            if current_time_us >= next_frame_time_us {
                next_frame_time_us += MICRO_SECONDS_PER_FRAME;
                break;
            }
            // WDT クリアのために必要
            FreeRtos::delay_ms(1);
        }
        frame_count += 1;
    }
}
