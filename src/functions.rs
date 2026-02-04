//! Scientific function implementations

use core::f64::consts::{E, PI};

/// Angle unit for trig functions
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum AngleMode {
    #[default]
    Degrees,
    Radians,
    Gradians,
}

impl AngleMode {
    pub fn cycle(&self) -> Self {
        match self {
            AngleMode::Degrees => AngleMode::Radians,
            AngleMode::Radians => AngleMode::Gradians,
            AngleMode::Gradians => AngleMode::Degrees,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AngleMode::Degrees => "DEG",
            AngleMode::Radians => "RAD",
            AngleMode::Gradians => "GRAD",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            AngleMode::Degrees => 0,
            AngleMode::Radians => 1,
            AngleMode::Gradians => 2,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => AngleMode::Radians,
            2 => AngleMode::Gradians,
            _ => AngleMode::Degrees,
        }
    }
}

/// Number display base
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum NumberBase {
    #[default]
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
}

impl NumberBase {
    pub fn cycle(&self) -> Self {
        match self {
            NumberBase::Decimal => NumberBase::Hexadecimal,
            NumberBase::Hexadecimal => NumberBase::Octal,
            NumberBase::Octal => NumberBase::Binary,
            NumberBase::Binary => NumberBase::Decimal,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NumberBase::Decimal => "DEC",
            NumberBase::Hexadecimal => "HEX",
            NumberBase::Octal => "OCT",
            NumberBase::Binary => "BIN",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            NumberBase::Decimal => 0,
            NumberBase::Hexadecimal => 1,
            NumberBase::Octal => 2,
            NumberBase::Binary => 3,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => NumberBase::Hexadecimal,
            2 => NumberBase::Octal,
            3 => NumberBase::Binary,
            _ => NumberBase::Decimal,
        }
    }
}

/// Scientific functions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Func {
    // Trigonometric
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    // Hyperbolic
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    // Logarithmic
    Log,
    Ln,
    Log2,
    // Exponential
    Exp,
    Exp10,
    // Power/Root
    Sqrt,
    Cbrt,
    Square,
    Cube,
    // Other
    Abs,
    Floor,
    Ceil,
    Round,
    Factorial,
    Reciprocal,
    Negate,
    // Constants (evaluated to values)
    Pi,
    E,
}

impl Func {
    /// Parse function name to Func
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "sin" => Some(Func::Sin),
            "cos" => Some(Func::Cos),
            "tan" => Some(Func::Tan),
            "asin" | "arcsin" => Some(Func::Asin),
            "acos" | "arccos" => Some(Func::Acos),
            "atan" | "arctan" => Some(Func::Atan),
            "sinh" => Some(Func::Sinh),
            "cosh" => Some(Func::Cosh),
            "tanh" => Some(Func::Tanh),
            "asinh" => Some(Func::Asinh),
            "acosh" => Some(Func::Acosh),
            "atanh" => Some(Func::Atanh),
            "log" => Some(Func::Log),
            "ln" => Some(Func::Ln),
            "log2" => Some(Func::Log2),
            "exp" => Some(Func::Exp),
            "sqrt" => Some(Func::Sqrt),
            "cbrt" => Some(Func::Cbrt),
            "abs" => Some(Func::Abs),
            "floor" => Some(Func::Floor),
            "ceil" => Some(Func::Ceil),
            "round" => Some(Func::Round),
            "pi" => Some(Func::Pi),
            "e" => Some(Func::E),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Asin => "asin",
            Func::Acos => "acos",
            Func::Atan => "atan",
            Func::Sinh => "sinh",
            Func::Cosh => "cosh",
            Func::Tanh => "tanh",
            Func::Asinh => "asinh",
            Func::Acosh => "acosh",
            Func::Atanh => "atanh",
            Func::Log => "log",
            Func::Ln => "ln",
            Func::Log2 => "log2",
            Func::Exp => "exp",
            Func::Exp10 => "10^",
            Func::Sqrt => "sqrt",
            Func::Cbrt => "cbrt",
            Func::Square => "x²",
            Func::Cube => "x³",
            Func::Abs => "abs",
            Func::Floor => "floor",
            Func::Ceil => "ceil",
            Func::Round => "round",
            Func::Factorial => "!",
            Func::Reciprocal => "1/x",
            Func::Negate => "neg",
            Func::Pi => "π",
            Func::E => "e",
        }
    }

    /// Is this a constant (no argument needed)?
    pub fn is_constant(&self) -> bool {
        matches!(self, Func::Pi | Func::E)
    }

    /// Evaluate unary function
    pub fn evaluate(&self, x: f64, angle_mode: AngleMode) -> Result<f64, CalcError> {
        match self {
            // Constants
            Func::Pi => Ok(PI),
            Func::E => Ok(E),

            // Trigonometric (input in current angle mode)
            Func::Sin => Ok(to_radians(x, angle_mode).sin()),
            Func::Cos => Ok(to_radians(x, angle_mode).cos()),
            Func::Tan => {
                let rad = to_radians(x, angle_mode);
                let cos = rad.cos();
                if cos.abs() < 1e-15 {
                    Err(CalcError::DomainError("tan undefined at 90°"))
                } else {
                    Ok(rad.tan())
                }
            }

            // Inverse trig (output in current angle mode)
            Func::Asin => {
                if x.abs() > 1.0 {
                    Err(CalcError::DomainError("asin domain [-1,1]"))
                } else {
                    Ok(from_radians(x.asin(), angle_mode))
                }
            }
            Func::Acos => {
                if x.abs() > 1.0 {
                    Err(CalcError::DomainError("acos domain [-1,1]"))
                } else {
                    Ok(from_radians(x.acos(), angle_mode))
                }
            }
            Func::Atan => Ok(from_radians(x.atan(), angle_mode)),

            // Hyperbolic
            Func::Sinh => Ok(x.sinh()),
            Func::Cosh => Ok(x.cosh()),
            Func::Tanh => Ok(x.tanh()),
            Func::Asinh => Ok(x.asinh()),
            Func::Acosh => {
                if x < 1.0 {
                    Err(CalcError::DomainError("acosh domain [1,∞)"))
                } else {
                    Ok(x.acosh())
                }
            }
            Func::Atanh => {
                if x.abs() >= 1.0 {
                    Err(CalcError::DomainError("atanh domain (-1,1)"))
                } else {
                    Ok(x.atanh())
                }
            }

            // Logarithmic
            Func::Ln => {
                if x <= 0.0 {
                    Err(CalcError::DomainError("ln domain (0,∞)"))
                } else {
                    Ok(x.ln())
                }
            }
            Func::Log => {
                if x <= 0.0 {
                    Err(CalcError::DomainError("log domain (0,∞)"))
                } else {
                    Ok(x.log10())
                }
            }
            Func::Log2 => {
                if x <= 0.0 {
                    Err(CalcError::DomainError("log2 domain (0,∞)"))
                } else {
                    Ok(x.log2())
                }
            }

            // Exponential
            Func::Exp => {
                let result = x.exp();
                if result.is_infinite() {
                    Err(CalcError::Overflow)
                } else {
                    Ok(result)
                }
            }
            Func::Exp10 => {
                let result = 10.0_f64.powf(x);
                if result.is_infinite() {
                    Err(CalcError::Overflow)
                } else {
                    Ok(result)
                }
            }

            // Roots
            Func::Sqrt => {
                if x < 0.0 {
                    Err(CalcError::DomainError("sqrt domain [0,∞)"))
                } else {
                    Ok(x.sqrt())
                }
            }
            Func::Cbrt => Ok(x.cbrt()),

            // Powers
            Func::Square => Ok(x * x),
            Func::Cube => Ok(x * x * x),

            // Other
            Func::Abs => Ok(x.abs()),
            Func::Floor => Ok(x.floor()),
            Func::Ceil => Ok(x.ceil()),
            Func::Round => Ok(x.round()),
            Func::Reciprocal => {
                if x == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(1.0 / x)
                }
            }
            Func::Negate => Ok(-x),
            Func::Factorial => factorial(x),
        }
    }
}

/// Convert angle to radians from current mode
fn to_radians(x: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => x,
        AngleMode::Degrees => x.to_radians(),
        AngleMode::Gradians => x * PI / 200.0,
    }
}

/// Convert radians to current angle mode
fn from_radians(x: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => x,
        AngleMode::Degrees => x.to_degrees(),
        AngleMode::Gradians => x * 200.0 / PI,
    }
}

/// Calculate factorial (gamma function for non-integers)
fn factorial(x: f64) -> Result<f64, CalcError> {
    if x < 0.0 {
        return Err(CalcError::DomainError("factorial domain [0,∞)"));
    }

    // Check if integer
    if x == x.floor() && x <= 170.0 {
        let n = x as u64;
        let mut result = 1.0_f64;
        for i in 2..=n {
            result *= i as f64;
        }
        if result.is_infinite() {
            Err(CalcError::Overflow)
        } else {
            Ok(result)
        }
    } else if x > 170.0 {
        Err(CalcError::Overflow)
    } else {
        // Use gamma function: n! = gamma(n+1)
        // Stirling approximation for non-integers
        let result = gamma(x + 1.0);
        if result.is_infinite() || result.is_nan() {
            Err(CalcError::Overflow)
        } else {
            Ok(result)
        }
    }
}

/// Gamma function approximation (Lanczos)
fn gamma(x: f64) -> f64 {
    // Lanczos approximation coefficients
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    if x < 0.5 {
        // Reflection formula
        PI / ((PI * x).sin() * gamma(1.0 - x))
    } else {
        let x = x - 1.0;
        let mut a = C[0];
        for i in 1..9 {
            a += C[i] / (x + i as f64);
        }
        let t = x + G + 0.5;
        (2.0 * PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
    }
}

/// Binary operators
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

impl Op {
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Add | Op::Sub => 1,
            Op::Mul | Op::Div | Op::Mod => 2,
            Op::Pow => 3,
        }
    }

    pub fn is_left_assoc(&self) -> bool {
        !matches!(self, Op::Pow)
    }

    pub fn symbol(&self) -> char {
        match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '×',
            Op::Div => '÷',
            Op::Pow => '^',
            Op::Mod => '%',
        }
    }

    pub fn evaluate(&self, a: f64, b: f64) -> Result<f64, CalcError> {
        match self {
            Op::Add => Ok(a + b),
            Op::Sub => Ok(a - b),
            Op::Mul => Ok(a * b),
            Op::Div => {
                if b == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(a / b)
                }
            }
            Op::Pow => {
                let result = a.powf(b);
                if result.is_infinite() {
                    Err(CalcError::Overflow)
                } else if result.is_nan() {
                    Err(CalcError::DomainError("invalid power"))
                } else {
                    Ok(result)
                }
            }
            Op::Mod => {
                if b == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(a % b)
                }
            }
        }
    }
}

/// Calculator errors
#[derive(Debug, Clone)]
pub enum CalcError {
    DivideByZero,
    DomainError(&'static str),
    Overflow,
    ParseError(alloc::string::String),
    SyntaxError(alloc::string::String),
    MemoryError,
}

extern crate alloc;

impl CalcError {
    pub fn message(&self) -> &str {
        match self {
            CalcError::DivideByZero => "ERR: DIV/0",
            CalcError::DomainError(msg) => msg,
            CalcError::Overflow => "ERR: OVERFLOW",
            CalcError::ParseError(_) => "ERR: PARSE",
            CalcError::SyntaxError(_) => "ERR: SYNTAX",
            CalcError::MemoryError => "ERR: MEMORY",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trig() {
        let sin90 = Func::Sin.evaluate(90.0, AngleMode::Degrees).unwrap();
        assert!((sin90 - 1.0).abs() < 1e-10);

        let cos0 = Func::Cos.evaluate(0.0, AngleMode::Degrees).unwrap();
        assert!((cos0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0.0).unwrap(), 1.0);
        assert_eq!(factorial(5.0).unwrap(), 120.0);
        assert_eq!(factorial(10.0).unwrap(), 3628800.0);
    }
}
