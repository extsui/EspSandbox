use std::sync::{Arc, Mutex};
use crate::KeyMatrix;
use crate::BuzzerDriver;
use crate::DisplayDriver;
use crate::LedDriver;
use crate::Volume;

pub struct AppContext {
    pub button: Arc<Mutex<KeyMatrix>>,
    pub buzzer: Arc<Mutex<BuzzerDriver>>,
    pub display: Arc<Mutex<DisplayDriver>>,
    pub led: Arc<Mutex<LedDriver>>,
    pub volume: Arc<Mutex<Volume>>,
}

pub trait AppFramework {
    fn get_name(&self) -> &str;
    fn initialize(&mut self, context: &AppContext) -> anyhow::Result<()>;
    fn update(&mut self, context: &AppContext, frame_count: u64) -> anyhow::Result<()>;
    fn finalize(&mut self, context: &AppContext) -> anyhow::Result<()>;
    fn is_finished(&self) -> bool;
}

