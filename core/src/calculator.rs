//! The calculator controller: keys in, state out. The app crate only draws
//! what this exposes and persists [`Saved`].

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use precursor_appkit_core::keys::Key;
use serde::{Deserialize, Serialize};

use crate::format::{format_decimal, format_number};
use crate::functions::{AngleMode, CalcError, Func, NumberBase, Op};
use crate::menu::{MenuAction, MenuId};
use crate::parser;
use crate::rpn::RpnStack;

/// Bump when [`Saved`] changes incompatibly.
pub const STATE_VERSION: u32 = 1;
/// Longest algebraic input, in characters.
pub const MAX_INPUT: usize = 120;
/// History entries kept in memory.
pub const HISTORY_MAX: usize = 50;
/// History entries persisted.
pub const HISTORY_SAVED: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Algebraic,
    Rpn,
}

/// What is drawn over the main display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    None,
    Menu(MenuId),
    /// Waiting for a register digit.
    Store,
    Recall,
}

/// Result of a key press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Stay,
    /// Hand focus back to the system (F4 on the main screen).
    Leave,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub expr: String,
    pub result: f64,
}

/// Persisted state. Field names of the version-0 format (`mode`, `angle_mode`,
/// `number_base`, `memory`, `ans`) are kept so old saves still load.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Saved {
    pub version: u32,
    /// 0 = algebraic, 1 = RPN.
    pub mode: u8,
    pub angle_mode: u8,
    pub number_base: u8,
    pub memory: [f64; 10],
    pub ans: f64,
    pub stack: [f64; 4],
    pub last_x: f64,
    pub history: Vec<HistoryEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// Written by a newer app; don't overwrite it.
    NewerVersion(u32),
}

fn clean(v: f64) -> f64 { if v.is_finite() { v } else { 0.0 } }

pub struct Calculator {
    mode: Mode,
    angle: AngleMode,
    base: NumberBase,
    input: String,
    ans: f64,
    rpn: RpnStack,
    memory: [f64; 10],
    history: Vec<HistoryEntry>,
    /// Index into `history` while browsing with Up/Down.
    browse: Option<usize>,
    overlay: Overlay,
    error: Option<&'static str>,
    overwrite_requested: bool,
}

impl Default for Calculator {
    fn default() -> Self { Self::new() }
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            mode: Mode::Algebraic,
            angle: AngleMode::Degrees,
            base: NumberBase::Decimal,
            input: String::new(),
            ans: 0.0,
            rpn: RpnStack::new(),
            memory: [0.0; 10],
            history: Vec::new(),
            browse: None,
            overlay: Overlay::None,
            error: None,
            overwrite_requested: false,
        }
    }

    // ---- view ----

    pub fn mode(&self) -> Mode { self.mode }

    pub fn angle(&self) -> AngleMode { self.angle }

    pub fn base(&self) -> NumberBase { self.base }

    pub fn input(&self) -> &str { &self.input }

    pub fn ans(&self) -> f64 { self.ans }

    pub fn rpn(&self) -> &RpnStack { &self.rpn }

    pub fn memory(&self) -> &[f64; 10] { &self.memory }

    pub fn overlay(&self) -> Overlay { self.overlay }

    pub fn error(&self) -> Option<&'static str> { self.error }

    /// The `n` most recent history lines, oldest first.
    pub fn history_lines(&self, n: usize) -> Vec<String> {
        let start = self.history.len().saturating_sub(n);
        self.history[start..]
            .iter()
            .map(|h| format!("{} = {}", h.expr, format_number(h.result, self.base)))
            .collect()
    }

    /// One-line mode summary for the header.
    pub fn status_line(&self) -> String {
        let mode = match self.mode {
            Mode::Algebraic => "ALG",
            Mode::Rpn => "RPN",
        };
        let mem = if self.memory.iter().any(|&v| v != 0.0) { " M" } else { "" };
        format!("{} {} {}{}", mode, self.angle.label(), self.base.label(), mem)
    }

    /// Display text of an RPN register (X shows the number being typed).
    pub fn rpn_register(&self, i: usize) -> String {
        if i == 0 {
            if let Some(e) = self.rpn.entry() {
                return format!("{e}_");
            }
        }
        format_number(self.rpn.stack()[i], self.base)
    }

    // ---- persistence ----

    pub fn to_saved(&self) -> Saved {
        let start = self.history.len().saturating_sub(HISTORY_SAVED);
        Saved {
            version: STATE_VERSION,
            mode: if self.mode == Mode::Rpn { 1 } else { 0 },
            angle_mode: self.angle.to_u8(),
            number_base: self.base.to_u8(),
            memory: self.memory.map(clean),
            ans: clean(self.ans),
            stack: self.rpn.stack().map(clean),
            last_x: clean(self.rpn.last_x()),
            history: self.history[start..].iter().filter(|h| h.result.is_finite()).cloned().collect(),
        }
    }

    /// Replace state from a save. Non-finite numbers become 0.
    pub fn load(&mut self, s: Saved) -> Result<(), LoadError> {
        if s.version > STATE_VERSION {
            return Err(LoadError::NewerVersion(s.version));
        }
        *self = Self::new();
        self.mode = if s.mode == 1 { Mode::Rpn } else { Mode::Algebraic };
        self.angle = AngleMode::from_u8(s.angle_mode);
        self.base = NumberBase::from_u8(s.number_base);
        self.memory = s.memory.map(clean);
        self.ans = clean(s.ans);
        self.rpn = RpnStack::from_registers(s.stack, s.last_x);
        self.history = s.history.into_iter().filter(|h| h.result.is_finite()).collect();
        let excess = self.history.len().saturating_sub(HISTORY_MAX);
        self.history.drain(..excess);
        Ok(())
    }

    // ---- input ----

    pub fn on_key(&mut self, key: Key) -> Action {
        self.error = None;
        match self.overlay {
            Overlay::Menu(menu) => {
                self.on_menu_key(menu, key);
                Action::Stay
            }
            Overlay::Store | Overlay::Recall => {
                self.on_register_key(key);
                Action::Stay
            }
            Overlay::None => self.on_main_key(key),
        }
    }

    /// Called by the shell when the saved data was locked read-only and the
    /// user chose "Overwrite unreadable save". Returns whether it was chosen
    /// since the last call.
    pub fn take_overwrite_request(&mut self) -> bool { core::mem::take(&mut self.overwrite_requested) }

    fn on_menu_key(&mut self, menu: MenuId, key: Key) {
        match key {
            Key::F(n) => match MenuId::from_fkey(n) {
                Some(m) if m != menu => self.overlay = Overlay::Menu(m),
                _ => self.overlay = Overlay::None, // same F-key again, or F4
            },
            Key::Backspace => self.overlay = Overlay::None,
            Key::Char(c) => {
                if let Some(action) = menu.find(c) {
                    self.overlay = Overlay::None;
                    self.menu_action(action);
                }
            }
            _ => {}
        }
    }

    fn on_register_key(&mut self, key: Key) {
        match key {
            Key::Char(c) if c.is_ascii_digit() => {
                let r = c as usize - '0' as usize;
                if self.overlay == Overlay::Store {
                    self.store(r);
                } else {
                    self.recall(r);
                }
                self.overlay = Overlay::None;
            }
            k if k.is_back() => self.overlay = Overlay::None,
            _ => {}
        }
    }

    fn on_main_key(&mut self, key: Key) -> Action {
        match key {
            Key::F(4) => return Action::Leave,
            Key::F(n) => {
                if let Some(m) = MenuId::from_fkey(n) {
                    self.overlay = Overlay::Menu(m);
                }
            }
            Key::Char('M') => self.toggle_mode(),
            Key::Char('A') => self.angle = self.angle.cycle(),
            Key::Char('B') => self.base = self.base.cycle(),
            Key::Char('S') => self.overlay = Overlay::Store,
            Key::Char('K') => self.overlay = Overlay::Recall,
            Key::Char('C') => self.clear_all(),
            _ => match self.mode {
                Mode::Algebraic => self.alg_key(key),
                Mode::Rpn => self.rpn_key(key),
            },
        }
        Action::Stay
    }

    fn alg_key(&mut self, key: Key) {
        match key {
            Key::Enter | Key::Char('=') => self.alg_evaluate(),
            Key::Backspace => {
                self.input.pop();
                self.browse = None;
            }
            Key::Char(' ') => {
                self.input.clear();
                self.browse = None;
            }
            Key::Up => self.browse_history(-1),
            Key::Down => self.browse_history(1),
            Key::Char(c) if is_expression_char(c) => self.alg_insert(&String::from(c)),
            _ => {} // ignored, never quits
        }
    }

    fn rpn_key(&mut self, key: Key) {
        let r = match key {
            Key::Char(c) if c.is_ascii_digit() => {
                self.rpn.digit(c);
                Ok(())
            }
            Key::Char('.') => {
                self.rpn.decimal_point();
                Ok(())
            }
            Key::Char('e') => {
                self.rpn.exponent();
                Ok(())
            }
            Key::Enter | Key::Char('=') => {
                self.rpn.enter();
                Ok(())
            }
            Key::Backspace => {
                self.rpn.backspace();
                Ok(())
            }
            Key::Char(' ') => {
                self.rpn.clear_x();
                Ok(())
            }
            Key::Char('n') => {
                self.rpn.change_sign();
                Ok(())
            }
            Key::Char('x') => {
                self.rpn.swap_xy();
                Ok(())
            }
            Key::Char('r') | Key::Down => {
                self.rpn.roll_down();
                Ok(())
            }
            Key::Char('u') | Key::Up => {
                self.rpn.roll_up();
                Ok(())
            }
            Key::Char('l') => {
                self.rpn.recall_last_x();
                Ok(())
            }
            Key::Char('!') => self.rpn.apply_unary(Func::Factorial, self.angle),
            Key::Char(c) => match Op::from_char(c) {
                Some(op) => self.rpn.apply_binary(op),
                None => Ok(()),
            },
            _ => Ok(()),
        };
        self.show(r);
    }

    fn show(&mut self, r: Result<(), CalcError>) {
        if let Err(e) = r {
            self.error = Some(e.message());
        }
    }

    fn menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::Func(f) => match self.mode {
                Mode::Algebraic => self.alg_insert(f.alg_insert()),
                Mode::Rpn => {
                    let r = self.rpn.apply_unary(f, self.angle);
                    self.show(r);
                }
            },
            MenuAction::ChangeSign => match self.mode {
                Mode::Algebraic => {
                    if self.input.is_empty() {
                        self.alg_insert("-");
                    } else {
                        let wrapped = format!("-({})", self.input);
                        if wrapped.chars().count() <= MAX_INPUT {
                            self.input = wrapped;
                        }
                    }
                }
                Mode::Rpn => self.rpn.change_sign(),
            },
            MenuAction::Store => self.overlay = Overlay::Store,
            MenuAction::Recall => self.overlay = Overlay::Recall,
            MenuAction::ClearMemory => self.memory = [0.0; 10],
            MenuAction::ToggleMode => self.toggle_mode(),
            MenuAction::CycleAngle => self.angle = self.angle.cycle(),
            MenuAction::CycleBase => self.base = self.base.cycle(),
            MenuAction::OverwriteSaved => self.overwrite_requested = true,
        }
    }

    fn alg_insert(&mut self, s: &str) {
        self.browse = None;
        if self.input.chars().count() + s.chars().count() <= MAX_INPUT {
            self.input.push_str(s);
        }
    }

    fn alg_evaluate(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        let expr = if parser::continues_ans(&self.input) {
            format!("ans{}", self.input.trim_start())
        } else {
            self.input.clone()
        };
        match parser::calculate(&expr, self.ans, self.angle) {
            Ok(v) => {
                self.ans = v;
                self.history.push(HistoryEntry { expr, result: v });
                if self.history.len() > HISTORY_MAX {
                    self.history.remove(0);
                }
                self.input.clear();
                self.browse = None;
            }
            Err(e) => self.error = Some(e.message()),
        }
    }

    /// Up (-1) / Down (+1) through previous expressions.
    fn browse_history(&mut self, dir: i32) {
        if self.history.is_empty() {
            return;
        }
        let last = self.history.len() - 1;
        let next = match (self.browse, dir < 0) {
            (None, true) => Some(last),
            (None, false) => None,
            (Some(i), true) => Some(i.saturating_sub(1)),
            (Some(i), false) if i < last => Some(i + 1),
            (Some(_), false) => None,
        };
        self.browse = next;
        self.input = match next {
            Some(i) => self.history[i].expr.clone(),
            None => String::new(),
        };
    }

    fn current_value(&mut self) -> f64 {
        match self.mode {
            Mode::Algebraic => self.ans,
            Mode::Rpn => {
                self.rpn.finish_entry();
                self.rpn.x()
            }
        }
    }

    fn store(&mut self, r: usize) {
        let v = self.current_value();
        if let Some(slot) = self.memory.get_mut(r) {
            *slot = v;
        }
    }

    fn recall(&mut self, r: usize) {
        let Some(&v) = self.memory.get(r) else { return };
        match self.mode {
            Mode::Algebraic => {
                let text = format_decimal(v);
                let after_number = self.input.chars().last().is_some_and(|c| c.is_ascii_alphanumeric() || c == '.');
                let insert = if v < 0.0 || after_number { format!("({text})") } else { text };
                self.alg_insert(&insert);
            }
            Mode::Rpn => self.rpn.push_value(v),
        }
    }

    fn toggle_mode(&mut self) {
        match self.mode {
            Mode::Algebraic => {
                self.rpn.finish_entry();
                // Carry ans over only if it isn't already in X, so toggling
                // back and forth doesn't keep pushing the stack.
                if self.rpn.x() != self.ans {
                    self.rpn.push_value(self.ans);
                }
                self.mode = Mode::Rpn;
            }
            Mode::Rpn => {
                self.rpn.finish_entry();
                self.ans = self.rpn.x();
                self.mode = Mode::Algebraic;
            }
        }
    }

    fn clear_all(&mut self) {
        match self.mode {
            Mode::Algebraic => {
                self.input.clear();
                self.ans = 0.0;
                self.browse = None;
            }
            Mode::Rpn => self.rpn.clear_all(),
        }
    }
}

/// Characters that may be typed into an algebraic expression. Upper-case
/// letters are commands; arrows and other symbols are ignored.
fn is_expression_char(c: char) -> bool {
    c.is_ascii_digit()
        || c.is_ascii_lowercase()
        || matches!(c, '.' | '(' | ')' | '[' | ']' | '!' | 'π')
        || Op::from_char(c).is_some()
}
