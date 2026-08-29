# Valo Roadmap

Valo's direction is to be a successor language to VB.NET with a standalone
runtime. VB.NET is the reference for syntax and semantics; VBA compatibility is a
migration bridge, and where the two disagree, Valo follows VB.NET.

This document tracks where that leaves the work.

## Phase 1: Core foundation

- [x] Lexer and recursive-descent parser
- [x] Semantic validation and type checking
- [x] Tree-walking interpreter
- [x] Object model: classes, properties, events, inheritance, interfaces
- [x] Generics for classes, structures, functions, and methods
- [x] Professional diagnostics with codes and source spans
- [x] Modules, imports, and namespaces
- [x] Whole-project semantic validation, including procedure bodies and
      cross-module resolution
- [x] VBA compatibility layer for `.bas` / `.cls` migration

## Phase 2: VB.NET language parity

The goal is that code written against the VB.NET language reference runs on Valo.

- [x] String interpolation, with format specifiers and alignment
- [x] Compound assignment (`+=`, `-=`, `*=`, `/=`, `\=`, `^=`, `&=`, `<<=`, `>>=`)
- [x] Shift operators (`<<`, `>>`)
- [x] `Continue For`, `Continue While`, `Continue Do`
- [x] `CType`, `DirectCast`, `TryCast`, `GetType`, `NameOf`
- [x] `Inherits` and `Implements` as standalone lines in a class body
- [x] Bitwise operators over every integral width
- [x] Multi-line lambdas (`Function() ... End Function`, `Sub() ... End Sub`)
- [x] Closures: lambdas capture their defining scope, by reference
- [x] Object initializers (`New Point With { .X = 1 }`)
- [ ] Anonymous types (`New With { Key .X = 1 }`)
- [x] Null-conditional access (`obj?.Member`)
- [ ] `Delegate Sub` / `Delegate Function` declarations
- [ ] Tuples and tuple returns
- [ ] `Overloads`, and deeper generic constraint enforcement
- [ ] Full LINQ query syntax (`From ... Where ... Select`)

## Phase 3: Performance

- [x] Remove quadratic array writes and per-statement allocations
      (see [Performance](performance.md))
- [x] Benchmark suite and a release profile tuned for the interpreter
- [ ] **Bytecode VM.** Move name resolution, statement-shape checks, and frame
      layout to a compile step instead of redoing them on every execution.
- [ ] Resolve identifiers to slot indices during validation
- [ ] Reference-counted strings, so passing them stops copying them
- [ ] Per-call-site method and property lookup caches

## Phase 4: Developer experience

- [ ] **Language server**, for completion, go-to-definition, and live diagnostics
- [ ] **Formatter and linter**
- [ ] **Package manager**, for distributing Valo libraries

## Phase 5: Integration and the standard library

- [x] FFI foundation: `Declare`, `PtrSafe`, `LongPtr`, `AddressOf`, callbacks,
      platform-aware library loading
- [x] Windows COM/OLE Automation through late-bound `Object` and `CreateObject`
- [x] `Collection` with keyed lookup, positional access, and enumeration
- [x] File-number I/O, including Random and Binary modes
- [ ] **Broader standard library**: networking, JSON, richer collections
- [ ] **FFI expansion**: broader marshalling, type-library tooling
- [ ] **Embedding API**, so Valo can be hosted inside a Rust application

## Vision

A language that reads like VB.NET, starts in milliseconds, ships as one binary,
talks to native code without ceremony, and keeps evolving.
