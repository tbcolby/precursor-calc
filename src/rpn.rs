//! RPN (Reverse Polish Notation) stack machine

use crate::functions::{AngleMode, CalcError, Func, Op};
use alloc::string::String;

/// Classic 4-level RPN stack (X, Y, Z, T)
pub struct RpnStack {
    /// Stack registers (index 0 = X/bottom, 3 = T/top)
    stack: [f64; 4],
    /// Last X value for recall
    last_x: f64,
    /// Currently entering a number
    entering: bool,
    /// Entry buffer for number being typed
    entry_buffer: String,
    /// Entry started (for push behavior)
    entry_started: bool,
}

impl Default for RpnStack {
    fn default() -> Self {
        Self::new()
    }
}

impl RpnStack {
    pub fn new() -> Self {
        Self {
            stack: [0.0; 4],
            last_x: 0.0,
            entering: false,
            entry_buffer: String::new(),
            entry_started: false,
        }
    }

    /// Get X register (bottom of stack)
    pub fn x(&self) -> f64 {
        self.stack[0]
    }

    /// Get Y register
    pub fn y(&self) -> f64 {
        self.stack[1]
    }

    /// Get Z register
    pub fn z(&self) -> f64 {
        self.stack[2]
    }

    /// Get T register (top of stack)
    pub fn t(&self) -> f64 {
        self.stack[3]
    }

    /// Get last X value
    pub fn last_x(&self) -> f64 {
        self.last_x
    }

    /// Is currently entering a number?
    pub fn is_entering(&self) -> bool {
        self.entering
    }

    /// Get entry buffer
    pub fn entry_buffer(&self) -> &str {
        &self.entry_buffer
    }

    /// Push value onto stack (lift stack)
    pub fn push(&mut self, value: f64) {
        // T is lost, others shift up
        self.stack[3] = self.stack[2];
        self.stack[2] = self.stack[1];
        self.stack[1] = self.stack[0];
        self.stack[0] = value;
        self.entering = false;
        self.entry_buffer.clear();
    }

    /// Pop value from stack (drop stack)
    pub fn pop(&mut self) -> f64 {
        let value = self.stack[0];
        // Others shift down, T duplicates
        self.stack[0] = self.stack[1];
        self.stack[1] = self.stack[2];
        self.stack[2] = self.stack[3];
        // T stays the same (classic HP behavior)
        value
    }

    /// Set X register directly (no stack lift)
    pub fn set_x(&mut self, value: f64) {
        self.stack[0] = value;
        self.entering = false;
        self.entry_buffer.clear();
    }

    /// Swap X and Y
    pub fn swap_xy(&mut self) {
        self.finish_entry();
        self.stack.swap(0, 1);
    }

    /// Roll stack down: T→Z→Y→X→T
    pub fn roll_down(&mut self) {
        self.finish_entry();
        let x = self.stack[0];
        self.stack[0] = self.stack[1];
        self.stack[1] = self.stack[2];
        self.stack[2] = self.stack[3];
        self.stack[3] = x;
    }

    /// Roll stack up: X→Y→Z→T→X
    pub fn roll_up(&mut self) {
        self.finish_entry();
        let t = self.stack[3];
        self.stack[3] = self.stack[2];
        self.stack[2] = self.stack[1];
        self.stack[1] = self.stack[0];
        self.stack[0] = t;
    }

    /// Clear X register
    pub fn clear_x(&mut self) {
        self.stack[0] = 0.0;
        self.entering = false;
        self.entry_buffer.clear();
        self.entry_started = false;
    }

    /// Clear all registers
    pub fn clear_all(&mut self) {
        self.stack = [0.0; 4];
        self.last_x = 0.0;
        self.entering = false;
        self.entry_buffer.clear();
        self.entry_started = false;
    }

    /// Enter key pressed - push or duplicate
    pub fn enter(&mut self) {
        if self.entering {
            // Finish entry and push (enables stack lift for next entry)
            self.finish_entry();
            // Push X to enable stack lift on next digit
            self.push(self.stack[0]);
            self.stack[0] = self.stack[1]; // Undo the double-push
        } else {
            // Duplicate X
            self.push(self.stack[0]);
        }
        self.entry_started = false;
    }

    /// Start entering a new number
    fn start_entry(&mut self) {
        if !self.entry_started {
            // Lift stack for new entry
            if !self.entering {
                self.stack[3] = self.stack[2];
                self.stack[2] = self.stack[1];
                self.stack[1] = self.stack[0];
            }
            self.entry_started = true;
        }
        self.entering = true;
    }

    /// Add digit to entry buffer
    pub fn digit(&mut self, c: char) {
        self.start_entry();
        self.entry_buffer.push(c);
        // Update X register with current entry
        if let Ok(value) = self.entry_buffer.parse::<f64>() {
            self.stack[0] = value;
        }
    }

    /// Add decimal point
    pub fn decimal_point(&mut self) {
        if !self.entry_buffer.contains('.') {
            self.start_entry();
            if self.entry_buffer.is_empty() {
                self.entry_buffer.push('0');
            }
            self.entry_buffer.push('.');
        }
    }

    /// Toggle sign
    pub fn change_sign(&mut self) {
        if self.entering {
            if self.entry_buffer.starts_with('-') {
                self.entry_buffer.remove(0);
            } else {
                self.entry_buffer.insert(0, '-');
            }
            if let Ok(value) = self.entry_buffer.parse::<f64>() {
                self.stack[0] = value;
            }
        } else {
            self.stack[0] = -self.stack[0];
        }
    }

    /// Backspace in entry
    pub fn backspace(&mut self) {
        if self.entering && !self.entry_buffer.is_empty() {
            self.entry_buffer.pop();
            if self.entry_buffer.is_empty() || self.entry_buffer == "-" {
                self.stack[0] = 0.0;
            } else if let Ok(value) = self.entry_buffer.parse::<f64>() {
                self.stack[0] = value;
            }
        }
    }

    /// Finish entry mode
    fn finish_entry(&mut self) {
        if self.entering {
            if let Ok(value) = self.entry_buffer.parse::<f64>() {
                self.stack[0] = value;
            }
            self.entering = false;
            self.entry_buffer.clear();
        }
    }

    /// Apply unary function to X
    pub fn apply_unary(&mut self, func: Func, angle_mode: AngleMode) -> Result<(), CalcError> {
        self.finish_entry();
        self.last_x = self.stack[0];
        let result = func.evaluate(self.stack[0], angle_mode)?;
        self.stack[0] = result;
        self.entry_started = false;
        Ok(())
    }

    /// Apply binary operator: Y op X → X
    pub fn apply_binary(&mut self, op: Op) -> Result<(), CalcError> {
        self.finish_entry();
        self.last_x = self.stack[0];
        let x = self.pop();
        let y = self.stack[0];
        let result = op.evaluate(y, x)?;
        self.stack[0] = result;
        self.entry_started = false;
        Ok(())
    }

    /// Recall last X
    pub fn recall_last_x(&mut self) {
        self.push(self.last_x);
    }

    /// Get all stack values for display [X, Y, Z, T]
    pub fn get_stack(&self) -> [f64; 4] {
        self.stack
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut stack = RpnStack::new();

        // 2 Enter 3 + = 5
        stack.digit('2');
        stack.enter();
        stack.digit('3');
        stack.apply_binary(Op::Add).unwrap();

        assert_eq!(stack.x(), 5.0);
    }

    #[test]
    fn test_stack_manipulation() {
        let mut stack = RpnStack::new();

        stack.push(1.0);
        stack.push(2.0);
        stack.push(3.0);
        stack.push(4.0);

        assert_eq!(stack.x(), 4.0);
        assert_eq!(stack.y(), 3.0);
        assert_eq!(stack.z(), 2.0);
        assert_eq!(stack.t(), 1.0);

        stack.swap_xy();
        assert_eq!(stack.x(), 3.0);
        assert_eq!(stack.y(), 4.0);
    }

    #[test]
    fn test_change_sign() {
        let mut stack = RpnStack::new();
        stack.digit('5');
        stack.change_sign();
        assert_eq!(stack.x(), -5.0);
        stack.change_sign();
        assert_eq!(stack.x(), 5.0);
    }
}
