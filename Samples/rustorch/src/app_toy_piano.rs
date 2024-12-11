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

    fn initialize(&mut self, _context: &AppContext) -> anyhow::Result<()> {
        self.finished = false;
        self.previous_key_status = 0;
        Ok(())
    }

    fn update(&mut self, context: &AppContext, _frame_count: u64) -> anyhow::Result<()> {
        let key_status = context.button.lock().unwrap().get_status();

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

        let format = format!("{:4}", raw_value);
        context.led.lock().unwrap().write_format(&format);

        self.previous_key_status = key_status;
        Ok(())
    }

    fn finalize(&mut self, context: &AppContext) -> anyhow::Result<()> {
        context.buzzer.lock().unwrap().stop_tone()?;
        context.led.lock().unwrap().clear();
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}
