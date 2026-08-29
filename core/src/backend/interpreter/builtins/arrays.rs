//! Builtins that build or measure arrays.
//!
//! Argument counts are checked against the registry before dispatch, so these
//! functions index the arguments their registry entry guarantees.

use super::super::Interpreter;
use super::{ValueFn, find_handler};
use crate::runtime::numeric::value_to_i64;
use crate::runtime::{ArrayValue, Diagnostic, Span, TypeName, Value};
use std::rc::Rc;

/// The builtins this module implements.
pub(super) const HANDLERS: &[(&str, ValueFn)] = &[
    ("Array", array),
    ("LBound", bound),
    ("UBound", bound),
    ("Split", split),
    ("Join", join),
];

pub(crate) fn eval_arrays(
    interpreter: &mut Interpreter,
    name: &str,
    args: &[Value],
    span: Span,
) -> Result<Option<Value>, Diagnostic> {
    match find_handler(HANDLERS, name) {
        Some(handler) => handler(interpreter, name, args, span).map(Some),
        None => Ok(None),
    }
}

/// Builds a zero-based `Variant` array from the arguments.
fn array(_: &mut Interpreter, _: &str, args: &[Value], _: Span) -> Result<Value, Diagnostic> {
    Ok(one_dimensional(TypeName::Variant, args.to_vec()))
}

/// Backs both `LBound` and `UBound`, which differ only in which end they read.
fn bound(_: &mut Interpreter, name: &str, args: &[Value], span: Span) -> Result<Value, Diagnostic> {
    let dimension = match args.get(1) {
        None => 1,
        Some(value) => value_to_i64(value).ok_or_else(|| {
            Diagnostic::new(
                crate::runtime::DiagnosticCode::TYPE_MISMATCH,
                "Array dimension must be Integer",
                Some(span),
            )
        })? as usize,
    };

    let bound = if name.eq_ignore_ascii_case("LBound") {
        super::super::arrays::lbound(&args[0], dimension, span)?
    } else {
        super::super::arrays::ubound(&args[0], dimension, span)?
    };
    Ok(Value::Int64(bound))
}

fn split(_: &mut Interpreter, _: &str, args: &[Value], _: Span) -> Result<Value, Diagnostic> {
    let expression = args[0].to_output_string();
    let delimiter = optional_delimiter(args.get(1));

    // Splitting on an empty delimiter has no separator to find, so the whole
    // string is the single part.
    let parts: Vec<Value> = if delimiter.is_empty() {
        vec![Value::String(expression)]
    } else {
        expression
            .split(&delimiter)
            .map(|part| Value::String(part.to_string()))
            .collect()
    };

    Ok(one_dimensional(TypeName::String, parts))
}

fn join(_: &mut Interpreter, _: &str, args: &[Value], span: Span) -> Result<Value, Diagnostic> {
    let Value::Array(array) = &args[0] else {
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "Join requires an array",
            Some(span),
        ));
    };
    let delimiter = optional_delimiter(args.get(1));
    let parts: Vec<String> = array
        .elements
        .iter()
        .map(|element| element.to_output_string())
        .collect();
    Ok(Value::String(parts.join(&delimiter)))
}

/// Reads a separator argument, defaulting to a single space when it is absent
/// or was left out at the call site.
fn optional_delimiter(argument: Option<&Value>) -> String {
    match argument {
        Some(value) if !matches!(value, Value::Missing) => value.to_output_string(),
        _ => " ".to_string(),
    }
}

/// Wraps elements in a zero-based, dynamically sized array.
fn one_dimensional(element_type: TypeName, elements: Vec<Value>) -> Value {
    let upper = elements.len() as i64 - 1;
    Value::Array(Rc::new(ArrayValue {
        element_type,
        elements,
        bounds: vec![crate::runtime::ArrayBound { lower: 0, upper }],
        allocated: true,
        dynamic: true,
    }))
}
