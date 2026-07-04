# Project DNA

## Project Name

<!-- The canonical name of this project. -->
My Project

## Stack

<!-- Languages, frameworks, runtimes, and key libraries in use. -->
- Language: Rust 1.78
- Framework: (none / axum / actix / etc.)
- Database: (none / sqlite / postgres / etc.)
- Key libs: (list them)

## Architecture

<!-- High-level shape: monolith, microservices, CLI tool, library, etc.
     Include a one-paragraph description of data flow or request lifecycle. -->
Single-binary CLI. Entry point in src/main.rs dispatches subcommands.
Core modules: config (path resolution), event_log (append-only JSONL log).

## Key Constraints

<!-- Hard limits the implementation must never violate. -->
- No external runtime dependencies (std-only crate)
- Must work on Windows, macOS, Linux
- Binary must start in < 50 ms
- Never panic in production paths — return Result/Option

## Design Principles

<!-- Guiding rules for everyday decisions. -->
1. Fail loudly in tests, fail gracefully in production.
2. Append-only for any log or event data.
3. Prefer explicit over clever.
4. Keep modules small and single-purpose.
