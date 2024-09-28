use esp_idf_hal::gpio::AnyInputPin;
use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::Pull;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::gpio_set_pull_mode;
use std::thread;
use std::sync::{Arc, Mutex};
use esp_idf_hal::ledc::*;
use esp_idf_hal::prelude::*;

struct LedPins {
    seg_a: AnyOutputPin,
    seg_b: AnyOutputPin,
    seg_c: AnyOutputPin,
    seg_d: AnyOutputPin,
    seg_e: AnyOutputPin,
    seg_f: AnyOutputPin,
    seg_g: AnyOutputPin,
    seg_dot: AnyOutputPin,
    seg_digit1: AnyOutputPin,
    seg_digit2: AnyOutputPin,
    seg_digit3: AnyOutputPin,
    seg_digit4: AnyOutputPin,
}

#[derive(Clone, Copy)]
struct LedDriver {
    data: [u8; 4],
}

impl LedDriver {
    pub fn new() -> Self {
        LedDriver {
            data: [ 0x00, 0x00, 0x00, 0x00 ],
        }
    }

    pub fn start_dynamic_lighting(self, pins: LedPins) -> anyhow::Result<()> {
        let _ = thread::spawn(move || -> anyhow::Result<()> {
            let mut seg_a = PinDriver::output(pins.seg_a)?;
            let mut seg_b = PinDriver::output(pins.seg_b)?;
            let mut seg_c = PinDriver::output(pins.seg_c)?;
            let mut seg_d = PinDriver::output(pins.seg_d)?;
            let mut seg_e = PinDriver::output(pins.seg_e)?;
            let mut seg_f = PinDriver::output(pins.seg_f)?;
            let mut seg_g = PinDriver::output(pins.seg_g)?;
            let mut seg_dot = PinDriver::output(pins.seg_dot)?;
            let mut seg_digit1 = PinDriver::output(pins.seg_digit1)?;
            let mut seg_digit2 = PinDriver::output(pins.seg_digit2)?;
            let mut seg_digit3 = PinDriver::output(pins.seg_digit3)?;
            let mut seg_digit4 = PinDriver::output(pins.seg_digit4)?;
        
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
        
            let mut display_number = 0;
        
            let mut i = 0;
            loop {
                if i % 100 == 0 {
                    display_number = (display_number + 1) % NUMBER_SEGMENT_TABLE.len();
        
                    log::info!("[led] {}", i);
                }
        
                let bit_pattern = NUMBER_SEGMENT_TABLE[display_number];
                if (bit_pattern & ((1 as u8) << 7)) != 0 { seg_a.set_high()? } else { seg_a.set_low()?; }
                if (bit_pattern & ((1 as u8) << 6)) != 0 { seg_b.set_high()? } else { seg_b.set_low()?; }
                if (bit_pattern & ((1 as u8) << 5)) != 0 { seg_c.set_high()? } else { seg_c.set_low()?; }
                if (bit_pattern & ((1 as u8) << 4)) != 0 { seg_d.set_high()? } else { seg_d.set_low()?; }
                if (bit_pattern & ((1 as u8) << 3)) != 0 { seg_e.set_high()? } else { seg_e.set_low()?; }
                if (bit_pattern & ((1 as u8) << 2)) != 0 { seg_f.set_high()? } else { seg_f.set_low()?; }
                if (bit_pattern & ((1 as u8) << 1)) != 0 { seg_g.set_high()? } else { seg_g.set_low()?; }
                if (bit_pattern & ((1 as u8) << 0)) != 0 { seg_dot.set_high()? } else { seg_dot.set_low()?; }
            
                // ON 期間
                if i % 4 == 0 {
                    seg_digit1.set_high()?;
                } else if i % 4 == 1 {
                    seg_digit2.set_high()?;
                } else if i % 4 == 2 {
                    seg_digit3.set_high()?;
                } else {
                    seg_digit4.set_high()?;
                }
                FreeRtos::delay_ms(4);
                
                // OFF 期間
                seg_digit1.set_low()?;
                seg_digit2.set_low()?;
                seg_digit3.set_low()?;
                seg_digit4.set_low()?;
        
                i += 1;
            }
        });
        Ok(())
    }

    pub fn write(self, data: [u8; 4]) {
        // TODO:
    }
}

struct KeyMatrixPins {
    key_in1: AnyInputPin,
    key_in2: AnyInputPin,
    key_in3: AnyInputPin,
    key_out1: AnyOutputPin,
    key_out2: AnyOutputPin,
}

struct KeyMatrix {
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
        led_driver_clone.lock().unwrap().start_dynamic_lighting(led_pins)?;
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
    
    // メインスレッド
    loop {
        // ボタン情報取得
        let status = key_matrix.lock().unwrap().get_status();
        log::info!("{:02x}", status);
        
        FreeRtos::delay_ms(100);
    }

/*
    let buzzer_pin = peripherals.pins.gpio4;
    let channel0 = peripherals.ledc.channel0;
    let timer0 = peripherals.ledc.timer0;
    
    let config = &config::TimerConfig::new().resolution(Resolution::Bits10).frequency(1.kHz().into());
    let mut timer = LedcTimerDriver::new(timer0, config)?;

    let mut channel = LedcDriver::new(
        channel0,
        &timer,
        buzzer_pin,
    )?;

    let max_duty = channel.get_max_duty();
    for frequency in [110, 220, 440, 880, 1760].iter().cycle() {
        log::info!("{}", frequency);
        timer.set_frequency(frequency.Hz())?;

        channel.set_duty(max_duty / 2)?;    // これが音出力のトリガーとなる
        FreeRtos::delay_ms(1000);
    }
*/
    Ok(())
}
