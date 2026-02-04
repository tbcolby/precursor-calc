//! PDDB settings persistence

use crate::functions::{AngleMode, NumberBase};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};

const DICT_NAME: &str = "calc.settings";
const KEY_NAME: &str = "state";

/// Persistent calculator settings
#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    /// 0 = Algebraic, 1 = RPN
    pub mode: u8,
    /// 0 = DEG, 1 = RAD, 2 = GRAD
    pub angle_mode: u8,
    /// 0 = DEC, 1 = HEX, 2 = OCT, 3 = BIN
    pub number_base: u8,
    /// Memory registers
    pub memory: [f64; 10],
    /// Last answer
    pub ans: f64,
}

impl Settings {
    pub fn is_rpn(&self) -> bool {
        self.mode == 1
    }

    pub fn set_rpn(&mut self, rpn: bool) {
        self.mode = if rpn { 1 } else { 0 };
    }

    pub fn get_angle_mode(&self) -> AngleMode {
        AngleMode::from_u8(self.angle_mode)
    }

    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.angle_mode = mode.to_u8();
    }

    pub fn get_number_base(&self) -> NumberBase {
        NumberBase::from_u8(self.number_base)
    }

    pub fn set_number_base(&mut self, base: NumberBase) {
        self.number_base = base.to_u8();
    }
}

/// Storage manager
pub struct Storage {
    pddb: pddb::Pddb,
}

impl Storage {
    pub fn new() -> Self {
        let pddb = pddb::Pddb::new();
        pddb.try_mount();
        Self { pddb }
    }

    /// Load settings from PDDB
    pub fn load(&self) -> Settings {
        match self.pddb.get(
            DICT_NAME,
            KEY_NAME,
            None,
            false,
            false,
            None,
            None::<fn()>,
        ) {
            Ok(mut key) => {
                let mut data = Vec::new();
                key.seek(SeekFrom::Start(0)).ok();
                if key.read_to_end(&mut data).is_ok() {
                    if let Ok(settings) = serde_json::from_slice::<Settings>(&data) {
                        return settings;
                    }
                }
                Settings::default()
            }
            Err(_) => Settings::default(),
        }
    }

    /// Save settings to PDDB
    pub fn save(&self, settings: &Settings) -> bool {
        let data = match serde_json::to_vec(settings) {
            Ok(d) => d,
            Err(_) => return false,
        };

        match self.pddb.get(
            DICT_NAME,
            KEY_NAME,
            None,
            true,
            true,
            Some(data.len()),
            None::<fn()>,
        ) {
            Ok(mut key) => {
                key.seek(SeekFrom::Start(0)).ok();
                if key.write_all(&data).is_ok() {
                    self.pddb.sync().is_ok()
                } else {
                    false
                }
            }
            Err(e) => {
                log::warn!("Failed to save settings: {:?}", e);
                false
            }
        }
    }
}

extern crate alloc;
