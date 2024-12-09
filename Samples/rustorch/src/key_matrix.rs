use esp_idf_hal::gpio::AnyInputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::Pull;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::sys::gpio_set_pull_mode;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct KeyMatrixPins {
    pub key_in1: AnyInputPin,
    pub key_in2: AnyInputPin,
    pub key_in3: AnyInputPin,
    pub key_out1: AnyOutputPin,
    pub key_out2: AnyOutputPin,
}

pub struct Button;
impl Button {
    pub const UP:    u8 = 0x01;
    pub const LEFT:  u8 = 0x02;
    pub const DOWN:  u8 = 0x04;
    pub const RIGHT: u8 = 0x08;
    pub const A:     u8 = 0x10;
    pub const B:     u8 = 0x20;
    pub const MASK:  u8 = 0x3F; // マスク操作用
}

const KEY_STATUS_HISTORY_COUNT: usize = 3;

pub struct KeyMatrix {
    status: Arc<Mutex<[u8; KEY_STATUS_HISTORY_COUNT]>>,
    released_event: Arc<Mutex<u8>>,
    is_scanning: bool,
}

impl KeyMatrix {
    pub fn new() -> Self {
        KeyMatrix {
            status: Arc::new(Mutex::new(Default::default())),
            released_event: Arc::new(Mutex::new(0)),
            is_scanning: false,
        }
    }

    pub fn start_scan(&mut self, pins: KeyMatrixPins) -> anyhow::Result<()> {
        assert!(!self.is_scanning);
        self.is_scanning = true;

        let status_clone = Arc::clone(&self.status);
        let released_event_clone = Arc::clone(&self.released_event);

        let _ = thread::spawn(move || -> anyhow::Result<()> {
            let in1 = PinDriver::input(pins.key_in1)?;
            let in2 = PinDriver::input(pins.key_in2)?;
            let in3 = PinDriver::input(pins.key_in3)?;
            let mut out1 = PinDriver::output(pins.key_out1)?;
            let mut out2 = PinDriver::output(pins.key_out2)?;
        
            // in1, in3 は外部プルアップ抵抗があるのでそのまま
            // in2 は外部プルアップ抵抗がないので内部プルアップを使う
            unsafe {
                // in2.set_pull() と書きたいのだが downgrade_input() 後は AnyInputPin 型になる
                // 一方で set_pull() は InputPin + OutputPin を要求してくるので呼び出せなくなる
                // --> set_pull() 内で呼び出している C 関数を直接呼び出すことで対処した
                gpio_set_pull_mode(in2.pin(), Pull::Up.into());
            }
        
            let mut i = 0;
        
            out1.set_high()?;
            out2.set_low()?;
        
            let mut button_out1: u8 = 0x00;
            let mut button_out2: u8 = 0x00;
        
            loop {
                // 出力端子切り替えから入力端子が安定するまでにある程度時間がかかるはずなので
                // 「出力端子切り替え -> ポーリング周期時間分ウェイト -> (ループ先頭) 入力取得」とする
                if i % 2 == 0 {
                    if in1.is_low() { button_out1 |= Button::UP; }
                    if in2.is_low() { button_out1 |= Button::RIGHT; }
                    if in3.is_low() { button_out1 |= Button::B; }
                    out1.set_high().unwrap();
                    out2.set_low().unwrap();
                } else {
                    if in1.is_low() { button_out2 |= Button::LEFT; }
                    if in2.is_low() { button_out2 |= Button::DOWN; }
                    if in3.is_low() { button_out2 |= Button::A; }
                    out1.set_low().unwrap();
                    out2.set_high().unwrap();
                }
        
                if i % 2 == 0 {
                    let button = button_out1 | button_out2;
                    button_out1 = 0;
                    button_out2 = 0;
                    
                    // インデックスが小さい方が新しい要素
                    let mut status = status_clone.lock().unwrap();
                    status.copy_within(0..KEY_STATUS_HISTORY_COUNT-1, 1);
                    status[0] = button;
                    
                    // 1 -> 1 -> 0 で離し検出
                    // 一度成立したらイベントを取得されるまでは有効のまま維持
                    let mut released_event = released_event_clone.lock().unwrap();
                    *released_event |= (!&status[0] & Button::MASK) &
                                       ( &status[1] & Button::MASK) &
                                       ( &status[2] & Button::MASK);
                    
                    log::debug!(
                        "[key] {} {}{}{}{}{}{}", i / 2,
                        if button & Button::UP    != 0 { '^' } else { ' ' },
                        if button & Button::LEFT  != 0 { '<' } else { ' ' },
                        if button & Button::DOWN  != 0 { 'v' } else { ' ' },
                        if button & Button::RIGHT != 0 { '>' } else { ' ' },
                        if button & Button::A     != 0 { 'A' } else { ' ' },
                        if button & Button::B     != 0 { 'B' } else { ' ' },
                    );
                    log::debug!("[{:02x}, {:02x}, {:02x}] -> {:02x}",
                        status[0], status[1], status[2], *released_event);
                }
        
                // OUT 信号線が 2 本なのでキースキャン周期は delay_ms の倍であることに注意
                FreeRtos::delay_ms(5);
        
                i += 1;
            }
        });
        Ok(())
    }

    // ボタンの現在の状態を取得する
    pub fn get_status(&self) -> u8 {
        self.status.lock().unwrap()[0]
    }

    pub fn is_pressed_all(&self) -> bool {
        self.get_status() == Button::MASK
    }

    // ボタンが押されて離されていたら true
    // 指定したボタンの情報は一度読み出すとクリアされる
    pub fn was_released(&self, button_mask: u8) -> u8 {
        let mut released_event = self.released_event.lock().unwrap();
        let released_event_masked = *released_event & button_mask;
        *released_event = 0;
        released_event_masked
    }

}
