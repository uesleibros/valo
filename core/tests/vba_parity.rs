use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use valo_core::backend::interpreter::run;
use valo_core::frontend::parser::Parser;
use valo_core::frontend::semantics::validate;
use valo_core::runtime::FileId;

fn exec(source: &str) -> Vec<String> {
    let program = Parser::parse_source(source, FileId::default()).expect("Parse failed");
    validate(&program).expect("Validation failed");
    run(&program).expect("Run failed")
}

fn exec_error(source: &str) -> String {
    let program = Parser::parse_source(source, FileId::default()).expect("Parse failed");
    validate(&program).expect("Validation failed");
    run(&program).expect_err("Run should fail").to_string()
}

fn unique_temp_path(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "valo_vba_parity_{}_{}_{}",
        std::process::id(),
        stamp,
        name
    ))
}

fn valo_string_literal(path: &std::path::Path) -> String {
    path.display().to_string().replace('"', "\"\"")
}

#[test]
fn test_lset_rset() {
    let source = r#"
        Sub Main()
            Dim s As String
            s = "12345"
            LSet s = "abc"
            Console.WriteLine(s)
            RSet s = "abc"
            Console.WriteLine(s)
            s = "12345"
            LSet s = "abcdefg"
            Console.WriteLine(s)
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "abc  ");
    assert_eq!(output[1], "  abc");
    assert_eq!(output[2], "abcde");
}

#[test]
fn test_math_function_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine(Abs(-5) & "," & Fix(-1.7) & "," & Int(-1.7) & "," & Sgn(-2))
            Console.WriteLine(Round(1.25, 1) & "," & Sqr(4) & "," & Log(1))
            Console.WriteLine(Sin(0) & "," & Cos(0) & "," & Tan(0) & "," & (Atn(1) > 0))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "5,-1,-2,-1");
    assert_eq!(output[1], "1.3,2,0");
    assert_eq!(output[2], "0,1,0,True");
}

#[test]
fn test_financial_function_parity() {
    let source = r#"
        Sub Main()
            Dim values As Variant
            values = Array(-100, 60, 60)
            Console.WriteLine(Round(SLN(1000, 100, 5), 2) & "," & Round(SYD(1000, 100, 5, 1), 2) & "," & Round(DDB(1000, 100, 5, 1), 2))
            Console.WriteLine(Round(FV(0.1, 2, -10), 2) & "," & Round(PV(0.1, 2, -10), 2) & "," & Round(Pmt(0.1, 2, 100), 2))
            Console.WriteLine(Round(NPV(0.1, values), 2) & "," & Round(IRR(values), 4) & "," & Round(Rate(2, -10, 17.355371900826), 4))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "180,300,400");
    assert_eq!(output[1], "21,17.36,-57.62");
    assert_eq!(output[2], "3.76,0.1307,0.1");
}

#[test]
fn test_financial_function_diagnostics() {
    let error = exec_error(
        r#"
        Sub Main()
            Console.WriteLine(IRR(Array(10, 20, 30)))
        End Sub
    "#,
    );
    assert!(error.contains("IRR requires at least one positive and one negative cash flow"));
}

#[test]
fn test_formatting_function_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine(Format(12.3, "0.00"))
            Console.WriteLine(FormatNumber(12.345, 1))
            Console.WriteLine(FormatCurrency(12.3, 2))
            Console.WriteLine(FormatPercent(0.125, 1))
            Console.WriteLine(FormatDateTime(DateSerial(2024, 2, 3), vbShortDate))
            Console.WriteLine(FormatDateTime(DateSerial(2024, 2, 3) + TimeSerial(1, 2, 3), vbLongTime))
            Console.WriteLine(Format(DateSerial(2024, 2, 3), "Long Date"))
            Console.WriteLine(Format(1234.5, "Standard"))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "12.30");
    assert_eq!(output[1], "12.3");
    assert_eq!(output[2], "$12.30");
    assert_eq!(output[3], "12.5%");
    assert_eq!(output[4], "2/3/2024");
    assert_eq!(output[5], "01:02:03");
    assert_eq!(output[6], "Saturday, February 3, 2024");
    assert_eq!(output[7], "1,234.50");
}

#[test]
fn test_date_interval_function_parity() {
    let source = r#"
        Sub Main()
            Dim d As Date
            d = DateSerial(2024, 1, 31)
            Console.WriteLine(Year(DateAdd("m", 1, d)) & "," & Month(DateAdd("m", 1, d)) & "," & Day(DateAdd("m", 1, d)))
            Console.WriteLine(Year(DateAdd("q", 1, d)) & "," & Month(DateAdd("q", 1, d)) & "," & Day(DateAdd("q", 1, d)))
            Console.WriteLine(Year(DateAdd("yyyy", 1, DateSerial(2024, 2, 29))) & "," & Month(DateAdd("yyyy", 1, DateSerial(2024, 2, 29))) & "," & Day(DateAdd("yyyy", 1, DateSerial(2024, 2, 29))))
            Console.WriteLine(DateDiff("d", DateSerial(2024, 1, 1), DateSerial(2024, 1, 10)))
            Console.WriteLine(DateDiff("ww", DateSerial(2024, 1, 1), DateSerial(2024, 1, 15)))
            Console.WriteLine(DatePart("yyyy", DateSerial(2024, 2, 3)) & "," & DatePart("m", DateSerial(2024, 2, 3)) & "," & DatePart("q", DateSerial(2024, 5, 1)))
            Console.WriteLine(Hour(TimeSerial(1, 2, 3)) & "," & Minute(TimeSerial(1, 2, 3)) & "," & Second(TimeSerial(1, 2, 3)))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "2024,2,29");
    assert_eq!(output[1], "2024,4,30");
    assert_eq!(output[2], "2025,2,28");
    assert_eq!(output[3], "9");
    assert_eq!(output[4], "2");
    assert_eq!(output[5], "2024,2,2");
    assert_eq!(output[6], "1,2,3");
}

#[test]
fn test_file_helper_parity() {
    let path = unique_temp_path("file.txt");
    fs::write(&path, b"hello").expect("write temp file");
    let literal = valo_string_literal(&path);
    let source = format!(
        r#"
        Sub Main()
            Console.WriteLine(FileLen("{literal}"))
            Console.WriteLine((GetAttr("{literal}") And vbDirectory) = 0)
            Console.WriteLine(Dir("{literal}") <> "")
        End Sub
    "#
    );
    let output = exec(&source);
    fs::remove_file(&path).expect("remove temp file");
    assert_eq!(output, vec!["5", "True", "True"]);
}

#[test]
fn test_settings_helper_parity() {
    let source = r#"
        Sub Main()
            Dim settings As Variant
            settings = GetAllSettings("valo", "missing")
            Console.WriteLine(GetSetting("valo", "missing", "key", "fallback"))
            Console.WriteLine(IsArray(settings))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output, vec!["fallback", "True"]);
}

#[test]
fn test_shell_helper_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine(Shell("exit 0") > 0)
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output, vec!["True"]);
}

#[test]
fn test_color_helper_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine(RGB(1, 2, 3))
            Console.WriteLine(QBColor(1) & "," & QBColor(15))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output, vec!["197121", "8388608,16777215"]);
}

#[test]
fn test_selection_helper_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine(Choose(2, "a", "b", "c"))
            Console.WriteLine(IsNull(Choose(4, "a", "b", "c")))
            Console.WriteLine(IIf(True, "yes", "no"))
            Console.WriteLine(Switch(False, "x", True, "y"))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output, vec!["b", "True", "yes", "y"]);
}

#[test]
fn test_unsupported_host_feature_diagnostics() {
    let error = exec_error(
        r#"
        Sub Main()
            Console.WriteLine(MacScript("return 1"))
        End Sub
    "#,
    );
    assert!(error.contains("Compatibility diagnostic: MacScript requires an Office VBA Mac host"));
    assert!(error.contains("replace MacScript"));
}

#[test]
fn test_like_advanced() {
    let source = r#"
        Sub Main()
            Console.WriteLine("1:" & ("abc" Like "a[a-z]c"))
            Console.WriteLine("2:" & ("a5c" Like "a[0-9]c"))
            Console.WriteLine("3:" & ("abc" Like "a[!0-9]c"))
            Console.WriteLine("4:" & ("a*c" Like "a[*]c"))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "1:True");
    assert_eq!(output[1], "2:True");
    assert_eq!(output[2], "3:True");
    assert_eq!(output[3], "4:True");
}

#[test]
fn test_builtins_parity() {
    let source = r#"
        Sub Main()
            Console.WriteLine("1:" & IsNumeric(" 123.4 "))
            Console.WriteLine("2:" & IsDate("2026-05-27"))
            Dim p As Variant
            p = Split("a b c", " ")
            Console.WriteLine("3:" & Join(p, ","))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "1:True");
    assert_eq!(output[1], "2:True");
    assert_eq!(output[2], "3:a,b,c");
}

#[test]
fn test_collection() {
    let source = r#"
        Sub Main()
            Dim c As New Collection
            c.Add "item1", "key1"
            c.Add "item2"
            Console.WriteLine("C1:" & c.Count)
            Console.WriteLine("C2:" & c("key1"))
            Console.WriteLine("C3:" & c(2))
            
            Dim s As String
            s = ""
            Dim v As Variant
            For Each v In c
                s = s & v & ","
            Next v
            Console.WriteLine("C4:" & s)
            
            c.Remove "key1"
            Console.WriteLine("C5:" & c.Count)
            Console.WriteLine("C6:" & c(1))
        End Sub
    "#;
    let output = exec(source);
    assert_eq!(output[0], "C1:2");
    assert_eq!(output[1], "C2:item1");
    assert_eq!(output[2], "C3:item2");
    assert_eq!(output[3], "C4:item1,item2,");
    assert_eq!(output[4], "C5:1");
    assert_eq!(output[5], "C6:item2");
}
