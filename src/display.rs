//! Number formatting and display utilities

use crate::functions::NumberBase;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

/// Format a number for display
pub fn format_number(value: f64, base: NumberBase) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return if value > 0.0 {
            String::from("∞")
        } else {
            String::from("-∞")
        };
    }

    match base {
        NumberBase::Decimal => format_decimal(value),
        NumberBase::Hexadecimal => format_hex(value),
        NumberBase::Octal => format_octal(value),
        NumberBase::Binary => format_binary(value),
    }
}

/// Format in decimal with smart scientific notation
fn format_decimal(value: f64) -> String {
    if value == 0.0 {
        return String::from("0");
    }

    let abs = value.abs();

    // Use scientific notation for very large or very small numbers
    if abs >= 1e10 || (abs != 0.0 && abs < 1e-4) {
        format_scientific(value)
    } else {
        format_fixed(value)
    }
}

/// Format in fixed-point notation, trimming trailing zeros
fn format_fixed(value: f64) -> String {
    let mut buf = String::new();
    write!(buf, "{:.10}", value).ok();
    trim_trailing_zeros(&mut buf);
    buf
}

/// Format in scientific notation
fn format_scientific(value: f64) -> String {
    let mut buf = String::new();
    write!(buf, "{:.7e}", value).ok();
    // Clean up exponent: e+07 -> e7, e-03 -> e-3
    buf = buf.replace("e+", "e").replace("e0", "e").replace("e-0", "e-");
    buf
}

/// Trim trailing zeros and unnecessary decimal point
fn trim_trailing_zeros(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// Format as hexadecimal (integer only)
fn format_hex(value: f64) -> String {
    let int_val = value as i64;
    let mut buf = String::new();
    if int_val < 0 {
        write!(buf, "-0x{:X}", -int_val).ok();
    } else {
        write!(buf, "0x{:X}", int_val).ok();
    }
    buf
}

/// Format as octal (integer only)
fn format_octal(value: f64) -> String {
    let int_val = value as i64;
    let mut buf = String::new();
    if int_val < 0 {
        write!(buf, "-0o{:o}", -int_val).ok();
    } else {
        write!(buf, "0o{:o}", int_val).ok();
    }
    buf
}

/// Format as binary (integer only)
fn format_binary(value: f64) -> String {
    let int_val = value as i64;
    let abs = int_val.abs();

    // Limit binary display length
    if abs > 0xFFFF {
        let mut buf = String::new();
        if int_val < 0 {
            write!(buf, "-0b{:b}", abs).ok();
        } else {
            write!(buf, "0b{:b}", abs).ok();
        }
        // Truncate if too long
        if buf.len() > 24 {
            buf.truncate(21);
            buf.push_str("...");
        }
        buf
    } else {
        let mut buf = String::new();
        if int_val < 0 {
            write!(buf, "-0b{:b}", abs).ok();
        } else {
            write!(buf, "0b{:b}", abs).ok();
        }
        buf
    }
}

/// Format for stack display (shorter, right-aligned)
pub fn format_stack_number(value: f64, base: NumberBase) -> String {
    let formatted = format_number(value, base);
    // Limit to reasonable display width
    if formatted.len() > 20 {
        let mut s = formatted;
        s.truncate(17);
        s.push_str("...");
        s
    } else {
        formatted
    }
}

/// Format entry buffer with cursor
pub fn format_entry(buffer: &str, has_cursor: bool) -> String {
    let mut s = String::from(buffer);
    if has_cursor {
        s.push('_');
    }
    s
}

/// Parse a number from input string, handling different bases
pub fn parse_number(input: &str) -> Option<f64> {
    let input = input.trim();

    if input.is_empty() {
        return None;
    }

    // Check for hex prefix
    if input.starts_with("0x") || input.starts_with("0X") {
        return i64::from_str_radix(&input[2..], 16).ok().map(|n| n as f64);
    }

    // Check for octal prefix
    if input.starts_with("0o") || input.starts_with("0O") {
        return i64::from_str_radix(&input[2..], 8).ok().map(|n| n as f64);
    }

    // Check for binary prefix
    if input.starts_with("0b") || input.starts_with("0B") {
        return i64::from_str_radix(&input[2..], 2).ok().map(|n| n as f64);
    }

    // Standard decimal parse
    input.parse::<f64>().ok()
}

/// History entry display
#[derive(Clone)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: f64,
}

impl HistoryEntry {
    pub fn new(expression: String, result: f64) -> Self {
        Self { expression, result }
    }

    pub fn format(&self, base: NumberBase) -> String {
        let mut buf = String::new();
        write!(buf, "{} = {}", self.expression, format_number(self.result, base)).ok();
        buf
    }
}

/// History tape manager
pub struct History {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl History {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn last_n(&self, n: usize) -> &[HistoryEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_decimal() {
        assert_eq!(format_decimal(0.0), "0");
        assert_eq!(format_decimal(42.0), "42");
        assert_eq!(format_decimal(3.14159), "3.14159");
        assert_eq!(format_decimal(-123.456), "-123.456");
    }

    #[test]
    fn test_format_scientific() {
        let s = format_number(1.23e15, NumberBase::Decimal);
        assert!(s.contains('e'));
    }

    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex(255.0), "0xFF");
        assert_eq!(format_hex(16.0), "0x10");
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Some(42.0));
        assert_eq!(parse_number("0xFF"), Some(255.0));
        assert_eq!(parse_number("0b1010"), Some(10.0));
        assert_eq!(parse_number("0o17"), Some(15.0));
    }
}
