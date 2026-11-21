//! Four-level RPN stack with HP-style stack lift.
//!
//! The rule that the original implementation missed: a `lift` flag decides
//! whether the next number typed (or recalled) pushes the stack or overwrites X.
//!
//! * ENTER copies X into Y and **disables** lift, so the next number replaces
//!   the copy in X: `1 ENTER 2 ENTER 3 + +` is 6.
//! * CLx (and Backspace that empties an entry) disables lift.
//! * Operations, functions, swaps, rolls and recalls **enable** lift.
//! * A failed operation leaves the stack untouched.

use alloc::string::String;

use crate::functions::{AngleMode, CalcError, Func, Op};

/// Longest number entry, in characters.
pub const MAX_ENTRY: usize = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct RpnStack {
    /// `[X, Y, Z, T]`.
    stack: [f64; 4],
    last_x: f64,
    /// Number being typed, if any. X mirrors its parsed value.
    entry: Option<String>,
    lift: bool,
}

impl Default for RpnStack {
    fn default() -> Self { Self::new() }
}

impl RpnStack {
    pub fn new() -> Self { Self { stack: [0.0; 4], last_x: 0.0, entry: None, lift: true } }

    /// Rebuild from saved registers (non-finite values become 0).
    pub fn from_registers(stack: [f64; 4], last_x: f64) -> Self {
        let clean = |v: f64| if v.is_finite() { v } else { 0.0 };
        Self { stack: stack.map(clean), last_x: clean(last_x), entry: None, lift: true }
    }

    pub fn x(&self) -> f64 { self.stack[0] }

    pub fn y(&self) -> f64 { self.stack[1] }

    pub fn stack(&self) -> [f64; 4] { self.stack }

    pub fn last_x(&self) -> f64 { self.last_x }

    pub fn lift_enabled(&self) -> bool { self.lift }

    /// The number being typed, if any.
    pub fn entry(&self) -> Option<&str> { self.entry.as_deref() }

    fn raise(&mut self) {
        self.stack[3] = self.stack[2];
        self.stack[2] = self.stack[1];
        self.stack[1] = self.stack[0];
    }

    /// Drop after a binary operation: Y←Z, Z←T, T stays.
    fn drop_stack(&mut self) {
        self.stack[1] = self.stack[2];
        self.stack[2] = self.stack[3];
    }

    /// Put a value in X the way a recall does: lift if enabled, then enable lift.
    pub fn push_value(&mut self, v: f64) {
        self.finish_entry();
        if self.lift {
            self.raise();
        }
        self.stack[0] = v;
        self.lift = true;
    }

    fn begin_entry(&mut self) -> &mut String {
        if self.entry.is_none() {
            if self.lift {
                self.raise();
            }
            self.lift = true;
            self.entry = Some(String::new());
        }
        self.entry.as_mut().expect("entry just created")
    }

    fn sync_x(&mut self) {
        if let Some(e) = &self.entry {
            self.stack[0] = parse_entry(e);
        }
    }

    pub fn digit(&mut self, c: char) {
        debug_assert!(c.is_ascii_digit());
        let e = self.begin_entry();
        if e.chars().count() < MAX_ENTRY {
            e.push(c);
        }
        self.sync_x();
    }

    pub fn decimal_point(&mut self) {
        let e = self.begin_entry();
        if !e.contains('.') && !e.contains('e') && e.chars().count() < MAX_ENTRY {
            if e.is_empty() || e == "-" {
                e.push('0');
            }
            e.push('.');
        }
        self.sync_x();
    }

    /// EEX: start the exponent of the number being typed.
    pub fn exponent(&mut self) {
        let e = self.begin_entry();
        if !e.contains('e') && e.chars().count() + 2 < MAX_ENTRY {
            if e.is_empty() || e == "-" {
                e.push('1');
            }
            e.push('e');
        }
        self.sync_x();
    }

    /// CHS: negate the mantissa (or the exponent once one is being typed), or X.
    pub fn change_sign(&mut self) {
        match &mut self.entry {
            Some(e) => {
                if let Some(pos) = e.find('e') {
                    if e[pos + 1..].starts_with('-') {
                        e.remove(pos + 1);
                    } else {
                        e.insert(pos + 1, '-');
                    }
                } else if e.starts_with('-') {
                    e.remove(0);
                } else {
                    e.insert(0, '-');
                }
                self.sync_x();
            }
            None => {
                if self.stack[0] != 0.0 {
                    self.stack[0] = -self.stack[0];
                }
            }
        }
    }

    /// Delete the last typed character. Outside an entry it acts as CLx.
    pub fn backspace(&mut self) {
        match &mut self.entry {
            Some(e) => {
                e.pop();
                if e.is_empty() || e == "-" {
                    self.entry = None;
                    self.stack[0] = 0.0;
                    self.lift = false;
                } else {
                    self.sync_x();
                }
            }
            None => self.clear_x(),
        }
    }

    /// End number entry; X keeps the typed value.
    pub fn finish_entry(&mut self) {
        if self.entry.take().is_some() {
            self.lift = true;
        }
    }

    /// ENTER: copy X to Y and disable lift.
    pub fn enter(&mut self) {
        self.finish_entry();
        self.raise();
        self.lift = false;
    }

    /// CLx: zero X and disable lift.
    pub fn clear_x(&mut self) {
        self.entry = None;
        self.stack[0] = 0.0;
        self.lift = false;
    }

    pub fn clear_all(&mut self) { *self = Self::new(); }

    pub fn swap_xy(&mut self) {
        self.finish_entry();
        self.stack.swap(0, 1);
        self.lift = true;
    }

    /// R↓: X←Y←Z←T←X.
    pub fn roll_down(&mut self) {
        self.finish_entry();
        self.stack.rotate_left(1);
        self.lift = true;
    }

    /// R↑: X→Y→Z→T→X.
    pub fn roll_up(&mut self) {
        self.finish_entry();
        self.stack.rotate_right(1);
        self.lift = true;
    }

    pub fn recall_last_x(&mut self) { self.push_value(self.last_x); }

    /// `f(X)`. On error the stack is unchanged.
    pub fn apply_unary(&mut self, f: Func, angle: AngleMode) -> Result<(), CalcError> {
        self.finish_entry();
        if f.is_constant() {
            self.push_value(f.evaluate(0.0, angle)?);
            return Ok(());
        }
        let r = f.evaluate(self.stack[0], angle)?;
        self.last_x = self.stack[0];
        self.stack[0] = r;
        self.lift = true;
        Ok(())
    }

    /// `Y op X`, then drop. On error the stack is unchanged.
    pub fn apply_binary(&mut self, op: Op) -> Result<(), CalcError> {
        self.finish_entry();
        let r = op.evaluate(self.stack[1], self.stack[0])?;
        self.last_x = self.stack[0];
        self.stack[0] = r;
        self.drop_stack();
        self.lift = true;
        Ok(())
    }
}

/// Value of a partial entry: tolerates a trailing `.`, `e` or `e-`.
fn parse_entry(e: &str) -> f64 {
    let t = e.trim_end_matches(['e', '-']);
    let t = if t.is_empty() || t == "-" { "0" } else { t };
    t.parse::<f64>().ok().filter(|v| v.is_finite()).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(s: &mut RpnStack, n: &str) {
        for c in n.chars() {
            match c {
                '.' => s.decimal_point(),
                'e' => s.exponent(),
                _ => s.digit(c),
            }
        }
    }

    #[test]
    fn chained_enter_disables_lift() {
        // Audit: 1 ENTER 2 ENTER 3 + + gave 7 with stack [7,1,1,1].
        let mut s = RpnStack::new();
        typed(&mut s, "1");
        s.enter();
        typed(&mut s, "2");
        s.enter();
        typed(&mut s, "3");
        assert_eq!(s.stack(), [3.0, 2.0, 1.0, 0.0]);
        s.apply_binary(Op::Add).unwrap();
        assert_eq!(s.stack(), [5.0, 1.0, 0.0, 0.0]);
        s.apply_binary(Op::Add).unwrap();
        assert_eq!(s.stack(), [6.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn two_enter_three_times_leaves_no_stray() {
        // Audit: 2 ENTER 3 * left a stray 2 in Y.
        let mut s = RpnStack::new();
        typed(&mut s, "2");
        s.enter();
        typed(&mut s, "3");
        s.apply_binary(Op::Mul).unwrap();
        assert_eq!(s.stack(), [6.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn clx_disables_lift() {
        // Audit: 7 ENTER 9 CLx 4 gave [4,0,7,7].
        let mut s = RpnStack::new();
        typed(&mut s, "7");
        s.enter();
        typed(&mut s, "9");
        s.clear_x();
        typed(&mut s, "4");
        assert_eq!(s.stack(), [4.0, 7.0, 0.0, 0.0]);
    }

    #[test]
    fn enter_duplicates_and_operation_uses_copy() {
        // 5 ENTER * squares.
        let mut s = RpnStack::new();
        typed(&mut s, "5");
        s.enter();
        assert_eq!(s.stack(), [5.0, 5.0, 0.0, 0.0]);
        assert!(!s.lift_enabled());
        s.apply_binary(Op::Mul).unwrap();
        assert_eq!(s.x(), 25.0);
    }

    #[test]
    fn number_after_operation_lifts() {
        let mut s = RpnStack::new();
        typed(&mut s, "2");
        s.enter();
        typed(&mut s, "3");
        s.apply_binary(Op::Add).unwrap(); // 5
        typed(&mut s, "4");
        assert_eq!(s.stack(), [4.0, 5.0, 0.0, 0.0]);
        s.apply_binary(Op::Mul).unwrap();
        assert_eq!(s.x(), 20.0);
    }

    #[test]
    fn classic_hp_example() {
        // (3 + 4) * (5 + 6) = 77
        let mut s = RpnStack::new();
        typed(&mut s, "3");
        s.enter();
        typed(&mut s, "4");
        s.apply_binary(Op::Add).unwrap();
        typed(&mut s, "5");
        s.enter();
        typed(&mut s, "6");
        s.apply_binary(Op::Add).unwrap();
        s.apply_binary(Op::Mul).unwrap();
        assert_eq!(s.stack(), [77.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn t_register_replicates_on_drop() {
        let mut s = RpnStack::new();
        for n in ["1", "2", "3", "4"] {
            typed(&mut s, n);
            s.enter();
        }
        // [4,4,3,2] after the last ENTER; type 5 overwrites X.
        typed(&mut s, "5");
        assert_eq!(s.stack(), [5.0, 4.0, 3.0, 2.0]);
        s.apply_binary(Op::Add).unwrap();
        assert_eq!(s.stack(), [9.0, 3.0, 2.0, 2.0]);
    }

    #[test]
    fn failed_operation_leaves_stack() {
        let mut s = RpnStack::new();
        typed(&mut s, "1");
        s.enter();
        typed(&mut s, "0");
        let before = s.stack();
        assert_eq!(s.apply_binary(Op::Div), Err(CalcError::DivideByZero));
        assert_eq!(s.stack(), before);
        assert!(s.apply_unary(Func::Sqrt, AngleMode::Degrees).is_ok());
        s.change_sign();
        s.push_value(-4.0);
        assert!(s.apply_unary(Func::Sqrt, AngleMode::Degrees).is_err());
        assert_eq!(s.x(), -4.0);
    }

    #[test]
    fn unary_sets_last_x_and_enables_lift() {
        let mut s = RpnStack::new();
        typed(&mut s, "9");
        s.apply_unary(Func::Sqrt, AngleMode::Degrees).unwrap();
        assert_eq!((s.x(), s.last_x()), (3.0, 9.0));
        typed(&mut s, "2");
        assert_eq!(s.stack(), [2.0, 3.0, 0.0, 0.0]);
        s.recall_last_x();
        assert_eq!(s.stack(), [9.0, 2.0, 3.0, 0.0]);
    }

    #[test]
    fn constants_push_like_recall() {
        let mut s = RpnStack::new();
        typed(&mut s, "2");
        s.apply_unary(Func::Pi, AngleMode::Degrees).unwrap();
        assert_eq!(s.y(), 2.0);
        s.apply_binary(Op::Mul).unwrap();
        assert!((s.x() - 2.0 * core::f64::consts::PI).abs() < 1e-12);
        // After ENTER a constant overwrites the duplicate.
        let mut s = RpnStack::new();
        typed(&mut s, "3");
        s.enter();
        s.apply_unary(Func::Pi, AngleMode::Degrees).unwrap();
        assert_eq!(s.y(), 3.0);
        assert_eq!(s.stack()[2], 0.0);
    }

    #[test]
    fn swap_and_rolls() {
        let mut s = RpnStack::from_registers([1.0, 2.0, 3.0, 4.0], 0.0);
        s.swap_xy();
        assert_eq!(s.stack(), [2.0, 1.0, 3.0, 4.0]);
        s.roll_down();
        assert_eq!(s.stack(), [1.0, 3.0, 4.0, 2.0]);
        s.roll_up();
        assert_eq!(s.stack(), [2.0, 1.0, 3.0, 4.0]);
    }

    #[test]
    fn change_sign_and_exponent_entry() {
        let mut s = RpnStack::new();
        typed(&mut s, "5");
        s.change_sign();
        assert_eq!(s.x(), -5.0);
        s.change_sign();
        assert_eq!(s.x(), 5.0);
        typed(&mut s, "e");
        assert_eq!(s.x(), 5.0);
        typed(&mut s, "3");
        assert_eq!(s.x(), 5000.0);
        s.change_sign();
        assert_eq!(s.entry(), Some("5e-3"));
        assert_eq!(s.x(), 0.005);
        // EEX on an empty entry means 1eN
        let mut s = RpnStack::new();
        typed(&mut s, "e6");
        assert_eq!(s.x(), 1e6);
        // CHS outside entry negates X without touching lift
        s.enter();
        s.change_sign();
        assert_eq!(s.stack()[..2], [-1e6, 1e6]);
        assert!(!s.lift_enabled());
    }

    #[test]
    fn backspace_edits_then_clears() {
        let mut s = RpnStack::new();
        typed(&mut s, "7");
        s.enter();
        typed(&mut s, "12");
        s.backspace();
        assert_eq!(s.x(), 1.0);
        s.backspace();
        assert_eq!(s.entry(), None);
        assert_eq!(s.x(), 0.0);
        typed(&mut s, "4");
        assert_eq!(s.stack(), [4.0, 7.0, 0.0, 0.0], "emptied entry must not lift");
        // outside an entry, backspace is CLx
        s.finish_entry();
        s.backspace();
        assert_eq!(s.x(), 0.0);
    }

    #[test]
    fn decimal_entry() {
        let mut s = RpnStack::new();
        typed(&mut s, ".5");
        assert_eq!(s.entry(), Some("0.5"));
        typed(&mut s, ".");
        assert_eq!(s.entry(), Some("0.5"), "second point ignored");
        assert_eq!(s.x(), 0.5);
    }

    #[test]
    fn entry_length_is_bounded() {
        let mut s = RpnStack::new();
        for _ in 0..100 {
            s.digit('9');
        }
        assert_eq!(s.entry().unwrap().len(), MAX_ENTRY);
        assert!(s.x().is_finite());
    }

    #[test]
    fn from_registers_sanitises() {
        let s = RpnStack::from_registers([f64::NAN, 1.0, f64::INFINITY, 2.0], f64::NEG_INFINITY);
        assert_eq!(s.stack(), [0.0, 1.0, 0.0, 2.0]);
        assert_eq!(s.last_x(), 0.0);
    }
}
