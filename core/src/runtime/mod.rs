//! Valo Runtime
//!
//! The runtime defines the core data model and behavior of the language,
//! including the value system, type definitions, and diagnostics.
//! It is designed to be independent of the execution backend.

pub mod builtins;
pub mod callable;
mod coerce;
pub mod com;
pub mod compare;
mod diagnostic;
pub mod ffi_platform;
pub mod naming;
pub mod numeric;
pub mod ops;
pub mod stdlib;
mod type_name;
mod value;
pub mod vba;
pub mod well_known;

pub use coerce::coerce_assignment;
pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticLabel, FileId, LabelStyle, RuntimeErrorInfo, Severity,
    SourceMap, SourcePos, Span, terminal_supports_color,
};
pub use naming::{fold, with_folded};
pub use type_name::TypeName;
pub use value::{
    ArrayValue, CollectionItem, CollectionValue, ComObjectValue, EventBinding, LambdaValue,
    ObjectValue, RecordValue, Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayBound {
    pub lower: i64,
    pub upper: i64,
}
