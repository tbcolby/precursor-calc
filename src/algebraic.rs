//! Algebraic (infix) expression parser and evaluator

use crate::functions::{AngleMode, CalcError, Func, Op};
use alloc::string::String;
use alloc::vec::Vec;

/// Token for expression parsing
#[derive(Clone, Debug)]
pub enum Token {
    Number(f64),
    Operator(Op),
    Function(Func),
    OpenParen,
    CloseParen,
    Ans,
}

/// Algebraic expression parser using shunting-yard algorithm
pub struct AlgebraicParser;

impl AlgebraicParser {
    /// Parse expression string into tokens
    pub fn tokenize(input: &str) -> Result<Vec<Token>, CalcError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' => {
                    chars.next();
                }
                '0'..='9' | '.' => {
                    let num = Self::parse_number(&mut chars)?;
                    tokens.push(Token::Number(num));
                }
                '+' => {
                    chars.next();
                    tokens.push(Token::Operator(Op::Add));
                }
                '-' => {
                    chars.next();
                    // Disambiguate unary minus vs subtraction
                    if Self::should_be_unary(&tokens) {
                        // Parse as negative number or unary function
                        if chars.peek().map_or(false, |c| c.is_ascii_digit() || *c == '.') {
                            let num = Self::parse_number(&mut chars)?;
                            tokens.push(Token::Number(-num));
                        } else {
                            tokens.push(Token::Function(Func::Negate));
                        }
                    } else {
                        tokens.push(Token::Operator(Op::Sub));
                    }
                }
                '*' | '×' => {
                    chars.next();
                    tokens.push(Token::Operator(Op::Mul));
                }
                '/' | '÷' => {
                    chars.next();
                    tokens.push(Token::Operator(Op::Div));
                }
                '^' => {
                    chars.next();
                    tokens.push(Token::Operator(Op::Pow));
                }
                '%' => {
                    chars.next();
                    tokens.push(Token::Operator(Op::Mod));
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::OpenParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::CloseParen);
                }
                'a'..='z' | 'A'..='Z' | 'π' => {
                    let name = Self::parse_identifier(&mut chars);
                    let token = Self::match_function_or_constant(&name)?;
                    tokens.push(token);
                }
                _ => {
                    return Err(CalcError::ParseError(alloc::format!(
                        "Unknown character: {}",
                        c
                    )));
                }
            }
        }

        Ok(tokens)
    }

    /// Parse a number from the character stream
    fn parse_number(
        chars: &mut core::iter::Peekable<core::str::Chars>,
    ) -> Result<f64, CalcError> {
        let mut num_str = String::new();
        let mut has_decimal = false;
        let mut has_exponent = false;

        while let Some(&c) = chars.peek() {
            match c {
                '0'..='9' => {
                    num_str.push(c);
                    chars.next();
                }
                '.' if !has_decimal && !has_exponent => {
                    num_str.push(c);
                    has_decimal = true;
                    chars.next();
                }
                'e' | 'E' if !has_exponent => {
                    num_str.push(c);
                    has_exponent = true;
                    chars.next();
                    // Handle optional sign after exponent
                    if let Some(&sign) = chars.peek() {
                        if sign == '+' || sign == '-' {
                            num_str.push(sign);
                            chars.next();
                        }
                    }
                }
                _ => break,
            }
        }

        num_str
            .parse::<f64>()
            .map_err(|_| CalcError::ParseError(alloc::format!("Invalid number: {}", num_str)))
    }

    /// Parse an identifier (function name or constant)
    fn parse_identifier(chars: &mut core::iter::Peekable<core::str::Chars>) -> String {
        let mut name = String::new();

        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() || c == 'π' || (c.is_ascii_digit() && !name.is_empty()) {
                name.push(c);
                chars.next();
            } else {
                break;
            }
        }

        name
    }

    /// Match identifier to function or constant
    fn match_function_or_constant(name: &str) -> Result<Token, CalcError> {
        let lower = name.to_lowercase();

        // Check for Ans
        if lower == "ans" {
            return Ok(Token::Ans);
        }

        // Check for π
        if name == "π" || lower == "pi" {
            return Ok(Token::Number(core::f64::consts::PI));
        }

        // Check for e constant
        if lower == "e" && name.len() == 1 {
            return Ok(Token::Number(core::f64::consts::E));
        }

        // Check for function
        if let Some(func) = Func::from_name(&lower) {
            return Ok(Token::Function(func));
        }

        Err(CalcError::ParseError(alloc::format!(
            "Unknown identifier: {}",
            name
        )))
    }

    /// Should the next minus be treated as unary?
    fn should_be_unary(tokens: &[Token]) -> bool {
        match tokens.last() {
            None => true,
            Some(Token::Operator(_)) => true,
            Some(Token::OpenParen) => true,
            Some(Token::Function(_)) => true,
            _ => false,
        }
    }

    /// Convert infix tokens to postfix using shunting-yard algorithm
    pub fn to_postfix(tokens: Vec<Token>) -> Result<Vec<Token>, CalcError> {
        let mut output: Vec<Token> = Vec::new();
        let mut op_stack: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(_) | Token::Ans => output.push(token),
                Token::Function(_) => op_stack.push(token),
                Token::Operator(op) => {
                    while let Some(top) = op_stack.last() {
                        match top {
                            Token::Operator(top_op) => {
                                if (op.is_left_assoc() && op.precedence() <= top_op.precedence())
                                    || op.precedence() < top_op.precedence()
                                {
                                    output.push(op_stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            Token::Function(_) => {
                                // Functions have higher precedence
                                break;
                            }
                            _ => break,
                        }
                    }
                    op_stack.push(token);
                }
                Token::OpenParen => op_stack.push(token),
                Token::CloseParen => {
                    let mut found_paren = false;
                    while let Some(top) = op_stack.pop() {
                        if matches!(top, Token::OpenParen) {
                            found_paren = true;
                            break;
                        }
                        output.push(top);
                    }
                    if !found_paren {
                        return Err(CalcError::SyntaxError("Mismatched parentheses".into()));
                    }
                    // Pop function if present after paren
                    if let Some(Token::Function(_)) = op_stack.last() {
                        output.push(op_stack.pop().unwrap());
                    }
                }
            }
        }

        // Pop remaining operators
        while let Some(top) = op_stack.pop() {
            if matches!(top, Token::OpenParen) {
                return Err(CalcError::SyntaxError("Mismatched parentheses".into()));
            }
            output.push(top);
        }

        Ok(output)
    }

    /// Evaluate postfix expression
    pub fn evaluate(
        postfix: Vec<Token>,
        ans: f64,
        angle_mode: AngleMode,
    ) -> Result<f64, CalcError> {
        let mut stack: Vec<f64> = Vec::new();

        for token in postfix {
            match token {
                Token::Number(n) => stack.push(n),
                Token::Ans => stack.push(ans),
                Token::Operator(op) => {
                    if stack.len() < 2 {
                        return Err(CalcError::SyntaxError("Not enough operands".into()));
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = op.evaluate(a, b)?;
                    stack.push(result);
                }
                Token::Function(func) => {
                    if func.is_constant() {
                        let result = func.evaluate(0.0, angle_mode)?;
                        stack.push(result);
                    } else {
                        if stack.is_empty() {
                            return Err(CalcError::SyntaxError("Not enough operands".into()));
                        }
                        let x = stack.pop().unwrap();
                        let result = func.evaluate(x, angle_mode)?;
                        stack.push(result);
                    }
                }
                Token::OpenParen | Token::CloseParen => {
                    // Should not appear in postfix
                    return Err(CalcError::SyntaxError("Unexpected parenthesis".into()));
                }
            }
        }

        if stack.len() != 1 {
            return Err(CalcError::SyntaxError("Invalid expression".into()));
        }

        Ok(stack.pop().unwrap())
    }

    /// Parse and evaluate an expression in one step
    pub fn calculate(input: &str, ans: f64, angle_mode: AngleMode) -> Result<f64, CalcError> {
        let tokens = Self::tokenize(input)?;
        let postfix = Self::to_postfix(tokens)?;
        Self::evaluate(postfix, ans, angle_mode)
    }
}

/// Algebraic mode state machine
pub struct AlgebraicState {
    /// Current input buffer
    input: String,
    /// Last result (Ans)
    ans: f64,
    /// Error message if any
    error: Option<String>,
}

impl Default for AlgebraicState {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgebraicState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            ans: 0.0,
            error: None,
        }
    }

    /// Get current input
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Get last answer
    pub fn ans(&self) -> f64 {
        self.ans
    }

    /// Get error if any
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Add character to input
    pub fn push(&mut self, c: char) {
        self.error = None;
        self.input.push(c);
    }

    /// Add string to input
    pub fn push_str(&mut self, s: &str) {
        self.error = None;
        self.input.push_str(s);
    }

    /// Remove last character
    pub fn backspace(&mut self) {
        self.error = None;
        self.input.pop();
    }

    /// Clear input
    pub fn clear(&mut self) {
        self.input.clear();
        self.error = None;
    }

    /// Clear all (input and ans)
    pub fn clear_all(&mut self) {
        self.input.clear();
        self.ans = 0.0;
        self.error = None;
    }

    /// Set ans directly (for memory recall etc)
    pub fn set_ans(&mut self, value: f64) {
        self.ans = value;
    }

    /// Evaluate current expression
    pub fn evaluate(&mut self, angle_mode: AngleMode) -> Option<f64> {
        if self.input.is_empty() {
            return Some(self.ans);
        }

        match AlgebraicParser::calculate(&self.input, self.ans, angle_mode) {
            Ok(result) => {
                self.ans = result;
                self.error = None;
                Some(result)
            }
            Err(e) => {
                self.error = Some(String::from(e.message()));
                None
            }
        }
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = AlgebraicParser::calculate("2+3", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 5.0);

        let result = AlgebraicParser::calculate("10-4", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 6.0);

        let result = AlgebraicParser::calculate("3*4", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 12.0);

        let result = AlgebraicParser::calculate("15/3", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_precedence() {
        // 2+3*4 should be 14, not 20
        let result = AlgebraicParser::calculate("2+3*4", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 14.0);

        // (2+3)*4 should be 20
        let result = AlgebraicParser::calculate("(2+3)*4", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_functions() {
        let result = AlgebraicParser::calculate("sqrt(16)", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 4.0);

        let result = AlgebraicParser::calculate("sin(90)", 0.0, AngleMode::Degrees).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_unary_minus() {
        let result = AlgebraicParser::calculate("-5", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, -5.0);

        let result = AlgebraicParser::calculate("3+-5", 0.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, -2.0);
    }

    #[test]
    fn test_ans() {
        let result = AlgebraicParser::calculate("ans+10", 5.0, AngleMode::Degrees).unwrap();
        assert_eq!(result, 15.0);
    }
}
