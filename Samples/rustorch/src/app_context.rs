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

/*
// TODO:

pub struct AppMetadata {
    pub name: String,
    pub icon: Option<[u8; 128 * 64]>,   // TODO:
}
*/

pub trait AppFramework {
    fn initialize(&mut self);
    fn update(&mut self, context: &AppContext) -> anyhow::Result<()>;
    fn is_finished(&self) -> bool;
}

