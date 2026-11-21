//! Controller-level tests: whole key sequences, as the device delivers them.

use alloc::string::String;
use alloc::vec::Vec;

use precursor_appkit_core::keys::{decode_rawkeys, Key};

use crate::calculator::{Action, Calculator, HistoryEntry, LoadError, Mode, Overlay, Saved, STATE_VERSION};
use crate::functions::{AngleMode, NumberBase};
use crate::menu::MenuId;

fn press(c: &mut Calculator, keys: &str) -> Vec<Action> {
    keys.chars().filter_map(Key::from_char).map(|k| c.on_key(k)).collect()
}

fn rpn() -> Calculator {
    let mut c = Calculator::new();
    press(&mut c, "M");
    assert_eq!(c.mode(), Mode::Rpn);
    c
}

#[test]
fn unknown_keys_never_leave() {
    // Audit: any unmapped key (arrows, ',', '?', 'Z', letters in RPN, non-digits
    // in a menu) terminated the app.
    for mode_keys in ["", "M"] {
        let mut c = Calculator::new();
        press(&mut c, mode_keys);
        let actions = press(&mut c, "←→↑↓,?Zqwyz#~@\u{F700}😊福");
        assert!(actions.iter().all(|a| *a == Action::Stay), "{:?}", actions);
        // in a menu too
        press(&mut c, "\u{11}");
        let actions = press(&mut c, "zq,?←\r");
        assert!(actions.iter().all(|a| *a == Action::Stay));
        assert_eq!(c.overlay(), Overlay::Menu(MenuId::Math), "unknown keys keep the menu open");
    }
}

#[test]
fn only_f4_on_main_screen_leaves() {
    let mut c = Calculator::new();
    assert_eq!(c.on_key(Key::F(4)), Action::Leave);
    // F4 in a menu closes the menu instead
    c.on_key(Key::F(1));
    assert_eq!(c.on_key(Key::F(4)), Action::Stay);
    assert_eq!(c.overlay(), Overlay::None);
    // Backspace never leaves
    for _ in 0..5 {
        assert_eq!(c.on_key(Key::Backspace), Action::Stay);
    }
}

#[test]
fn function_keys_are_real_codes() {
    // Audit: F1-F4 were matched as U+F704..F707; the device sends U+0011..0014.
    let mut c = Calculator::new();
    let keys: Vec<Key> = decode_rawkeys([0x11, 0, 0, 0]).collect();
    c.on_key(keys[0]);
    assert_eq!(c.overlay(), Overlay::Menu(MenuId::Math));
    c.on_key(Key::F(2));
    assert_eq!(c.overlay(), Overlay::Menu(MenuId::Trig), "another F-key switches menus");
    c.on_key(Key::F(2));
    assert_eq!(c.overlay(), Overlay::None, "same F-key closes");
    c.on_key(Key::F(3));
    c.on_key(Key::Backspace);
    assert_eq!(c.overlay(), Overlay::None);
}

#[test]
fn rpn_chain_through_keys() {
    let mut c = rpn();
    press(&mut c, "1\r2\r3++");
    assert_eq!(c.rpn().stack(), [6.0, 0.0, 0.0, 0.0]);
}

#[test]
fn rpn_functions_via_menu() {
    // Audit: RPN had no way to apply any function.
    let mut c = rpn();
    press(&mut c, "16");
    press(&mut c, "\u{11}1"); // F1, 1 = sqrt
    assert_eq!(c.rpn().x(), 4.0);
    press(&mut c, "\u{12}1"); // F2, 1 = sin (of 4 degrees)
    assert!((c.rpn().x() - 4f64.to_radians().sin()).abs() < 1e-15);
    press(&mut c, "C5n"); // change sign
    assert_eq!(c.rpn().x(), -5.0);
    press(&mut c, "!");
    assert!(c.error().is_some(), "(-5)! is a domain error");
    assert_eq!(c.rpn().x(), -5.0);
}

#[test]
fn rpn_division_by_zero_shown_not_panicking() {
    let mut c = rpn();
    press(&mut c, "1\r0/");
    assert_eq!(c.error(), Some("Error: divide by zero"));
    assert_eq!(c.rpn().stack()[..2], [0.0, 1.0]);
    press(&mut c, "1");
    assert_eq!(c.error(), None, "error clears on next key");
}

#[test]
fn alg_evaluate_and_errors() {
    let mut c = Calculator::new();
    press(&mut c, "2+3*4\r");
    assert_eq!(c.ans(), 14.0);
    assert_eq!(c.input(), "");
    press(&mut c, "1/0\r");
    assert_eq!(c.error(), Some("Error: divide by zero"));
    assert_eq!(c.input(), "1/0", "input kept for correction");
    assert_eq!(c.ans(), 14.0);
    press(&mut c, " 9^999\r");
    assert_eq!(c.error(), Some("Error: overflow"));
}

#[test]
fn alg_menu_insertions_evaluate() {
    // Audit: Square inserted "x²(" (ERR: PARSE); factorial unusable in ALG.
    let mut c = Calculator::new();
    press(&mut c, "3\u{11}2\r"); // 3, F1-2 (x^2)
    assert_eq!(c.ans(), 9.0);
    press(&mut c, "4\u{11}8\r"); // 4!
    assert_eq!(c.ans(), 24.0);
    press(&mut c, "\u{11}34)\r"); // 1/(4)
    assert_eq!(c.ans(), 0.25);
    press(&mut c, "\u{13}6\r"); // F3-6 inserts "^3"; leading operator continues ans
    assert_eq!(c.ans(), 0.015625);
}

#[test]
fn alg_leading_operator_uses_ans() {
    let mut c = Calculator::new();
    press(&mut c, "5\r*2\r");
    assert_eq!(c.ans(), 10.0);
    press(&mut c, "-3\r");
    assert_eq!(c.ans(), -3.0, "leading minus is negation, not ans-3");
}

#[test]
fn alg_unary_minus_and_prefix_functions() {
    let mut c = Calculator::new();
    press(&mut c, "5\r-ans+1\r");
    assert_eq!(c.ans(), -4.0);
    press(&mut c, "-2^2\r");
    assert_eq!(c.ans(), -4.0);
    press(&mut c, "-sqrt(4)+1\r");
    assert_eq!(c.ans(), -1.0);
}

#[test]
fn alg_change_sign_wraps_whole_input() {
    let mut c = Calculator::new();
    press(&mut c, "2+3\u{13}8\r");
    assert_eq!(c.ans(), -5.0);
}

#[test]
fn memory_store_and_recall_use_pressed_digit() {
    // Audit: in the MEM menu the pressed digit was discarded.
    let mut c = Calculator::new();
    press(&mut c, "42\rS3");
    assert_eq!(c.memory()[3], 42.0);
    press(&mut c, "K3+1\r");
    assert_eq!(c.ans(), 43.0);
    // via the F3 menu
    press(&mut c, "\u{13}97"); // STO 7
    assert_eq!(c.memory()[7], 43.0);
    press(&mut c, "\u{13}07*2\r"); // RCL 7
    assert_eq!(c.ans(), 86.0);
    // F4 cancels a register prompt without leaving
    assert_eq!(c.on_key(Key::Char('S')), Action::Stay);
    assert_eq!(c.on_key(Key::F(4)), Action::Stay);
    assert_eq!(c.overlay(), Overlay::None);
}

#[test]
fn recall_of_large_and_negative_values_parses() {
    // Audit: recalling 1e300 inserted a 301-character literal.
    let mut c = Calculator::new();
    press(&mut c, "1e300\rS1 2*K1\r");
    assert_eq!(c.ans(), 2e300);
    press(&mut c, "-7\rS2 3-K2\r");
    assert_eq!(c.ans(), 10.0);
    press(&mut c, "2K2\r"); // after a digit: implicit multiply, not "2-7"
    assert_eq!(c.ans(), -14.0);
}

#[test]
fn recall_in_rpn_respects_lift() {
    let mut c = rpn();
    press(&mut c, "9S0");
    press(&mut c, "C1\rK0");
    assert_eq!(c.rpn().stack()[..2], [9.0, 1.0], "after ENTER, RCL overwrites the copy");
}

#[test]
fn mode_toggle_does_not_keep_pushing() {
    // Audit: each ALG->RPN toggle pushed ans and dropped T.
    let mut c = Calculator::new();
    press(&mut c, "5\r");
    press(&mut c, "MMMMMM");
    assert_eq!(c.mode(), Mode::Algebraic);
    press(&mut c, "M");
    assert_eq!(c.rpn().stack(), [5.0, 0.0, 0.0, 0.0]);
    press(&mut c, "3*M");
    assert_eq!(c.ans(), 15.0, "RPN X carries back to ans");
}

#[test]
fn history_browse_with_arrows() {
    let mut c = Calculator::new();
    press(&mut c, "1+1\r2+2\r");
    press(&mut c, "↑");
    assert_eq!(c.input(), "2+2");
    press(&mut c, "↑↑");
    assert_eq!(c.input(), "1+1");
    press(&mut c, "↓↓");
    assert_eq!(c.input(), "");
    assert_eq!(c.history_lines(5), ["1+1 = 2", "2+2 = 4"]);
}

#[test]
fn input_is_bounded() {
    let mut c = Calculator::new();
    for _ in 0..500 {
        press(&mut c, "9");
    }
    assert_eq!(c.input().len(), crate::calculator::MAX_INPUT);
}

#[test]
fn modes_cycle_and_status() {
    let mut c = Calculator::new();
    press(&mut c, "AB");
    assert_eq!(c.angle(), AngleMode::Radians);
    assert_eq!(c.base(), NumberBase::Hexadecimal);
    press(&mut c, "255\r");
    assert_eq!(c.status_line(), "ALG RAD HEX");
    press(&mut c, "S0");
    assert_eq!(c.status_line(), "ALG RAD HEX M");
    assert_eq!(c.history_lines(1), ["255 = 0xFF"]);
}

#[test]
fn saved_round_trip_through_json() {
    let mut c = Calculator::new();
    press(&mut c, "2+2\rS4AB");
    press(&mut c, "M7\r8");
    let saved = c.to_saved();
    let json = serde_json::to_string(&saved).unwrap();
    let mut d = Calculator::new();
    d.load(serde_json::from_str(&json).unwrap()).unwrap();
    assert_eq!(d.to_saved(), saved);
    assert_eq!(d.mode(), Mode::Rpn);
    assert_eq!(d.rpn().stack()[..2], [8.0, 7.0]);
}

#[test]
fn version_zero_save_still_loads() {
    // The original app stored {mode, angle_mode, number_base, memory, ans}.
    let old = r#"{"mode":1,"angle_mode":1,"number_base":0,"memory":[1,0,0,0,0,0,0,0,0,2],"ans":3.5}"#;
    let mut c = Calculator::new();
    c.load(serde_json::from_str(old).unwrap()).unwrap();
    assert_eq!((c.mode(), c.angle(), c.ans()), (Mode::Rpn, AngleMode::Radians, 3.5));
    assert_eq!(c.memory()[9], 2.0);
}

#[test]
fn null_numbers_in_old_save_are_not_fatal_after_resave() {
    // A non-finite value can no longer reach the saved state.
    let mut c = Calculator::new();
    press(&mut c, "1e308*10\r");
    let json = serde_json::to_string(&c.to_saved()).unwrap();
    assert!(!json.contains("null"), "{}", json);
}

#[test]
fn newer_version_is_refused() {
    let mut c = Calculator::new();
    let s = Saved { version: STATE_VERSION + 1, ..Saved::default() };
    assert_eq!(c.load(s), Err(LoadError::NewerVersion(STATE_VERSION + 1)));
}

#[test]
fn load_sanitises_and_bounds_history() {
    let mut c = Calculator::new();
    let history = (0..80).map(|i| HistoryEntry { expr: String::from("1"), result: i as f64 }).collect();
    let s = Saved { version: 1, ans: f64::NAN, memory: [f64::INFINITY; 10], history, ..Saved::default() };
    c.load(s).unwrap();
    assert_eq!(c.ans(), 0.0);
    assert_eq!(c.memory()[0], 0.0);
    assert_eq!(c.to_saved().history.len(), crate::calculator::HISTORY_SAVED);
}

#[test]
fn overwrite_request_is_one_shot() {
    let mut c = Calculator::new();
    assert!(!c.take_overwrite_request());
    press(&mut c, "\u{13}w");
    assert!(c.take_overwrite_request());
    assert!(!c.take_overwrite_request());
}
