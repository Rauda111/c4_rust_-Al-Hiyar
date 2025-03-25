# C4 Rust Compiler

## Project Overview & Objectives

This project is a Rust reimplementation of the original C4 compiler—a minimal C compiler that supports a small subset of the C language. The objectives of this project are:

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

