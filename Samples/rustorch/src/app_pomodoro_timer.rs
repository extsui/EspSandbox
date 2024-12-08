use crate::app_context::AppContext;
use crate::app_context::AppFramework;

use crate::Button;
use crate::Volume;

use embedded_graphics::prelude::*;

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

pub struct PomodoroTimer {
    state: State,
    remaining_time: u32,
    finished: bool,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        PomodoroTimer {
            state: State::Preparing,
            remaining_time: 25 * 60,
            finished: false,
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

impl AppFramework for PomodoroTimer {
    fn get_name(&self) -> &str {
        "Pomodoro timer"
    }

    fn initialize(&mut self) {

    }

    fn update(&mut self, context: &AppContext, frame_count: u64) -> anyhow::Result<()> {
        let key_status = context.button.lock().unwrap().get_status();
        if key_status == Button::MASK {
            self.finished = true;
            return Ok(());
        }

        let released_button = context.button.lock().unwrap().was_released(Button::MASK);
        let was_start_stop_button_pressed = released_button & Button::A != 0x00;
        let was_reset_button_pressed      = released_button & Button::B != 0x00;
        let was_down_button_pressed       = released_button & Button::DOWN != 0x00;

        let raw_value = context.volume.lock().unwrap().read_raw();
        let percent = Volume::to_percent(raw_value) as u8;

        let brightness = percent;
        context.led.lock().unwrap().set_brightness([ brightness, brightness, brightness, brightness ]);

        let sub_frame = frame_count % 60;
        let with_dot = sub_frame < 30;

        let do_count_down = |_remaining_time: &mut u32, _sub_frame: &u64| {
            if *_sub_frame == 0 {
                *_remaining_time -= 1;
            }
        };

        // DEBUG: 時間短縮用
        if was_down_button_pressed {
            if self.remaining_time > 60 {
                self.remaining_time -= 60;
            } else if self.remaining_time > 10 {
                self.remaining_time -= 10;
            }
            let display_data = convert_to_display_data(self.remaining_time, false);
            context.led.lock().unwrap().write(display_data);
        }

        // 強制リセット
        if was_reset_button_pressed {
            self.state = State::Preparing;
            self.remaining_time = 25 * 60;
            let display_data = convert_to_display_data(self.remaining_time, false);
            context.led.lock().unwrap().write(display_data);
            return Ok(());
        }

        match self.state {
            State::Preparing => {
                if was_start_stop_button_pressed {
                    self.state = State::Working;
                    {
                        let mut locked = context.display.lock().unwrap();
                        locked.clear()?;
                        locked.draw_image(include_bytes!("../asserts/images/pomodoro_working.bmp"), Point::new(0, 0))?;
                        locked.update()?;
                    }
                }
            },
            State::Working => {
                do_count_down(&mut self.remaining_time, &sub_frame);
                if was_start_stop_button_pressed {
                    self.state = State::WorkingPaused;
                }
                if self.remaining_time == 0 {
                    self.remaining_time = 5 * 60;
                    self.state = State::Resting;
                    {
                        let mut locked = context.display.lock().unwrap();
                        locked.clear()?;
                        locked.draw_image(include_bytes!("../asserts/images/pomodoro_resting.bmp"), Point::new(0, 0))?;
                        locked.update()?;
                    }
                }
                let display_data = convert_to_display_data(self.remaining_time, with_dot);
                context.led.lock().unwrap().write(display_data);
            },
            State::WorkingPaused => {
                if was_start_stop_button_pressed {
                    self.state = State::Working;
                }
            },
            State::Resting => {
                do_count_down(&mut self.remaining_time, &sub_frame);
                if was_start_stop_button_pressed {
                    self.state = State::RestingPaused;
                }
                if self.remaining_time == 0 {
                    self.remaining_time = 25 * 60;
                    self.state = State::Preparing;
                }
                let display_data = convert_to_display_data(self.remaining_time, with_dot);
                context.led.lock().unwrap().write(display_data);
            },
            State::RestingPaused => {
                if was_start_stop_button_pressed {
                    self.state = State::Resting;
                }
            },
        }
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}
