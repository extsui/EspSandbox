use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{Gpio6, Gpio7};
use esp_idf_hal::i2c::I2C0;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::i2c::I2cDriver;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    image::Image,
};
use tinybmp::Bmp;

use ssd1306::{
    prelude::*,
    I2CDisplayInterface,
    Ssd1306
};

use esp_idf_hal::delay::FreeRtos;

pub struct DisplayDriver {

}

impl DisplayDriver {
    pub fn new() -> Self {
        DisplayDriver {
        }
    }

    pub fn start_thread(&mut self, i2c0: I2C0, sda: Gpio6, scl: Gpio7) -> anyhow::Result<()> {
        let i2c_config = I2cConfig::new().baudrate(400.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
        let i2c = I2cDriver::new(i2c0, sda, scl, &i2c_config)?;
        let i2c_interface = I2CDisplayInterface::new(i2c);
       
        let mut display = Ssd1306::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();

        display.clear(BinaryColor::On).unwrap();
        display.flush().unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let _ = std::thread::spawn(move || {

            // デフォルトの優先度が 5 なのでそれより低くしておく
            unsafe { esp_idf_sys::vTaskPrioritySet(std::ptr::null_mut(), 4); };

            loop {
                // ESP 環境では std::time::Instant::now() で起動してからの時刻を取得できなかった
                let uptime_us = unsafe { esp_idf_sys::esp_timer_get_time() };
                let text = format!("{} [ms]", uptime_us / 1000);

                println!("{}", text);

                display.clear(BinaryColor::Off).unwrap();
                
                let working_bmp = Bmp::from_slice(include_bytes!("../asserts/images/pomodoro_working.bmp")).unwrap();
                let working_img: Image<Bmp<BinaryColor>> = Image::new(&working_bmp, Point::new(0, 0));
                working_img.draw(&mut display).unwrap();
    
                Text::with_baseline(&text, Point::new(0, 0), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
        
                display.flush().unwrap();
                
                FreeRtos::delay_ms(30);
            }
        });
        Ok(())
    }

    // 画像描画
    pub fn draw_image(&mut self) {
        // TODO:
    }

    // テキスト描画
    pub fn draw_text(&mut self) {
        // TODO:
    }

    // 画面更新
    pub fn update(&mut self) {
        // TODO:
    }
}
