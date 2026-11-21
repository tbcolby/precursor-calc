//! Pure logic for the Precursor calculator. Nothing here depends on Xous, so
//! `cargo test` runs on any host.
//!
//! * [`functions`]: unary functions, binary operators, errors (never NaN/inf).
//! * [`parser`]: infix expressions with correct unary minus, prefix functions,
//!   postfix `!` and implicit multiplication.
//! * [`rpn`]: four-level stack with HP-style stack lift.
//! * [`format`]: display formatting (12 significant digits, bases).
//! * [`menu`]: the F1-F3 menus.
//! * [`calculator`]: the key-driven controller and its saved state.

// std is needed for f64 transcendental functions (sin, powf, ln).

extern crate alloc;

pub mod calculator;
pub mod format;
pub mod functions;
pub mod menu;
pub mod parser;
pub mod rpn;

pub use calculator::{Action, Calculator, LoadError, Mode, Overlay, Saved, STATE_VERSION};

#[cfg(test)]
mod tests;
