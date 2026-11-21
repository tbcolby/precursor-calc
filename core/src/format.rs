//! Number formatting for the display.

use alloc::format;
use alloc::string::String;

use crate::functions::NumberBase;

/// Significant digits shown in fixed and scientific notation. An f64 holds
/// 15-17; showing 12 hides representation noise such as `12345678.9000000004`.
pub const SIG_DIGITS: i32 = 12;

/// Values at or above this magnitude switch to scientific notation.
const SCI_ABOVE: f64 = 1e12;
/// Values below this magnitude (other than 0) switch to scientific notation.
const SCI_BELOW: f64 = 1e-4;

/// Format a value for display in `base`.
pub fn format_number(value: f64, base: NumberBase) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "inf" } else { "-inf" });
    }
    match base {
        NumberBase::Decimal => format_decimal(value),
        NumberBase::Hexadecimal => format_radix(value, 16),
        NumberBase::Octal => format_radix(value, 8),
        NumberBase::Binary => format_radix(value, 2),
    }
}

/// Decimal with up to [`SIG_DIGITS`] significant digits; the output always
/// parses back with the expression parser.
pub fn format_decimal(value: f64) -> String {
    if value == 0.0 {
        return String::from("0");
    }
    let abs = value.abs();
    if !(SCI_BELOW..SCI_ABOVE).contains(&abs) {
        return format_scientific(value);
    }
    let int_digits = abs.log10().floor() as i32 + 1;
    let decimals = (SIG_DIGITS - int_digits).clamp(0, 16) as usize;
    let mut s = format!("{value:.decimals$}");
    trim_fraction(&mut s);
    if s == "-0" {
        s = String::from("0");
    }
    s
}

/// Scientific notation with a trimmed mantissa: `1.23e15`, `-5e-7`.
pub fn format_scientific(value: f64) -> String {
    let s = format!("{:.*e}", (SIG_DIGITS - 1) as usize, value);
    match s.split_once('e') {
        Some((mantissa, exp)) => {
            let mut m = String::from(mantissa);
            trim_fraction(&mut m);
            format!("{m}e{exp}")
        }
        None => s,
    }
}

fn trim_fraction(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// Integer display in base 16/8/2. Values that aren't integers are shown
/// truncated with a leading `~`; values outside i64 show a range message.
fn format_radix(value: f64, radix: u32) -> String {
    // i64 covers [-2^63, 2^63); 2^63 itself is exactly representable as f64.
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    let t = value.trunc();
    if !(-LIMIT..LIMIT).contains(&t) {
        return String::from("out of range");
    }
    let n = t as i64;
    let mag = n.unsigned_abs();
    let digits = match radix {
        16 => format!("0x{mag:X}"),
        8 => format!("0o{mag:o}"),
        _ => format!("0b{mag:b}"),
    };
    let mut s = String::new();
    if t != value {
        s.push('~');
    }
    if n < 0 {
        s.push('-');
    }
    s.push_str(&digits);
    s
}

/// Shorten to at most `max` characters (char-safe), marking the cut with `…`.
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return String::from(s);
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::calculate;
    use crate::functions::AngleMode;

    fn dec(v: f64) -> String { format_number(v, NumberBase::Decimal) }

    #[test]
    fn basic_decimal() {
        assert_eq!(dec(0.0), "0");
        assert_eq!(dec(-0.0), "0");
        assert_eq!(dec(42.0), "42");
        assert_eq!(dec(3.14159), "3.14159");
        assert_eq!(dec(-123.456), "-123.456");
        assert_eq!(dec(0.1 + 0.2), "0.3");
        assert_eq!(dec(1.0 / 3.0), "0.333333333333");
    }

    #[test]
    fn no_representation_noise() {
        // Audit: 12345678.9 showed 12345678.9000000004.
        assert_eq!(dec(12_345_678.9), "12345678.9");
        assert_eq!(dec(1_234_567_890.123), "1234567890.12");
    }

    #[test]
    fn scientific_is_trimmed_and_late() {
        // Audit: 1e10 showed 1.0000000e10; 123456789012 showed 1.2345679e11.
        assert_eq!(dec(1e10), "10000000000");
        assert_eq!(dec(123_456_789_012.0), "123456789012");
        assert_eq!(dec(1e12), "1e12");
        assert_eq!(dec(1.23e15), "1.23e15");
        assert_eq!(dec(-5e-7), "-5e-7");
        assert_eq!(dec(0.0001), "0.0001");
        assert_eq!(dec(1e300), "1e300");
    }

    #[test]
    fn decimal_output_parses_back() {
        for v in [1e300, -2.5e-9, 12_345_678.9, 1.0 / 7.0, 6.02214076e23, 1e12] {
            let s = dec(v);
            let back = calculate(&s, 0.0, AngleMode::Degrees).unwrap();
            assert!(((back - v) / v).abs() < 1e-11, "{} -> {} -> {}", v, s, back);
        }
    }

    #[test]
    fn radix_bases() {
        assert_eq!(format_number(255.0, NumberBase::Hexadecimal), "0xFF");
        assert_eq!(format_number(8.0, NumberBase::Octal), "0o10");
        assert_eq!(format_number(5.0, NumberBase::Binary), "0b101");
        assert_eq!(format_number(-16.0, NumberBase::Hexadecimal), "-0x10");
    }

    #[test]
    fn radix_marks_fractions_and_range() {
        // Audit: 0.5 showed 0x0, 1e30 showed 0x7FFF..., -1e30 panicked in debug.
        assert_eq!(format_number(0.5, NumberBase::Hexadecimal), "~0x0");
        assert_eq!(format_number(1e30, NumberBase::Hexadecimal), "out of range");
        assert_eq!(format_number(-1e30, NumberBase::Hexadecimal), "out of range");
        // i64::MIN itself is fine (unsigned_abs, no overflow)
        assert_eq!(
            format_number(-9_223_372_036_854_775_808.0, NumberBase::Hexadecimal),
            "-0x8000000000000000"
        );
        assert_eq!(format_number(9_223_372_036_854_775_808.0, NumberBase::Binary), "out of range");
    }

    #[test]
    fn ellipsize_is_char_safe() {
        assert_eq!(ellipsize("abc", 5), "abc");
        assert_eq!(ellipsize("abcdef", 4), "abc…");
        assert_eq!(ellipsize("ππππππ", 3), "ππ…");
    }

    #[test]
    fn non_finite_is_labelled() {
        assert_eq!(dec(f64::NAN), "NaN");
        assert_eq!(dec(f64::INFINITY), "inf");
    }
}
