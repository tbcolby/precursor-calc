//! CalcApp - main application state and mode dispatch

use crate::algebraic::AlgebraicState;
use crate::display::{format_number, format_stack_number, History, HistoryEntry};
use crate::functions::{AngleMode, Func, NumberBase, Op};
use crate::keymap::{get_menu_items, KeyAction, KeyState};
use crate::memory::Memory;
use crate::rpn::RpnStack;
use crate::storage::{Settings, Storage};
use crate::ui;

use alloc::string::String;
use alloc::vec::Vec;
use gam::Gam;

/// Calculator operating mode
#[derive(Clone, Copy, PartialEq, Default)]
pub enum CalcMode {
    #[default]
    Algebraic,
    Rpn,
}

/// Calculator state machine
#[derive(Clone, Copy, PartialEq)]
pub enum CalcState {
    Normal,
    FnMenu(u8),
    WaitingStore,
    WaitingRecall,
}

/// Main calculator application
pub struct CalcApp {
    // Mode and settings
    mode: CalcMode,
    angle_mode: AngleMode,
    number_base: NumberBase,

    // State
    state: CalcState,
    key_state: KeyState,

    // Mode-specific state
    algebraic: AlgebraicState,
    rpn: RpnStack,

    // Shared
    memory: Memory,
    history: History,
    error: Option<String>,

    // Storage
    storage: Storage,
}

impl CalcApp {
    pub fn new() -> Self {
        let storage = Storage::new();
        let settings = storage.load();

        let mode = if settings.is_rpn() {
            CalcMode::Rpn
        } else {
            CalcMode::Algebraic
        };

        let mut memory = Memory::new();
        memory.set_all(settings.memory);

        let mut algebraic = AlgebraicState::new();
        algebraic.set_ans(settings.ans);

        Self {
            mode,
            angle_mode: settings.get_angle_mode(),
            number_base: settings.get_number_base(),
            state: CalcState::Normal,
            key_state: KeyState::new(),
            algebraic,
            rpn: RpnStack::new(),
            memory,
            history: History::new(50),
            error: None,
            storage,
        }
    }

    /// Save current state to PDDB
    pub fn save_state(&self) {
        let settings = Settings {
            mode: if self.mode == CalcMode::Rpn { 1 } else { 0 },
            angle_mode: self.angle_mode.to_u8(),
            number_base: self.number_base.to_u8(),
            memory: *self.memory.get_all(),
            ans: self.algebraic.ans(),
        };
        self.storage.save(&settings);
    }

    /// Handle a key press
    pub fn handle_key(&mut self, c: char) -> bool {
        // Check for special states
        match self.state {
            CalcState::WaitingStore => {
                if let Some(digit) = c.to_digit(10) {
                    let value = self.current_value();
                    self.memory.store(digit as usize, value);
                    self.state = CalcState::Normal;
                    return true;
                } else {
                    self.state = CalcState::Normal;
                    return true;
                }
            }
            CalcState::WaitingRecall => {
                if let Some(digit) = c.to_digit(10) {
                    if let Some(value) = self.memory.recall(digit as usize) {
                        self.insert_value(value);
                    }
                    self.state = CalcState::Normal;
                    return true;
                } else {
                    self.state = CalcState::Normal;
                    return true;
                }
            }
            CalcState::FnMenu(menu) => {
                if let Some(digit) = c.to_digit(10) {
                    let action = crate::keymap::map_fn_menu_key(menu, digit as u8);
                    self.state = CalcState::Normal;
                    return self.handle_action(action);
                } else if c == '\u{001B}' || c == '∴' {
                    self.state = CalcState::Normal;
                    return true;
                }
                return false;
            }
            CalcState::Normal => {}
        }

        let action = crate::keymap::map_key(c, &mut self.key_state, self.mode == CalcMode::Rpn);
        self.handle_action(action)
    }

    /// Handle a key action
    fn handle_action(&mut self, action: KeyAction) -> bool {
        self.error = None;

        match action {
            KeyAction::Digit(d) => {
                self.input_digit(d);
                true
            }
            KeyAction::Letter(c) => {
                // Letters are for function names in algebraic mode
                if self.mode == CalcMode::Algebraic {
                    self.algebraic.push(c);
                }
                true
            }
            KeyAction::DecimalPoint => {
                self.input_decimal();
                true
            }
            KeyAction::Operator(op) => {
                self.apply_operator(op);
                true
            }
            KeyAction::Function(func) => {
                self.apply_function(func);
                true
            }
            KeyAction::OpenParen => {
                if self.mode == CalcMode::Algebraic {
                    self.algebraic.push('(');
                }
                true
            }
            KeyAction::CloseParen => {
                if self.mode == CalcMode::Algebraic {
                    self.algebraic.push(')');
                }
                true
            }
            KeyAction::Execute => {
                self.execute();
                true
            }
            KeyAction::Backspace => {
                self.backspace();
                true
            }
            KeyAction::ClearEntry => {
                self.clear_entry();
                true
            }
            KeyAction::ClearAll => {
                self.clear_all();
                true
            }
            KeyAction::ChangeSign => {
                self.change_sign();
                true
            }
            KeyAction::Ans => {
                self.insert_ans();
                true
            }
            KeyAction::ToggleMode => {
                self.toggle_mode();
                true
            }
            KeyAction::CycleAngle => {
                self.angle_mode = self.angle_mode.cycle();
                true
            }
            KeyAction::CycleBase => {
                self.number_base = self.number_base.cycle();
                true
            }
            KeyAction::SwapXY => {
                if self.mode == CalcMode::Rpn {
                    self.rpn.swap_xy();
                }
                true
            }
            KeyAction::RollDown => {
                if self.mode == CalcMode::Rpn {
                    self.rpn.roll_down();
                }
                true
            }
            KeyAction::RollUp => {
                if self.mode == CalcMode::Rpn {
                    self.rpn.roll_up();
                }
                true
            }
            KeyAction::LastX => {
                if self.mode == CalcMode::Rpn {
                    self.rpn.recall_last_x();
                }
                true
            }
            KeyAction::Store => {
                self.state = CalcState::WaitingStore;
                true
            }
            KeyAction::Recall => {
                self.state = CalcState::WaitingRecall;
                true
            }
            KeyAction::FnMenu(n) => {
                self.state = CalcState::FnMenu(n);
                true
            }
            KeyAction::Cancel => {
                self.state = CalcState::Normal;
                self.key_state.reset();
                true
            }
            KeyAction::Quit => false, // Signal to quit
            KeyAction::None | KeyAction::MenuSelect(_) => false,
        }
    }

    /// Get current display value
    fn current_value(&self) -> f64 {
        match self.mode {
            CalcMode::Algebraic => self.algebraic.ans(),
            CalcMode::Rpn => self.rpn.x(),
        }
    }

    /// Insert a value (for memory recall, etc)
    fn insert_value(&mut self, value: f64) {
        match self.mode {
            CalcMode::Algebraic => {
                use core::fmt::Write;
                let mut buf = String::new();
                write!(buf, "{}", value).ok();
                self.algebraic.push_str(&buf);
            }
            CalcMode::Rpn => {
                self.rpn.push(value);
            }
        }
    }

    /// Input a digit
    fn input_digit(&mut self, d: char) {
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.push(d);
            }
            CalcMode::Rpn => {
                self.rpn.digit(d);
            }
        }
    }

    /// Input decimal point
    fn input_decimal(&mut self) {
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.push('.');
            }
            CalcMode::Rpn => {
                self.rpn.decimal_point();
            }
        }
    }

    /// Apply binary operator
    fn apply_operator(&mut self, op: Op) {
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.push(op.symbol());
            }
            CalcMode::Rpn => {
                if let Err(e) = self.rpn.apply_binary(op) {
                    self.error = Some(String::from(e.message()));
                }
            }
        }
    }

    /// Apply unary function
    fn apply_function(&mut self, func: Func) {
        match self.mode {
            CalcMode::Algebraic => {
                // Insert function name with open paren
                self.algebraic.push_str(func.name());
                if !func.is_constant() {
                    self.algebraic.push('(');
                }
            }
            CalcMode::Rpn => {
                if let Err(e) = self.rpn.apply_unary(func, self.angle_mode) {
                    self.error = Some(String::from(e.message()));
                }
            }
        }
    }

    /// Execute/Enter
    fn execute(&mut self) {
        match self.mode {
            CalcMode::Algebraic => {
                let expr = self.algebraic.input().to_string();
                if let Some(result) = self.algebraic.evaluate(self.angle_mode) {
                    if !expr.is_empty() {
                        self.history.add(HistoryEntry::new(expr, result));
                    }
                    self.algebraic.clear();
                } else if let Some(err) = self.algebraic.error() {
                    self.error = Some(String::from(err));
                }
            }
            CalcMode::Rpn => {
                self.rpn.enter();
            }
        }
    }

    /// Backspace
    fn backspace(&mut self) {
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.backspace();
            }
            CalcMode::Rpn => {
                self.rpn.backspace();
            }
        }
    }

    /// Clear entry
    fn clear_entry(&mut self) {
        self.error = None;
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.clear();
            }
            CalcMode::Rpn => {
                self.rpn.clear_x();
            }
        }
    }

    /// Clear all
    fn clear_all(&mut self) {
        self.error = None;
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.clear_all();
            }
            CalcMode::Rpn => {
                self.rpn.clear_all();
            }
        }
    }

    /// Change sign
    fn change_sign(&mut self) {
        match self.mode {
            CalcMode::Algebraic => {
                // Insert negation
                let input = self.algebraic.input();
                if input.is_empty() {
                    self.algebraic.push('-');
                } else {
                    self.algebraic.push_str("*(-1)");
                }
            }
            CalcMode::Rpn => {
                self.rpn.change_sign();
            }
        }
    }

    /// Insert Ans
    fn insert_ans(&mut self) {
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.push_str("ans");
            }
            CalcMode::Rpn => {
                // In RPN, treat as LastX
                self.rpn.recall_last_x();
            }
        }
    }

    /// Toggle between algebraic and RPN modes
    fn toggle_mode(&mut self) {
        // Transfer current value between modes
        let value = self.current_value();

        self.mode = match self.mode {
            CalcMode::Algebraic => CalcMode::Rpn,
            CalcMode::Rpn => CalcMode::Algebraic,
        };

        // Set value in new mode
        match self.mode {
            CalcMode::Algebraic => {
                self.algebraic.set_ans(value);
            }
            CalcMode::Rpn => {
                self.rpn.push(value);
            }
        }
    }

    /// Draw the calculator UI
    pub fn draw(&self, gam: &Gam, gid: gam::Gid) {
        ui::clear_screen(gam, gid);

        // Status bar
        let mode_label = match self.mode {
            CalcMode::Algebraic => "ALG",
            CalcMode::Rpn => "RPN",
        };
        ui::draw_status_bar(
            gam,
            gid,
            mode_label,
            self.angle_mode.label(),
            self.number_base.label(),
            self.memory.has_stored_value(),
        );

        // Main display based on mode
        match self.mode {
            CalcMode::Algebraic => {
                let result = format_number(self.algebraic.ans(), self.number_base);
                ui::draw_algebraic_display(
                    gam,
                    gid,
                    self.algebraic.input(),
                    &result,
                    self.error.as_deref().or(self.algebraic.error()),
                );
            }
            CalcMode::Rpn => {
                let stack = self.rpn.get_stack();
                let stack_strs: [String; 4] = [
                    format_stack_number(stack[0], self.number_base),
                    format_stack_number(stack[1], self.number_base),
                    format_stack_number(stack[2], self.number_base),
                    format_stack_number(stack[3], self.number_base),
                ];
                let last_x = format_stack_number(self.rpn.last_x(), self.number_base);

                let entry = if self.rpn.is_entering() {
                    self.rpn.entry_buffer()
                } else {
                    &stack_strs[0]
                };

                ui::draw_rpn_display(
                    gam,
                    gid,
                    [&stack_strs[0], &stack_strs[1], &stack_strs[2], &stack_strs[3]],
                    entry,
                    self.rpn.is_entering(),
                    &last_x,
                    self.error.as_deref(),
                );
            }
        }

        // History
        let history_entries: Vec<String> = self
            .history
            .last_n(10)
            .iter()
            .map(|e| e.format(self.number_base))
            .collect();
        let history_refs: Vec<&str> = history_entries.iter().map(|s| s.as_str()).collect();
        ui::draw_history(gam, gid, &history_refs);

        // Menu bar
        ui::draw_menu_bar(gam, gid);

        // Function menu overlay if active
        if let CalcState::FnMenu(menu) = self.state {
            let title = match menu {
                1 => "MATH Menu",
                2 => "TRIG Menu",
                3 => "MODE Menu",
                4 => "MEM Menu",
                _ => "Menu",
            };
            let items = get_menu_items(menu);
            ui::draw_fn_menu(gam, gid, title, items);
        }

        // Store/Recall prompt
        match self.state {
            CalcState::WaitingStore => {
                ui::draw_fn_menu(gam, gid, "Store to M#", &[("0-9", "Select register")]);
            }
            CalcState::WaitingRecall => {
                ui::draw_fn_menu(gam, gid, "Recall M#", &[("0-9", "Select register")]);
            }
            _ => {}
        }

        gam.redraw().ok();
    }
}

extern crate alloc;

// Helper function needed by keymap
pub fn map_fn_menu_key(menu: u8, key: u8) -> KeyAction {
    crate::keymap::map_fn_menu_key(menu, key)
}
