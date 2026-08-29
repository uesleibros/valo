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

### Closures

A lambda closes over the scope that created it, so it can use the variables in
that scope as well as its own parameters.

```vb
Dim factor As Long = 3
Dim triple = Function(x As Long) x * factor

Console.WriteLine(triple(5))   ' 15
```

Capture is by reference, not by copy. The lambda sees assignments made after it
was created, and the scope sees what the lambda assigns:

```vb
factor = 10
Console.WriteLine(triple(5))   ' 50

Dim total As Long = 0
Dim add = Sub(n As Long)
    total = total + n
End Sub

add(4)
add(6)
Console.WriteLine(total)       ' 10
```

A parameter, or a variable declared inside the lambda, shadows a captured name
of the same spelling and leaves the outer variable untouched:

```vb
Dim value As Long = 1
Dim shadowed = Function(value As Long) value

Console.WriteLine(shadowed(7))   ' 7
Console.WriteLine(value)         ' 1
```

Captures survive the scope that created them, so a lambda can be returned from
the function that built it, and a lambda inside a lambda captures from both:

```vb
Function MakeAdder(ByVal delta As Long) As Variant
    Dim base As Long = 100
    Return Function(x As Long) x + delta + base
End Function
```

## Overloads

Several procedures may share one name as long as their parameters differ. The
call decides which is meant, by how many arguments it passes and what type they
have:

```vb
Function Area(ByVal side As Double) As Double
    Return side * side
End Function

Function Area(ByVal width As Double, ByVal height As Double) As Double
    Return width * height
End Function

Console.WriteLine(Area(3))      ' 9
Console.WriteLine(Area(3, 4))   ' 12
```

`Overloads` may be written before `Sub` or `Function` to say the sharing is
deliberate. It is optional -- Valo works overloading out from the declarations
themselves -- and is accepted so VB.NET source carries over unchanged.

### How one is chosen

Each candidate is measured argument by argument. A candidate wins when it is at
least as good as every rival everywhere and strictly better somewhere:

| Fit | Example |
|---|---|
| the same type | `Long` argument, `Long` parameter |
| a conversion that loses nothing, nearest first | `Integer` prefers `Long` over `Double` |
| `Variant` on either side | resolved by whatever the value turns out to be |
| a conversion that can lose something | `Double` argument, `Long` parameter |

A candidate better for one argument and worse for another does not win on
balance; the call is reported as ambiguous, and a conversion or a named
argument says which was meant. Two procedures whose parameters have the same
types are rejected where they are declared, since no call could tell them
apart.

### What does not overload

- A `Sub` and a `Function` cannot share a name. A bare `f()` in an expression
  has to mean one or the other.
- `AddressOf` on an overloaded name is rejected: taking an address passes no
  arguments, so nothing says which one is meant. Wrap the one you want in a
  lambda.
- Class methods do not overload yet; this is module-level procedures, including
  `Declare` and extension methods.

## Related

- [Example: overloads](../../examples/overloads.valo)
- [Example: lambdas](../../examples/lambdas.valo)
