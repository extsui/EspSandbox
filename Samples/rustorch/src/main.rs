use app_context::AppContext;
use app_context::AppFramework;
use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripherals::Peripherals;
use std::sync::{Arc, Mutex};

mod key_matrix;
use key_matrix::KeyMatrix;
use key_matrix::KeyMatrixPins;
use key_matrix::Button;

mod led_driver;
use led_driver::LedDriver;
use led_driver::LedPins;

mod buzzer_driver;
use buzzer_driver::BuzzerDriver;

mod volume;
use volume::Volume;

mod display_driver;
use display_driver::DisplayDriver;
use embedded_graphics::prelude::*;

mod app_context;

mod app_toy_piano;
use app_toy_piano::ToyPiano;

use esp_idf_hal::delay::FreeRtos;

fn print_freertos_tasks() {
    let mut buf = [0u8; 1024];
    unsafe { esp_idf_sys::vTaskList(buf.as_mut_ptr() as *mut i8) };
    log::info!("tasks:\n \
               name          state  priority stack hwm id\n{}",
         String::from_utf8_lossy(&buf).replace('\r', "")
    );
}

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
        led_driver_clone.lock().unwrap().start_dynamic_lighting(led_pins, peripherals.timer10)?;
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

    let display_driver = Arc::new(Mutex::new(DisplayDriver::new()));
    {
        let i2c0 = peripherals.i2c0;
        let sda = peripherals.pins.gpio6;
        let scl = peripherals.pins.gpio7;
        let display_driver_clone = Arc::clone(&display_driver);
        display_driver_clone.lock().unwrap().start_thread(i2c0, sda, scl)?;
        {
            let mut locked = display_driver.lock().unwrap();
            locked.clear()?;
            locked.draw_text("Rustorch startup...".to_string(), Point::new(0, 0))?;
            locked.update()?;
        }
    }

    let buzzer_driver = Arc::new(Mutex::new(BuzzerDriver::new()));
    {
        let buzzer_pin = peripherals.pins.gpio4;
        let channel0 = peripherals.ledc.channel0;
        let timer0 = peripherals.ledc.timer0;
        let buzzer_driver_clone = Arc::clone(&buzzer_driver);
        buzzer_driver_clone.lock().unwrap().start_thread(buzzer_pin, channel0, timer0)?;
    }

    let volume = Arc::new(Mutex::new(Volume::new(peripherals.adc1, peripherals.pins.gpio5)));

    let context = AppContext {
        button: key_matrix,
        buzzer: buzzer_driver,
        display: display_driver,
        led: led_driver,
        volume: volume,
    };

    print_freertos_tasks();

    // フレームの概念を導入する
    const MICRO_SECONDS_PER_FRAME : i64 = 16667;
    let mut next_frame_time_us = unsafe { esp_idf_sys::esp_timer_get_time() } + MICRO_SECONDS_PER_FRAME;

    //let mut frame_count = 0u64;

    // TODO: 他の Application も追加する
    let mut toy_piano = ToyPiano::new();
    toy_piano.initialize();
    
    loop {
        toy_piano.update(&context)?;

        if toy_piano.is_finished() {
            log::info!("Finished!");
            break Ok(());
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
        //frame_count += 1;
    }
}
