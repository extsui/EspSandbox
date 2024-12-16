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
mod app_pomodoro_timer;
use app_pomodoro_timer::PomodoroTimer;
mod app_slot_game;
use app_slot_game::SlotGame;

use esp_idf_hal::delay::FreeRtos;

fn print_freertos_tasks() {
    let mut buf = [0u8; 1024];
    unsafe { esp_idf_sys::vTaskList(buf.as_mut_ptr() as *mut i8) };
    log::info!("tasks:\n \
               name          state  priority stack hwm id\n{}",
         String::from_utf8_lossy(&buf).replace('\r', "")
    );
}

fn draw_menu(display: &Arc<Mutex<DisplayDriver>>, app_names: &Vec<&str>, selected_index: usize) {
    let mut locked = display.lock().unwrap();
    locked.clear().unwrap();
    locked.draw_text("== Menu ==".to_string(), Point::new(0, 0)).unwrap();
    for (i, name) in app_names.iter().enumerate() {
        if i == selected_index {
            locked.draw_text(format!("> {}", name), Point::new(0, (i + 1) as i32 * 10)).unwrap();
        } else {
            locked.draw_text(format!("  {}", name), Point::new(0, (i + 1) as i32 * 10)).unwrap();
        }
    }
    locked.update().unwrap();
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

    // 各アプリケーション
    let mut apps: Vec<Box<dyn AppFramework>> = vec![
        Box::new(PomodoroTimer::new()),
        Box::new(ToyPiano::new()),
        Box::new(SlotGame::new()),
    ];
    // 選択中のアプリケーション
    let mut selected_index = 0 as usize;
    
    enum MenuState {
        Selection,      // メニュー選択
        AppRunning,     // アプリ実行中
        ReturnToMenu,   // メニューへの遷移中 (キー入力無効状態)
    }

    let mut menu_state = MenuState::Selection;

    /*
    // TODO: なんかうまくいかん...
    let app_names: Vec<&String> = apps.iter().map(|app| app.get_name().clone()).collect();
    let app_names_refs: Vec<&str> = app_names.iter().map(|name| name.as_str()).collect();
    */
    // TODO: 本当は apps から get_name() で取得したベクタにしたい
    let app_names = vec![
        "Pomodoro timer",
        "Toy piano",
        "Slot game",
    ];
    // メニュー画面を表示
    draw_menu(&context.display, &app_names, selected_index);

    // フレームの概念を導入する
    const MICRO_SECONDS_PER_FRAME: i64 = 16667;
    const MICRO_SECONDS_PER_SECONDS_FRAME_TIME_ADJUSTMENT: i64 = 1000 * 1000 - MICRO_SECONDS_PER_FRAME * 59;   // 誤差調整用

    let mut next_frame_time_us = unsafe { esp_idf_sys::esp_timer_get_time() } + MICRO_SECONDS_PER_FRAME;
    let mut frame_count = 0u64;

    let mut return_to_menu_time = 0u64;

    loop {
        match menu_state {
            MenuState::Selection => {
                let adc_value = context.volume.lock().unwrap().read_raw();
                let percent = Volume::to_percent(adc_value) as u8;

                let format = format!("{:3}.{:1}", (frame_count / 60) % 1000, frame_count / 6 % 10);
                context.led.lock().unwrap().write_format(&format);
                context.led.lock().unwrap().set_brightness([ percent, percent, percent, percent ]);

                let button = context.button.lock().unwrap().was_released(Button::MASK);
                let is_up_event = button & Button::UP != 0;
                let is_down_event = button & Button::DOWN != 0;
                let is_run_event = button & Button::A != 0;
                if is_up_event || is_down_event {
                    let direction= if is_down_event { 1 } else { apps.len() - 1 };
                    selected_index = (selected_index + direction) % apps.len();
                    draw_menu(&context.display, &app_names, selected_index);
                    log::info!("[menu] Selection index: -> {}", selected_index);
                }
                if is_run_event {
                    // 共通処理
                    {
                        let mut locked = context.display.lock().unwrap();
                        locked.clear().unwrap();
                        locked.update().unwrap();
                    }
    
                    let app = &mut apps[selected_index];
                    app.initialize(&context)?;
                    menu_state = MenuState::AppRunning;
                    log::info!("[menu] -> AppRunning");
                }
            },
            MenuState::AppRunning => {
                let app = &mut apps[selected_index];
                app.update(&context, frame_count)?;

                if app.is_finished() || context.button.lock().unwrap().is_pressed_all() {
                    app.finalize(&context)?;
                    draw_menu(&context.display, &app_names, selected_index);
                    return_to_menu_time = frame_count + (60 / 2);   // 0.5秒待ち
                    menu_state = MenuState::ReturnToMenu;
                    log::info!("[menu] -> ReturnToMenu (current: {}, end: {})", frame_count, return_to_menu_time);
                }
            },
            MenuState::ReturnToMenu => {
                // 入力の読み捨て
                let _ = context.button.lock().unwrap().was_released(0);
                if frame_count >= return_to_menu_time {
                    menu_state = MenuState::Selection;
                    log::info!("[menu] -> Selection (current: {})", frame_count);
                }
            },
        }

        // 次のフレームまで待つ
        loop {
            let current_time_us = unsafe { esp_idf_sys::esp_timer_get_time() };
            if current_time_us >= next_frame_time_us {
                if frame_count % 60 == 0 {
                    // 1 秒に 1 回蓄積の誤差をリセットするフレームを入れる
                    next_frame_time_us += MICRO_SECONDS_PER_SECONDS_FRAME_TIME_ADJUSTMENT;
                } else {
                    next_frame_time_us += MICRO_SECONDS_PER_FRAME;
                }
                break;
            }
            // WDT クリアのために必要
            FreeRtos::delay_ms(1);
        }
        frame_count += 1;
    }
}
