# Language Reference

Documentation for the Valo language features.

*   **[Syntax Overview](syntax.md):** Basic types, variables, and control flow.
*   **[Expressions](expressions.md):** Operators, precedence, compound assignment, shifts, short-circuiting (`AndAlso`, `OrElse`), and numeric behavior.
*   **[Strings and Interpolation](strings.md):** String literals, concatenation, and `$"..."` with format specifiers and alignment.
*   **[Types and Conversions](types.md):** Native structures, nullable types (`T?`), byte arrays, and `CType` / `DirectCast` / `TryCast` / `GetType` / `NameOf`.
*   **[Generics](generics.md):** VB.NET-style generic classes, structures, functions, lambdas, and runtime strategy.
*   **[Functions](functions.md):** Procedures, lambdas (`Function(x) ...`), argument passing, and optional arguments.
*   **[Async and Await](async.md):** VB.NET-style async declarations, await validation, and current interpreter behavior.
*   **[Classes and Objects](classes.md):** Lifecycle, auto-properties, events (`AddHandler`), and visibility.
*   **[Properties](properties.md):** Property Get/Let/Set compatibility rules.
*   **[Inheritance](inheritance.md):** Base classes, overrides, abstract members, and protected visibility.
*   **[Modules and Imports](modules.md):** Project organization and dependency management.
*   **[Error Handling](error-handling.md):** Robust runtime failure management.
*   **[VBA Compatibility](vba-compat.md):** The migration bridge from VBA: `.bas`/`.cls` loading, built-in constants, runtime functions, file I/O, COM, and FFI bridge behavior. Where VBA and VB.NET disagree, Valo follows VB.NET.
*   **[Standard Library Reference](standard-library.md):** Built-in functions, constants, compatibility levels, and known caveats.
*   **[COM Automation](com.md):** Windows COM/OLE Automation through `Object`, `CreateObject`, and late-bound calls.
*   **[FFI](ffi.md):** Native library declarations, pointer types, callbacks, and platform-aware loading.
