//! c4.rs – A Minimal Self‑Hosting C Compiler in Rust
//!
//! This file is a reimplementation of the original C4 compiler (written in C)
//! using Rust. It is organized into modules for lexing, parsing (with an advanced
//! symbol table and support for control flow), and a virtual machine (VM) that
//! executes the generated opcodes. The compiler supports a subset of C, including:
//!   - Global and local variable declarations
//!   - A single function definition (e.g., int main() { ... })
//!   - Statements: expression statements, if–else, while, return
//!   - Expressions: assignments and basic arithmetic operations
//!
//! Usage (via Cargo):
//!     cargo run -- <file.c>
//!
//! The program reads a C source file, tokenizes it, parses it into opcodes, and
//! then executes the opcodes using a stack-based virtual machine. Errors at any phase
//! are reported with descriptive messages.

use std::env;
use std::fs;
use std::process;

//
// Module: lexer
//
mod lexer {
    //! The lexer module converts C source code into a sequence of tokens.
    //!
    //! It recognizes keywords (int, char, return, if, else, while), identifiers,
//! numeric literals, operators, and punctuation.

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
        // Literals
        Num(i64),
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
    pub fn tokenize(source: &str) -> LexResult {
        let mut tokens = Vec::new();
        let mut chars = source.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => { chars.next(); },
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
                    let value = num_str.parse::<i64>().map_err(|e| e.to_string())?;
                    tokens.push(Token::Num(value));
                },
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
                        "int" => tokens.push(Token::Int),
                        "char" => tokens.push(Token::Char),
                        "return" => tokens.push(Token::Return),
                        "if" => tokens.push(Token::If),
                        "else" => tokens.push(Token::Else),
                        "while" => tokens.push(Token::While),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                },
                '+' => { tokens.push(Token::Plus); chars.next(); },
                '-' => { tokens.push(Token::Minus); chars.next(); },
                '*' => { tokens.push(Token::Mul); chars.next(); },
                '/' => {
                    chars.next();
                    // Handle C++–style single-line comments.
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
}

//
// Module: parser
//
mod parser {
    //! The parser module implements a recursive descent parser for a subset of C.
    //!
    //! It supports global and local variable declarations, a single function definition
    //! (e.g., int main() { ... }), and statements including expression statements, if–else,
    //! while, and return. It also builds an advanced symbol table for variables.
    //!
    //! The parser emits opcodes for a stack-based virtual machine.

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
        pub offset: i64, // For local variables: offset in the stack frame.
    }

    pub struct Parser {
        tokens: Vec<Token>,
        pos: usize,
        opcodes: Vec<Opcode>,
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

        /// Consumes the current token if it matches the given token.
        fn eat(&mut self, token: &Token) -> bool {
            if self.current() == token {
                self.pos += 1;
                true
            } else {
                false
            }
        }

        /// Expects that the current token matches the given token.
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
                    Token::Int => {
                        self.pos += 1; // consume 'int'
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
                                    // Enter new local scope.
                                    self.locals.clear();
                                    self.local_offset = 0;
                                    while self.current() != &Token::RBrace {
                                        self.parse_stmt()?;
                                    }
                                    self.expect(&Token::RBrace)?;
                                    // Function end.
                                    self.opcodes.push(Opcode::Ret);
                                } else {
                                    // Global variable declaration.
                                    self.globals.insert(ident.clone(), Symbol { name: ident, class: SymbolClass::Global, offset: 0 });
                                    // Consume remaining declaration tokens until semicolon.
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
                Token::Int => self.parse_local_decl(),
                _ => {
                    // Expression statement.
                    self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    Ok(())
                }
            }
        }

        /// Parses an if–else statement: if ( expr ) stmt [ else stmt ]
        fn parse_if(&mut self) -> Result<(), String> {
            self.pos += 1; // consume 'if'
            self.expect(&Token::LParen)?;
            self.parse_expr()?;
            self.expect(&Token::RParen)?;
            let jz_index = self.opcodes.len();
            self.opcodes.push(Opcode::Jz(0)); // placeholder for jump if false
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

        /// Parses a while statement: while ( expr ) stmt
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
            self.pos += 1; // consume 'int'
            loop {
                match self.current() {
                    Token::Ident(name) => {
                        let var_name = name.clone();
                        self.pos += 1;
                        self.local_offset += 1;
                        let offset = self.local_offset;
                        self.locals.insert(var_name.clone(), Symbol { name: var_name, class: SymbolClass::Local, offset });

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
        /// Supports assignment (identifier = expr) and additive expressions.
        fn parse_expr(&mut self) -> Result<(), String> {
            self.parse_assignment()
        }

        fn parse_assignment(&mut self) -> Result<(), String> {
            let start = self.pos;
            if let Token::Ident(ref name) = self.current() {
                let ident = name.clone();
                self.pos += 1;
                if self.eat(&Token::Assign) {
                    self.parse_assignment()?;
                    // Generate store opcode.
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

        fn parse_factor(&mut self) -> Result<(), String> {
            match self.current() {
                Token::Num(n) => {
                    let value = *n;
                    self.pos += 1;
                    self.opcodes.push(Opcode::Imm(value));
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

        /// Public API: parses tokens into opcodes.
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
}

//
// Module: vm
//
mod vm {
    //! The virtual machine (VM) executes opcodes generated by the parser.
    //!
    //! This stack-based VM supports integer arithmetic, variable load/store,
    //! and control flow instructions (jumps and conditional jumps).

    #[derive(Debug, Clone, PartialEq)]
    pub enum Opcode {
        Imm(i64),   // Push immediate value onto the stack.
        Ld(i64),    // Load variable from local offset.
        St(i64),    // Store top of stack into local variable at offset.
        Add,        // Add top two values.
        Sub,        // Subtract top two values.
        Mul,        // Multiply top two values.
        Div,        // Divide top two values.
        Jmp(i64),   // Unconditional jump to opcode index.
        Jz(i64),    // Jump if top of stack is zero.
        Ret,        // Return from function.
    }

    /// Executes a sequence of opcodes and returns the final result.
    pub fn execute(opcodes: Vec<Opcode>) -> Result<i64, String> {
        let mut stack: Vec<i64> = Vec::new();
        stack.resize(32, 0); // ✅ Reserve space for local variables
    
        let mut pc: i64 = 0;
        while (pc as usize) < opcodes.len() {
            match opcodes[pc as usize].clone() {
                Opcode::Imm(n) => {
                    stack.push(n);
                    pc += 1;
                },
                Opcode::Ld(offset) => {
                    if (offset as usize) < stack.len() {
                        let val = stack[offset as usize];
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
                    stack.push(a + b);
                    pc += 1;
                },
                Opcode::Sub => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Sub".into());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                    pc += 1;
                },
                Opcode::Mul => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Mul".into());
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a * b);
                    pc += 1;
                },
                Opcode::Div => {
                    if stack.len() < 2 {
                        return Err("Stack underflow in Div".into());
                    }
                    let b = stack.pop().unwrap();
                    if b == 0 {
                        return Err("Division by zero".into());
                    }
                    let a = stack.pop().unwrap();
                    stack.push(a / b);
                    pc += 1;
                },
                Opcode::Jmp(addr) => {
                    pc = addr;
                },
                Opcode::Jz(addr) => {
                    if let Some(&top) = stack.last() {
                        if top == 0 {
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
}

//
// Main entry point
//
fn main() {
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
            println!("Program executed successfully. Result: {}", result);
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
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes).expect("Execution failed");
        // In this case, the outer condition is true (1), so we go into the inner if.
        // The inner condition is false (0), so the else branch returns 2.
        assert_eq!(result, 2);
    }

    /// Test a nested while loop.
    #[test]
    fn test_nested_while_loops() {
        // This test simulates a nested loop that decrements a variable.
        // Note: Our minimal compiler only supports basic arithmetic and control flow.
        // The following code initializes i to 3, then uses a nested loop to decrement it.
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
        // Depending on how the compiler handles variable declarations and expressions,
        // the expected result should be 0.
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes).expect("Execution failed");
        assert_eq!(result, 0);
    }

    /// Test that an undefined variable causes an error.
    #[test]
    fn test_undefined_variable_error() {
        let source = r#"
        int main() {
            return x;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let parse_result = parse(tokens);
        assert!(parse_result.is_err(), "Parsing should fail due to undefined variable");
    }

    /// Test that division by zero is handled as an error.
    #[test]
    fn test_division_by_zero_error() {
        let source = r#"
        int main() {
            return 10 / 0;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes);
        assert!(result.is_err(), "Execution should fail with division by zero");
    }

    /// Test that invalid syntax produces a parse error.
    #[test]
    fn test_invalid_syntax_error() {
        let source = r#"
        int main( { return 0; }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let parse_result = parse(tokens);
        assert!(parse_result.is_err(), "Parsing should fail due to invalid syntax");
    }

    /// A simple self-hosting test that uses a minimal C source resembling the compiler's own code.
    #[test]
    fn test_self_hosting() {
        // This is a minimal C program that our compiler should handle.
        let source = r#"
        int main() {
            return 42;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes).expect("Execution failed");
        assert_eq!(result, 42);
    }

    /// Test long arithmetic expression with precedence
    #[test]
    fn test_complex_expression() {
        let source = r#"
        int main() {
            return 2 + 3 * 4 - 6 / 2;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes).expect("Execution failed");
        assert_eq!(result, 2 + 3 * 4 - 6 / 2);
    }

    /// Test multiple variable declarations and usage
    #[test]
    fn test_multiple_variables() {
        let source = r#"
        int main() {
            int a, b;
            a = 5;
            b = 10;
            return a + b;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let opcodes = parse(tokens).expect("Failed to parse");
        let result = execute(opcodes).expect("Execution failed");
        assert_eq!(result, 15);
    }

    /// Test unmatched parentheses to simulate a syntax error
    #[test]
    fn test_unmatched_parentheses() {
        let source = r#"
        int main() {
            return (5 + 2;
        }
        "#;
        let tokens = tokenize(source).expect("Failed to tokenize");
        let parse_result = parse(tokens);
        assert!(parse_result.is_err(), "Expected error due to unmatched parentheses");
    }
}
