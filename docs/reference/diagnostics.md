# Diagnostic Codes

Every diagnostic Valo reports carries a code. Codes are stable: once released, a
code keeps its meaning, and a retired one is never reused. That makes them safe
to search for, to match on in tooling, and to cite in a bug report.

Codes are declared in one place,
[`core/src/runtime/diagnostic.rs`](../../core/src/runtime/diagnostic.rs), which
produces both the constants the compiler uses and the table this page is checked
against. A test fails if a code is added without appearing here.

`V0001` is the one code with no specific meaning: it marks a diagnostic that has
not been given a better code yet. Seeing it is a signal that the diagnostic
deserves one.


## Syntax

Raised before a program has any meaning: the source does not parse, or a directive is malformed.

| Code | Name | Meaning |
|---|---|---|
| `V0001` | `GENERIC` | An error that has not been given a more specific code yet. |
| `V0100` | `PARSE` | The source does not form a valid program. |
| `V0101` | `OPTION` | An `Option` directive is misplaced, repeated, or unrecognized. |
| `V0102` | `PREPROCESSOR` | A conditional-compilation directive is malformed. |


## Semantics

Raised by the analyzer, and by the interpreter where the same rule is enforced again at run time.

| Code | Name | Meaning |
|---|---|---|
| `V1001` | `UNKNOWN_NAME` | A name is used but never declared. |
| `V1002` | `DUPLICATE_DECLARATION` | A name is declared twice in the same scope. |
| `V1003` | `MEMBER_IS_PRIVATE` | A member exists but is not visible from here. |
| `V1100` | `TYPE_MISMATCH` | A value cannot be used where that type is required. |
| `V1101` | `INVALID_ASSIGNMENT` | The target of an assignment cannot be assigned to. |
| `V1102` | `ARGUMENT_NOT_OPTIONAL` | A required argument was omitted. |
| `V1103` | `ARGUMENT_COUNT` | A call passes a number of arguments the callee does not accept. |
| `V1104` | `ARITHMETIC` | An arithmetic operation has no defined result, such as division by zero. |
| `V1200` | `ARRAY` | An array is indexed, sized, or used incorrectly. |
| `V1300` | `CONTROL_FLOW` | A control-flow statement appears where it cannot apply. |
| `V1400` | `MEMBER_ACCESS` | A type does not have the member being accessed. |
| `V1500` | `SELECT_CASE` | A `Select Case` arm is malformed. |
| `V1600` | `MODULE_NOT_FOUND` | An imported module could not be located. |
| `V1601` | `DUPLICATE_IMPORT` | The same import alias is bound twice. |
| `V1602` | `IMPORT_CYCLE` | Modules import each other in a cycle. |
| `V1603` | `AMBIGUOUS_IMPORT` | A name is provided by more than one import. |
| `V1604` | `CASE_COLLISION` | Two names differ only by case, which cannot be told apart. |
| `V1605` | `UNKNOWN_QUALIFIED_SYMBOL` | A qualified name does not exist in that module. |
| `V1606` | `INVALID_QUALIFIED_ACCESS` | A qualified name exists but cannot be used this way. |


## Native interop

Raised by `Declare` and the calls it introduces.

| Code | Name | Meaning |
|---|---|---|
| `V3001` | `FFI_LIBRARY_NOT_FOUND` | A `Declare` names a library that could not be loaded. |
| `V3002` | `FFI_SYMBOL_NOT_FOUND` | A `Declare` names a symbol the library does not export. |
| `V3003` | `FFI_UNSUPPORTED_MARSHALING` | A `Declare` uses a type Valo cannot pass to native code. |
| `V3004` | `FFI_CALL` | A native call failed. |


## Runtime

Raised while a program runs.

| Code | Name | Meaning |
|---|---|---|
| `V9000` | `RUNTIME` | A runtime failure with no more specific code. |
| `V9001` | `RUNTIME_ERROR` | An error raised by the program itself, through `Err.Raise` or `Throw`. |
