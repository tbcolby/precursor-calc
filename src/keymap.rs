//! Keyboard to operation mapping

use crate::functions::{Func, Op};

/// Key action result
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// Digit 0-9
    Digit(char),
    /// Letter for expression input (algebraic mode)
    Letter(char),
    /// Decimal point
    DecimalPoint,
    /// Binary operator
    Operator(Op),
    /// Scientific function
    Function(Func),
    /// Open parenthesis
    OpenParen,
    /// Close parenthesis
    CloseParen,
    /// Execute/Enter
    Execute,
    /// Backspace
    Backspace,
    /// Clear entry (CLx)
    ClearEntry,
    /// Clear all (AC)
    ClearAll,
    /// Toggle sign (+/-)
    ChangeSign,
    /// Insert "Ans"
    Ans,
    /// Toggle mode (ALG/RPN)
    ToggleMode,
    /// Cycle angle mode (DEG/RAD/GRAD)
    CycleAngle,
    /// Cycle number base (DEC/HEX/OCT/BIN)
    CycleBase,
    /// RPN: Swap X↔Y
    SwapXY,
    /// RPN: Roll down
    RollDown,
    /// RPN: Roll up
    RollUp,
    /// RPN: Last X
    LastX,
    /// Store to memory (followed by digit)
    Store,
    /// Recall from memory (followed by digit)
    Recall,
    /// Open function menu (F1-F4)
    FnMenu(u8),
    /// Menu selection (0-9)
    MenuSelect(u8),
    /// Cancel/Escape
    Cancel,
    /// Quit app
    Quit,
    /// No action
    None,
}

/// State for tracking shift and 2nd function
#[derive(Default)]
pub struct KeyState {
    /// Shift key held
    pub shift: bool,
    /// Waiting for memory register digit
    pub waiting_store: bool,
    pub waiting_recall: bool,
    /// Current function menu (0 = none, 1-4 = F1-F4)
    pub fn_menu: u8,
}

impl KeyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.shift = false;
        self.waiting_store = false;
        self.waiting_recall = false;
        self.fn_menu = 0;
    }
}

/// Map keyboard character to action
pub fn map_key(c: char, state: &mut KeyState, is_rpn: bool) -> KeyAction {
    // Handle shift indicator
    if c == '\u{0F}' {
        state.shift = true;
        return KeyAction::None;
    }

    // Handle function menu selection
    if state.fn_menu > 0 {
        if let Some(digit) = c.to_digit(10) {
            let action = map_fn_menu_key(state.fn_menu, digit as u8);
            state.fn_menu = 0;
            return action;
        }
        if c == '\u{001B}' || c == '∴' {
            // ESC or menu key cancels
            state.fn_menu = 0;
            return KeyAction::Cancel;
        }
    }

    // Handle memory register selection
    if state.waiting_store {
        if let Some(digit) = c.to_digit(10) {
            state.waiting_store = false;
            return KeyAction::MenuSelect(digit as u8); // Will be interpreted as store to M[digit]
        }
        state.waiting_store = false;
        return KeyAction::Cancel;
    }
    if state.waiting_recall {
        if let Some(digit) = c.to_digit(10) {
            state.waiting_recall = false;
            return KeyAction::MenuSelect(digit as u8); // Will be interpreted as recall from M[digit]
        }
        state.waiting_recall = false;
        return KeyAction::Cancel;
    }

    let shift = state.shift;
    state.shift = false;

    // Shifted keys
    if shift {
        return map_shifted_key(c);
    }

    // Normal keys
    match c {
        // Digits
        '0'..='9' => KeyAction::Digit(c),
        '.' => KeyAction::DecimalPoint,

        // Operators
        '+' => KeyAction::Operator(Op::Add),
        '-' => KeyAction::Operator(Op::Sub),
        '*' | '×' => KeyAction::Operator(Op::Mul),
        '/' | '÷' => KeyAction::Operator(Op::Div),
        '^' => KeyAction::Operator(Op::Pow),
        '%' => KeyAction::Operator(Op::Mod),

        // Parentheses
        '(' | '[' => KeyAction::OpenParen,
        ')' | ']' => KeyAction::CloseParen,

        // Control
        '\r' | '\n' | '=' => KeyAction::Execute,
        '\u{0008}' => KeyAction::Backspace, // Backspace
        ' ' => KeyAction::ClearEntry,

        // In RPN mode, letters are commands
        // In algebraic mode, lowercase letters are for function names (sin, cos, sqrt, etc.)

        // Clear - uppercase only to allow 'c' in expressions like "acos"
        'C' => KeyAction::ClearAll,

        // Mode toggles - uppercase works in both modes
        'M' => KeyAction::ToggleMode,
        'A' => KeyAction::CycleAngle,
        'B' => KeyAction::CycleBase,

        // RPN specific commands (only in RPN mode)
        'x' | 'X' if is_rpn => KeyAction::SwapXY,
        'r' | 'R' if is_rpn => KeyAction::RollDown,
        'l' | 'L' if is_rpn => KeyAction::LastX,

        // Memory - uppercase only
        'S' => KeyAction::Store,
        'K' => KeyAction::Recall,

        // Ans - uppercase only in RPN, lowercase allowed in algebraic for expression
        'N' if is_rpn => KeyAction::Ans,

        // In algebraic mode, pass lowercase letters through for function names
        'a'..='z' if !is_rpn => KeyAction::Letter(c),

        // In RPN mode, some letters are commands, others are ignored
        'a' if is_rpn => KeyAction::CycleAngle,
        'b' if is_rpn => KeyAction::CycleBase,
        'n' if is_rpn => KeyAction::Ans,

        // Function keys (using F1-F4 scan codes may vary)
        '\u{F704}' => KeyAction::FnMenu(1), // F1
        '\u{F705}' => KeyAction::FnMenu(2), // F2
        '\u{F706}' => KeyAction::FnMenu(3), // F3
        '\u{F707}' => KeyAction::FnMenu(4), // F4

        // Also support 1-4 with some modifier for function menus
        // In practice we'll use Esc-prefix or similar

        // Cancel/Quit - use menu key (∴) to exit app
        '\u{001B}' => KeyAction::Cancel, // ESC

        // Menu key - exit the calculator
        '∴' => KeyAction::Quit, // Precursor menu key exits app

        _ => KeyAction::None,
    }
}

/// Map shifted keys
fn map_shifted_key(c: char) -> KeyAction {
    match c {
        // Trig functions
        '1' => KeyAction::Function(Func::Sin),
        '2' => KeyAction::Function(Func::Cos),
        '3' => KeyAction::Function(Func::Tan),

        // Log functions
        '4' => KeyAction::Function(Func::Ln),
        '5' => KeyAction::Function(Func::Log),

        // Other functions
        '6' => KeyAction::Function(Func::Sqrt),
        '7' => KeyAction::Function(Func::Square),
        '8' => KeyAction::Function(Func::Reciprocal),

        // Alt parens
        '9' => KeyAction::OpenParen,
        '0' => KeyAction::CloseParen,

        // Constants
        '.' => KeyAction::Function(Func::Pi),
        'e' | 'E' => KeyAction::Function(Func::E),

        // Change sign
        '-' => KeyAction::ChangeSign,

        // More functions via letters
        's' | 'S' => KeyAction::Function(Func::Asin),
        'c' | 'C' => KeyAction::Function(Func::Acos),
        't' | 'T' => KeyAction::Function(Func::Atan),
        'l' | 'L' => KeyAction::Function(Func::Log2),
        'x' | 'X' => KeyAction::Function(Func::Exp),
        'r' | 'R' => KeyAction::Function(Func::Cbrt),
        'f' | 'F' => KeyAction::Function(Func::Factorial),
        'a' | 'A' => KeyAction::Function(Func::Abs),

        _ => KeyAction::None,
    }
}

/// Map function menu key (menu number, digit pressed)
pub fn map_fn_menu_key(menu: u8, key: u8) -> KeyAction {
    match menu {
        1 => {
            // MATH menu
            match key {
                1 => KeyAction::Function(Func::Abs),
                2 => KeyAction::Function(Func::Floor),
                3 => KeyAction::Function(Func::Ceil),
                4 => KeyAction::Function(Func::Round),
                5 => KeyAction::Operator(Op::Mod),
                6 => KeyAction::Function(Func::Factorial),
                7 => KeyAction::Function(Func::Exp),
                8 => KeyAction::Function(Func::Exp10),
                9 => KeyAction::Function(Func::Cube),
                0 => KeyAction::Function(Func::Cbrt),
                _ => KeyAction::None,
            }
        }
        2 => {
            // TRIG menu
            match key {
                1 => KeyAction::Function(Func::Sin),
                2 => KeyAction::Function(Func::Cos),
                3 => KeyAction::Function(Func::Tan),
                4 => KeyAction::Function(Func::Asin),
                5 => KeyAction::Function(Func::Acos),
                6 => KeyAction::Function(Func::Atan),
                7 => KeyAction::Function(Func::Sinh),
                8 => KeyAction::Function(Func::Cosh),
                9 => KeyAction::Function(Func::Tanh),
                0 => KeyAction::None, // Could add more
                _ => KeyAction::None,
            }
        }
        3 => {
            // MODE menu (handled specially)
            match key {
                1 => KeyAction::ToggleMode,
                2 => KeyAction::CycleAngle,
                3 => KeyAction::CycleBase,
                _ => KeyAction::None,
            }
        }
        4 => {
            // MEM menu
            match key {
                0..=9 => KeyAction::Recall, // Will need register number
                _ => KeyAction::None,
            }
        }
        _ => KeyAction::None,
    }
}

/// Get function menu display items
pub fn get_menu_items(menu: u8) -> &'static [(&'static str, &'static str)] {
    match menu {
        1 => &[
            ("1", "abs"),
            ("2", "floor"),
            ("3", "ceil"),
            ("4", "round"),
            ("5", "mod"),
            ("6", "n!"),
            ("7", "exp"),
            ("8", "10^x"),
            ("9", "x³"),
            ("0", "³√x"),
        ],
        2 => &[
            ("1", "sin"),
            ("2", "cos"),
            ("3", "tan"),
            ("4", "asin"),
            ("5", "acos"),
            ("6", "atan"),
            ("7", "sinh"),
            ("8", "cosh"),
            ("9", "tanh"),
        ],
        3 => &[
            ("1", "ALG/RPN"),
            ("2", "DEG/RAD"),
            ("3", "DEC/HEX"),
        ],
        4 => &[
            ("0-9", "Recall M#"),
        ],
        _ => &[],
    }
}
