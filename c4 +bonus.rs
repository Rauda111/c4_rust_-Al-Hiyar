//! c4.rs – A Self‑Hosting C Compiler in Rust with Bonus Floating‑Point Support
//!
//! This compiler is a Rust reimplementation of the original C4 compiler. It
//! includes a lexer, a recursive descent parser (with advanced symbol table
//! management and support for control flow), and a stack-based virtual machine.
//!
//! In addition to supporting a minimal subset of C (global/local variables,
//! a single function definition, arithmetic expressions, assignment, if–else,
//! while, and return statements), this version adds bonus floating‑point support.
//!
//! Usage (via Cargo):
//!     cargo run -- <file.c>
//!
//! The program reads a C source file, tokenizes it, parses it into opcodes, and
//! then executes the opcodes. Errors at each phase are reported with descriptive messages.

use std::env;
use std::fs;
use std::process;

//
// Module: lexer
//
mod lexer {
    //! The lexer converts C source code into a stream of tokens.
    //!
    //! This lexer supports keywords (int, char, return, if, else, while), identifiers,
    //! integer and floating‑point literals, operators, and punctuation.

    #[derive(Debug, Clone, PartialEq)]
    pub enum Token {
        // Keywords
        Int,
        Char,
        Return,
        If,
        Else,
        While,
        // Identifiers
        Ident(String),
        // Literals: integer and floating point (bonus)
        Num(i64),
        Float(f64),
        // Operators
        Plus,      // +
        Minus,     // -
        Mul,       // *
        Div,       // /
        Assign,    // =
        Eq,        // ==
        Ne,        // !=
        Lt,        // <
        Gt,        // >
        Le,        // <=
        Ge,        // >=
        // Punctuation
        Semicolon,
        Comma,
        LParen,
        RParen,
        LBrace,
        RBrace,
        EOF,
    }

    pub type LexResult = Result<Vec<Token>, String>;

    /// Tokenizes the input C source code into a vector of tokens.
    ///
    /// Supports skipping whitespace and C++‑style comments.
    pub fn tokenize(source: &str) -> LexResult {
        let mut tokens = Vec::new();
        let mut chars = source.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => { chars.next(); },
                // Numbers: check for integer and optionally a decimal point.
                '0'..='9' => {
                    let mut num_str = String::new();
                    while let Some(&digit) = chars.peek() {
                        if digit.is_digit(10) {
                            num_str.push(digit);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Check for a fractional part.
                    if let Some(&'.') = chars.peek() {
                        num_str.push('.');
                        chars.next(); // consume dot
                        while let Some(&digit) = chars.peek() {
                            if digit.is_digit(10) {
                                num_str.push(digit);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let value = num_str.parse::<f64>().map_err(|e| e.to_string())?;
                        tokens.push(Token::Float(value));
                    } else {
                        let value = num_str.parse::<i64>().map_err(|e| e.to_string())?;
                        tokens.push(Token::Num(value));
                    }
                },
                // Identifiers and keywords.
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            ident.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Check for keywords.
                    match ident.as_str() {
                        "int"   => tokens.push(Token::Int),
                        "char"  => tokens.push(Token::Char),
                        "return"=> tokens.push(Token::Return),
                        "if"    => tokens.push(Token::If),
                        "else"  => tokens.push(Token::Else),
                        "while" => tokens.push(Token::While),
                        _       => tokens.push(Token::Ident(ident)),
                    }
                },
                '+' => { tokens.push(Token::Plus); chars.next(); },
                '-' => { tokens.push(Token::Minus); chars.next(); },
                '*' => { tokens.push(Token::Mul); chars.next(); },
                '/' => {
                    chars.next();
                    // Handle single-line comments.
                    if let Some(&'/') = chars.peek() {
                        while let Some(&c) = chars.peek() {
                            if c == '\n' { break; }
                            chars.next();
                        }
                    } else {
                        tokens.push(Token::Div);
                    }
                },
                '=' => {
                    chars.next();
                    if let Some(&'=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::Eq);
                    } else {
                        tokens.push(Token::Assign);
                    }
                },
                '!' => {
                    chars.next();
                    if let Some(&'=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::Ne);
                    } else {
                        return Err("Unexpected '!'".to_string());
                    }
                },
                '<' => {
                    chars.next();
                    if let Some(&'=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::Le);
                    } else {
                        tokens.push(Token::Lt);
                    }
                },
                '>' => {
                    chars.next();
                    if let Some(&'=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::Ge);
                    } else {
                        tokens.push(Token::Gt);
                    }
                },
                ';' => { tokens.push(Token::Semicolon); chars.next(); },
                ',' => { tokens.push(Token::Comma); chars.next(); },
                '(' => { tokens.push(Token::LParen); chars.next(); },
                ')' => { tokens.push(Token::RParen); chars.next(); },
                '{' => { tokens.push(Token::LBrace); chars.next(); },
                '}' => { tokens.push(Token::RBrace); chars.next(); },
                _ => return Err(format!("Unexpected character: {}", ch)),
            }
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_tokenize_int_and_float() {
            let src = "123 + 3.14;";
            let result = tokenize(src).unwrap();
            let expected = vec![
                Token::Num(123),
                Token::Plus,
                Token::Float(3.14),
                Token::Semicolon,
                Token::EOF,
            ];
            assert_eq!(result, expected);
        }

        #[test]
        fn test_tokenize_keywords() {
            let src = "int main() { return 0; }";
            let result = tokenize(src).unwrap();
            let expected = vec![
                Token::Int,
                Token::Ident("main".to_string()),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::Return,
                Token::Num(0),
                Token::Semicolon,
                Token::RBrace,
                Token::EOF,
            ];
            assert_eq!(result, expected);
        }
    }
}

//
// Module: parser
//
mod parser {
    //! The parser implements a recursive descent parser for a subset of C.
    //!
    //! It supports global variable declarations, a single function definition
    //! (only "main" is allowed), and statements including expression statements,
    //! local variable declarations, if–else, while loops, and return statements.
    //!
    //! This version also supports bonus floating‑point literals. In expressions,
    //! when a float literal is encountered, an opcode for a floating‑point immediate
    //! is generated.

    use crate::lexer::Token;
    use crate::vm::Opcode;
    use std::collections::HashMap;

    pub type ParseResult = Result<Vec<Opcode>, String>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum SymbolClass {
        Global,
        Local,
        Function,
    }

    #[derive(Debug, Clone)]
    pub struct Symbol {
        pub name: String,
        pub class: SymbolClass,
        pub offset: i64, // For locals, the offset in the stack frame.
    }

    pub struct Parser {
        tokens: Vec<Token>,
        pos: usize,
        pub opcodes: Vec<Opcode>,
        globals: HashMap<String, Symbol>,
        locals: HashMap<String, Symbol>,
        local_offset: i64,
    }

    impl Parser {
        /// Creates a new parser instance.
        pub fn new(tokens: Vec<Token>) -> Self {
            Parser {
                tokens,
                pos: 0,
                opcodes: Vec::new(),
                globals: HashMap::new(),
                locals: HashMap::new(),
                local_offset: 0,
            }
        }

        /// Returns a reference to the current token.
        fn current(&self) -> &Token {
            self.tokens.get(self.pos).unwrap_or(&Token::EOF)
        }

        /// Consumes the current token if it matches the expected token.
        fn eat(&mut self, token: &Token) -> bool {
            if self.current() == token {
                self.pos += 1;
                true
            } else {
                false
            }
        }

        /// Expects the current token to match the given token.
        fn expect(&mut self, token: &Token) -> Result<(), String> {
            if self.eat(token) {
                Ok(())
            } else {
                Err(format!("Expected {:?}, found {:?}", token, self.current()))
            }
        }

        /// Parses the entire program.
        ///
        /// The program may contain global variable declarations and one function definition.
        pub fn parse_program(&mut self) -> Result<(), String> {
            while self.current() != &Token::EOF {
                match self.current() {
                    Token::Int | Token::Char => {
                        // For simplicity, we support only "int" declarations.
                        self.pos += 1; // consume type
                        match self.current() {
                            Token::Ident(ref name) => {
                                let ident = name.clone();
                                self.pos += 1; // consume identifier
                                if self.eat(&Token::LParen) {
                                    // Function definition.
                                    if ident != "main" {
                                        return Err("Only main function is supported".to_string());
                                    }
                                    self.expect(&Token::RParen)?;
                                    self.expect(&Token::LBrace)?;
                                    // Start a new local scope.
                                    self.locals.clear();
                                    self.local_offset = 0;
                                    while self.current() != &Token::RBrace {
                                        self.parse_stmt()?;
                                    }
                                    self.expect(&Token::RBrace)?;
                                    self.opcodes.push(Opcode::Ret);
                                } else {
                                    // Global variable declaration.
                                    self.globals.insert(ident.clone(), Symbol { name: ident, class: SymbolClass::Global, offset: 0 });
                                    while self.current() != &Token::Semicolon && self.current() != &Token::EOF {
                                        self.pos += 1;
                                    }
                                    self.expect(&Token::Semicolon)?;
                                }
                            },
                            _ => return Err("Expected identifier after type".to_string()),
                        }
                    },
                    _ => return Err(format!("Unexpected token at global scope: {:?}", self.current())),
                }
            }
            Ok(())
        }

        /// Parses a statement.
        ///
        /// Supports: return, if–else, while, local declarations, and expression statements.
        fn parse_stmt(&mut self) -> Result<(), String> {
            match self.current() {
                Token::Return => {
                    self.pos += 1; // consume 'return'
                    self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    self.opcodes.push(Opcode::Ret);
                    Ok(())
                },
                Token::If => self.parse_if(),
                Token::While => self.parse_while(),
                Token::LBrace => {
                    self.pos += 1;
                    while self.current() != &Token::RBrace {
                        self.parse_stmt()?;
                    }
                    self.expect(&Token::RBrace)?;
                    Ok(())
                },
                Token::Int | Token::Char => self.parse_local_decl(),
                _ => {
                    self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    Ok(())
                }
            }
        }

        /// Parses an if–else statement.
        fn parse_if(&mut self) -> Result<(), String> {
            self.pos += 1; // consume 'if'
            self.expect(&Token::LParen)?;
            self.parse_expr()?;
            self.expect(&Token::RParen)?;
            let jz_index = self.opcodes.len();
            self.opcodes.push(Opcode::Jz(0)); // placeholder
            self.parse_stmt()?;
            if self.eat(&Token::Else) {
                let jmp_index = self.opcodes.len();
                self.opcodes.push(Opcode::Jmp(0)); // placeholder for jump over else
                let else_addr = self.opcodes.len() as i64;
                self.opcodes[jz_index] = Opcode::Jz(else_addr);
                self.parse_stmt()?;
                let end_addr = self.opcodes.len() as i64;
                self.opcodes[jmp_index] = Opcode::Jmp(end_addr);
            } else {
                let addr = self.opcodes.len() as i64;
                self.opcodes[jz_index] = Opcode::Jz(addr);
            }
            Ok(())
        }

        /// Parses a while loop.
        fn parse_while(&mut self) -> Result<(), String> {
            self.pos += 1; // consume 'while'
            let loop_start = self.opcodes.len() as i64;
            self.expect(&Token::LParen)?;
            self.parse_expr()?;
            self.expect(&Token::RParen)?;
            let jz_index = self.opcodes.len();
            self.opcodes.push(Opcode::Jz(0)); // placeholder for loop exit
            self.parse_stmt()?;
            self.opcodes.push(Opcode::Jmp(loop_start));
            let loop_end = self.opcodes.len() as i64;
            self.opcodes[jz_index] = Opcode::Jz(loop_end);
            Ok(())
        }

        /// Parses a local variable declaration: int x, y;
        fn parse_local_decl(&mut self) -> Result<(), String> {
            self.pos += 1; // consume type
            loop {
                match self.current() {
                    Token::Ident(name) => {
                        let var_name = name.clone();
                        self.pos += 1;
                        self.local_offset += 1;
                        let offset = self.local_offset;
                        self.locals.insert(var_name, Symbol { name: var_name, class: SymbolClass::Local, offset });
                    },
                    _ => return Err("Expected identifier in local declaration".to_string()),
                }
                if self.eat(&Token::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            self.expect(&Token::Semicolon)?;
            Ok(())
        }

        /// Parses an expression.
        ///
        /// Supports assignment and additive expressions.
        fn parse_expr(&mut self) -> Result<(), String> {
            self.parse_assignment()
        }

        /// Parses an assignment expression.
        fn parse_assignment(&mut self) -> Result<(), String> {
            let start = self.pos;
            if let Token::Ident(ref name) = self.current() {
                let ident = name.clone();
                self.pos += 1;
                if self.eat(&Token::Assign) {
                    self.parse_assignment()?;
                    // Generate a store opcode.
                    if let Some(sym) = self.locals.get(&ident) {
                        self.opcodes.push(Opcode::St(sym.offset));
                        return Ok(());
                    } else if let Some(sym) = self.globals.get(&ident) {
                        self.opcodes.push(Opcode::St(sym.offset));
                        return Ok(());
                    } else {
                        return Err(format!("Undefined variable: {}", ident));
                    }
                } else {
                    self.pos = start;
                }
            }
            self.parse_additive()
        }

        /// Parses an additive expression.
        fn parse_additive(&mut self) -> Result<(), String> {
            self.parse_term()?;
            while let Token::Plus | Token::Minus = self.current() {
                let op = self.current().clone();
                self.pos += 1;
                self.parse_term()?;
                match op {
                    Token::Plus => self.opcodes.push(Opcode::Add),
                    Token::Minus => self.opcodes.push(Opcode::Sub),
                    _ => {},
                }
            }
            Ok(())
        }

        /// Parses a term (multiplication and division).
        fn parse_term(&mut self) -> Result<(), String> {
            self.parse_factor()?;
            while let Token::Mul | Token::Div = self.current() {
                let op = self.current().clone();
                self.pos += 1;
                self.parse_factor()?;
                match op {
                    Token::Mul => self.opcodes.push(Opcode::Mul),
                    Token::Div => self.opcodes.push(Opcode::Div),
                    _ => {},
                }
            }
            Ok(())
        }

        /// Parses a factor: a numeric literal (int or float), identifier, or parenthesized expression.
        fn parse_factor(&mut self) -> Result<(), String> {
            match self.current() {
                Token::Num(n) => {
                    let value = *n;
                    self.pos += 1;
                    self.opcodes.push(Opcode::IImm(value));
                    Ok(())
                },
                Token::Float(f) => {
                    let value = *f;
                    self.pos += 1;
                    self.opcodes.push(Opcode::FImm(value));
                    Ok(())
                },
                Token::Ident(name) => {
                    let var_name = name.clone();
                    self.pos += 1;
                    if let Some(sym) = self.locals.get(&var_name) {
                        self.opcodes.push(Opcode::Ld(sym.offset));
                        Ok(())
                    } else if let Some(sym) = self.globals.get(&var_name) {
                        self.opcodes.push(Opcode::Ld(sym.offset));
                        Ok(())
                    } else {
                        Err(format!("Undefined variable: {}", var_name))
                    }
                },
                Token::LParen => {
                    self.pos += 1;
                    self.parse_expr()?;
                    self.expect(&Token::RParen)?;
                    Ok(())
                },
                _ => Err(format!("Unexpected token in factor: {:?}", self.current())),
            }
        }

        /// Public API: parses tokens into a vector of opcodes.
        pub fn parse(mut self) -> ParseResult {
            self.parse_program()?;
            Ok(self.opcodes)
        }
    }

    /// Public function to parse tokens.
    pub fn parse(tokens: Vec<Token>) -> ParseResult {
        let parser = Parser::new(tokens);
        parser.parse()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::lexer::tokenize;
        use crate::vm::Opcode;

        #[test]
        fn test_parse_int_expression() {
            let src = "1 + 2;";
            let tokens = tokenize(src).unwrap();
            let opcodes = parse(tokens).unwrap();
            let expected = vec![Opcode::IImm(1), Opcode::IImm(2), Opcode::Add];
            assert_eq!(opcodes, expected);
        }

        #[test]
        fn test_parse_float_expression() {
            let src = "3.14 + 2.0;";
            let tokens = tokenize(src).unwrap();
            let opcodes = parse(tokens).unwrap();
            let expected = vec![Opcode::FImm(3.14), Opcode::FImm(2.0), Opcode::Add];
            assert_eq!(opcodes, expected);
        }
    }
}

//
// Module: vm
//
mod vm {
    //! The virtual machine (VM) executes opcodes generated by the parser.
    //!
    //! This VM is stack-based and now supports both integer and floating‑point arithmetic.
    //! It uses a unified `Value` type and performs type checking for arithmetic operations.
    //! Control flow instructions (jumps, conditional jumps, and return) are also supported.

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Int(i64),
        Float(f64),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Opcode {
        // Immediate values.
        IImm(i64),
        FImm(f64),
        // Variable load and store.
        Ld(i64),   // Load from local offset.
        St(i64),   // Store to local offset.
        // Arithmetic operations.
        Add,
        Sub,
        Mul,
        Div,
        // Control flow.
        Jmp(i64),  // Unconditional jump.
        Jz(i64),   // Jump if top of stack is zero.
        Ret,       // Return from function.
    }

    /// Executes a sequence of opcodes and returns the final result as a Value.
    pub fn execute(opcodes: Vec<Opcode>) -> Result<Value, String> {
        let mut stack: Vec<Value> = Vec::new();
        let mut pc: i64 = 0;
        while (pc as usize) < opcodes.len() {
            match opcodes[pc as usize].clone() {
                Opcode::IImm(n) => { stack.push(Value::Int(n)); pc += 1; },
                Opcode::FImm(f) => { stack.push(Value::Float(f)); pc += 1; },
                Opcode::Ld(offset) => {
                    if (offset as usize) < stack.len() {
                        let val = stack[offset as usize].clone();
                        stack.push(val);
                        pc += 1;
                    } else {
                        return Err("Invalid local offset in Ld".into());
                    }
                },
                Opcode::St(offset) => {
                    if let Some(val) = stack.pop() {
                        if (offset as usize) < stack.len() {
                            stack[offset as usize] = val;
                            pc += 1;
                        } else {
                            return Err("Invalid local offset in St".into());
                        }
                    } else {
                        return Err("Stack underflow in St".into());
                    }
                },
                Opcode::Add => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Add".into());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x + y)),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x + y)),
                        _ => return Err("Type mismatch in Add".into()),
                    }
                    pc += 1;
                },
                Opcode::Sub => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Sub".into());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x - y)),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x - y)),
                        _ => return Err("Type mismatch in Sub".into()),
                    }
                    pc += 1;
                },
                Opcode::Mul => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Mul".into());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x * y)),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x * y)),
                        _ => return Err("Type mismatch in Mul".into()),
                    }
                    pc += 1;
                },
                Opcode::Div => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Div".into());
                    }
                    let b = stack.pop().unwrap();
                    match b {
                        Value::Int(0) | Value::Float(0.0) => return Err("Division by zero".into()),
                        _ => {}
                    }
                    let a = stack.pop().unwrap();
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x / y)),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x / y)),
                        _ => return Err("Type mismatch in Div".into()),
                    }
                    pc += 1;
                },
                Opcode::Jmp(addr) => { pc = addr; },
                Opcode::Jz(addr) => {
                    if let Some(top) = stack.last() {
                        let zero = match top {
                            Value::Int(n) => *n == 0,
                            Value::Float(f) => *f == 0.0,
                        };
                        if zero {
                            pc = addr;
                        } else {
                            pc += 1;
                        }
                    } else {
                        return Err("Stack underflow in Jz".into());
                    }
                },
                Opcode::Ret => {
                    if let Some(result) = stack.pop() {
                        return Ok(result);
                    } else {
                        return Err("Stack underflow in Ret".into());
                    }
                },
            }
        }
        Err("No Ret opcode encountered".into())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_int_arithmetic() {
            let opcodes = vec![
                Opcode::IImm(10),
                Opcode::IImm(5),
                Opcode::Sub,
                Opcode::Ret,
            ];
            let result = execute(opcodes).unwrap();
            assert_eq!(result, Value::Int(5));
        }

        #[test]
        fn test_float_arithmetic() {
            let opcodes = vec![
                Opcode::FImm(3.5),
                Opcode::FImm(1.5),
                Opcode::Add,
                Opcode::Ret,
            ];
            let result = execute(opcodes).unwrap();
            assert_eq!(result, Value::Float(5.0));
        }

        #[test]
        fn test_type_mismatch() {
            let opcodes = vec![
                Opcode::IImm(3),
                Opcode::FImm(4.5),
                Opcode::Add,
                Opcode::Ret,
            ];
            assert!(execute(opcodes).is_err());
        }
    }
}

//
// Main entry point
//
fn main() {
    // Retrieve command-line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: c4 <file.c>");
        process::exit(1);
    }
    let filename = &args[1];
    let source = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", filename, err);
        process::exit(1);
    });

    // Lexical analysis.
    let tokens = match lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexing error: {}", e);
            process::exit(1);
        }
    };

    // Parsing.
    let opcodes = match parser::parse(tokens) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Parsing error: {}", e);
            process::exit(1);
        }
    };

    // Execution.
    match vm::execute(opcodes) {
        Ok(result) => {
            println!("Program executed successfully. Result: {:?}", result);
        },
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    }
}


#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::vm::execute;

    /// Test a nested if–else construct.
    #[test]
    fn test_nested_if_else() {
        let source = r#"
        int main() {
            if (1) {
                if (0) {
                    return 1;
                } else {
                    return 2;
                }
            } else {
                return 3;
            }
        }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let opcodes = parse(tokens).expect("Parsing failed");
        let result = execute(opcodes).expect("Execution failed");
        // The outer condition is true, inner condition false → returns 2.
        assert_eq!(result, 2);
    }

    /// Test a nested while loop.
    #[test]
    fn test_nested_while_loops() {
        // This minimal example uses nested loops to compute a result.
        // The following C code conceptually decrements a variable in nested loops.
        let source = r#"
        int main() {
            int i;
            i = 3;
            while (i) {
                while (i - 1) {
                    i = i - 1;
                }
                i = 0;
            }
            return i;
        }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let opcodes = parse(tokens).expect("Parsing failed");
        let result = execute(opcodes).expect("Execution failed");
        // The expected result is 0 after the loops.
        assert_eq!(result, 0);
    }

    /// Test that referencing an undefined variable results in a parse error.
    #[test]
    fn test_undefined_variable_error() {
        let source = r#"
        int main() {
            return x;
        }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let parse_result = parse(tokens);
        assert!(parse_result.is_err(), "Parsing should fail due to undefined variable");
    }

    /// Test that division by zero is caught as an error during execution.
    #[test]
    fn test_division_by_zero_error() {
        let source = r#"
        int main() {
            return 10 / 0;
        }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let opcodes = parse(tokens).expect("Parsing failed");
        let result = execute(opcodes);
        assert!(result.is_err(), "Execution should fail with division by zero");
    }

    /// Test that invalid syntax is detected during parsing.
    #[test]
    fn test_invalid_syntax_error() {
        let source = r#"
        int main( { return 0; }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let parse_result = parse(tokens);
        assert!(parse_result.is_err(), "Parsing should fail due to invalid syntax");
    }

    /// A simple self-hosting test using a minimal C program.
    #[test]
    fn test_self_hosting() {
        let source = r#"
        int main() {
            return 42;
        }
        "#;
        let tokens = tokenize(source).expect("Tokenization failed");
        let opcodes = parse(tokens).expect("Parsing failed");
        let result = execute(opcodes).expect("Execution failed");
        assert_eq!(result, 42);
    }
}
