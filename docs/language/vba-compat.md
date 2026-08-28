# VBA Compatibility

Valo follows VB.NET. VBA compatibility exists so an existing VBA codebase can move
to Valo incrementally, not to keep VBA semantics alive where they conflict with
VB.NET.

> **When VBA and VB.NET disagree, Valo follows VB.NET.**

That rule decides the cases this document does not enumerate. `.bas` and `.cls`
files load and run, VBA runtime functions and constants are available, and
classic idioms such as `On Error` and function-name assignment work — but the
language those files are read as is Valo, and Valo's reference is VB.NET.

This document outlines the bridge layer between `.valo` and `.bas`/`.cls` files,
the supported compatibility surface, and the intentional differences.

## The Bridge Layer

Valo distinguishes between modern native code and legacy compatibility code primarily through file extensions and specific syntax choices.

### Source Modes
*   **`.valo` Files:** Modern native syntax. Prefers class `Sub New`/`Sub Terminate`, `Structure`, `Default` keyword, and structured imports.
*   **`.bas` / `.cls` Files:** VBA compatibility mode. Supports `Attribute VB_*` metadata, `Class_Initialize`, `Class_Terminate`, `Type`, `Declare`, and common exported-module encodings.

### Feature Comparison

| Feature | Native Valo (`.valo`) | VBA Compatibility (`.bas`/`.cls`) |
|---------|----------------------|-----------------------------------|
| Constructor | `Public Sub New()` | `Private Sub Class_Initialize()` |
| Destructor | `Public Sub Terminate()` | `Private Sub Class_Terminate()` |
| Default Member | `Public Default Property Get Item()` | `Attribute Item.VB_UserMemId = 0` on the Get/Let/Set property group |
| Value Records | `Public Structure Point` | `Public Type Point` |
| Byte Arrays | `Dim data() As Byte` | `Dim data() As Byte` |
| Debug Output | `Console.WriteLine` | `Debug.Print` |
| Error Handling | `Try/Catch/Finally` | `On Error GoTo` |
| Array Bounds | `1 To N` (optional) | `1 To N` (optional) |

### Built-in Compatibility
- VBA project-style module groups: importing any `.bas` or `.cls` file automatically loads sibling `.bas` and `.cls` files from the same directory. Those sibling compatibility modules can reference each other's public declarations without explicit `Imports`, matching the flat project namespace used by exported VBA projects while keeping modern `.valo` imports explicit.
- The [standard library reference](standard-library.md) tracks built-in functions, constants, and compatibility levels as Implemented, Recognized limited, Platform-dependent, or Stub / diagnostic.
- `Debug.Print`: Available in all file modes, outputs to the standard console. Supports multiple comma-separated arguments.
- `Err` Object: Full support for `Err.Raise`, `Err.Number`, and `Err.Description` in all modes.
- `Array Built-ins`: `Array`, `Split`, `Join`, `Filter`, `LBound`, and `UBound` behave according to standard VBA semantics.
- VBA runtime constants: the Microsoft Learn VBA constant groups are exposed case-insensitively, both unqualified and through the `VBA.` namespace. This includes Calendar, CallType, Color, Comparison, Date/DateFormat, Dir/GetAttr/SetAttr, DriveType, File Attribute, File I/O, Form, IMEStatus, Keycode, Miscellaneous, MsgBox, QueryClose, Shell, SpecialFolder, StrConv, System Color, TriState, and VarType constants.
- VBA runtime functions: all function names from the Microsoft Learn VBA function index are recognized by semantic validation. The runtime implements the practical cross-platform subset across arrays, strings, math, financial calculations, conversion, type checking, date/time, date intervals, formatting, selection helpers, file helpers, dialogs, color helpers, shell helpers, settings helpers, COM activation, and pointer helpers. VBA `$` string-returning spellings such as `Left$`, `Chr$`, `Hex$`, and `Trim$` parse and dispatch through the same runtime intrinsics.
- String and formatting helpers: `Len`, `LenB`, `Left`, `Right`, `Mid`, `Trim`, `LTrim`, `RTrim`, `UCase`, `LCase`, `Replace`, `InStr`, `InStrRev`, `Space`, `String`, `StrConv`, `StrReverse`, `Chr`, `ChrW`, `Asc`, `AscW`, `Val`, `Str`, `Hex`, `Oct`, `StrComp`, `Format`, `FormatNumber`, `FormatCurrency`, `FormatPercent`, `FormatDateTime`, and `Partition` are available. Formatting is deterministic and intentionally less locale-dependent than Office VBA.
- Math, financial, and selection helpers: `Abs`, `Atn`, `Cos`, `Exp`, `Fix`, `Int`, `Log`, `Round`, `Sgn`, `Sin`, `Sqr`, `Tan`, `DDB`, `FV`, `IPmt`, `IRR`, `MIRR`, `NPer`, `NPV`, `Pmt`, `PPmt`, `PV`, `Rate`, `SLN`, `SYD`, `Choose`, `IIf`, and `Switch` are available.
- Conversion and type helpers: `CBool`, `CByte`, `CCur`, `CDate`, `CDbl`, `CDec`, `CInt`, `CLng`, `CLngLng`, `CLngPtr`, `CStr`, `CVar`, `CVErr`, `IsArray`, `IsDate`, `IsEmpty`, `IsError`, `IsMissing`, `IsNull`, `IsNumeric`, `IsObject`, `TypeName`, and `VarType` are available.
- Classic VBA file I/O: file-number I/O is supported for the practical local-file subset: `FreeFile`, `Open ... For Input|Output|Append|Binary|Random [Access ...] [Lock ...] As #n [Len = n]`, `Close`, `Line Input #`, `Input #`, `Input(...)`, `Print #`, `Write #`, `Get #`, `Put #`, `EOF`, `FileAttr`, `LOF`, `Loc`, `Seek(n)`, and `Seek #n, position`. File state is owned by the interpreter instance rather than global mutable process state.
- Binary and Random files: `Get #` and `Put #` use explicit little-endian serialization for scalar `Byte`, `Integer`, `Long`, `Int64`, `Single`, `Double`, `Boolean`, strings, and Byte arrays. `Random` mode requires `Len =`, uses 1-based record numbers, and pads/truncates written record bytes to the record length. User-defined record binary layout is intentionally deferred until Valo has an explicit portable layout contract.
- Access and Lock clauses: `Access Read`, `Access Write`, and `Access Read Write` are parsed and enforced by the Valo runtime for reads and writes. `Lock Read`, `Lock Write`, and `Lock Read Write` are parsed and stored as advisory compatibility metadata; Valo does not yet claim cross-process OS file locking parity.
- Filesystem helpers: `Dir`, `GetAttr`, `Kill`, `FileLen`, `FileDateTime`, `CurDir`, `ChDir`, `MkDir`, `RmDir`, and `Name old As new` are available for local filesystem compatibility. `Dir` supports exact paths, `*`/`?` wildcard enumeration, repeated `Dir()` continuation calls, `vbDirectory`, basic read-only filtering, Unix-style dotfile hidden matching, and Windows hidden metadata where available.
- Date/time helpers: `Timer`, `Now`, `Date`, `Time`, `DateAdd`, `DateDiff`, `DatePart`, `DateSerial`, `TimeSerial`, `DateValue`, `TimeValue`, `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`, `Weekday`, `MonthName`, and `WeekdayName` are implemented using Valo's VBA serial `Date` value. `DateValue`/`TimeValue` intentionally accept a small deterministic parse subset rather than locale-dependent VBA parsing.
- Host and platform helpers: `Command`, `DoEvents`, `Environ`, `Error`, `GetSetting`, `GetAllSettings`, `IMEStatus`, `InputBox`, `MacID`, `MacScript`, `MsgBox`, `QBColor`, `RGB`, `Shell`, `Spc`, and `Tab` are recognized. Some host-specific behavior is intentionally pragmatic: `MacScript` reports an unsupported-runtime diagnostic, `GetSetting` returns the supplied default when no host settings store exists, and `GetAllSettings` returns an empty array.
- `Option Private Module`: Parsed and tracked on imported modules. Same-project explicit imports and implicit `.bas` sibling access remain available, while project resolution is import-driven rather than treating every loaded module as an ambient global candidate.
- `Multidimensional Arrays`: Fully supported with `ReDim Preserve` compatibility (last-dimension only resizing).
- `New ClassName`: Parentheses are optional for zero-argument construction, matching VBA (`Set v = New Vec2`).
- `Const`: Module, local, and class-scope constants are supported, including multi-Const declarations such as `Public Const PI = 3.14, E = 2.71`.
- `^` and unary signs: Exponent expressions are supported and evaluate through numeric promotion. Unary `+` and `-` accept all numeric literal suffixes and numeric expression forms; exponentiation binds tighter than unary sign, so `-2 ^ 2` evaluates as `-(2 ^ 2)`.
- `Declare`/`PtrSafe`: `Declare Function` and `Declare Sub` are callable at runtime through the native FFI layer. Private declares are visible inside their module, public declares can be imported, and declare functions support expression calls, bare statement calls, and `Call`. `Lib`, `Alias`, `PtrSafe`, `LongPtr`, `LongLong`, `As Any`, ByVal/ByRef parameters, `StdCall`, and the `CDecl` extension are supported with clean diagnostics for unsupported marshaling.
- Memory and Pointers: `VarPtr`, `StrPtr`, and `ObjPtr` are supported as builtins. `StrPtr` accepts string variables and temporary string expressions such as literals, `CStr(...)`, and `Left$(...)`; temporaries are owned by the interpreter for the statement call duration. `AddressOf` generates libffi closure trampolines, enabling robust, native callbacks.
- Property procedures: `Property Let` and `Property Set` accept omitted `ByVal` on the value parameter for VBA import compatibility.
- Source encodings and diagnostics: `.bas` and `.cls` imports accept UTF-8, UTF-8 BOM, UTF-16 LE/BE BOM, and Windows-1252/ANSI fallback, with normalized line endings. Parser and runtime diagnostics report the exact source file path, line, and column for errors inside loaded compatibility modules.

`Structure` is the native Valo value type and supports methods, properties, constructors, and copy semantics. `Type` remains the VBA-compatible fields-only record syntax.
Structure fields may use constant-expression defaults, for example `Public X As Double = 0#`.

## Modern Bridging

Valo provides modern features that complement legacy VBA code, making the transition to standalone development smoother.

### Short-circuiting and Null Safety
`AndAlso` and `OrElse` allow for safer object access patterns than legacy `And` and `Or`, which always evaluate both sides. This is especially useful when bridging between nullable Valo types and legacy VBA objects.

### Partial Classes
The `Partial` keyword allows splitting large legacy classes into multiple files, making them easier to refactor into modern Valo code over time.

### Collection Initializers
`New Collection() From { ... }` simplifies the common VBA pattern of repeated `.Add` calls when initializing global or class-level state.

## Intentional Differences

While Valo strives for high compatibility, it is not a "bug-for-bug" clone. Some differences are intentional to improve safety and performance:

1.  **Strict Validation:** Valo performs comprehensive semantic analysis before execution. Many errors that VBA only catches at runtime (like type mismatches in assignments) are caught during compilation in Valo.
2.  **Explicit Scoping:** In modern `.valo` files, cross-module access requires explicit `Import` statements, whereas VBA modules share a global namespace.
3.  **COM Automation:** Valo supports OLE Automation on Windows via `CreateObject`, late-binding, and `For Each` enumeration, providing a familiar experience for VBA developers controlling external applications.
4.  **Modern Keywords:** Keywords like `Return` are preferred for returning values from functions and properties, although name-based assignment is still supported for compatibility.
5.  **Native Boundary Diagnostics:** VBA may crash or corrupt state on an invalid external declaration. Valo reports loader, symbol, ABI, pointer-safety, and marshaling failures as diagnostics where it can detect them.
6.  **Runtime Function Scope:** Valo recognizes the full Microsoft Learn VBA function index and implements the practical standalone subset, but some functions are host-specific, platform-dependent, or intentionally deterministic rather than Office/locale dependent. `MacScript`, registry-backed settings functions, COM activation, dialog behavior, formatting, financial edge cases, and shell behavior should be treated as compatibility surfaces that may keep gaining parity.
7.  **File I/O Scope:** `Open` currently focuses on deterministic local file access. Exact print zones, full OS file-locking semantics, platform-specific archive/system/volume/alias attributes, locale-dependent date parsing, and portable binary UDT layout are not claimed as complete VBA parity. `ChDir` changes the process working directory, matching classic behavior but remaining process-wide.

## Compatibility Goals

*   **Migration Support:** Allow existing `.bas` and `.cls` files to be dropped into a Valo project and "just work" where practical.
*   **Ergonomic Bridge:** Native Valo code should be able to call into VBA-style modules and vice-versa seamlessly.
*   **Modern Foundation:** Ensure that the compatibility layer doesn't compromise the safety or performance of the core runtime.
