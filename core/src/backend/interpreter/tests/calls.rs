use crate::backend::interpreter::tests::helpers::*;

#[test]
fn runs_simple_sub_call() {
    let output = run_source(
        r#"
Sub SayHello()
    Console.WriteLine("Hello")
End Sub

Sub Main()
    SayHello()
End Sub
"#,
    );

    assert_eq!(output, vec!["Hello"]);
}

#[test]
fn runs_sub_call_with_byval_parameter() {
    let output = run_source(
        r#"
Sub Show(ByVal value As String)
    Console.WriteLine(value)
End Sub

Sub Main()
    Show("Valo")
End Sub
"#,
    );

    assert_eq!(output, vec!["Valo"]);
}

#[test]
fn test_runtime_function_assignment() {
    let source = r#"
        Function Soma(ByVal a As Long, ByVal b As Long) As Long
            Soma = a + b
        End Function

        Sub Main()
            Console.WriteLine(Soma(10, 20))
        End Sub
    "#;
    let output = run_source(source);
    assert_eq!(output, vec!["30"]);
}

#[test]
fn function_can_recurse_through_its_own_name() {
    let output = run_source(
        r#"
Function Fib(ByVal n As Long) As Long
    If n < 2 Then
        Fib = n
    Else
        Fib = Fib(n - 1) + Fib(n - 2)
    End If
End Function

Sub Main()
    Console.WriteLine(Fib(10))
End Sub
"#,
    );

    assert_eq!(output, vec!["55"]);
}

#[test]
fn function_returning_an_array_is_still_indexable_by_its_own_name() {
    let output = run_source(
        r#"
Function Build() As Long
    Dim slots(0 To 2) As Long
    slots(1) = 42
    Build = slots(1)
End Function

Sub Main()
    Console.WriteLine(Build())
End Sub
"#,
    );

    assert_eq!(output, vec!["42"]);
}

#[test]
fn multi_line_function_lambda_returns_a_value() {
    let output = run_source(
        r#"
Sub Main()
    Dim classify = Function(n As Long) As String
        If n < 0 Then
            Return "negative"
        ElseIf n = 0 Then
            Return "zero"
        End If
        Return "positive"
    End Function

    Console.WriteLine(classify(-5))
    Console.WriteLine(classify(0))
    Console.WriteLine(classify(7))
End Sub
"#,
    );

    assert_eq!(output, vec!["negative", "zero", "positive"]);
}

#[test]
fn multi_line_sub_lambda_runs_as_a_statement() {
    let output = run_source(
        r#"
Sub Main()
    Dim shout = Sub(message As String)
        Dim loud As String = UCase(message)
        Console.WriteLine(loud & "!")
    End Sub

    shout("hello")
End Sub
"#,
    );

    assert_eq!(output, vec!["HELLO!"]);
}

#[test]
fn single_expression_lambdas_still_work() {
    let output = run_source(
        r#"
Sub Main()
    Dim twice = Function(x As Long) x * 2
    Console.WriteLine(twice(21))
End Sub
"#,
    );

    assert_eq!(output, vec!["42"]);
}

#[test]
fn a_lambda_must_be_closed_by_the_matching_end_keyword() {
    let diagnostic = source_diagnostic(
        r#"
Sub Main()
    Dim broken = Function(x As Long) As Long
        Return x
    End Sub
End Sub
"#,
    );

    assert_eq!(diagnostic.code, crate::runtime::DiagnosticCode::PARSE);
}
