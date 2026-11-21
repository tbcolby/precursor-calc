//! Glue between keys, `calc-core`, the PDDB and the screen.

use calc_core::{Action, Calculator, LoadError, Saved};
use precursor_appkit::keys::Key;
use precursor_appkit::screen::Screen;
use precursor_appkit::store::Store;

/// PDDB dictionary and key. Same names as the original app so old settings load.
pub const DICT: &str = "calc.settings";
const KEY: &str = "state";

pub struct App {
    screen: Screen,
    store: Store,
    calc: Calculator,
    /// What is on flash, so we only write when something changed.
    last_saved: Saved,
    /// Saved data couldn't be used; don't overwrite it until the user says so.
    read_only: bool,
    status: String,
}

impl App {
    pub fn new(screen: Screen, store: Store) -> Self {
        let calc = Calculator::new();
        let last_saved = calc.to_saved();
        let mut app = Self { screen, store, calc, last_saved, read_only: false, status: String::new() };
        app.reload();
        app.draw();
        app
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    /// (Re)load from the PDDB. Called at start and when the set of open bases
    /// changes, so state from a just-locked basis doesn't linger.
    pub fn reload(&mut self) {
        self.calc = Calculator::new();
        self.read_only = false;
        self.status.clear();
        match self.store.load::<Saved>(KEY) {
            Ok(Some(saved)) => match self.calc.load(saved) {
                Ok(()) => {}
                Err(LoadError::NewerVersion(v)) => {
                    log::warn!("saved state version {} is newer than this app", v);
                    self.read_only = true;
                    self.status = "Saved data is from a newer version; not saving".into();
                }
            },
            Ok(None) => {}
            Err(e) => {
                log::error!("couldn't load saved state: {:?}", e);
                self.read_only = true;
                self.status = "Saved data unreadable; not saving (F3 w overwrites)".into();
            }
        }
        self.last_saved = self.calc.to_saved();
    }

    /// Write state if it changed since the last save.
    pub fn save(&mut self) {
        if self.read_only {
            return;
        }
        let now = self.calc.to_saved();
        if now == self.last_saved {
            return;
        }
        match self.store.save(KEY, &now) {
            Ok(()) => {
                self.last_saved = now;
                self.status.clear();
            }
            Err(e) => {
                log::error!("save failed: {:?}", e);
                self.status = "Save failed".into();
            }
        }
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        let action = self.calc.on_key(key);
        if self.calc.take_overwrite_request() && self.read_only {
            self.read_only = false;
            self.last_saved = Saved::default(); // force the next save
            self.status = "Saved data will be overwritten".into();
        }
        action
    }

    pub fn on_foreground(&mut self) {
        self.screen.refresh_size();
        self.draw();
    }

    pub fn draw(&self) { crate::ui::draw(&self.screen, &self.calc, &self.status); }
}
