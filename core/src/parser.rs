//! Infix expression parser and evaluator (recursive descent).
//!
//! Grammar, lowest to highest precedence:
//!
//! ```text
//! expr    := term (('+' | '-') term)*
//! term    := unary (('*' | '/' | '%' | <implicit>) unary)*
//! unary   := ('-' | '+') unary | power
//! power   := postfix ('^' unary)?          right-associative: 2^3^2 = 2^9
//! postfix := primary '!'*
//! primary := number | 'ans' | constant | '(' expr ')' | function unary
//! ```
//!
//! So unary minus binds looser than `^` (`-2^2 = -4`) and tighter than
//! `*`, and a function applies to the operand right after it, not to the rest
//! of the expression (`sin 30+1 = sin(30)+1`, `-sqrt(4)+1 = -1`).
//! Implicit multiplication covers `2pi`, `2(3)`, `(1+1)(2)` and `2sqrt(9)`.

use alloc::vec::Vec;

use crate::functions::{finite, AngleMode, CalcError, Func, Op};

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Op(Op),
    Bang,
    Open,
    Close,
    Ans,
    Func(Func),
}

fn tokenize(input: &str) -> Result<Vec<Tok>, CalcError> {
    let mut out = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                let mut seen_dot = false;
                while i < chars.len() && (chars[i].is_ascii_digit() || (chars[i] == '.' && !seen_dot)) {
                    seen_dot |= chars[i] == '.';
                    i += 1;
                }
                // Exponent: only when 'e' is followed by a digit or a signed digit,
                // so `2e` stays "2 times e".
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    let mut j = i + 1;
                    if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                        j += 1;
                    }
                    if j < chars.len() && chars[j].is_ascii_digit() {
                        i = j;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                let s: alloc::string::String = chars[start..i].iter().collect();
                if matches!(out.last(), Some(Tok::Num(_))) {
                    return Err(CalcError::Parse); // "1..2", "1.2.3"
                }
                out.push(Tok::Num(s.parse::<f64>().map_err(|_| CalcError::Parse)?));
            }
            '!' => {
                out.push(Tok::Bang);
                i += 1;
            }
            '(' | '[' => {
                out.push(Tok::Open);
                i += 1;
            }
            ')' | ']' => {
                out.push(Tok::Close);
                i += 1;
            }
            'π' => {
                out.push(Tok::Func(Func::Pi));
                i += 1;
            }
            c if c.is_ascii_alphabetic() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                    i += 1;
                }
                let name: alloc::string::String = chars[start..i].iter().collect::<alloc::string::String>().to_lowercase();
                out.push(match name.as_str() {
                    "ans" => Tok::Ans,
                    n => Tok::Func(Func::from_name(n).ok_or(CalcError::Parse)?),
                });
            }
            c => match Op::from_char(c) {
                Some(op) => {
                    out.push(Tok::Op(op));
                    i += 1;
                }
                None => return Err(CalcError::Parse),
            },
        }
    }
    Ok(out)
}

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
    ans: f64,
    angle: AngleMode,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> { self.toks.get(self.pos) }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> Result<f64, CalcError> {
        let mut v = self.term()?;
        while let Some(Tok::Op(op @ (Op::Add | Op::Sub))) = self.peek().cloned() {
            self.pos += 1;
            let rhs = self.term()?;
            v = op.evaluate(v, rhs)?;
        }
        Ok(v)
    }

    fn starts_operand(t: Option<&Tok>) -> bool { matches!(t, Some(Tok::Num(_) | Tok::Ans | Tok::Func(_) | Tok::Open)) }

    fn term(&mut self) -> Result<f64, CalcError> {
        let mut v = self.unary()?;
        loop {
            match self.peek().cloned() {
                Some(Tok::Op(op @ (Op::Mul | Op::Div | Op::Mod))) => {
                    self.pos += 1;
                    let rhs = self.unary()?;
                    v = op.evaluate(v, rhs)?;
                }
                t if Self::starts_operand(t.as_ref()) => {
                    let rhs = self.unary()?;
                    v = Op::Mul.evaluate(v, rhs)?;
                }
                _ => return Ok(v),
            }
        }
    }

    fn unary(&mut self) -> Result<f64, CalcError> {
        match self.peek() {
            Some(Tok::Op(Op::Sub)) => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some(Tok::Op(Op::Add)) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<f64, CalcError> {
        let base = self.postfix()?;
        if let Some(Tok::Op(Op::Pow)) = self.peek() {
            self.pos += 1;
            let exp = self.unary()?;
            return Op::Pow.evaluate(base, exp);
        }
        Ok(base)
    }

    fn postfix(&mut self) -> Result<f64, CalcError> {
        let mut v = self.primary()?;
        while let Some(Tok::Bang) = self.peek() {
            self.pos += 1;
            v = Func::Factorial.evaluate(v, self.angle)?;
        }
        Ok(v)
    }

    fn primary(&mut self) -> Result<f64, CalcError> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(n),
            Some(Tok::Ans) => Ok(self.ans),
            Some(Tok::Open) => {
                let v = self.expr()?;
                match self.next() {
                    Some(Tok::Close) => Ok(v),
                    _ => Err(CalcError::Syntax),
                }
            }
            Some(Tok::Func(f)) if f.is_constant() => f.evaluate(0.0, self.angle),
            Some(Tok::Func(f)) => {
                let arg = self.unary()?;
                f.evaluate(arg, self.angle)
            }
            _ => Err(CalcError::Syntax),
        }
    }
}

/// Parse and evaluate `input`. `ans` is the value of the `ans` keyword.
pub fn calculate(input: &str, ans: f64, angle: AngleMode) -> Result<f64, CalcError> {
    let toks = tokenize(input)?;
    if toks.is_empty() {
        return Err(CalcError::Syntax);
    }
    let mut p = Parser { toks: &toks, pos: 0, ans, angle };
    let v = p.expr()?;
    if p.pos != toks.len() {
        return Err(CalcError::Syntax);
    }
    finite(v)
}

/// True if an expression starts with a binary operator that should apply to
/// the previous answer (`+2` means `ans+2`). A leading `-` stays a negation.
pub fn continues_ans(input: &str) -> bool {
    matches!(input.trim_start().chars().next(), Some('+' | '*' | '×' | '/' | '÷' | '^' | '%' | '!'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(s: &str) -> Result<f64, CalcError> { calculate(s, 0.0, AngleMode::Degrees) }

    fn calc_ans(s: &str, ans: f64) -> f64 { calculate(s, ans, AngleMode::Degrees).unwrap() }

    #[test]
    fn arithmetic_and_precedence() {
        assert_eq!(calc("2+3"), Ok(5.0));
        assert_eq!(calc("10-4"), Ok(6.0));
        assert_eq!(calc("2+3*4"), Ok(14.0));
        assert_eq!(calc("(2+3)*4"), Ok(20.0));
        assert_eq!(calc("10-4-3"), Ok(3.0));
        assert_eq!(calc("100/10/5"), Ok(2.0));
        assert_eq!(calc("2^3^2"), Ok(512.0));
        assert_eq!(calc("2 × 3 ÷ 4"), Ok(1.5));
    }

    #[test]
    fn unary_minus_binds_looser_than_power() {
        // Audit: -2^2 gave 4 while -ans^2 gave -25.
        assert_eq!(calc("-2^2"), Ok(-4.0));
        assert_eq!(calc_ans("-ans^2", 5.0), -25.0);
        assert_eq!(calc("(-2)^2"), Ok(4.0));
        assert_eq!(calc("2^-1"), Ok(0.5));
        assert_eq!(calc("3+-5"), Ok(-2.0));
        assert_eq!(calc("3--5"), Ok(8.0));
        assert_eq!(calc("-5"), Ok(-5.0));
        assert_eq!(calc("2*-3"), Ok(-6.0));
    }

    #[test]
    fn prefix_functions_apply_to_next_operand_only() {
        // Audit: -ans+1 (ans=5) gave -6, -pi+1 gave -4.14, -sqrt(4)+1 gave -3,
        // sin 30+1 gave sin(31).
        assert_eq!(calc_ans("-ans+1", 5.0), -4.0);
        assert!((calc("-pi+1").unwrap() - (1.0 - core::f64::consts::PI)).abs() < 1e-12);
        assert_eq!(calc("-sqrt(4)+1"), Ok(-1.0));
        assert!((calc("sin 30+1").unwrap() - 1.5).abs() < 1e-12);
        assert!((calc("sin(30)+1").unwrap() - 1.5).abs() < 1e-12);
        assert_eq!(calc("neg(3)+1"), Ok(-2.0));
        assert_eq!(calc("sqrt 16 * 2"), Ok(8.0));
    }

    #[test]
    fn menu_insertions_parse() {
        // Audit: Square inserted "x²(" and factorial was unusable in ALG mode.
        assert_eq!(calc("3^2"), Ok(9.0));
        assert_eq!(calc("2^3"), Ok(8.0));
        assert_eq!(calc("5!"), Ok(120.0));
        assert_eq!(calc("3!!"), Ok(720.0));
        assert_eq!(calc("fact(4)"), Ok(24.0));
        assert_eq!(calc("1/(4)"), Ok(0.25));
        assert_eq!(calc("10^(2)"), Ok(100.0));
        assert_eq!(calc("exp10(3)"), Ok(1000.0));
        assert_eq!(calc("-3!"), Ok(-6.0));
    }

    #[test]
    fn implicit_multiplication() {
        assert!((calc("2pi").unwrap() - 2.0 * core::f64::consts::PI).abs() < 1e-12);
        assert_eq!(calc("2(3)"), Ok(6.0));
        assert_eq!(calc("(1+1)(2+1)"), Ok(6.0));
        assert_eq!(calc("2sqrt(9)"), Ok(6.0));
        assert_eq!(calc_ans("3ans", 2.0), 6.0);
        assert!((calc("2e").unwrap() - 2.0 * core::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn scientific_literals() {
        assert_eq!(calc("1e3"), Ok(1000.0));
        assert_eq!(calc("1.5e-3"), Ok(0.0015));
        assert_eq!(calc("2E+2"), Ok(200.0));
        assert_eq!(calc("1e300"), Ok(1e300));
    }

    #[test]
    fn errors_not_panics() {
        assert_eq!(calc("1/0"), Err(CalcError::DivideByZero));
        assert_eq!(calc("1e308*10"), Err(CalcError::Overflow));
        assert_eq!(calc("(1+2"), Err(CalcError::Syntax));
        assert_eq!(calc("1+2)"), Err(CalcError::Syntax));
        assert_eq!(calc("2+"), Err(CalcError::Syntax));
        assert_eq!(calc(""), Err(CalcError::Syntax));
        assert_eq!(calc("foo(2)"), Err(CalcError::Parse));
        assert_eq!(calc("2#3"), Err(CalcError::Parse));
        assert_eq!(calc("1..2"), Err(CalcError::Parse));
        assert!(calc("sqrt(-1)").is_err());
        assert_eq!(calc("NaN"), Err(CalcError::Parse));
        assert_eq!(calc("inf"), Err(CalcError::Parse));
    }

    #[test]
    fn leading_operator_continues_ans() {
        assert!(continues_ans("+2"));
        assert!(continues_ans("*3"));
        assert!(continues_ans("!"));
        assert!(!continues_ans("-2"));
        assert!(!continues_ans("2+2"));
    }

    #[test]
    fn functions_and_constants() {
        assert_eq!(calc("sqrt(16)"), Ok(4.0));
        assert_eq!(calc("sin(90)"), Ok(1.0));
        assert_eq!(calc("abs(-3)"), Ok(3.0));
        assert_eq!(calc("log(1000)"), Ok(3.0));
        assert_eq!(calc("PI"), calc("pi"));
        assert_eq!(calc("π"), calc("pi"));
    }
}
