# Properties

Properties use VBA-compatible `Property Get`, `Property Let`, and `Property Set` procedures.

```vb
Property Let Name(value As String)
    m_Name = value
End Property
```

The final value parameter for `Property Let` and `Property Set` may omit `ByVal`, matching common VBA and exported `.cls` code. Explicit `ByVal` and `ByRef` remain accepted.

## Indexed properties

A property may take arguments, which makes it an indexer:

```vb
Class Store
    Private Slots(9) As String

    Public Property Get Item(ByVal index As Long) As String
        Return Slots(index)
    End Property

    Public Property Let Item(ByVal index As Long, ByVal value As String)
        Slots(index) = value
    End Property
End Class

Dim s As New Store()
s.Item(2) = "two"
Console.WriteLine(s.Item(2))
```

The value a write carries is the accessor's last parameter, after however many
indices it takes. A plain property has only that one.

## Overloaded accessors

Accessors of one kind may share a name when their parameters differ, and the use
site picks between them the way a call to an overloaded method does:

```vb
Public Property Get Item(ByVal index As Long) As String
Public Property Get Item(ByVal key As String) As String
```

Reading `Item(2)` reaches the first and `Item("k")` the second. A write is
resolved by its whole shape, indices and value together, so `Let` accessors
overload the same way. Two accessors of one kind with the same parameter types
are rejected where they are declared, since no use could choose between them.
