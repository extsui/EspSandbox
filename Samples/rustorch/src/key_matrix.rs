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

pub struct KeyMatrix {
    status: Arc<Mutex<u8>>,
    is_scanning: bool,
}

impl KeyMatrix {
    pub fn new() -> Self {
        KeyMatrix {
            status: Arc::new(Mutex::new(0)),
            is_scanning: false,
        }
    }

    pub fn start_scan(&mut self, pins: KeyMatrixPins) -> anyhow::Result<()> {
        assert!(!self.is_scanning);
        self.is_scanning = true;

        let status_clone = Arc::clone(&self.status);

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
        
            // 0: UP 
            // 1: LEFT
            // 2: DOWN
            // 3: RIGHT
            // 4: A
            // 5: B
            // 6: -
            // 7: -
            let mut button_out1: u8 = 0x00;
            let mut button_out2: u8 = 0x00;
        
            loop {
                // 出力端子切り替えから入力端子が安定するまでにある程度時間がかかるはずなので
                // 「出力端子切り替え -> ポーリング周期時間分ウェイト -> (ループ先頭) 入力取得」とする
                if i % 2 == 0 {
                    if in1.is_low() { button_out1 |= 0x01 as u8; }   // SW_UP
                    if in2.is_low() { button_out1 |= 0x08 as u8; }   // SW_RIGHT
                    if in3.is_low() { button_out1 |= 0x10 as u8; }   // SW_A
                    out1.set_high().unwrap();
                    out2.set_low().unwrap();
                } else {
                    if in1.is_low() { button_out2 |= 0x02 as u8; }   // SW_LEFT
                    if in2.is_low() { button_out2 |= 0x04 as u8; }   // SW_DOWN
                    if in3.is_low() { button_out2 |= 0x20 as u8; }   // SW_B
                    out1.set_low().unwrap();
                    out2.set_high().unwrap();
                }
        
                if i % 2 == 0 {
                    let button = button_out1 | button_out2;
        
                    let mut status = status_clone.lock().unwrap();
                    *status = button;

                    log::debug!(
                        "[key] {} {}{}{}{}{}{}", i / 2,
                        if button & 0x01 != 0 { '^' } else { ' ' },
                        if button & 0x02 != 0 { '<' } else { ' ' },
                        if button & 0x04 != 0 { 'v' } else { ' ' },
                        if button & 0x08 != 0 { '>' } else { ' ' },
                        if button & 0x10 != 0 { 'A' } else { ' ' },
                        if button & 0x20 != 0 { 'B' } else { ' ' },
                    );

                    button_out1 = 0;
                    button_out2 = 0;
                }
        
                FreeRtos::delay_ms(10);
        
                i += 1;
            }
        });
        Ok(())
    }

    pub fn get_status(&self) -> u8 {
        *self.status.lock().unwrap()
    }
}
