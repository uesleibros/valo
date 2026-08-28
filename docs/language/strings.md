# Strings and Interpolation

Valo strings are `String` values. They compare and concatenate with `&`, and they
support VB.NET-style interpolated literals.

## String literals

A string literal is written with double quotes. A doubled quote inside a literal
produces one quote character.

```vb
Console.WriteLine("Hello")
Console.WriteLine("She said ""hello""")
```

## Concatenation

Use `&` to concatenate. It converts non-string operands to their display form, so
it is the safe choice for building text out of mixed values.

```vb
Dim count As Long = 3
Console.WriteLine("You have " & count & " items")
```

`&=` appends in place:

```vb
Dim message As String = "Valo"
message &= " 1.0"
```

`+` also concatenates two strings, but it is addition first and concatenation
second. Prefer `&` for text.

## Interpolated strings

An interpolated string is a literal prefixed with `$`. Expressions inside braces
are evaluated and rendered in place.

```vb
Sub Main()
    Dim user As String = "Ada"
    Dim items As Long = 3

    Console.WriteLine($"Hello, {user}! You have {items} items.")
End Sub
```

```txt
Hello, Ada! You have 3 items.
```

A hole holds any expression, including calls and arithmetic:

```vb
Console.WriteLine($"Upper: {UCase(user)}")
Console.WriteLine($"Next year: {items + 1}")
Console.WriteLine($"First letter: {Left(user, 1)}")
```

### Escapes

A doubled brace produces a literal brace, and a doubled quote produces a literal
quote, as in an ordinary string literal.

```vb
Console.WriteLine($"Literal braces: {{not a hole}}")
Console.WriteLine($"She said ""hello"", {user}.")
```

```txt
Literal braces: {not a hole}
She said "hello", Ada.
```

### Format specifiers

A hole may carry a format specifier after a colon. The specifier is handed to the
same implementation that backs the `Format` function, so `$"{value:0.00}"` and
`Format(value, "0.00")` always produce the same text.

```vb
Dim price As Double = 12.5
Console.WriteLine($"Total: {price:0.00}")
```

```txt
Total: 12.50
```

### Alignment

A hole may carry an alignment after a comma. A positive width pads on the left
(right-aligns the value) and a negative width pads on the right. Alignment never
truncates: a value wider than the field is written in full.

```vb
Dim name As String = "Valo"
Console.WriteLine($"[{name,10}]")
Console.WriteLine($"[{name,-10}]")
```

```txt
[      Valo]
[Valo      ]
```

Alignment and a format specifier can be combined, alignment first:

```vb
Console.WriteLine($"{item.Name,-12}{item.Price,10:0.00}")
```

This makes fixed-width tables straightforward:

```vb
Console.WriteLine($"{"Product",-12}{"Price",10}{"Stock",8}")
Console.WriteLine($"{"Keyboard",-12}{49.9,10:0.00}{12,8}")
```

```txt
Product          Price   Stock
Keyboard         49.90      12
```

### Restrictions

An interpolated string is evaluated at run time, so it cannot initialize a
`Const`:

```vb
Const Greeting As String = $"Hello, {name}"  ' error
```

Each hole must contain exactly one expression, and an empty hole (`{}`) is a
parse error.

## Related

- [Expressions and operators](expressions.md)
- [Types and conversions](types.md)
- [Example: string interpolation](../../examples/string_interpolation.valo)
