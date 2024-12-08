use crate::app_context::AppContext;
use crate::app_context::AppFramework;

use crate::Button;

pub struct ToyPiano {
    finished: bool,
    previous_key_status: u8,
}

impl ToyPiano {
    pub fn new() -> Self {
        ToyPiano {
            finished: false,
            previous_key_status: 0,
        }
    }
}

impl AppFramework for ToyPiano {
    fn get_name(&self) -> &str {
        "Toy piano"
    }

    fn initialize(&mut self) {

    }

    fn update(&mut self, context: &AppContext, _frame_count: u64) -> anyhow::Result<()> {
        // ボタン情報取得
        let key_status = context.button.lock().unwrap().get_status();
        if key_status == Button::MASK {
            self.finished = true;
            return Ok(());
        }

        // 7セグ輝度調整用
        // 理論上は 0V ~ 3.3V (=3300) だが実際は 3.26V あたりでサチるので
        // 0% ~ 100% の範囲に入れるために最大値より少し小さい値で % を計算
        let raw_value = context.volume.lock().unwrap().read_raw();

        // TODO: モード選択を実装して ToyPiano モードで↓が実行されるようにする

        let octave =
            if      raw_value < 1000 { 0.5 }
            else if raw_value < 2000 { 1.0 }
            else if raw_value < 3000 { 2.0 }
            else                     { 4.0 };
        
        if self.previous_key_status != key_status {
            if key_status == 0 {
                // 全てのボタンを離した
                context.buzzer.lock().unwrap().stop_tone()?;
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
                    context.buzzer.lock().unwrap().start_tone(frequency)?;
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

        context.led.lock().unwrap().write(display_data);

        self.previous_key_status = key_status;
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}
