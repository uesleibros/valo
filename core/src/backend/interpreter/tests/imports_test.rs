use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::run_file;

fn temp_project() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("valo_imports_test_{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &std::path::Path, name: &str, source: &str) {
    fs::write(dir.join(name), source).unwrap();
}

#[test]
fn imports_modern_syntax_works() {
    let dir = temp_project();
    write(
        &dir,
        "Math.valo",
        "Namespace Math\nPublic Function Add(a As Integer, b As Integer) As Integer\nReturn a + b\nEnd Function\nEnd Namespace",
    );
    write(
        &dir,
        "main.valo",
        "Imports Math\n\nSub Main()\nConsole.WriteLine(Math.Add(1, 2))\nEnd Sub\n",
    );

    assert_eq!(
        run_file(dir.join("main.valo")).unwrap(),
        vec!["3".to_string()]
    );
}

#[test]
fn imports_with_alias_works() {
    let dir = temp_project();
    write(
        &dir,
        "Math.valo",
        "Namespace Math\nPublic Function Add(a As Integer, b As Integer) As Integer\nReturn a + b\nEnd Function\nEnd Namespace",
    );
    write(
        &dir,
        "main.valo",
        "Imports M = Math\n\nSub Main()\nConsole.WriteLine(M.Add(1, 2))\nEnd Sub\n",
    );

    assert_eq!(
        run_file(dir.join("main.valo")).unwrap(),
        vec!["3".to_string()]
    );
}

#[test]
fn an_imported_module_can_name_types_from_its_own_imports() {
    let dir = temp_project();
    write(
        &dir,
        "Shapes.valo",
        "Public Class Dot\nPublic N As Double\nEnd Class\n",
    );
    write(
        &dir,
        "Middle.valo",
        "Imports Shapes\n\nPublic Function MakeDot() As Dot\nDim d As New Dot()\nd.N = 42\nReturn d\nEnd Function\n",
    );
    write(
        &dir,
        "main.valo",
        "Imports Middle\n\nSub Main()\nConsole.WriteLine(Middle.MakeDot().N)\nEnd Sub\n",
    );

    assert_eq!(
        run_file(dir.join("main.valo")).unwrap(),
        vec!["42".to_string()]
    );
}

#[test]
fn a_module_can_name_its_own_enum_and_an_importer_can_too() {
    let dir = temp_project();
    write(
        &dir,
        "Kinds.valo",
        "Public Enum Kind\nFirst_ = 1\nSecond_ = 2\nEnd Enum\n\n\
         Public Function Named() As Long\nReturn Kind.Second_\nEnd Function\n",
    );
    write(
        &dir,
        "main.valo",
        "Imports Kinds\n\nSub Main()\nConsole.WriteLine(Kinds.Named())\n\
         Console.WriteLine(Kind.First_)\nEnd Sub\n",
    );

    assert_eq!(
        run_file(dir.join("main.valo")).unwrap(),
        vec!["2".to_string(), "1".to_string()]
    );
}

#[test]
fn an_imported_module_can_use_its_own_imports_in_a_signature() {
    let dir = temp_project();
    write(
        &dir,
        "Shapes.valo",
        "Public Structure Point_\nPublic X As Double\nEnd Structure\n",
    );
    // Middle names Shapes' type in an interface and a class. A third module
    // importing Middle knows nothing of Shapes, and does not have to.
    write(
        &dir,
        "Middle.valo",
        "Imports Shapes\n\n\
         Public Interface IHasPoint\nFunction Where() As Point_\nEnd Interface\n\n\
         Public Class Marker\nImplements IHasPoint\n\
         Public Function Where() As Point_ Implements IHasPoint.Where\n\
         Dim p As Point_\np.X = 3\nReturn p\nEnd Function\nEnd Class\n",
    );
    write(
        &dir,
        "main.valo",
        "Imports Middle\n\nSub Main()\nDim m As New Marker()\n\
         Console.WriteLine(m.Where().X)\nEnd Sub\n",
    );

    assert_eq!(
        run_file(dir.join("main.valo")).unwrap(),
        vec!["3".to_string()]
    );
}
