use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;

    let mut seg_a = PinDriver::output(peripherals.pins.gpio21)?;
    let mut seg_b = PinDriver::output(peripherals.pins.gpio22)?;
    let mut seg_c = PinDriver::output(peripherals.pins.gpio23)?;
    let mut seg_d = PinDriver::output(peripherals.pins.gpio15)?;
    let mut seg_e = PinDriver::output(peripherals.pins.gpio17)?;
    let mut seg_f = PinDriver::output(peripherals.pins.gpio16)?;
    let mut seg_g = PinDriver::output(peripherals.pins.gpio3)?;
    let mut seg_dot = PinDriver::output(peripherals.pins.gpio2)?;
    
    let mut seg_digit1 = PinDriver::output(peripherals.pins.gpio11)?;
    let mut seg_digit2 = PinDriver::output(peripherals.pins.gpio18)?;
    let mut seg_digit3 = PinDriver::output(peripherals.pins.gpio19)?;
    let mut seg_digit4 = PinDriver::output(peripherals.pins.gpio20)?;

    seg_a.set_low()?;
    seg_b.set_low()?;
    seg_c.set_low()?;
    seg_d.set_low()?;
    seg_e.set_low()?;
    seg_f.set_low()?;
    seg_g.set_low()?;
    seg_dot.set_low()?;

    seg_digit1.set_low()?;
    seg_digit2.set_low()?;
    seg_digit3.set_low()?;
    seg_digit4.set_low()?;

    let mut polarity = true;

    let mut i = 0;
    loop {
        log::info!("{}", i);

        if i % 50 == 0 {
            polarity ^= true;
        }

        if polarity {
            seg_a.set_high()?;
            seg_b.set_high()?;
            seg_c.set_high()?;
            seg_d.set_high()?;
            seg_e.set_high()?;
            seg_f.set_high()?;
            seg_g.set_high()?;
            seg_dot.set_high()?;
        } else {
            seg_a.set_low()?;
            seg_b.set_low()?;
            seg_c.set_low()?;
            seg_d.set_low()?;
            seg_e.set_low()?;
            seg_f.set_low()?;
            seg_g.set_low()?;
            seg_dot.set_low()?;
        }
    
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
        FreeRtos::delay_ms(5);
        
        // OFF 期間
        seg_digit1.set_low()?;
        seg_digit2.set_low()?;
        seg_digit3.set_low()?;
        seg_digit4.set_low()?;
        FreeRtos::delay_ms(0);

        i += 1;
    }
}
