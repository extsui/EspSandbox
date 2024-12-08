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

    let mut volume = Volume::new(peripherals.adc1, peripherals.pins.gpio5);

    print_freertos_tasks();

    // フレームの概念を導入する
    const MICRO_SECONDS_PER_FRAME : i64 = 16667;
    let mut next_frame_time_us = unsafe { esp_idf_sys::esp_timer_get_time() } + MICRO_SECONDS_PER_FRAME;

    let mut frame_count = 0u64;

    //============================================================
    //  ポモドーロタイマ
    //============================================================

    enum State {
        // 準備中
        Preparing,
        // 作業中 (典型的には 25 分)
        Working,
        // 作業中断中
        WorkingPaused,
        // 休憩中 (典型的には 5 分)
        Resting,
        // 休憩中断中
        RestingPaused,
    }

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

    struct PomodoroTimer {
        pub state: State,
        pub remaining_time: u32,
    }

    impl PomodoroTimer {
        pub fn new() -> Self {
            PomodoroTimer {
                state: State::Preparing,
                remaining_time: 25 * 60,
            }
        }
    }

    fn convert_to_display_data(time: u32, with_dot: bool) -> [u8; 4] {
        let minutes: u32 = time / 60;
        let seconds: u32 = time % 60;
        [
            // 1桁目は10分未満になったら消灯
            if minutes / 10 == 0 { 0x00 } else { NUMBER_SEGMENT_TABLE[(minutes / 10) as usize] },
            // 2桁目のドットは動作中表現用
            NUMBER_SEGMENT_TABLE[(minutes % 10) as usize] | if with_dot { 0x01 } else { 0x00 },
            // 秒以降はそのまま
            NUMBER_SEGMENT_TABLE[(seconds / 10) as usize],
            NUMBER_SEGMENT_TABLE[(seconds % 10) as usize],
        ]
    }

    let mut context = PomodoroTimer::new();
    led_driver.lock().unwrap().write(convert_to_display_data(context.remaining_time, false));

    let mut previous_key_status = 0u8;

    loop {
        let released_button = key_matrix.lock().unwrap().was_released(Button::MASK);
        let was_start_stop_button_pressed = released_button & Button::A != 0x00;
        let was_reset_button_pressed      = released_button & Button::B != 0x00;
        let was_down_button_pressed       = released_button & Button::DOWN != 0x00;
        
        // 7セグ輝度調整用
        // 理論上は 0V ~ 3.3V (=3300) だが実際は 3.26V あたりでサチるので
        // 0% ~ 100% の範囲に入れるために最大値より少し小さい値で % を計算
        let raw_value = volume.read_raw();
        let percent = Volume::to_percent(raw_value);

        // TODO: モード選択を実装して ToyPiano モードで↓が実行されるようにする

        // ボタン情報取得
        let key_status = key_matrix.lock().unwrap().get_status();
        let octave =
            if      raw_value < 1000 { 0.5 }
            else if raw_value < 2000 { 1.0 }
            else if raw_value < 3000 { 2.0 }
            else                     { 4.0 };
        
        if previous_key_status != key_status {
            if key_status == 0 {
                // 全てのボタンを離した
                buzzer_driver.lock().unwrap().stop_tone()?;
            } else {
                // ボタンが 6 個しかないのでシを AB 同時押しで表現する
                let frequency_base = match key_status {
                    // 同時押しなので優先的に判定
                    _ if key_status == Button::B | Button::A => Some(988),  // B
                    // 以降は単押し判定
                    _ if key_status == Button::UP    => Some(523),  // C
                    _ if key_status == Button::LEFT  => Some(587),  // D
                    _ if key_status == Button::DOWN  => Some(659),  // E
                    _ if key_status == Button::RIGHT => Some(698),  // F
                    _ if key_status == Button::B     => Some(783),  // G
                    _ if key_status == Button::A     => Some(880),  // A
                    _ => None,
                };
                if let Some(value) = frequency_base {
                    let frequency = (value as f32 * octave) as u32;
                    buzzer_driver.lock().unwrap().start_tone(frequency)?;
                }
            }
        }

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
            NUMBER_SEGMENT_TABLE[(raw_value / 1000 % 10) as usize],
            NUMBER_SEGMENT_TABLE[(raw_value / 100  % 10) as usize],
            NUMBER_SEGMENT_TABLE[(raw_value / 10   % 10) as usize],
            NUMBER_SEGMENT_TABLE[(raw_value / 1    % 10) as usize],
        ];
        if      raw_value < 10   { display_data[0..3].fill(0); }
        else if raw_value < 100  { display_data[0..2].fill(0); }
        else if raw_value < 1000 { display_data[0..1].fill(0); }

        led_driver.lock().unwrap().write(display_data);

        previous_key_status = key_status;
        
/*
        let brightness = percent;
        led_driver.lock().unwrap().set_brightness([ brightness, brightness, brightness, brightness ]);

        let sub_frame = frame_count % 60;
        let with_dot = sub_frame < 30;

        let do_count_down = |_remaining_time: &mut u32, _sub_frame: &u64| {
            if *_sub_frame == 0 {
                *_remaining_time -= 1;
            }
        };

        // DEBUG: 時間短縮用
        if was_down_button_pressed {
            if context.remaining_time > 60 {
                context.remaining_time -= 60;
            } else if context.remaining_time > 10 {
                context.remaining_time -= 10;
            }
            let display_data = convert_to_display_data(context.remaining_time, false);
            led_driver.lock().unwrap().write(display_data);
        }

        // 強制リセット
        if was_reset_button_pressed {
            context.state = State::Preparing;
            context.remaining_time = 25 * 60;
            let display_data = convert_to_display_data(context.remaining_time, false);
            led_driver.lock().unwrap().write(display_data);
            continue;
        }

        match context.state {
            State::Preparing => {
                if was_start_stop_button_pressed {
                    context.state = State::Working;
                    {
                        let mut locked = display_driver.lock().unwrap();
                        locked.clear()?;
                        locked.draw_image(include_bytes!("../asserts/images/pomodoro_working.bmp"), Point::new(0, 0))?;
                        locked.update()?;
                    }
                }
            },
            State::Working => {
                do_count_down(&mut context.remaining_time, &sub_frame);
                if was_start_stop_button_pressed {
                    context.state = State::WorkingPaused;
                }
                if context.remaining_time == 0 {
                    context.remaining_time = 5 * 60;
                    context.state = State::Resting;
                    {
                        let mut locked = display_driver.lock().unwrap();
                        locked.clear()?;
                        locked.draw_image(include_bytes!("../asserts/images/pomodoro_resting.bmp"), Point::new(0, 0))?;
                        locked.update()?;
                    }
                }
                let display_data = convert_to_display_data(context.remaining_time, with_dot);
                led_driver.lock().unwrap().write(display_data);
            },
            State::WorkingPaused => {
                if was_start_stop_button_pressed {
                    context.state = State::Working;
                }
            },
            State::Resting => {
                do_count_down(&mut context.remaining_time, &sub_frame);
                if was_start_stop_button_pressed {
                    context.state = State::RestingPaused;
                }
                if context.remaining_time == 0 {
                    context.remaining_time = 25 * 60;
                    context.state = State::Preparing;
                }
                let display_data = convert_to_display_data(context.remaining_time, with_dot);
                led_driver.lock().unwrap().write(display_data);
            },
            State::RestingPaused => {
                if was_start_stop_button_pressed {
                    context.state = State::Resting;
                }
            },
        }
*/
        
/*
    //============================================================
    //  スロットマシン
    //============================================================
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

        // TODO: 最終的にモジュール化が必要

        // フレーム毎に呼び出される処理
        pub fn update(&self, ) {

        }
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
*/
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
