use esp_idf_hal::gpio::Gpio5;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::adc::oneshot::AdcChannelDriver;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;

pub struct Volume {
    // ADC 関連の構造体を完全隠蔽するために Box + 'static が必要
    adc_driver: Box<AdcDriver<'static, ADC1>>,
    adc_channel_driver: AdcChannelDriver<'static, Gpio5, &'static AdcDriver<'static, ADC1>>,
}

impl Volume {
    pub fn new(adc: ADC1, pin: Gpio5) -> Self {
        let adc_config = AdcChannelConfig {
            attenuation: DB_11,
            calibration: true,
            ..Default::default()
        };
        // AdcDriver は AdcChannelDriver 生成で渡す時と read() 時のそれぞれで必要になる
        // --> AdcDriver をヒープ上に確保 + Box::leak() + 生ポインタ化で無理矢理共有する
        let adc_driver = Box::new(AdcDriver::new(adc).unwrap());
        let adc_driver_ref: &'static AdcDriver<'static, ADC1> = Box::leak(adc_driver);
        let adc_channel_driver = AdcChannelDriver::new(adc_driver_ref, pin, &adc_config).unwrap();
        Volume {
            adc_driver: unsafe { Box::from_raw(adc_driver_ref as *const _ as *mut _) },
            adc_channel_driver,            
        }
    }

    pub fn read_raw(&mut self) -> u16 {
        self.adc_driver.read(&mut self.adc_channel_driver).unwrap()
    }

    pub fn to_percent(raw_value: u16) -> u16 {
        (raw_value as u64 * 100 / 3270) as u16
    }
}    

