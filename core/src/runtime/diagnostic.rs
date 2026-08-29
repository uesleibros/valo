use std::fmt;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file_id: FileId,
    pub start: SourcePos,
    pub end: SourcePos,
}

impl Span {
    pub fn new(file_id: FileId, start: SourcePos, end: SourcePos) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    pub fn empty(file_id: FileId) -> Self {
        Self::new(file_id, SourcePos::new(1, 1), SourcePos::new(1, 1))
    }
}

#[derive(Debug, Clone)]
pub struct SourceMap {
    sources: Vec<Source>,
}

#[derive(Debug, Clone)]
struct Source {
    name: String,
    content: String,
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, content: String) -> FileId {
        let id = FileId(self.sources.len() as u32);
        self.sources.push(Source { name, content });
        id
    }

    pub fn get_name(&self, id: FileId) -> Option<&str> {
        self.sources.get(id.0 as usize).map(|s| s.name.as_str())
    }

    pub fn get_content(&self, id: FileId) -> Option<&str> {
        self.sources.get(id.0 as usize).map(|s| s.content.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: Box<str>,
    pub span: Option<Box<Span>>,
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub runtime_error: Option<Box<RuntimeErrorInfo>>,
    pub labels: Box<Vec<DiagnosticLabel>>,
    pub notes: Box<Vec<String>>,
    pub helps: Box<Vec<String>>,
    pub related: Box<Vec<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeErrorInfo {
    pub number: i64,
    pub source: String,
    pub description: String,
    pub help_file: String,
    pub help_context: i64,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            message: message.into().into_boxed_str(),
            span: span.map(Box::new),
            severity: Severity::Error,
            labels: Box::default(),
            notes: Box::default(),
            helps: Box::default(),
            related: Box::default(),
            runtime_error: None,
        }
    }

    pub fn with_code(mut self, code: DiagnosticCode) -> Self {
        self.code = code;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_primary_label(mut self, message: impl Into<String>) -> Self {
        if let Some(span) = &self.span {
            self.labels.push(DiagnosticLabel::primary(**span, message));
        }
        self
    }

    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel::secondary(span, message));
        self
    }

    pub fn with_note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(message.into());
        self
    }

    pub fn with_help(mut self, message: impl Into<String>) -> Self {
        self.helps.push(message.into());
        self
    }

    pub fn with_name_suggestion<'a>(
        self,
        misspelled: &str,
        candidates: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        if let Some(candidate) = suggest_name(misspelled, candidates) {
            self.with_help(format!("did you mean '{candidate}'?"))
        } else {
            self
        }
    }

    pub fn with_available_items<'a>(
        mut self,
        label: &str,
        items: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let mut items: Vec<_> = items.into_iter().collect();
        items.sort_unstable_by_key(|item| item.to_ascii_lowercase());
        items.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        if !items.is_empty() {
            let mut note = String::from(label);
            note.push(':');
            for item in items {
                note.push_str("\n  - ");
                note.push_str(item);
            }
            self.notes.push(note);
        }
        self
    }

    pub fn with_related(mut self, diagnostic: Diagnostic) -> Self {
        self.related.push(diagnostic);
        self
    }

    pub fn with_runtime_error(mut self, info: RuntimeErrorInfo) -> Self {
        self.runtime_error = Some(Box::new(info));
        self
    }

    pub fn render(&self, source_map: &SourceMap) -> String {
        self.render_colored(source_map, terminal_supports_color())
    }

    pub fn render_colored(&self, source_map: &SourceMap, use_color: bool) -> String {
        let mut out = String::new();
        let color = ColorSupport::new(use_color);
        let gutter_width = diagnostic_gutter_width(self);

        out.push_str(&format!(
            "{}[{}]: {}{}{}\n",
            color.severity(self.severity),
            self.code,
            color.bold(""),
            self.message,
            color.reset()
        ));

        if let Some(span) = &self.span {
            let source_name = source_map.get_name(span.file_id).unwrap_or("<unknown>");
            out.push_str(&format!(
                "{}{}--> {}{}:{}:{}{}\n",
                " ".repeat(gutter_width),
                color.gutter(),
                color.bold(""),
                source_name,
                span.start.line,
                span.start.column,
                color.reset()
            ));
            render_empty_gutter(&mut out, gutter_width, &color);

            if let Some(source) = source_map.get_content(span.file_id) {
                render_span_lines(&mut out, source, **span, &self.labels, &color, gutter_width);
            }
        }

        for note in self.notes.iter() {
            out.push_str(&format!(
                "{}{}= {}note{}: {}\n",
                " ".repeat(gutter_width),
                color.gutter(),
                color.note(),
                color.reset(),
                note
            ));
        }
        for help in self.helps.iter() {
            out.push_str(&format!(
                "{}{}= {}help{}: {}\n",
                " ".repeat(gutter_width),
                color.gutter(),
                color.help(),
                color.reset(),
                help
            ));
        }
        for related in self.related.iter() {
            out.push_str(&related.render_colored(source_map, use_color));
            out.push('\n');
        }

        out.trim_end().to_string()
    }
}

pub fn suggest_name<'a>(
    misspelled: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let needle = misspelled.to_ascii_lowercase();
    let max_distance = (needle.chars().count() / 3).clamp(2, 3);
    candidates
        .into_iter()
        .filter(|candidate| !candidate.is_empty())
        .map(|candidate| {
            (
                candidate,
                edit_distance(&needle, &candidate.to_ascii_lowercase()),
            )
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .min_by_key(|(candidate, distance)| (*distance, candidate.len()))
        .map(|(candidate, _)| candidate.to_string())
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_ch) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_ch) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_ch != *right_ch);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution_cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

pub fn terminal_supports_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() || !std::io::stderr().is_terminal() {
        return false;
    }
    #[cfg(windows)]
    {
        if std::env::var_os("WT_SESSION").is_some()
            || std::env::var_os("ANSICON").is_some()
            || std::env::var_os("ConEmuANSI").is_some()
            || std::env::var("TERM")
                .map(|term| term != "dumb")
                .unwrap_or(false)
        {
            return true;
        }
        false
    }
    #[cfg(not(windows))]
    {
        std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(false)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some(span) => write!(
                f,
                "{}[{}]: {} at line {}, column {}",
                self.severity, self.code, self.message, span.start.line, span.start.column
            ),
            None => write!(f, "{}[{}]: {}", self.severity, self.code, self.message),
        }
    }
}

impl std::error::Error for Diagnostic {}

struct ColorSupport {
    enabled: bool,
}

impl ColorSupport {
    fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    fn bold(&self, code: &str) -> String {
        if self.enabled {
            format!("{}{}", code, "\x1b[1m")
        } else {
            String::new()
        }
    }

    fn gutter(&self) -> &str {
        if self.enabled { "\x1b[34;1m" } else { "" }
    }

    fn note(&self) -> &str {
        if self.enabled { "\x1b[36;1m" } else { "" }
    }

    fn help(&self) -> &str {
        if self.enabled { "\x1b[32;1m" } else { "" }
    }

    fn primary(&self) -> &str {
        if self.enabled { "\x1b[31;1m" } else { "" }
    }

    fn secondary(&self) -> &str {
        if self.enabled { "\x1b[33;1m" } else { "" }
    }

    fn severity(&self, severity: Severity) -> String {
        if !self.enabled {
            return severity.to_string();
        }
        let code = match severity {
            Severity::Error => "\x1b[31;1m",
            Severity::Warning => "\x1b[33;1m",
            Severity::Note => "\x1b[36;1m",
            Severity::Help => "\x1b[32;1m",
        };
        format!("{code}{severity}\x1b[0m")
    }

    fn reset(&self) -> &str {
        if self.enabled { "\x1b[0m" } else { "" }
    }
}

fn diagnostic_gutter_width(diagnostic: &Diagnostic) -> usize {
    max_diagnostic_line(diagnostic).to_string().len().max(1)
}

fn max_diagnostic_line(diagnostic: &Diagnostic) -> usize {
    let mut max_line = diagnostic
        .span
        .as_ref()
        .map(|span| span.start.line.max(span.end.line))
        .unwrap_or(0);

    for label in diagnostic.labels.iter() {
        max_line = max_line.max(label.span.start.line).max(label.span.end.line);
    }
    for related in diagnostic.related.iter() {
        max_line = max_line.max(max_diagnostic_line(related));
    }

    max_line
}

fn render_empty_gutter(out: &mut String, gutter_width: usize, color: &ColorSupport) {
    out.push_str(&format!(
        "{}{} |{}\n",
        " ".repeat(gutter_width),
        color.gutter(),
        color.reset()
    ));
}

fn render_span_lines(
    out: &mut String,
    source: &str,
    primary: Span,
    labels: &[DiagnosticLabel],
    color: &ColorSupport,
    gutter_width: usize,
) {
    let primary_label = labels
        .iter()
        .find(|label| label.style == LabelStyle::Primary && label.span == primary)
        .map(|label| label.message.as_str())
        .unwrap_or("");

    render_labeled_span(
        out,
        source,
        primary,
        primary_label,
        LabelStyle::Primary,
        color,
        gutter_width,
    );

    for label in labels
        .iter()
        .filter(|label| label.style == LabelStyle::Secondary)
    {
        render_empty_gutter(out, gutter_width, color);
        render_labeled_span(
            out,
            source,
            label.span,
            &label.message,
            LabelStyle::Secondary,
            color,
            gutter_width,
        );
    }

    render_empty_gutter(out, gutter_width, color);
}

fn render_labeled_span(
    out: &mut String,
    source: &str,
    span: Span,
    label: &str,
    style: LabelStyle,
    color: &ColorSupport,
    gutter_width: usize,
) {
    let source_line = source
        .lines()
        .nth(span.start.line.saturating_sub(1))
        .unwrap_or("");
    let displayed_line = expand_tabs(source_line);
    let marker_offset = visual_offset_for_column(source_line, span.start.column);
    let marker_width = visual_span_width(source_line, span).max(1);
    let marker = match style {
        LabelStyle::Primary => "^",
        LabelStyle::Secondary => "-",
    };
    let marker_color = match style {
        LabelStyle::Primary => color.primary(),
        LabelStyle::Secondary => color.secondary(),
    };
    let label_suffix = if label.is_empty() {
        String::new()
    } else {
        format!(" {label}")
    };

    out.push_str(&format!(
        "{}{:>width$} |{} {}\n",
        color.gutter(),
        span.start.line,
        color.reset(),
        displayed_line,
        width = gutter_width
    ));
    out.push_str(&format!(
        "{}{} |{} {}{}{}{}{}\n",
        " ".repeat(gutter_width),
        color.gutter(),
        color.reset(),
        " ".repeat(marker_offset),
        marker_color,
        marker.repeat(marker_width),
        color.reset(),
        label_suffix
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        };
        write!(f, "{text}")
    }
}

/// Diagnostic code scheme:
/// V0000 generic diagnostics, V0100 syntax/options/preprocessor,
/// V1000 name/declaration/member lookup, V1100 typing/assignment,
/// V1200 arrays, V1300 control flow, V3000 native FFI, and V9000 runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCode(pub &'static str);

/// Declares every diagnostic code once.
///
/// The macro produces both the constants the compiler refers to and a table the
/// documentation and tests read, so a code cannot exist without a summary and
/// the two cannot fall out of step. Codes are permanent: a released code keeps
/// its meaning, and a retired one is not reused.
macro_rules! diagnostic_codes {
    ($($name:ident = $code:literal, $summary:literal;)*) => {
        impl DiagnosticCode {
            $(pub const $name: Self = Self($code);)*
        }

        /// Every diagnostic code, with the constant that names it and a summary.
        pub const ALL_DIAGNOSTIC_CODES: &[(DiagnosticCode, &str, &str)] = &[
            $((DiagnosticCode($code), stringify!($name), $summary),)*
        ];
    };
}

diagnostic_codes! {
    GENERIC = "V0001", "An error that has not been given a more specific code yet.";
    PARSE = "V0100", "The source does not form a valid program.";
    OPTION = "V0101", "An `Option` directive is misplaced, repeated, or unrecognized.";
    PREPROCESSOR = "V0102", "A conditional-compilation directive is malformed.";
    UNKNOWN_NAME = "V1001", "A name is used but never declared.";
    DUPLICATE_DECLARATION = "V1002", "A name is declared twice in the same scope.";
    MEMBER_IS_PRIVATE = "V1003", "A member exists but is not visible from here.";
    INVALID_DECLARATION = "V1004", "A declaration parses but is not valid, such as an `Optional` parameter before a required one.";
    ENTRY_POINT = "V1005", "The program has no `Sub Main()`, or the one it has cannot serve as the entry point.";
    TYPE_MISMATCH = "V1100", "A value cannot be used where that type is required.";
    INVALID_ASSIGNMENT = "V1101", "The target of an assignment cannot be assigned to.";
    ARGUMENT_NOT_OPTIONAL = "V1102", "A required argument was omitted.";
    ARGUMENT_COUNT = "V1103", "A call passes a number of arguments the callee does not accept.";
    ARITHMETIC = "V1104", "An arithmetic operation has no defined result, such as division by zero.";
    MISSING_RETURN = "V1105", "A `Function` can finish without returning a value.";
    NAMED_ARGUMENT = "V1106", "A named argument does not match a parameter, or names the same one twice.";
    AMBIGUOUS_OVERLOAD = "V1107", "A call fits more than one overload equally well.";
    ARRAY = "V1200", "An array is indexed, sized, or used incorrectly.";
    CONTROL_FLOW = "V1300", "A control-flow statement appears where it cannot apply.";
    MEMBER_ACCESS = "V1400", "A type does not have the member being accessed.";
    SELECT_CASE = "V1500", "A `Select Case` arm is malformed.";
    MODULE_NOT_FOUND = "V1600", "An imported module could not be located.";
    DUPLICATE_IMPORT = "V1601", "The same import alias is bound twice.";
    IMPORT_CYCLE = "V1602", "Modules import each other in a cycle.";
    AMBIGUOUS_IMPORT = "V1603", "A name is provided by more than one import.";
    CASE_COLLISION = "V1604", "Two names differ only by case, which cannot be told apart.";
    UNKNOWN_QUALIFIED_SYMBOL = "V1605", "A qualified name does not exist in that module.";
    INVALID_QUALIFIED_ACCESS = "V1606", "A qualified name exists but cannot be used this way.";
    PACKAGE_MANIFEST = "V1607", "`valo.toml` is malformed.";
    FFI_LIBRARY_NOT_FOUND = "V3001", "A `Declare` names a library that could not be loaded.";
    FFI_SYMBOL_NOT_FOUND = "V3002", "A `Declare` names a symbol the library does not export.";
    FFI_UNSUPPORTED_MARSHALING = "V3003", "A `Declare` uses a type Valo cannot pass to native code.";
    FFI_CALL = "V3004", "A native call failed.";
    COM = "V3100", "A COM object could not be created, or a call into one failed.";
    RUNTIME = "V9000", "A runtime failure with no more specific code.";
    RUNTIME_ERROR = "V9001", "An error raised by the program itself, through `Err.Raise` or `Throw`.";
    FILE_IO = "V9002", "A file or directory operation failed.";
    UNSUPPORTED = "V9003", "A VBA feature the standalone Valo runtime does not provide.";
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

impl DiagnosticLabel {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

fn visual_span_width(source_line: &str, span: Span) -> usize {
    if span.start.line == span.end.line {
        let start = visual_offset_for_column(source_line, span.start.column);
        let end = visual_offset_for_column(source_line, span.end.column);
        end.saturating_sub(start).max(1)
    } else {
        1
    }
}

fn visual_offset_for_column(source_line: &str, column: usize) -> usize {
    source_line
        .chars()
        .take(column.saturating_sub(1))
        .fold(0, |offset, ch| offset + char_width(ch, offset))
}

fn expand_tabs(source_line: &str) -> String {
    let mut expanded = String::with_capacity(source_line.len());
    let mut offset = 0;
    for ch in source_line.chars() {
        if ch == '\t' {
            let width = char_width(ch, offset);
            expanded.push_str(&" ".repeat(width));
            offset += width;
        } else {
            expanded.push(ch);
            offset += char_width(ch, offset);
        }
    }
    expanded
}

fn char_width(ch: char, offset: usize) -> usize {
    if ch == '\t' { 4 - (offset % 4) } else { 1 }
}

#[cfg(test)]
mod tests {
    /// A code is an identity, so two diagnostics must never share one.
    ///
    /// They are hand-assigned strings, and a duplicate would be invisible:
    /// both diagnostics would report the same code and neither would be
    /// distinguishable by tooling.
    #[test]
    fn every_diagnostic_code_is_unique() {
        let mut seen = std::collections::HashMap::new();
        for (code, name, _) in ALL_DIAGNOSTIC_CODES {
            if let Some(previous) = seen.insert(code.0, *name) {
                panic!("code {} is used by both {previous} and {name}", code.0);
            }
        }
    }

    #[test]
    fn every_diagnostic_code_is_well_formed_and_described() {
        for (code, name, summary) in ALL_DIAGNOSTIC_CODES {
            assert!(
                code.0.starts_with('V')
                    && code.0.len() == 5
                    && code.0[1..].chars().all(|c| c.is_ascii_digit()),
                "{name} has a malformed code: {}",
                code.0
            );
            assert!(!summary.is_empty(), "{name} has no summary");
            assert!(
                summary.ends_with('.'),
                "{name}'s summary should read as a sentence: {summary}"
            );
        }
    }

    /// The compiler may not print. Everything it has to say is a diagnostic.
    ///
    /// A stray `println!` from a debugging session is invisible until it turns
    /// up in the middle of a program's own output -- and it did: the generic
    /// constraint checker printed a line for every constraint it looked at.
    /// The interpreter is exempt, since running a program is how `Debug.Print`
    /// and `MsgBox` reach a terminal in the first place.
    #[test]
    fn the_compiler_says_what_it_has_to_say_in_diagnostics() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        collect_prints(&root.join("frontend"), &mut offenders);

        assert!(
            offenders.is_empty(),
            "these print from the compiler; report it as a diagnostic instead,              or delete the line if it was for debugging:
{}",
            offenders.join("
")
        );
    }

    fn collect_prints(dir: &std::path::Path, offenders: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "tests") {
                    continue;
                }
                collect_prints(&path, offenders);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            let mut in_tests = false;
            for (index, line) in source.lines().enumerate() {
                if line.trim_start().starts_with("mod tests") {
                    in_tests = true;
                }
                if in_tests {
                    continue;
                }
                if line.contains("println!") || line.contains("eprintln!") || line.contains("dbg!")
                {
                    offenders.push(format!("  {}:{}", path.display(), index + 1));
                }
            }
        }
    }

    /// Nothing may reach for the code that means "no code was chosen".
    ///
    /// `GENERIC` is the escape hatch, and an escape hatch in easy reach gets
    /// used: this codebase once emitted 67 diagnostics under it, which told a
    /// reader nothing and could not be matched on. Every one now names what
    /// went wrong. Adding another is a deliberate act, not a default.
    #[test]
    fn no_diagnostic_is_left_without_a_code() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        collect_generic_uses(&root, &mut offenders);

        assert!(
            offenders.is_empty(),
            "these emit DiagnosticCode::GENERIC; give each one a code that says              what went wrong, and add it to docs/reference/diagnostics.md:
{}",
            offenders.join("
")
        );
    }

    fn collect_generic_uses(dir: &std::path::Path, offenders: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            // Test modules are exempt: a test that renders a diagnostic needs
            // some code to render, and `GENERIC` is the one that means nothing.
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "tests") {
                    continue;
                }
                collect_generic_uses(&path, offenders);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs")
                // This file declares the code and tests the renderer with it.
                || path.file_name().is_some_and(|name| name == "diagnostic.rs")
            {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (index, line) in source.lines().enumerate() {
                if line.contains("DiagnosticCode::GENERIC") {
                    offenders.push(format!("  {}:{}", path.display(), index + 1));
                }
            }
        }
    }

    /// The published reference must list every code the compiler can emit.
    ///
    /// Documentation that silently falls behind is worse than none: a reader
    /// who cannot find a code assumes it does not exist.
    #[test]
    fn the_diagnostics_reference_lists_every_code() {
        const REFERENCE: &str = include_str!("../../../docs/reference/diagnostics.md");

        for (code, name, summary) in ALL_DIAGNOSTIC_CODES {
            assert!(
                REFERENCE.contains(code.0),
                "docs/reference/diagnostics.md does not list {} ({name})",
                code.0
            );
            assert!(
                REFERENCE.contains(summary),
                "docs/reference/diagnostics.md does not carry the summary for {} ({name})",
                code.0
            );
        }
    }

    use super::*;

    #[test]
    fn renders_code_labels_notes_and_help() {
        let mut source_map = SourceMap::new();
        let file_id = source_map.add(
            "test.valo".to_string(),
            "Dim age As Integer\n    age = \"Valo\"".to_string(),
        );

        let span = Span::new(file_id, SourcePos::new(2, 5), SourcePos::new(2, 8));
        let other = Span::new(file_id, SourcePos::new(1, 1), SourcePos::new(1, 4));
        let diagnostic = Diagnostic::new(
            DiagnosticCode::GENERIC,
            "cannot assign String to Integer",
            Some(span),
        )
        .with_code(DiagnosticCode::TYPE_MISMATCH)
        .with_primary_label("expected Integer, found String")
        .with_secondary_label(other, "variable declared here")
        .with_note("assignment types match")
        .with_help("change the variable type or assign an Integer value");

        let rendered = diagnostic.render_colored(&source_map, false);

        assert!(rendered.contains("error[V1100]: cannot assign String to Integer"));
        assert!(rendered.contains("--> test.valo:2:5"));
        assert!(rendered.contains("expected Integer, found String"));
        assert!(rendered.contains("variable declared here"));
        assert!(rendered.contains("note: assignment types match"));
        assert!(rendered.contains("help: change the variable type"));
    }

    #[test]
    fn renders_wide_line_gutters_without_shifting_bars() {
        let mut source_map = SourceMap::new();
        let mut source = "\n".repeat(999);
        source.push_str("Dim answer As Integer\n");
        let file_id = source_map.add("large.valo".to_string(), source);
        let span = Span::new(file_id, SourcePos::new(1000, 5), SourcePos::new(1000, 11));
        let diagnostic = Diagnostic::new(DiagnosticCode::TYPE_MISMATCH, "bad value", Some(span))
            .with_primary_label("expected Integer");

        let rendered = diagnostic.render_colored(&source_map, false);

        assert!(rendered.contains("    |"));
        assert!(rendered.contains("1000 | Dim answer As Integer"));
        assert!(rendered.contains("    |     ^^^^^^ expected Integer"));
    }

    #[test]
    fn expands_tabs_before_rendering_caret_markers() {
        let mut source_map = SourceMap::new();
        let file_id = source_map.add("tabs.valo".to_string(), "\tvalue = \"x\"".to_string());
        let span = Span::new(file_id, SourcePos::new(1, 2), SourcePos::new(1, 7));
        let diagnostic = Diagnostic::new(DiagnosticCode::TYPE_MISMATCH, "bad value", Some(span))
            .with_primary_label("here");

        let rendered = diagnostic.render_colored(&source_map, false);

        assert!(rendered.contains("1 |     value = \"x\""));
        assert!(rendered.contains("  |     ^^^^^ here"));
    }
}
