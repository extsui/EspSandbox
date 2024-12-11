use crate::app_context::AppContext;
use crate::app_context::AppFramework;

use crate::Button;

enum State {
    // [---] 起動状態
    Startup,
    // [***] 回転中
    Rolling,
    // [nnn] 全桁確定 (再開待ち)
    Fixed,
}

pub struct SlotGame {
    finished: bool,

    state: State,
    // 何桁目まで確定したか
    fixed_digit_count: u8,
    // 各桁の数字の内部カウンタ、7セグの回転表示に使用
    internal_number: [u32; 3],
    // 確定された数字の格納先
    fixed_number: [u8; 3],

    animation_delay_param: u32
}

impl SlotGame {
    pub fn new() -> Self {
        SlotGame {
            finished: false,
            state: State::Startup,
            fixed_digit_count: 0,
            internal_number: Default::default(),
            fixed_number: Default::default(),
            animation_delay_param: 5,   // TORIAEZU: 初期値は適当
        }
    }
}

impl AppFramework for SlotGame {
    fn get_name(&self) -> &str {
        "Slot game"
    }

    fn initialize(&mut self, _context: &AppContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, context: &AppContext, _frame_count: u64) -> anyhow::Result<()> {
        let released_button = context.button.lock().unwrap().was_released(Button::MASK);
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

        match self.state {
            State::Startup => {
                if was_rolling_started {
                    self.state = State::Rolling;
                    self.fixed_digit_count = 0;
                    self.fixed_number = Default::default();
                    self.internal_number = Default::default();
                    println!("-> Rolling");
                }
            },
            State::Rolling => {
                // アニメーションの速度を動的に変更 (主にデバッグ用)
                if was_button_up_pressed {
                    if self.animation_delay_param > 1 {
                        self.animation_delay_param -= 1;
                    }
                }
                if was_button_down_pressed {
                    self.animation_delay_param += 1;
                }

                // 桁確定判定
                if was_number_selected {
                    let index = self.fixed_digit_count as usize;
                    // TODO: 内部数値から確定数値への変換処理を仕上げる (演出関連)
                    self.fixed_number[index] = (self.internal_number[index] / self.animation_delay_param / NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as u8;
                    self.fixed_digit_count += 1;

                    println!("fixed_digit_count : {} -> {}", index, index + 1);
                }

                // 定常処理

                // 内部数値のカウントアップ処理
                // TODO: 要パラメータ調整
                for value in self.internal_number.iter_mut() {
                    *value += 1;
                    if *value >= self.animation_delay_param * NUMBER_SEGMENT_TABLE.len() as u32 * NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32 {
                        *value = 0;
                    }
                }

                // 確定済み桁はその数字を表示、未確定の桁は遷移中のパターンを表示
                let mut display_data = [0u8; 4];
                for i in 0..display_data.len()-1 {
                    display_data[i] = if i < (self.fixed_digit_count as usize) {
                        let number_index = self.fixed_number[i] as usize;
                        NUMBER_SEGMENT_TABLE[number_index]
                    } else {
                        let number_index    = (self.internal_number[i] / self.animation_delay_param / NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as usize;
                        let animation_index = (self.internal_number[i] / self.animation_delay_param % NUMBER_SEGMENT_SLOT_TABLE[0].len() as u32) as usize;
                        NUMBER_SEGMENT_SLOT_TABLE[number_index][animation_index]
                    };
                }

                context.led.lock().unwrap().write_data(display_data);

                if self.fixed_digit_count == 3 {
                    self.state = State::Fixed;
                    println!("-> Fixed");
                }
            },
            State::Fixed => {
                // TODO: 結果に対して何かしらのアニメーションさせる?
                // TORIAEZU: 現状は NOP で遷移
                self.state = State::Startup;
                println!("-> Startup");
            },
        }
        Ok(())
    }

    fn finalize(&mut self, context: &AppContext) -> anyhow::Result<()> {
        context.led.lock().unwrap().clear();
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}
