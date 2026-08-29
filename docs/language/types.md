# Types

Valo has native scalar types, arrays, classes, enums, and value types.

## Native Structures

Use `Structure` for new value-type code:

```vb
Public Structure Point
    Public X As Integer
    Public Y As Integer

    Public Sub New(ByVal x As Integer, ByVal y As Integer)
        X = x
        Y = y
    End Sub

    Public Function Sum() As Integer
        Return X + Y
    End Function

    Public Property Get IsZero() As Boolean
        Return X = 0 And Y = 0
    End Property
End Structure

Dim p As New Point(10, 20)
Console.WriteLine(p.Sum())
```

Structures are value types. Assignment, `ByVal` parameter passing, and function returns copy the value. `ByRef` parameters and mutating methods called on a variable can update the original structure.

Structures support:

- Fields
- `Sub` and `Function` methods
- `Property Get` and `Property Let`
- `Sub New` constructors with at least one parameter
- Default `Property Get` indexers
- Module imports, including qualified construction

Structures do not support inheritance, interfaces, events, `WithEvents`, `Class_Initialize`, `Class_Terminate`, `Terminate`, reference identity with `Is`, `Set` assignment, or `Nothing`.

Calling a mutating structure method requires an assignable receiver such as a variable or `ByRef` parameter. Temporary values are treated as copies.

## VBA-Compatible Types

`Type ... End Type` remains supported for VBA compatibility and is fields-only:

```vb
Public Type Point
    X As Integer
    Y As Integer
End Type
```

Fields inside a `Type` use plain VBA UDT syntax. Do not prefix fields with `Public`, `Private`, or `Dim`.

Prefer `Structure` in new `.valo` code and keep `Type` for migrated VBA code.

## Class vs Structure vs Type

- `Class`: reference type with identity, lifecycle, events, default properties, and object references.
- `Structure`: native value type with fields, methods, properties, constructors, and copy semantics.
- `Type`: VBA-compatible fields-only record syntax.

## Nullable Types

Valo supports VB.NET-style nullable types for both value and reference types using the `?` suffix.

```vb
Dim age As Integer? = Nothing
If age Is Nothing Then
    Console.WriteLine("Age is unknown")
End If

age = 25
If age.HasValue Then
    Console.WriteLine("Age is: " & age.Value)
End If
```

### Synthetic Properties

Nullable types expose two read-only properties:

- `.HasValue`: Returns `True` if the variable contains a value, `False` if it is `Nothing`.
- `.Value`: Returns the underlying value. Accessing `.Value` when the variable is `Nothing` will result in a runtime error.

### Lifted Operators

Arithmetic and logical operators are "lifted" for nullable types. If either operand is `Nothing`, the result of the operation is `Nothing`.

```vb
Dim a As Integer? = 10
Dim b As Integer? = Nothing
Dim sum As Integer? = a + b ' Result is Nothing
```

### Reference Types

Reference types (like `String` or classes) can also use the nullable suffix for clarity, though they already support `Nothing`.

```vb
Dim s As String? = Nothing
```

## Byte Arrays

Use Basic-style array syntax for byte buffers:

```vb
Dim data() As Byte
ReDim data(0 To 15)
data(0) = CByte(255)
```

C-style square-bracket array spelling is not the official Valo syntax.

## Conversions

### CType

`CType(value, Type)` converts between compatible types, following the same rules
as assignment and the `C*` builtins.

```vb
Dim count As Integer = CType("42", Integer)
Dim text As String = CType(7, String)
```

`CType` is the right choice for value types and for anything that needs an actual
conversion rather than a reinterpretation.

### DirectCast

`DirectCast(value, Type)` reinterprets a reference without converting it. It
succeeds when the value already is of the target type, following the inheritance
chain, and reports a type mismatch otherwise.

```vb
Dim rex As New Dog()
Dim animal As Animal = DirectCast(rex, Animal)
```

Because it never converts, `DirectCast` is both cheaper and stricter than
`CType`. Reach for it when you know the runtime type and want the compiler and
runtime to hold you to it.

### TryCast

`TryCast(value, Type)` behaves like `DirectCast` but answers `Nothing` instead of
failing when the value is not of the target type. It requires a reference type,
since a value type has no `Nothing` to answer with.

```vb
Dim maybeDog As Dog = TryCast(animal, Dog)

If maybeDog IsNot Nothing Then
    Console.WriteLine(maybeDog.Breed)
End If
```

### Choosing between them

| | Converts | On mismatch |
|---|---|---|
| `CType` | yes | converts, or reports a type mismatch |
| `DirectCast` | no | reports a type mismatch |
| `TryCast` | no | answers `Nothing` |

## Reflection

### GetType

`GetType(Type)` reports a type's name, using the casing it was declared with. It
is resolved at compile time.

```vb
Console.WriteLine(GetType(Dog))       ' Dog
Console.WriteLine(GetType(Integer))   ' Integer
```

For the type of a *value* rather than a type name, use `TypeName(value)`.

### NameOf

`NameOf(x)` reports the source name of its operand as a string, resolved at
compile time. For a member access it names the final member, matching VB.NET.

```vb
Dim total As Long
Console.WriteLine(NameOf(total))             ' total
Console.WriteLine(NameOf(customer.Address))  ' Address
```

`NameOf` is useful for diagnostics and validation messages that should survive a
rename.

## Related

- [Expressions and operators](expressions.md)
- [Example: conversions and reflection](../../examples/conversions.valo)

## Tuples

A tuple groups a fixed number of values without declaring a type for them:

```vb
Dim pair = (1, "two")
Console.WriteLine(pair.Item1)      ' 1
Console.WriteLine(pair.Item2)      ' two
```

Elements can be named, which gives each a second way in; the positional name
still works:

```vb
Dim point = (X := 3, Y := 4)
Console.WriteLine(point.X)         ' 3
Console.WriteLine(point.Item1)     ' 3
```

The type is written out where it is used, including as a return type:

```vb
Function Divide(ByVal a As Long, ByVal b As Long) As (Quotient As Long, Remainder As Long)
    Return (Quotient := a \ b, Remainder := a Mod b)
End Function
```

`Dim` with parentheses names the elements of a tuple as separate variables,
which usually reads better than reaching through the tuple:

```vb
Dim (q, r) = Divide(17, 5)
Console.WriteLine(q & " remainder " & r)
```

Tuples are structural: two tuple types are the same when their elements are,
whatever the elements are called. Elements convert the way any other value
does, so `(1, 2)` fits `(Double, Double)`. A tuple is copied when it is
assigned, as a `Structure` is, and it cannot cross the native boundary --
pass its elements instead.

## Related

- [Example: tuples](../../examples/tuples.valo)
