<div align="center">

  <img src="assets/valo-mascot.png" width="140" alt="Valo mascot">

  # The Valo Programming Language

  A successor language to VB.NET, with a standalone runtime written in Rust

</div>

<p align="center">
  <a href="#what-is-valo">What is Valo?</a> |
  <a href="#why-build-valo">Why?</a> |
  <a href="#language-goals">Goals</a> |
  <a href="#project-status">Status</a> |
  <a href="#getting-started">Getting started</a> |
  <a href="#performance">Performance</a> |
  <a href="#documentation">Documentation</a> |
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/valolang/valo/actions/workflows/ci.yml">
    <img src="https://github.com/valolang/valo/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
  <a href="https://github.com/valolang/valo/actions/workflows/release.yml">
    <img src="https://github.com/valolang/valo/actions/workflows/release.yml/badge.svg" alt="Release Status">
  </a>
  <a href="https://github.com/valolang/valo/releases">
    <img src="https://img.shields.io/github/v/release/valolang/valo?include_prereleases&label=release" alt="Release">
  </a>
  <a href="https://github.com/valolang/valo/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/valolang/valo" alt="License">
  </a>
  <img src="https://img.shields.io/badge/runtime-Rust-orange" alt="Runtime">
  <img src="https://img.shields.io/badge/status-experimental-blue" alt="Status">
</p>

> [!NOTE]
> Valo is experimental and not production-ready yet. APIs, syntax, runtime behavior, and compatibility details may change quickly.

## What is Valo?

**Valo** is a programming language that takes the syntax of VB.NET and gives it a
standalone runtime, written in Rust, that runs anywhere.

The design question behind Valo is narrow on purpose:

> What would VB.NET look like if it were not tied to .NET?

VB.NET is a genuinely good language. It is readable, explicit, and productive,
with a type system, generics, properties, events, and interfaces that hold up
well. What it lacks is a life outside its runtime: no small standalone binaries,
no practical embedding, no direct native interop that avoids marshalling layers,
and no path to run where .NET does not.

Valo follows the VB.NET language as its reference, then adds what only makes
sense once you own the runtime:

- native interop as a language feature, through `Declare`, `PtrSafe`, `LongPtr`,
  `AddressOf`, and callbacks
- a single self-contained binary with no runtime to install
- an interpreter designed to grow into a bytecode VM, rather than a language
  fitted onto a VM it did not choose

This is the move [Carbon](https://github.com/carbon-language/carbon-lang) makes
for C++ and [Mojo](https://www.modular.com/mojo) makes for Python: keep the
syntax people already know, and change what sits underneath it.

## Why build Valo?

![Building with Valo](assets/building.png)

The Basic family has never had a language that is both modern and unencumbered.

- **VB6** was standalone but is long dead.
- **VBA** is alive and widely used, but locked inside Office and COM.
- **VB.NET** is modern, but inseparable from .NET, and its language design has
  stopped moving.

Valo starts from VB.NET because it is the most complete of the three, and
rebuilds underneath it:

| | VB6 | VBA | VB.NET | Valo |
|---|---|---|---|---|
| Standalone binaries | yes | no | no | yes |
| Modern type system | no | no | yes | yes |
| Generics | no | no | yes | yes |
| Direct native interop | limited | `Declare` | via P/Invoke | `Declare`, first-class |
| Runs without a host runtime | yes | no | no | yes |
| Actively evolving | no | no | no | yes |

Existing VBA code is not abandoned: `.bas` and `.cls` files still load, so a
codebase can migrate gradually. But compatibility is a migration path, not the
design target. When VBA and VB.NET disagree, Valo follows VB.NET.

## Language goals

- **Follow VB.NET.** Its syntax and semantics are the reference. Code that reads
  like VB.NET should behave like VB.NET.
- **Own the runtime.** A standalone Rust runtime, with no host environment and no
  framework to install.
- **Make native interop first-class.** `Declare` is a language feature, not an
  escape hatch.
- **Be fast enough to be boring.** Performance is tracked by benchmarks in the
  repository.
- **Diagnose precisely.** Explicit codes, source spans, and actionable help.
- **Keep the migration door open.** `.bas` and `.cls` load, so existing VBA can
  come along.

## Project status

Valo is an experimental language and runtime in active development.

Implemented today:

- lexer, recursive-descent parser, semantic validator, and tree-walking interpreter
- classes, interfaces, inheritance, structures, properties, events, and lifecycle hooks
- generics for classes, structures, functions, and methods, with constraints that
  give a type parameter its bound's members
- overloading for procedures, methods, constructors, and `Shared` members
- delegates, tuples and tuple returns, anonymous types, and query syntax
  (`From … Where … Order By … Select`)
- `Option Strict`, which rejects silent narrowing and late binding
- modules, imports, namespaces, and `Module ... End Module` blocks
- string interpolation, compound assignment, shift operators, and `Continue`
- `CType`, `DirectCast`, `TryCast`, `GetType`, and `NameOf`
- lambdas, extension methods, operator overloading, partial classes, iterators, nullable types, and collection initializers
- `Try` / `Catch` / `Finally` alongside VBA-style `On Error`
- deterministic cleanup and `Using`
- native FFI through `Declare`, `PtrSafe`, callbacks, and pointer helpers
- Windows COM/OLE Automation through late-bound `Object` and `CreateObject`
- a diagnostics engine, a REPL, and a CLI
- a VBA compatibility layer for `.bas` and `.cls` migration

Not there yet: a bytecode VM, a package manager, a language server, a formatter,
and a complete standard library. See the [roadmap](docs/architecture/roadmap.md).

## Feature highlights

### String interpolation

```vb
Sub Main()
    Dim user As String = "Ada"
    Dim price As Double = 12.5

    Console.WriteLine($"Hello, {user}!")
    Console.WriteLine($"Total: {price:0.00}")
    Console.WriteLine($"[{user,10}] right-aligned, [{user,-10}] left-aligned")
End Sub
```

Format specifiers go through the same engine as `Format`, so the two always agree.

### Compound assignment and shifts

```vb
Sub Main()
    Dim total As Long = 10
    total += 5
    total *= 2
    total >>= 1

    Dim message As String = "Valo"
    message &= " 1.0"

    Dim flags As Long = (1 << 0) Or (1 << 3)
End Sub
```

`+=`, `-=`, `*=`, `/=`, `\=`, `^=`, `&=`, `<<=`, and `>>=` are all supported, as
are the `<<` and `>>` operators.

### Conversions and reflection

```vb
Sub Main()
    Dim rex As New Dog()

    Dim asAnimal As Animal = DirectCast(rex, Animal)
    Dim maybeDog As Dog = TryCast(asAnimal, Dog)
    Dim count As Integer = CType("42", Integer)

    Console.WriteLine(GetType(Dog))
    Console.WriteLine(NameOf(count))
End Sub
```

`CType` converts, `DirectCast` reinterprets along the inheritance chain and fails
otherwise, and `TryCast` answers `Nothing` instead of failing.

### Control flow

```vb
Sub Main()
    Dim i As Integer

    For i = 1 To 10
        If i Mod 2 = 0 Then Continue For
        Console.WriteLine(i)
    Next i

    Try
        DangerousOperation()
    Catch ex As Error
        Console.WriteLine(ex.Message)
    Finally
        Console.WriteLine("cleanup")
    End Try
End Sub
```

`Continue For`, `Continue While`, and `Continue Do` each advance the loop they
name, so `Continue For` inside a nested `Do` advances the outer `For`.

### Generics, constraints, and inheritance

```vb
Class Box(Of T)
    Public Value As T
End Class

Class Dog
    Inherits Animal

    Public Overrides Sub Speak()
        Console.WriteLine(Name & " says woof")
    End Sub
End Class

Interface IProducer(Of Out T)
    Function Current() As T
End Interface

Class Repository(Of T As {Class, New})
    Public Current As T
End Class
```

Generic substitution is active for classes, structures, fields, parameters,
properties, return values, inheritance, constructor arguments, and nested generic
instances. Constraint and variance syntax is accepted by the parser; deeper
compile-time constraint enforcement is still being expanded.

### Native FFI

```vb
Declare PtrSafe Function strlen Lib "libc" CDecl (
    ByVal value As String
) As Long

Sub Main()
    Console.WriteLine(strlen("Valo"))
End Sub
```

Native interop is a language feature rather than a library: `Declare`, `PtrSafe`,
`LongPtr`, `AddressOf`, callbacks, `VarPtr`, `StrPtr`, and `ObjPtr` are all part
of the language.

### Modules and namespaces

```vb
Module MathTools
    Public Function Add(ByVal left As Integer, ByVal right As Integer) As Integer
        Return left + right
    End Function
End Module

Namespace Game
Namespace Graphics

Public Class Sprite
End Class

End Namespace
End Namespace
```

Nested namespace wrappers flatten into a qualified namespace such as
`Game.Graphics`.

### More

Valo also has examples and tests for `AndAlso` / `OrElse` short-circuiting,
nullable `T?` values, default properties, static locals, multidimensional arrays
and `ReDim Preserve`, collection initializers, LINQ-style extension APIs, async
declaration syntax, and `#If` / `#Const` / `Option Base` / `Option Compare`.

See the [examples](examples/README.md) and the [language docs](docs/language).

## Performance

Performance is treated as a feature and tracked by a benchmark suite in
[`bench/`](bench/README.md).

```sh
cargo build --release -p valo_cli
./scripts/bench.sh
```

Recent work removed a quadratic array-write path and the per-statement
allocations on the interpreter's hot path:

| Benchmark | Before | After | Speedup |
|---|---|---|---|
| Array fill and read (100k elements) | 195 s | 0.72 s | 270x |
| Nested arithmetic loop (2M iterations) | 4.74 s | 1.16 s | 4.1x |
| String building (40k iterations) | 0.25 s | 0.07 s | 3.6x |

Both columns were measured on the same machine, back to back, comparing the
commit before this work against the current build.

The interpreter is still a tree walker. A bytecode VM is the next significant
step, and these benchmarks exist to keep that work honest.

## Supported file types

| Extension | Mode | Purpose |
|---|---|---|
| `.valo` | Native Valo | The language; VB.NET syntax is the reference |
| `.bas` | VBA compatibility | Legacy standard modules, for migration |
| `.cls` | VBA compatibility | Exported class modules, for migration |

## CLI

```sh
valo run examples/hello.valo           # run a program
valo check examples/generic_box.valo   # validate without running
valo repl                              # interactive REPL
valo version
valo help
```

`valo check` runs the full semantic analysis: declarations, procedure bodies,
types, and cross-module resolution.

```txt
valo> Dim x As Integer
valo> x = 10
valo> Console.WriteLine(x)
10
```

## Getting started

### Install via script

Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/valolang/valo/main/scripts/install.sh | bash
```

Windows (PowerShell):

```powershell
iwr -UseBasicParsing https://raw.githubusercontent.com/valolang/valo/main/scripts/install.ps1 | iex
```

Restart your terminal to apply PATH changes, then verify:

```sh
valo version
```

### Manual download

Release assets are published at <https://github.com/valolang/valo/releases>:

| Platform | Asset |
|---|---|
| Linux x64 | `valo-linux-x64.tar.gz` |
| Linux x86 | `valo-linux-x86.tar.gz` |
| macOS ARM64 | `valo-macos-arm64.tar.gz` |
| macOS x64 | `valo-macos-x64.tar.gz` |
| Windows x64 | `valo-windows-x64.zip` |
| Windows x86 | `valo-windows-x86.zip` |

### Build from source

Requires Rust stable.

```sh
git clone https://github.com/valolang/valo
cd valo
cargo build --release
./target/release/valo version
```

## Your first Valo program

Create `hello.valo`:

```vb
Sub Main()
    Dim name As String = "Valo"
    Console.WriteLine($"Hello, {name}")
End Sub
```

Run it:

```sh
valo run hello.valo
```

```txt
Hello, Valo
```

## Diagnostics

Diagnostics carry explicit codes, source spans, suggestions, stack traces, and
import-cycle reporting.

```txt
error[V1100]: Cannot assign String value to Integer variable
 --> examples/demo.valo:4:9
  |
4 |     age = "twenty"
  |         ^^^^^^^^^ expected Integer
```

Codes are stable: once released, a code keeps its meaning and a retired one is
never reused, so they are safe to search for, match on in tooling, and cite in a
bug report. Every code is listed in
[the diagnostics reference](docs/reference/diagnostics.md), which a test keeps in
step with the compiler.

## VBA compatibility

VBA compatibility is a migration bridge. It exists so an existing codebase can
move to Valo incrementally, not to keep VBA semantics alive where they conflict
with VB.NET. **When the two disagree, Valo follows VB.NET.**

`.bas` and `.cls` files load directly, including exported class modules,
`Attribute VB_Name`, `Attribute VB_UserMemId`, and classic function-assignment
semantics. The compatibility runtime covers `On Error` / `Err` / `Resume` / `Erl`,
`Debug.Print`, `Variant` / `Object` / `Empty` / `Null`, file-number I/O (`Open`,
`Input #`, `Line Input #`, `Print #`, `Write #`, `Get #`, `Put #`, with Random and
Binary modes), Windows COM automation through `CreateObject`, and a broad surface
of Microsoft-indexed VBA constants and functions spanning math, financial, date,
formatting, string, conversion, type-checking, file, dialog, color, shell, and
pointer helpers.

Coverage is pragmatic, and some host-specific functions are deliberately partial:
`MacScript` reports an unsupported-runtime diagnostic, `GetSetting` returns the
supplied default when no host settings store exists, and COM depends on the host
platform.

Full detail lives in [docs/language/vba-compat.md](docs/language/vba-compat.md).

## Documentation

Language:

- [Getting started](docs/getting-started.md)
- [Syntax](docs/language/syntax.md)
- [Expressions and operators](docs/language/expressions.md)
- [Strings and interpolation](docs/language/strings.md)
- [Types and conversions](docs/language/types.md)
- [Classes and objects](docs/language/classes.md)
- [Inheritance](docs/language/inheritance.md)
- [Generics](docs/language/generics.md)
- [Modules and imports](docs/language/modules.md)
- [Error handling](docs/language/error-handling.md)
- [FFI](docs/language/ffi.md)
- [VBA compatibility](docs/language/vba-compat.md)
- [REPL](docs/repl.md)
- [Diagnostic codes](docs/reference/diagnostics.md)
- [Examples](examples/README.md)

Architecture:

- [Overview](docs/architecture/README.md)
- [Frontend](docs/architecture/frontend.md)
- [Backend](docs/architecture/backend.md)
- [Runtime](docs/architecture/runtime.md)
- [Parser](docs/architecture/parser.md)
- [Diagnostics](docs/architecture/diagnostics.md)
- [Module system](docs/architecture/modules.md)
- [Platform](docs/architecture/platform.md)
- [Performance](docs/architecture/performance.md)
- [Roadmap](docs/architecture/roadmap.md)

## Contributing

Valo is in active development. Useful places to start:

- VB.NET syntax and semantics that Valo does not cover yet
- language tests and examples
- diagnostics quality
- interpreter performance, measured against [`bench/`](bench/README.md)
- runtime builtins and the standard library
- CLI and REPL improvements

Before submitting changes:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
cargo test -p valo_core --test examples -- --nocapture
cargo build --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

## License

Valo is licensed under the [MIT License](LICENSE).
