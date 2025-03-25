# C4 Rust Compiler

## Project Overview & Objectives

This project is a Rust reimplementation of the original C4 compiler a minimal C compiler that supports a small subset of the C language. The objectives of this project are:

- **Rewriting the C4 Compiler in Rust:**  
  Translate the C4 compiler (lexer, parser, virtual machine, etc.) from C into Rust while preserving functional equivalence and ensuring self-hosting capability.
  
- **Leveraging Rust Features:**  
  Use Rust idioms (ownership, pattern matching, error handling with `Result`/`Option`) to improve safety, clarity, and maintainability.
  
- **Implementing Unit Testing & Documentation:**  
  Provide comprehensive tests for each component and detailed documentation for future maintainers.
  
- **Bonus Feature – Floating‑Point Support:**  
  Extend the compiler to support floating‑point literals and arithmetic in addition to the original C subset.

## Repository Structure

The repository is organized as follows:

c4_rust_-Al-Hiyar (GitHub repository name)

├── Cargo.toml (Cargo configuration (file Main Rust source file implementing the compiler)



├── c4.rs (Main Rust source file implementing the compiler)


├── c4 +bonus.rs (Rust source file implementing the compiler (includes bonus floating‑point support))


├── README.md (This file)


└── c4_rust_comparison.pdf (Comparison report between the Rust implementation and the original C4 compiler)


## Building the Project

To build the project, run the following command in the repository directory:

```bash
cargo build
   ```
This command will compile your Rust source code and produce an executable.

## Running the Compiler

To run the compiler on a C source file (e.g., sample.c), use the following command:

```bash
cargo run -- sample.c
 ```

The compiler will:

- Read and tokenize the provided C source code.

- Parse the tokens into opcodes.

- Execute the opcodes using a stack-based virtual machine.

- Print the result of the program or any errors encountered during lexing, parsing, or execution.

## Running Unit Tests

Unit tests are provided for the lexer, parser, and virtual machine components. To run all tests, execute:

```bash
cargo test
```

Ensure all tests pass to verify that the compiler processes the supported subset of C correctly, including the bonus floating‑point functionality.




# Bonus Feature: Floating‑Point Support

## Overview

In addition to supporting the original C subset (integers, basic arithmetic, control flow, and self-hosting), this compiler includes bonus support for floating‑point numbers. The bonus feature enables the compiler to:

- Recognize floating‑point literals (e.g., 3.14, 0.001) during lexing.

- Generate appropriate opcodes (FImm) for floating‑point immediates in the parser.

- Perform floating‑point arithmetic operations (addition, subtraction, multiplication, division) in the virtual machine with proper type checking.

## How to Test/Use Floating‑Point Support

You can test floating‑point support by providing a C source file containing floating‑point expressions. For example, create a file float_test.c with the following content:
```bash
int main() {
    return 3.14 + 2.0;
}
```
Run the compiler with:

```bash
cargo run -- float_test.c
```
The expected output should show that the program executed successfully and returned the result of the floating‑point arithmetic.

 # **Additional Documentation**
 
- Rust Documentation:
  
The source code is fully documented using Rust doc comments. To generate HTML documentation, run:

```bash
cargo doc --open
```
- **Comparison Report:**

The file c4_rust_comparison.pdf provides an in-depth comparison between this Rust implementation and the original C4 compiler, including design decisions, performance insights, and challenges encountered during the rewrite.











