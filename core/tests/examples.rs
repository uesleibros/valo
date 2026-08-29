use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_official_examples() {
    let examples_dir = examples_dir();
    let golden_dir = examples_dir.join("golden");
    let bless = std::env::var_os(BLESS_VARIABLE).is_some();
    let entries = runnable_examples(&examples_dir);

    let mut failures = Vec::new();
    let mut count = 0;
    let mut skipped = 0;

    for path in entries {
        let file_name = path.file_name().and_then(|s| s.to_str());

        if should_skip_example(file_name) {
            skipped += 1;
            continue;
        }

        count += 1;
        match valo_core::run_file(&path) {
            Ok(output) => {
                if let Err(mismatch) = check_transcript(&golden_dir, &path, &output, bless) {
                    failures.push(mismatch);
                }
            }
            Err(diag) => {
                failures.push(format!("{:?}: Failed with error: {}", path, diag));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Integration tests failed for {}/{} examples:\n\n{}",
            failures.len(),
            count,
            failures.join("\n")
        );
    }

    println!("Successfully ran {} examples ({} skipped).", count, skipped);
}

fn examples_dir() -> PathBuf {
    let path = Path::new("examples");
    if path.exists() {
        return path.to_path_buf();
    }
    Path::new("../examples").to_path_buf()
}

fn runnable_examples(examples_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_runnable_examples(examples_dir, examples_dir, &mut paths);
    paths.sort();
    paths
}

fn collect_runnable_examples(root: &Path, dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("Failed to read examples directory {:?}: {err}", dir))
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            collect_runnable_examples(root, &path, paths);
        } else if is_runnable_example(root, &path) {
            paths.push(path);
        }
    }
}

fn is_runnable_example(root: &Path, path: &Path) -> bool {
    if !is_source_file(path) {
        return false;
    }
    if !has_sub_main(path) {
        return false;
    }

    match path.parent() {
        Some(parent) if parent == root => true,
        _ => path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("main")),
    }
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("valo" | "bas" | "cls")
    )
}

fn has_sub_main(path: &Path) -> bool {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read example {:?}: {err}", path));
    source.lines().any(|line| {
        line.trim_start()
            .to_ascii_lowercase()
            .starts_with("sub main")
    })
}

/// Setting this records the transcripts instead of comparing against them.
const BLESS_VARIABLE: &str = "VALO_BLESS";

/// Compares an example's output against its recorded transcript.
///
/// Running an example only proves it does not crash. An example that quietly
/// changed what it prints would still pass, which is no use as a safety net for
/// work on the interpreter: the transcripts make the current behaviour explicit,
/// so a change to it has to be one someone meant to make.
///
/// Run the suite with `VALO_BLESS=1` to record a new example or accept an
/// intended change.
fn check_transcript(
    golden_dir: &Path,
    example: &Path,
    output: &[String],
    bless: bool,
) -> Result<(), String> {
    let name = example
        .file_name()
        .and_then(|name| name.to_str())
        .expect("example paths are valid UTF-8");
    let transcript_path = golden_dir.join(format!("{name}.txt"));
    let actual = transcript(output);

    if bless {
        fs::create_dir_all(golden_dir)
            .map_err(|err| format!("could not create {golden_dir:?}: {err}"))?;
        fs::write(&transcript_path, &actual)
            .map_err(|err| format!("could not write {transcript_path:?}: {err}"))?;
        return Ok(());
    }

    let Ok(expected) = fs::read_to_string(&transcript_path) else {
        return Err(format!(
            "{example:?} has no recorded transcript. Run the suite with {BLESS_VARIABLE}=1 to record one."
        ));
    };

    // Transcripts are compared as text so a mismatch reads as a diff rather
    // than as two debug-printed vectors.
    let expected = expected.replace("\r\n", "\n");
    if expected == actual {
        return Ok(());
    }

    Err(format!(
        "{example:?} no longer prints what was recorded.\n\
         --- recorded ---\n{expected}\
         --- actual ---\n{actual}\
         Run the suite with {BLESS_VARIABLE}=1 if this change was intended."
    ))
}

/// Renders output as one escaped line per printed line.
///
/// Escaping matters: some examples print the VBA carriage-return constants, so
/// a transcript can contain a CR that is *data*. Written raw it would be
/// indistinguishable from a line ending, and any normalisation of line endings
/// -- which a checkout may perform -- would silently corrupt it.
fn transcript(output: &[String]) -> String {
    let mut text = String::new();
    for line in output {
        let escaped = line
            .replace('\\', "\\\\")
            .replace('\r', "\\r")
            .replace('\n', "\\n");
        text.push_str(&escaped);
        text.push('\n');
    }
    text
}

fn should_skip_example(file_name: Option<&str>) -> bool {
    if let Some(name) = file_name
        && cfg!(not(windows))
        && name.starts_with("com_")
    {
        return true;
    }
    false
}
