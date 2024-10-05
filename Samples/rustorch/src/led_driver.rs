
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::delay::FreeRtos;
use std::thread;
use std::sync::{Arc, Mutex};

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
}

impl LedDriver {
    pub fn new() -> Self {
        LedDriver {
            display_data: Arc::new(Mutex::new([ 0, 0, 0, 0 ])),
        }
    }

    pub fn start_dynamic_lighting(&mut self, pins: LedPins) -> anyhow::Result<()> {
        let display_data_clone = Arc::clone(&self.display_data);
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

    pub fn write(&mut self, data: [u8; 4]) {
        *self.display_data.lock().unwrap() = data;
    }
}
