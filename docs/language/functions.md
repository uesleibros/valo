# Functions and Argument Passing

Valo keeps VBA-compatible parameter defaults: omitted `ByVal`/`ByRef` is parsed as `ByRef`, but expression arguments, literals, coercions, and incompatible variable types are passed through temporary copy values where VBA would commonly do so.

Optional parameters preserve omitted state for `IsMissing`. If an omitted optional value is used where a concrete value is required, diagnostics explain that the optional argument was omitted instead of reporting a generic variable error.

## Lambdas

A lambda is an inline procedure. It comes in a single-expression form and a
multi-line form.

### Single-expression lambdas

The body is one expression, and its value is the result.

```vb
Dim twice = Function(x As Long) x * 2
Console.WriteLine(twice(21))   ' 42
```

### Multi-line lambdas

When a newline follows the header, the body is a statement block closed by
`End Function`. The result comes from `Return`.

```vb
Dim classify = Function(n As Long) As String
    If n < 0 Then
        Return "negative"
    ElseIf n = 0 Then
        Return "zero"
    End If
    Return "positive"
End Function
```

The result type after `As` is optional; the interpreter infers the result from
the value that is returned.

A `Sub` lambda has no result and is always multi-line. It is invoked as a
statement rather than in an expression.

```vb
Dim banner = Sub(title As String)
    Console.WriteLine(UCase(title))
End Sub

banner("done")
```

A lambda must be closed by the keyword that opened it: `Function` with
`End Function`, `Sub` with `End Sub`.

### Passing lambdas around

A lambda is a value, so it can be stored, passed, and returned.

```vb
Function Accumulate(ByVal count As Long, ByVal transform As Variant) As Long
    Dim total As Long = 0
    Dim i As Long
    For i = 1 To count
        total += transform(i)
    Next i
    Return total
End Function

Console.WriteLine(Accumulate(5, Function(n As Long) n * n))   ' 55
```

Closures are not captured yet: a lambda sees its parameters, not the variables
of the scope that created it.

## Related

- [Example: lambdas](../../examples/lambdas.valo)
