use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;

struct Leds {
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

fn run_thread(pins: Leds) -> anyhow::Result<()> {
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
        log::info!("{}", i);

        if i % 100 == 0 {
            display_number = (display_number + 1) % NUMBER_SEGMENT_TABLE.len();
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
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;

    let leds = Leds {
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

    let handle = thread::spawn(move || run_thread(leds));
    let _ = handle.join();

    Ok(())
}
