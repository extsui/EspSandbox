use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::Gpio4;
use esp_idf_hal::ledc::*;
use esp_idf_hal::ledc::config::TimerConfig;

use std::sync::mpsc;
use std::sync::mpsc::SendError;

pub enum BuzzerCommand {
    StartTone { frequency: u32 },
    StopTone,

    // TODO:
    //Play { score? },
    //Cancel,
    //QueryStatus,
}

pub struct BuzzerDriver {
    sender: Option<mpsc::SyncSender<BuzzerCommand>>,
}

impl BuzzerDriver {
    pub fn new() -> Self {
        Self {
            sender: None,
        }
    }
    
    pub fn start_thread(&mut self, buzzer_pin: Gpio4, channel: CHANNEL0, timer: TIMER0) -> anyhow::Result<()> {
        let timer_config = &TimerConfig::new().resolution(Resolution::Bits10).frequency(1.kHz().into());
        let mut timer = LedcTimerDriver::new(timer, timer_config)?;
        let mut driver = LedcDriver::new(
            channel,
            &timer,
            buzzer_pin,
        )?;
        
        let (tx, rx) = mpsc::sync_channel::<BuzzerCommand>(5);
        let _ = std::thread::spawn(move || {
            for command in rx {
                match command {
                    BuzzerCommand::StartTone { frequency } => {
                        let max_duty = driver.get_max_duty();
                        timer.set_frequency(frequency.Hz()).unwrap();
                        driver.set_duty(max_duty / 2).unwrap();    // これが音出力のトリガーとなる
                        timer.resume().unwrap();

                        log::info!("[buz] start: {} Hz", frequency);
                    },
                    BuzzerCommand::StopTone => {
                        timer.pause().unwrap();

                        log::info!("[buz] stop");
                    },
                }
            }
        });
        self.sender = Some(tx);
        Ok(())
    }
    
    pub fn start_tone(&mut self, frequency: u32) -> Result<(), SendError<BuzzerCommand>> {
        self.sender.as_mut().unwrap().send(BuzzerCommand::StartTone { frequency })
    }

    pub fn stop_tone(&mut self) -> Result<(), SendError<BuzzerCommand>> {
        self.sender.as_mut().unwrap().send(BuzzerCommand::StopTone)
    }
}
