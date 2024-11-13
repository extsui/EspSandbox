
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::AnyOutputPin;
use std::thread;
use std::sync::{Arc, Mutex};

use esp_idf_sys::xTaskDelayUntil;
use esp_idf_sys::xTaskGetTickCount;
use esp_idf_sys::TickType_t;

use esp_idf_hal::timer::*;
use esp_idf_hal::task::notification::Notification;
use std::num::NonZeroU32;

pub struct LedPins {
    pub seg_a: AnyOutputPin,
    pub seg_b: AnyOutputPin,
    pub seg_c: AnyOutputPin,
    pub seg_d: AnyOutputPin,
    pub seg_e: AnyOutputPin,
    pub seg_f: AnyOutputPin,
    pub seg_g: AnyOutputPin,
    pub seg_dot: AnyOutputPin,
    pub seg_digit1: AnyOutputPin,
    pub seg_digit2: AnyOutputPin,
    pub seg_digit3: AnyOutputPin,
    pub seg_digit4: AnyOutputPin,
}

pub struct LedDriver {
    display_data: Arc<Mutex<[u8; 4]>>,
    brightness: Arc<Mutex<[u8; 4]>>,
}

impl LedDriver {
    pub fn new() -> Self {
        LedDriver {
            display_data: Arc::new(Mutex::new([ 0, 0, 0, 0 ])),
            brightness: Arc::new(Mutex::new([ 100, 100, 100, 100 ])),
        }
    }

    pub fn start_dynamic_lighting(&mut self, pins: LedPins, timer10: TIMER10) -> anyhow::Result<()> {
        let display_data_clone = Arc::clone(&self.display_data);
        let brightness_clone = Arc::clone(&self.brightness);

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
        
            // タイマ本数が余ってないので周期は固定
            // 1000HZ 設定なので tick == ミリ秒
            const PERIOD: TickType_t = 4;

            // 7 セグ 1 桁毎の輝度制御に使用 (us ~ ms) 
            let timer_config = esp_idf_hal::timer::config::Config::new().auto_reload(false).divider(2); // Divider の最小値は 2
            let mut timer = esp_idf_hal::timer::TimerDriver::new(timer10, &timer_config)?;

            let notification = Notification::new();
            let notifer = notification.notifier();
            unsafe {
                timer.subscribe(move || {
                    notifer.notify_and_yield(NonZeroU32::new(1).unwrap());
                })?;
            }

            let mut last_wake_time: TickType_t = unsafe { xTaskGetTickCount() };
            
            let mut i = 0;
            loop {
                let bit_pattern = display_data_clone.lock().unwrap()[(i % 4) as usize];
                if (bit_pattern & ((1 as u8) << 7)) != 0 { seg_a.set_high()? } else { seg_a.set_low()?; }
                if (bit_pattern & ((1 as u8) << 6)) != 0 { seg_b.set_high()? } else { seg_b.set_low()?; }
                if (bit_pattern & ((1 as u8) << 5)) != 0 { seg_c.set_high()? } else { seg_c.set_low()?; }
                if (bit_pattern & ((1 as u8) << 4)) != 0 { seg_d.set_high()? } else { seg_d.set_low()?; }
                if (bit_pattern & ((1 as u8) << 3)) != 0 { seg_e.set_high()? } else { seg_e.set_low()?; }
                if (bit_pattern & ((1 as u8) << 2)) != 0 { seg_f.set_high()? } else { seg_f.set_low()?; }
                if (bit_pattern & ((1 as u8) << 1)) != 0 { seg_g.set_high()? } else { seg_g.set_low()?; }
                if (bit_pattern & ((1 as u8) << 0)) != 0 { seg_dot.set_high()? } else { seg_dot.set_low()?; }
            
                // 0 ~ 100% の割合で表す
                let brightness = brightness_clone.lock().unwrap()[(i % 4) as usize];

                // 1秒あたり timer.tick_hz() だけカウントされる
                // 1usあたり timer.tick_hz() / 1000_000 だけカウントされる
                // 100% の時 PERIOD*1000 [us] で、0% の時 0 [us] とする
                let to_counter_value = |_percent: u8| -> u64 {
                    let us_per_count = timer.tick_hz() / 1000_000;
                    (PERIOD as u64 * 1000 * _percent as u64 / 100) * us_per_count
                };
                
                if i % 25 == 0 {
                    log::debug!("[led] br: {}%, cnt: {}", brightness, to_counter_value(brightness));
                }

                if brightness > 0 {
                    timer.set_alarm(to_counter_value(brightness))?;
                    timer.set_counter(0)?;
                    timer.enable_interrupt()?;
                    timer.enable_alarm(true)?;
                    timer.enable(true)?;

                    // ON 期間
                    match i % 4 {
                        0 => seg_digit1.set_high()?,
                        1 => seg_digit2.set_high()?,
                        2 => seg_digit3.set_high()?,
                        3 => seg_digit4.set_high()?,
                        _ => (),
                    }

                    // auto_reload 無しなので割り込み後にタイマは自動停止するはず
                    let _ = notification.wait(esp_idf_hal::delay::BLOCK);
                }

                // OFF 期間
                seg_digit1.set_low()?;
                seg_digit2.set_low()?;
                seg_digit3.set_low()?;
                seg_digit4.set_low()?;
                
                unsafe {
                    xTaskDelayUntil(&mut last_wake_time, PERIOD)
                };

                i += 1;
            }
        });
        Ok(())
    }

    pub fn write(&mut self, data: [u8; 4]) {
        *self.display_data.lock().unwrap() = data;
    }

    pub fn set_brightness(&mut self, brightness: [u8; 4]) {
        *self.brightness.lock().unwrap() = brightness;
    }
}
