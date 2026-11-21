//! Scientific functions, binary operators and calculator errors.
//!
//! Every evaluation returns `Err` instead of a non-finite value, so `ans`,
//! the RPN stack and memory registers never hold NaN or infinity (which
//! `serde_json` cannot round-trip).

use core::f64::consts::{E, PI};

/// Angle unit for trig functions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AngleMode {
    #[default]
    Degrees,
    Radians,
    Gradians,
}

impl AngleMode {
    pub fn cycle(self) -> Self {
        match self {
            AngleMode::Degrees => AngleMode::Radians,
            AngleMode::Radians => AngleMode::Gradians,
            AngleMode::Gradians => AngleMode::Degrees,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AngleMode::Degrees => "DEG",
            AngleMode::Radians => "RAD",
            AngleMode::Gradians => "GRAD",
        }
    }

    pub fn to_u8(self) -> u8 {
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

    /// Units in a full turn, or `None` for radians (not exactly representable).
    fn full_turn(self) -> Option<f64> {
        match self {
            AngleMode::Degrees => Some(360.0),
            AngleMode::Gradians => Some(400.0),
            AngleMode::Radians => None,
        }
    }
}

/// Number display base. Entry is always decimal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NumberBase {
    #[default]
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
}

impl NumberBase {
    pub fn cycle(self) -> Self {
        match self {
            NumberBase::Decimal => NumberBase::Hexadecimal,
            NumberBase::Hexadecimal => NumberBase::Octal,
            NumberBase::Octal => NumberBase::Binary,
            NumberBase::Binary => NumberBase::Decimal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            NumberBase::Decimal => "DEC",
            NumberBase::Hexadecimal => "HEX",
            NumberBase::Octal => "OCT",
            NumberBase::Binary => "BIN",
        }
    }

    pub fn to_u8(self) -> u8 {
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

/// Unary functions and constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Log,
    Ln,
    Log2,
    Exp,
    Exp10,
    Sqrt,
    Cbrt,
    Square,
    Cube,
    Abs,
    Floor,
    Ceil,
    Round,
    Factorial,
    Reciprocal,
    Negate,
    Pi,
    E,
}

impl Func {
    /// Function for a name typed in an expression (lower case).
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" | "arcsin" => Func::Asin,
            "acos" | "arccos" => Func::Acos,
            "atan" | "arctan" => Func::Atan,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "asinh" => Func::Asinh,
            "acosh" => Func::Acosh,
            "atanh" => Func::Atanh,
            "log" => Func::Log,
            "ln" => Func::Ln,
            "log2" => Func::Log2,
            "exp" => Func::Exp,
            "exp10" => Func::Exp10,
            "sqrt" => Func::Sqrt,
            "cbrt" => Func::Cbrt,
            "sq" => Func::Square,
            "cube" => Func::Cube,
            "abs" => Func::Abs,
            "floor" => Func::Floor,
            "ceil" => Func::Ceil,
            "round" => Func::Round,
            "fact" => Func::Factorial,
            "recip" => Func::Reciprocal,
            "neg" => Func::Negate,
            "pi" => Func::Pi,
            "e" => Func::E,
            _ => return None,
        })
    }

    /// Short label for menus.
    pub fn label(self) -> &'static str {
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
            Func::Exp => "e^x",
            Func::Exp10 => "10^x",
            Func::Sqrt => "sqrt",
            Func::Cbrt => "cbrt",
            Func::Square => "x^2",
            Func::Cube => "x^3",
            Func::Abs => "abs",
            Func::Floor => "floor",
            Func::Ceil => "ceil",
            Func::Round => "round",
            Func::Factorial => "n!",
            Func::Reciprocal => "1/x",
            Func::Negate => "+/-",
            Func::Pi => "pi",
            Func::E => "e",
        }
    }

    /// Text inserted into an algebraic expression when the function is chosen
    /// from a menu. Every insertion is parseable by [`crate::parser`].
    pub fn alg_insert(self) -> &'static str {
        match self {
            Func::Square => "^2",
            Func::Cube => "^3",
            Func::Factorial => "!",
            Func::Reciprocal => "1/(",
            Func::Exp10 => "10^(",
            Func::Pi => "pi",
            Func::E => "e",
            Func::Sin => "sin(",
            Func::Cos => "cos(",
            Func::Tan => "tan(",
            Func::Asin => "asin(",
            Func::Acos => "acos(",
            Func::Atan => "atan(",
            Func::Sinh => "sinh(",
            Func::Cosh => "cosh(",
            Func::Tanh => "tanh(",
            Func::Asinh => "asinh(",
            Func::Acosh => "acosh(",
            Func::Atanh => "atanh(",
            Func::Log => "log(",
            Func::Ln => "ln(",
            Func::Log2 => "log2(",
            Func::Exp => "exp(",
            Func::Sqrt => "sqrt(",
            Func::Cbrt => "cbrt(",
            Func::Abs => "abs(",
            Func::Floor => "floor(",
            Func::Ceil => "ceil(",
            Func::Round => "round(",
            Func::Negate => "neg(",
        }
    }

    /// Constants take no argument.
    pub fn is_constant(self) -> bool { matches!(self, Func::Pi | Func::E) }

    /// Evaluate. Never returns a non-finite value.
    pub fn evaluate(self, x: f64, angle: AngleMode) -> Result<f64, CalcError> {
        if !self.is_constant() && !x.is_finite() {
            return Err(CalcError::Overflow);
        }
        let r = match self {
            Func::Pi => PI,
            Func::E => E,
            Func::Sin => sin(x, angle),
            Func::Cos => cos(x, angle),
            Func::Tan => tan(x, angle)?,
            Func::Asin => {
                if x.abs() > 1.0 {
                    return Err(CalcError::Domain("asin needs -1..1"));
                }
                from_radians(x.asin(), angle)
            }
            Func::Acos => {
                if x.abs() > 1.0 {
                    return Err(CalcError::Domain("acos needs -1..1"));
                }
                from_radians(x.acos(), angle)
            }
            Func::Atan => from_radians(x.atan(), angle),
            Func::Sinh => x.sinh(),
            Func::Cosh => x.cosh(),
            Func::Tanh => x.tanh(),
            Func::Asinh => x.asinh(),
            Func::Acosh => {
                if x < 1.0 {
                    return Err(CalcError::Domain("acosh needs x>=1"));
                }
                x.acosh()
            }
            Func::Atanh => {
                if x.abs() >= 1.0 {
                    return Err(CalcError::Domain("atanh needs -1<x<1"));
                }
                x.atanh()
            }
            Func::Ln | Func::Log | Func::Log2 => {
                if x <= 0.0 {
                    return Err(CalcError::Domain("log needs x>0"));
                }
                match self {
                    Func::Ln => x.ln(),
                    Func::Log => x.log10(),
                    _ => x.log2(),
                }
            }
            Func::Exp => x.exp(),
            Func::Exp10 => 10f64.powf(x),
            Func::Sqrt => {
                if x < 0.0 {
                    return Err(CalcError::Domain("sqrt needs x>=0"));
                }
                x.sqrt()
            }
            Func::Cbrt => x.cbrt(),
            Func::Square => x * x,
            Func::Cube => x * x * x,
            Func::Abs => x.abs(),
            Func::Floor => x.floor(),
            Func::Ceil => x.ceil(),
            Func::Round => x.round(),
            Func::Reciprocal => {
                if x == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                1.0 / x
            }
            Func::Negate => -x,
            Func::Factorial => factorial(x)?,
        };
        finite(r)
    }
}

/// Map a non-finite result to `Overflow` (or `Domain` for NaN).
pub fn finite(r: f64) -> Result<f64, CalcError> {
    if r.is_nan() {
        Err(CalcError::Domain("undefined"))
    } else if r.is_infinite() {
        Err(CalcError::Overflow)
    } else {
        Ok(r)
    }
}

fn to_radians(x: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => x,
        AngleMode::Degrees => x.to_radians(),
        AngleMode::Gradians => x * PI / 200.0,
    }
}

fn from_radians(x: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => x,
        AngleMode::Degrees => x.to_degrees(),
        AngleMode::Gradians => x * 200.0 / PI,
    }
}

/// Angle reduced to `[0, full)` and its exact quarter-turn index, if any.
fn reduce(x: f64, full: f64) -> (f64, Option<u8>) {
    let r = x.rem_euclid(full);
    let quarter = full / 4.0;
    let q = r / quarter;
    if q == q.trunc() {
        (r, Some(q as u8 % 4))
    } else {
        (r, None)
    }
}

/// Snap float residue at radian multiples of pi (only for |x| >= 1 so the
/// small-angle region keeps full precision).
fn snap(r: f64, x: f64) -> f64 {
    if x.abs() >= 1.0 && r.abs() < 1e-15 {
        0.0
    } else {
        r
    }
}

fn sin(x: f64, angle: AngleMode) -> f64 {
    match angle.full_turn() {
        Some(full) => match reduce(x, full) {
            (_, Some(q)) => [0.0, 1.0, 0.0, -1.0][q as usize],
            (r, None) => to_radians(r, angle).sin(),
        },
        None => snap(x.sin(), x),
    }
}

fn cos(x: f64, angle: AngleMode) -> f64 {
    match angle.full_turn() {
        Some(full) => match reduce(x, full) {
            (_, Some(q)) => [1.0, 0.0, -1.0, 0.0][q as usize],
            (r, None) => to_radians(r, angle).cos(),
        },
        None => snap(x.cos(), x),
    }
}

fn tan(x: f64, angle: AngleMode) -> Result<f64, CalcError> {
    match angle.full_turn() {
        Some(full) => match reduce(x, full) {
            (_, Some(1)) | (_, Some(3)) => Err(CalcError::Domain("tan undefined here")),
            (_, Some(_)) => Ok(0.0),
            (r, None) => Ok(to_radians(r, angle).tan()),
        },
        None => {
            // Reduce modulo the period before testing for a pole.
            let r = x.rem_euclid(PI);
            if (r - PI / 2.0).abs() < 1e-12 {
                Err(CalcError::Domain("tan undefined here"))
            } else {
                Ok(snap(r.tan(), x))
            }
        }
    }
}

/// n! for integers up to 170, gamma(n+1) for other non-negative values.
fn factorial(x: f64) -> Result<f64, CalcError> {
    if x < 0.0 {
        return Err(CalcError::Domain("n! needs n>=0"));
    }
    if x > 170.0 {
        return Err(CalcError::Overflow);
    }
    if x == x.floor() {
        let mut result = 1.0_f64;
        for i in 2..=(x as u32) {
            result *= i as f64;
        }
        Ok(result)
    } else {
        finite(gamma(x + 1.0))
    }
}

/// Lanczos approximation of the gamma function.
fn gamma(x: f64) -> f64 {
    const G: f64 = 7.0;
    #[allow(clippy::excessive_precision)] // published Lanczos coefficients, kept verbatim
    const C: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_13,
        -176.615_029_162_140_59,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_571_6e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        PI / ((PI * x).sin() * gamma(1.0 - x))
    } else {
        let x = x - 1.0;
        let mut a = C[0];
        for (i, c) in C.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        let t = x + G + 0.5;
        (2.0 * PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
    }
}

/// Binary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    /// Remainder with the sign of the dividend (`-7 % 3 = -1`), not percent.
    Mod,
}

impl Op {
    pub fn symbol(self) -> char {
        match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
            Op::Pow => '^',
            Op::Mod => '%',
        }
    }

    pub fn from_char(c: char) -> Option<Op> {
        Some(match c {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' | '×' => Op::Mul,
            '/' | '÷' => Op::Div,
            '^' => Op::Pow,
            '%' => Op::Mod,
            _ => return None,
        })
    }

    /// `a op b`. Never returns a non-finite value.
    pub fn evaluate(self, a: f64, b: f64) -> Result<f64, CalcError> {
        if !a.is_finite() || !b.is_finite() {
            return Err(CalcError::Overflow);
        }
        let r = match self {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div | Op::Mod => {
                if b == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                if self == Op::Div {
                    a / b
                } else {
                    a % b
                }
            }
            Op::Pow => {
                if a == 0.0 && b < 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                let r = a.powf(b);
                if r.is_nan() {
                    return Err(CalcError::Domain("invalid power"));
                }
                r
            }
        };
        finite(r)
    }
}

/// Calculator errors, shown on screen instead of a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    DivideByZero,
    Domain(&'static str),
    Overflow,
    Parse,
    Syntax,
}

impl CalcError {
    pub fn message(&self) -> &'static str {
        match self {
            CalcError::DivideByZero => "Error: divide by zero",
            CalcError::Domain(msg) => msg,
            CalcError::Overflow => "Error: overflow",
            CalcError::Parse => "Error: can't parse",
            CalcError::Syntax => "Error: syntax",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEG: AngleMode = AngleMode::Degrees;
    const RAD: AngleMode = AngleMode::Radians;
    const GRAD: AngleMode = AngleMode::Gradians;

    #[test]
    fn exact_trig_at_quadrants_in_degrees() {
        // Audit: sin(180) showed 1.22e-16 and cos(90) 6.1e-17.
        assert_eq!(Func::Sin.evaluate(180.0, DEG), Ok(0.0));
        assert_eq!(Func::Cos.evaluate(90.0, DEG), Ok(0.0));
        assert_eq!(Func::Sin.evaluate(90.0, DEG), Ok(1.0));
        assert_eq!(Func::Sin.evaluate(-90.0, DEG), Ok(-1.0));
        assert_eq!(Func::Cos.evaluate(720.0, DEG), Ok(1.0));
        assert_eq!(Func::Sin.evaluate(200.0, GRAD), Ok(0.0));
        let s30 = Func::Sin.evaluate(30.0, DEG).unwrap();
        assert!((s30 - 0.5).abs() < 1e-15);
    }

    #[test]
    fn radian_residue_snapped() {
        assert_eq!(Func::Sin.evaluate(PI, RAD), Ok(0.0));
        // small angles keep precision
        assert_eq!(Func::Sin.evaluate(1e-20, RAD), Ok(1e-20));
    }

    #[test]
    fn tan_poles_reduced_mod_period() {
        // Audit: tan(810000000090) returned ~1e6 instead of an error.
        assert!(Func::Tan.evaluate(90.0, DEG).is_err());
        assert!(Func::Tan.evaluate(810_000_000_090.0, DEG).is_err());
        assert!(Func::Tan.evaluate(-270.0, DEG).is_err());
        assert!(Func::Tan.evaluate(100.0, GRAD).is_err());
        assert!(Func::Tan.evaluate(PI / 2.0, RAD).is_err());
        assert_eq!(Func::Tan.evaluate(180.0, DEG), Ok(0.0));
        let t45 = Func::Tan.evaluate(45.0, DEG).unwrap();
        assert!((t45 - 1.0).abs() < 1e-15);
        // Message does not claim degrees in every mode.
        assert!(!Func::Tan.evaluate(PI / 2.0, RAD).unwrap_err().message().contains('°'));
    }

    #[test]
    fn inverse_trig_in_angle_mode() {
        assert!((Func::Asin.evaluate(1.0, DEG).unwrap() - 90.0).abs() < 1e-12);
        assert!((Func::Acos.evaluate(0.0, GRAD).unwrap() - 100.0).abs() < 1e-12);
        assert!(Func::Asin.evaluate(1.5, DEG).is_err());
    }

    #[test]
    fn overflow_is_an_error_not_infinity() {
        // Audit: 1e308*10 showed infinity and then saved "ans":null.
        assert_eq!(Op::Mul.evaluate(1e308, 10.0), Err(CalcError::Overflow));
        assert_eq!(Op::Add.evaluate(1.7e308, 1.7e308), Err(CalcError::Overflow));
        assert_eq!(Op::Sub.evaluate(-1.7e308, 1.7e308), Err(CalcError::Overflow));
        assert_eq!(Func::Square.evaluate(1e200, DEG), Err(CalcError::Overflow));
        assert_eq!(Func::Cube.evaluate(1e110, DEG), Err(CalcError::Overflow));
        assert_eq!(Func::Sinh.evaluate(1000.0, DEG), Err(CalcError::Overflow));
        assert_eq!(Func::Cosh.evaluate(1000.0, DEG), Err(CalcError::Overflow));
        assert_eq!(Func::Exp.evaluate(1000.0, DEG), Err(CalcError::Overflow));
        assert_eq!(Func::Exp10.evaluate(400.0, DEG), Err(CalcError::Overflow));
        assert_eq!(Op::Pow.evaluate(10.0, 400.0), Err(CalcError::Overflow));
        assert_eq!(Func::Factorial.evaluate(171.0, DEG), Err(CalcError::Overflow));
    }

    #[test]
    fn division_by_zero_reported() {
        assert_eq!(Op::Div.evaluate(1.0, 0.0), Err(CalcError::DivideByZero));
        assert_eq!(Op::Mod.evaluate(1.0, 0.0), Err(CalcError::DivideByZero));
        assert_eq!(Func::Reciprocal.evaluate(0.0, DEG), Err(CalcError::DivideByZero));
        // Audit: 0^-1 reported Overflow.
        assert_eq!(Op::Pow.evaluate(0.0, -1.0), Err(CalcError::DivideByZero));
    }

    #[test]
    fn domain_errors() {
        assert!(Func::Sqrt.evaluate(-1.0, DEG).is_err());
        assert!(Func::Ln.evaluate(0.0, DEG).is_err());
        assert!(Func::Factorial.evaluate(-1.0, DEG).is_err());
        assert!(Op::Pow.evaluate(-8.0, 0.5).is_err());
        assert!(Func::Sin.evaluate(f64::INFINITY, DEG).is_err());
    }

    #[test]
    fn factorial_and_gamma() {
        assert_eq!(Func::Factorial.evaluate(0.0, DEG), Ok(1.0));
        assert_eq!(Func::Factorial.evaluate(5.0, DEG), Ok(120.0));
        assert_eq!(Func::Factorial.evaluate(10.0, DEG), Ok(3_628_800.0));
        // 0.5! = gamma(1.5) = sqrt(pi)/2
        let half = Func::Factorial.evaluate(0.5, DEG).unwrap();
        assert!((half - PI.sqrt() / 2.0).abs() < 1e-12);
    }

    #[test]
    fn modulo_sign_follows_dividend() {
        assert_eq!(Op::Mod.evaluate(-7.0, 3.0), Ok(-1.0));
        assert_eq!(Op::Mod.evaluate(7.0, 3.0), Ok(1.0));
    }

    #[test]
    fn every_alg_insert_names_a_known_function() {
        use Func::*;
        for f in [
            Sin, Cos, Tan, Asin, Acos, Atan, Sinh, Cosh, Tanh, Asinh, Acosh, Atanh, Log, Ln, Log2, Exp, Sqrt,
            Cbrt, Abs, Floor, Ceil, Round, Negate,
        ] {
            let name = f.alg_insert().trim_end_matches('(');
            assert_eq!(Func::from_name(name), Some(f), "{:?}", f);
        }
        assert_eq!(Func::from_name("fact"), Some(Factorial));
        assert_eq!(Func::from_name("exp10"), Some(Exp10));
    }

    #[test]
    fn mode_round_trip() {
        for m in [DEG, RAD, GRAD] {
            assert_eq!(AngleMode::from_u8(m.to_u8()), m);
        }
        let mut b = NumberBase::Decimal;
        for _ in 0..4 {
            assert_eq!(NumberBase::from_u8(b.to_u8()), b);
            b = b.cycle();
        }
        assert_eq!(b, NumberBase::Decimal);
    }
}
