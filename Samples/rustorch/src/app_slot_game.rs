/*
use app_context;

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

    pub state: State,
    // 何桁目まで確定したか
    pub fixed_digit_count: u8,
    // 各桁の数字の内部カウンタ、7セグの回転表示に使用
    pub internal_number: [u32; 3],
    // 確定された数字の格納先
    pub fixed_number: [u8; 3],
}

impl SlotGame {
    pub fn new() -> Self {
        SlotGame {
            finished: false,
            state: State::Startup,
            fixed_digit_count: 0,
            internal_number: Default::default(),
            fixed_number: Default::default(),
        }
    }
}

impl AppFramework for SlotGame {
    fn initialize(&mut self) {

    }

    fn update(&mut self, context: &AppContext) {
        let mut animation_delay_param: u32 = 5; // TORIAEZU: 初期値は適当
    
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
    }

    fn is_finished(&self) -> bool {

    }
}
*/