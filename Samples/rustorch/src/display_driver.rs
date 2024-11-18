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

use std::sync::mpsc;
use std::sync::mpsc::SendError;

pub enum DisplayCommand {
    Clear,
    DrawImage,  // TODO:
    DrawText { text: String, point: Point },
    Update,
}

pub struct DisplayDriver {
    sender: Option<mpsc::SyncSender<DisplayCommand>>,
}

impl DisplayDriver {
    pub fn new() -> Self {
        Self {
            sender: None,
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
        
        let (tx, rx) = mpsc::sync_channel::<DisplayCommand>(10);

        let _ = std::thread::spawn(move || {

            // デフォルトの優先度が 5 なのでそれより低くしておく
            unsafe { esp_idf_sys::vTaskPrioritySet(std::ptr::null_mut(), 4); };

            for command in rx {
                match command {
                    DisplayCommand::Clear => {
                        display.clear(BinaryColor::Off).unwrap();
                    }
                    DisplayCommand::DrawImage => {
                        //let bmp = Bmp::from_slice(include_bytes!("../asserts/images/pomodoro_working.bmp")).unwrap();
                        //let gfx_img: Image<Bmp<BinaryColor>> = Image::new(&bmp, Point::new(0, 0));
                        //gfx_img.draw(&mut display).unwrap();
                    }
                    DisplayCommand::DrawText { text, point } => {
                        let text_img = Text::with_baseline(&text, point, text_style, Baseline::Top);
                        text_img.draw(&mut display).unwrap();
                    }
                    DisplayCommand::Update => {
                        display.flush().unwrap();
                    }
                }
            }
        });
        self.sender = Some(tx);
        Ok(())
    }

    // 描画系の前に一度だけ呼び出すこと
    pub fn clear(&mut self) -> Result<(), SendError<DisplayCommand>> {
        self.sender.as_mut().unwrap().send(DisplayCommand::Clear)
    }

    // 画像描画
    //pub fn draw_image(&mut self, image, point: Point) {
    //    self.sender.as_mut().unwrap().send(DisplayCommand::DrawImage {});
    //}

    // テキスト描画
    pub fn draw_text(&mut self, text: String, point: Point) -> Result<(), SendError<DisplayCommand>> {
        self.sender.as_mut().unwrap().send(DisplayCommand::DrawText { text, point })
    }

    // 画面の更新
    // - 画面更新に数十ミリ秒かかる
    // - 画面描画が完了するまでは次の描画依頼を出しても詰まることに注意
    pub fn update(&mut self) -> Result<(), SendError<DisplayCommand>> {
        self.sender.as_mut().unwrap().send(DisplayCommand::Update)
    }
}
