# Valo Roadmap

Valo is a successor language to VB.NET with a standalone runtime. VB.NET is the
reference for syntax and semantics; VBA compatibility is a migration bridge, and
where the two disagree, Valo follows VB.NET.

This document is ordered by what unblocks what, not by what would be nicest to
have. Items other items depend on come first.

## Where things stand

| | |
|---|---|
| Language surface against VB.NET | roughly 80% of what is commonly used |
| Runtime | tree-walking interpreter; no VM |
| Standard library | `Collection` plus the VBA runtime surface |
| Tooling | CLI, REPL, diagnostics; no language server, formatter, or package manager |

The language is well ahead of the ecosystem. Someone can write real VB.NET-shaped
code today; nobody can distribute a library, edit with completion, or rely on
VM-class performance.

## Open defects

These are known wrong, not merely missing. All were found by writing a whole
program — [the Breakout demo](../../game/README.md) — rather than by reading code.

- [ ] 67 diagnostics still report `V0001`, the code that means "no code was
      chosen". Each needs judgement, not a script.

## Phase 1: Foundations for a new execution engine

Nothing here is a VM. All of it is what a VM needs to exist first, and the first
two are worth doing even if the VM never happens.

- [x] **Pin the semantics with golden outputs.** Every example's output is now
      recorded in `examples/golden/` and compared on each run, so a new engine
      can be told apart from a broken one. `VALO_BLESS=1` records or updates
      them.
- [ ] **Resolve identifiers to slot indices.** Every variable read hashes a
      string today. A VM's locals are indices, so this is a hard prerequisite —
      and it speeds up the current interpreter immediately. The analyzer already
      computes scopes; it just does not record positions.
      *This is where `ProjectIndex` belongs.* It is built on every validation and
      the result is thrown away (`let _project_index = …` in `validate.rs`).
- [ ] **Make `Value` cheap to copy.** `Value::String(String)` copies the text on
      every clone. A VM moves values constantly; this wants to be `Rc<str>`.
- [ ] **Per-call-site method and property caches.** A method call costs about
      4.9µs against 1.2µs for a field read, most of it resolution.

## Phase 2: Bytecode VM

Only after Phase 1. The headroom is measured, not assumed: a field read costs
1.17µs today, where a bytecode VM should reach tens of nanoseconds.

- [ ] **Define the instruction set and the lowering pass.**
- [ ] **Frame layout**: a contiguous stack with frame pointers, replacing the
      per-call `HashMap`.

The parts that need a decision rather than an implementation:

| Feature | Why it resists straightforward lowering |
|---|---|
| `On Error Resume Next`, `Resume` | Needs resume points at instruction granularity. The interpreter already tracks a statement index, which maps over. |
| `ByRef` aliasing | `VariableCell` holds views into an array element or a record field. A slot must be able to hold an alias, not only a value. |
| `AddressOf` | Native callbacks have to re-enter the VM. |
| Late-bound `Object`, COM, `CallByName` | Cannot be compiled statically; stay as runtime helpers the VM calls into. |

## Phase 3: JIT

Deliberately after the VM, and driven by profiling rather than ambition.

- [ ] Type feedback through inline caches — `Variant` is dynamic, so specialized
      code has to know what actually flows through a site.
- [ ] A deoptimization path for when speculation is wrong.

Worth saying plainly: for a language whose programs are mostly I/O and native
calls — the Breakout demo spends 6ms a frame inside SDL, which no JIT touches —
a good VM likely captures most of the available win. JIT is the step after the
VM proves there is still a bottleneck.

## Phase 4: Finishing VB.NET

None of this blocks the VM, and the VM blocks none of it.

- [ ] **`Overloads`** — several procedures sharing a name, resolved by signature.
      The largest item here: signature collection assumes one name is one
      procedure, so this touches the analyzer and dispatch together. It is also
      the first thing someone coming from VB.NET reaches for.
- [ ] **`Delegate Sub` / `Delegate Function`** — named callable types. Lambdas
      already carry parameters and a body, so this is mostly a type story.
- [ ] **Tuples and tuple returns** — `(a, b)` literals, `As (X As Long, …)`
      types, and destructuring.
- [ ] **Anonymous types** — `New With { Key .X = 1 }`. The object-initializer
      machinery exists; what is missing is a synthesized type.
- [ ] **LINQ query syntax** — `From … Where … Select`. Now viable: it desugars to
      the extension methods and lambdas that already work, and closures are what
      made those lambdas useful.
- [ ] **Deeper generic constraint enforcement** — constraints parse but are not
      fully checked.
- [ ] **`Option Strict`**, jagged-array parity, and the rest of the `My.*`
      surface, as they come up.

## Phase 5: Beyond VB.NET

Things worth adding *because* Valo owns its runtime — the reason to be a
successor rather than a reimplementation. Each came from a real obstacle, not
from a wish list.

- [ ] **A memory buffer type.** The SDL demo needed a 128-byte `SDL_Event`, and
      expressing it took a `Type` with 24 padding fields. A first-class fixed
      buffer with a defined layout would make native ABIs natural to describe.
- [ ] **Reading and writing through a pointer.** `VarPtr` and `StrPtr` hand out
      addresses that nothing can then dereference, so an API returning a pointer
      into an array — `SDL_GetKeyboardState` is the example — cannot be used at
      all.
- [ ] **Native callbacks with checked signatures.** `AddressOf` works, but the
      shapes it accepts are not checked against what the native side expects.
- [ ] **An embedding API**, so Valo can be hosted inside a Rust application.
- [ ] **Compile-time evaluation** beyond `Const`, so table-driven code does not
      pay for itself at run time.

## Phase 6: Ecosystem

- [ ] **Language server** — completion, go-to-definition, live diagnostics. The
      diagnostics engine already carries codes and spans, which is the hard part.
- [ ] **Formatter**, and a linter built on the same analysis.
- [ ] **Package manager** for distributing Valo libraries.
- [ ] **Standard library**: networking, JSON, and generic `List(Of T)` and
      `Dictionary(Of K, V)` beside `Collection`.

## Done

Kept short; the detail lives in the commits.

**Foundation** — lexer, parser, semantic validation, tree-walking interpreter,
the object model, generics, modules and namespaces, diagnostics with stable
codes, and the VBA `.bas` / `.cls` bridge. Whole-project validation now checks
procedure bodies and resolves across modules; it used to stop at declarations,
so `valo check` reported success on code that failed as soon as it ran.

**VB.NET parity** — string interpolation with alignment and format specifiers,
compound assignment, shift operators, `Continue`, `CType` / `DirectCast` /
`TryCast` / `GetType` / `NameOf`, object initializers, multi-line lambdas,
closures capturing by reference, null-conditional access, standalone `Inherits`,
and bitwise operators over every integral width.

**Architecture** — one builtin registry that the analyzer and interpreter both
read, with seven invariants held by tests; well-known names declared once;
built-in types described declaratively; diagnostic codes generated from a single
table alongside a reference page that cannot drift from it.

**Performance** — quadratic array writes removed, per-statement allocations
removed, and classes and method bodies no longer copied on every call. A method
call went from 10.7µs to 4.9µs; the Breakout demo from 86ms of work per frame to
24ms.

## How to work on this

Two things this codebase learned the hard way, both worth keeping:

**Measure before optimizing, and measure more than once.** A single timing run on
a developer machine varies by tens of percent — enough that one measurement here
showed field access getting *slower* from a change that does not touch that path.
Every performance figure quoted in this repository is the best of several runs.

**Write whole programs, not only feature tests.** The suite of 587 tests and 117
single-feature examples did not catch five defects that one Breakout demo found
in an afternoon. Features interacting is where the bugs are.

## Vision

A language that reads like VB.NET, starts in milliseconds, ships as one binary,
talks to native code without ceremony, and keeps evolving.
