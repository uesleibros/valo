# Architecture Overview

Valo's architecture is divided into three primary layers to ensure modularity, portability, and future-readiness.

## 1. Frontend (`core/src/frontend/`)
The Frontend is responsible for transforming raw source code into a validated, semantically-rich Abstract Syntax Tree (AST). It is entirely independent of how the code is eventually executed.

*   **Preprocessor:** Handles conditional compilation (`#If`) and source transformations.
*   **Lexer:** Tokenizes source code into discrete elements.
*   **Parser:** Performs recursive descent to build the AST.
*   **AST:** The structural representation of Valo code.
*   **Semantics:** Validates symbols, types, and control flow.
*   **Module System:** Discovers and resolves module dependencies.

Learn more in **[Frontend Architecture](frontend.md)**.

## 2. Runtime (`core/src/runtime/`)
The Runtime defines the core data model and behavior of the Valo language. It is shared by both the Frontend (for type-checking) and all Backends (for execution).

*   **Value System:** Defines `Value`, `ObjectValue`, and type coercion rules.
*   **Diagnostics:** Provides source-aware error reporting.
*   **Operations:** Centralized logic for arithmetic, comparison, and coercion.
*   **Resource Model:** Manages deterministic cleanup via `Using` and `Dispose`.

Learn more in **[Runtime Architecture](runtime.md)**.

## 3. Backend (`core/src/backend/`)
The Backend is the execution engine that consumes the validated AST (or future intermediate representations) and performs the actual work.

*   **Interpreter:** The current reference execution engine (tree-walking).
*   **Future VM:** A planned bytecode virtual machine for higher performance.
*   **Future Backends:** Potential WASM and Native compilation targets.

Learn more in **[Backend Architecture](backend.md)**.

## 4. Platform
The Platform layer is the emerging project/package, namespace, tooling, standard-library, and interop architecture that turns Valo from a single-file language runtime into an ecosystem.

*   **Package Identity:** `valo.toml` project roots and entrypoint resolution.
*   **Semantic IDs/HIR:** Stable project-wide identities for tooling and future VM lowering.
*   **Namespaces:** Logical API identity decoupled from filenames.
*   **Runtime Services:** Standard-library boundaries shared by interpreter, VM, and embedders.
*   **Interop:** COM/type-library architecture layered beside native FFI.

Learn more in **[Platform Architecture](platform.md)**.

## Cross-cutting: name folding

Basic is case-insensitive, so `Total`, `total`, and `TOTAL` are one name. Every
symbol table in the compiler and runtime is therefore keyed by a case-folded
name, and all three layers must agree on what folding means.

That rule lives in one place, [`core/src/runtime/naming.rs`](../../core/src/runtime/naming.rs).
The preprocessor, the semantic analyzer, and the interpreter all delegate to it
rather than each folding names their own way. `with_folded` is the form to reach
for on hot paths: it folds into a stack buffer instead of allocating, which
matters because identifier lookup is the single most frequent operation the
interpreter performs.

## Cross-cutting: validation

Both entry points — `validate` for a single program and `validate_project` for a
loaded project — run the same body validation over every procedure, function,
structure, and class. The project path additionally brings imported types,
callables, and extension methods into scope, and completes a `Partial Class` from
its halves in other modules.

Keeping the two paths on one implementation is deliberate. They diverged once,
with project validation stopping before procedure bodies, and the result was that
`valo check` reported success on programs that failed the moment they ran.

Learn more in **[Performance](performance.md)** and **[Diagnostics](diagnostics.md)**.

## Cross-cutting: the builtin registry

A builtin has a name, an arity, a result type, and an implementation. Those used
to live in three places — name lists in `runtime::builtins`, arity and result
type as `if` branches in the analyzer, and the implementation as more `if`
branches in the interpreter — with nothing tying them together. Adding a builtin
meant three edits in three files, and nothing detected it when they disagreed.

[`runtime::builtins`](../../core/src/runtime/builtins.rs) is now the single
declaration. Each row states the name, arity, result type, whether the builtin
may stand alone as a statement, and which dispatcher handles it:

```rust
f("Mid", 2, 3, BuiltinReturn::String),
grouped(f("DateAdd", 3, 3, BuiltinReturn::Date), BuiltinGroup::DateTime),
```

The analyzer looks the name up, checks the arity, and reports the result type.
The interpreter routes on the group. Neither keeps its own list.

A handful of builtins constrain an argument beyond counting it — `LBound`,
`UBound`, and `Filter` take an array; `IsMissing` takes an optional parameter;
`IsArray` accepts anything by design. Those are named explicitly in the
analyzer, so the exceptions are visible rather than buried in a chain of a
thousand lines.

Two tests in the interpreter's builtin module hold the registry to its promise:
every declared builtin is reachable through the analyzer, and the arity a row
declares is the arity actually enforced.
