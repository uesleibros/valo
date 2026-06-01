//! Valo Builtins
//!
//! Standard library functions and procedures.
//!
//! Value-level builtins are kept backend-neutral where practical. A small
//! dispatch layer still handles lazy/special forms that need AST or frame access
//! (`IIf`, `CallByName`, pointer builtins, and host-owned services).

use super::{ControlFlow, Frame, Interpreter};
use crate::runtime::builtins::{
    DATE_TIME_FUNCTIONS, DIALOG_STATEMENT_FUNCTIONS, FILE_FUNCTIONS,
    FILE_SYSTEM_STATEMENT_FUNCTIONS, is_builtin_function, is_name_in, strip_vba_namespace,
};
use crate::runtime::{Diagnostic, Value};
use crate::{Expr, ExprKind};

const DEFAULT_DIALOG_TITLE: &str = "Valo";

pub(crate) fn dispatch_stmt(
    interpreter: &mut Interpreter,
    object_name: &str,
    method: &str,
    args: &[Expr],
    frame: &mut Frame,
    span: crate::runtime::Span,
) -> Result<Option<ControlFlow>, Diagnostic> {
    if object_name.is_empty() && is_name_in(method, DIALOG_STATEMENT_FUNCTIONS) {
        dispatch_function(interpreter, method, args, frame, span)?;
        return Ok(Some(ControlFlow::Continue));
    }
    // Handle VBA namespace fallback: VBA.MsgBox(...) -> MsgBox(...)
    let effective_object_name = if object_name.eq_ignore_ascii_case("VBA") {
        "VBA"
    } else {
        object_name
    };

    if effective_object_name.eq_ignore_ascii_case("Console")
        || effective_object_name.eq_ignore_ascii_case("Debug")
    {
        if effective_object_name.eq_ignore_ascii_case("Debug")
            && method.eq_ignore_ascii_case("Assert")
        {
            dispatch_function(interpreter, "Debug.Assert", args, frame, span)?;
            return Ok(Some(ControlFlow::Continue));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            let val = interpreter.eval_expr(arg, frame)?;
            let resolved = interpreter.resolve_default_value(val, frame, arg.span)?;
            values.push(resolved);
        }

        if effective_object_name.eq_ignore_ascii_case("Console") {
            if method.eq_ignore_ascii_case("ReadLine") {
                let result = console::exec_console(method, &values, span)?;
                if let Some(_line) = result {
                    // This is a bit tricky: ReadLine as a statement? Usually it's a function.
                    // If called as a statement, we just ignore the result.
                    return Ok(Some(ControlFlow::Continue));
                }
            }
            if let Some(line) = console::exec_console(method, &values, span)? {
                interpreter.emit_output(line);
                return Ok(Some(ControlFlow::Continue));
            } else if method.eq_ignore_ascii_case("Write") {
                return Ok(Some(ControlFlow::Continue));
            }
        } else {
            if let Some(line) = debug::exec_debug(method, &values, span)? {
                interpreter.emit_output(line);
                return Ok(Some(ControlFlow::Continue));
            }
        }
    }

    if effective_object_name.eq_ignore_ascii_case("Err") {
        return err::exec_err(interpreter, method, args, frame, span);
    }

    if effective_object_name.eq_ignore_ascii_case("VBA") {
        if is_name_in(method, DIALOG_STATEMENT_FUNCTIONS) {
            dispatch_function(interpreter, method, args, frame, span)?;
            return Ok(Some(ControlFlow::Continue));
        }

        // VBA.Randomize 123
        if let Some(val) = dispatch_function(interpreter, method, args, frame, span)? {
            // If it returns a value but was called as a stmt, we just ignore the value
            // (or maybe check if it's a valid stmt builtin)
            if matches!(val, Value::Empty) || method.eq_ignore_ascii_case("Randomize") {
                return Ok(Some(ControlFlow::Continue));
            }
        }
    }

    Ok(None)
}

pub(crate) fn dispatch_function(
    interpreter: &mut Interpreter,
    name: &str,
    args: &[Expr],
    frame: &mut Frame,
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    // Handle VBA namespace fallback: VBA.Join(...) -> Join(...)
    let effective_name = strip_vba_namespace(name);

    // Special forms that require lazy evaluation or direct Expr access
    if effective_name.eq_ignore_ascii_case("IIf") {
        expect_arg_count(effective_name, args, 3, span)?;
        let condition = interpreter.eval_expr(&args[0], frame)?.is_truthy();
        let value_expr = if condition { &args[1] } else { &args[2] };
        return Ok(Some(interpreter.eval_expr(value_expr, frame)?));
    }

    if effective_name.eq_ignore_ascii_case("Choose") {
        if args.len() < 2 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "Choose expects an index and at least one choice",
                Some(span),
            ));
        }
        let index = integer_arg(
            effective_name,
            &interpreter.eval_expr(&args[0], frame)?,
            span,
        )?;
        if index < 1 || index as usize >= args.len() {
            return Ok(Some(Value::Null));
        }
        return Ok(Some(interpreter.eval_expr(&args[index as usize], frame)?));
    }

    if effective_name.eq_ignore_ascii_case("Switch") {
        if args.is_empty() || !args.len().is_multiple_of(2) {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "Switch expects expression/value pairs",
                Some(span),
            ));
        }
        for pair in args.chunks(2) {
            if interpreter.eval_expr(&pair[0], frame)?.is_truthy() {
                return Ok(Some(interpreter.eval_expr(&pair[1], frame)?));
            }
        }
        return Ok(Some(Value::Null));
    }

    if effective_name.eq_ignore_ascii_case("CallByName") {
        return dispatch_callbyname(interpreter, effective_name, args, frame, span);
    }

    if effective_name.eq_ignore_ascii_case("VarPtr") {
        expect_arg_count(effective_name, args, 1, span)?;
        return Ok(Some(Value::Ptr(interpreter.varptr_expr(&args[0], frame)?)));
    }

    if effective_name.eq_ignore_ascii_case("StrPtr") {
        expect_arg_count(effective_name, args, 1, span)?;
        let arg = &args[0];
        let value = interpreter.eval_expr(arg, frame)?;
        let text = match value {
            Value::String(text) => text,
            Value::Empty => String::new(),
            Value::Null | Value::Nothing | Value::Missing => return Ok(Some(Value::Ptr(0))),
            other => other.to_output_string(),
        };
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.push(0);
        interpreter.temporary_wide_strings.push(wide);
        let ptr = interpreter
            .temporary_wide_strings
            .last()
            .map(|text| text.as_ptr() as usize)
            .unwrap_or(0);
        return Ok(Some(Value::Ptr(ptr)));
    }

    if effective_name.eq_ignore_ascii_case("ObjPtr") {
        expect_arg_count(effective_name, args, 1, span)?;
        let value = interpreter.eval_expr(&args[0], frame)?;
        match value {
            Value::Object(obj) => {
                let ptr = std::rc::Rc::as_ptr(&obj) as usize;
                return Ok(Some(Value::Ptr(ptr)));
            }
            Value::Collection(coll) => {
                let ptr = std::rc::Rc::as_ptr(&coll) as usize;
                return Ok(Some(Value::Ptr(ptr)));
            }
            Value::ComObject(com) => {
                let ptr = std::rc::Rc::as_ptr(&com) as usize;
                return Ok(Some(Value::Ptr(ptr)));
            }
            Value::Nothing => {
                return Ok(Some(Value::Ptr(0)));
            }
            _ => {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "ObjPtr requires an object",
                    Some(span),
                ));
            }
        }
    }

    if effective_name.eq_ignore_ascii_case("DoEvents") {
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
            };
            unsafe {
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        return Ok(Some(Value::Int16(0)));
    }

    if effective_name.eq_ignore_ascii_case("MsgBox") {
        if args.is_empty() || args.len() > 5 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "MsgBox expects 1 to 5 arguments",
                Some(span),
            ));
        }
        let prompt = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        let title = if args.len() >= 3 && !matches!(args[2].kind, ExprKind::Missing) {
            interpreter.eval_expr(&args[2], frame)?.to_output_string()
        } else {
            DEFAULT_DIALOG_TITLE.to_string()
        };

        #[cfg(windows)]
        {
            use crate::runtime::{TypeName, coerce_assignment};
            let buttons_val = if args.len() >= 2 && !matches!(args[1].kind, ExprKind::Missing) {
                interpreter.eval_expr(&args[1], frame)?
            } else {
                Value::Int32(0) // vbOKOnly
            };
            let buttons = coerce_assignment(&TypeName::Long, buttons_val, span)?
                .to_output_string()
                .parse::<i32>()
                .unwrap_or(0);

            use windows::Win32::UI::WindowsAndMessaging::{MESSAGEBOX_STYLE, MessageBoxW};
            use windows::core::HSTRING;
            let result = unsafe {
                MessageBoxW(
                    None,
                    &HSTRING::from(prompt),
                    &HSTRING::from(title),
                    MESSAGEBOX_STYLE(buttons as u32),
                )
            };
            return Ok(Some(Value::Int16(result.0 as i16)));
        }
        #[cfg(not(windows))]
        {
            // Evaluate buttons anyway to maintain side-effects parity
            if args.len() >= 2 && !matches!(args[1].kind, ExprKind::Missing) {
                let _ = interpreter.eval_expr(&args[1], frame)?;
            }

            println!("{title}: {prompt}");
            return Ok(Some(Value::Int16(1))); // vbOK
        }
    }

    if effective_name.eq_ignore_ascii_case("InputBox") {
        if args.is_empty() || args.len() > 7 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "InputBox expects 1 to 7 arguments",
                Some(span),
            ));
        }
        let prompt = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        let title = if args.len() >= 2 && !matches!(args[1].kind, ExprKind::Missing) {
            interpreter.eval_expr(&args[1], frame)?.to_output_string()
        } else {
            DEFAULT_DIALOG_TITLE.to_string()
        };
        let default = if args.len() >= 3 && !matches!(args[2].kind, ExprKind::Missing) {
            interpreter.eval_expr(&args[2], frame)?.to_output_string()
        } else {
            String::new()
        };

        // For now, let's use console ReadLine as a fallback for InputBox
        println!("{title}: {prompt}");
        if !default.is_empty() {
            println!("Default: {default}");
        }
        print!("> ");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let result = input.trim_end_matches(['\r', '\n']).to_string();
        if result.is_empty() && !default.is_empty() {
            return Ok(Some(Value::String(default)));
        }
        return Ok(Some(Value::String(result)));
    }

    if effective_name.eq_ignore_ascii_case("Command") {
        expect_arg_count(effective_name, args, 0, span)?;
        let command = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
        return Ok(Some(Value::String(command)));
    }

    if effective_name.eq_ignore_ascii_case("Error") {
        if args.len() > 1 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "Error expects 0 to 1 arguments",
                Some(span),
            ));
        }
        let number = if let Some(arg) = args.first() {
            integer_arg(effective_name, &interpreter.eval_expr(arg, frame)?, span)?
        } else {
            0
        };
        return Ok(Some(Value::String(error_description(number))));
    }

    if effective_name.eq_ignore_ascii_case("Input") {
        expect_arg_count(effective_name, args, 2, span)?;
        let count = integer_arg(
            effective_name,
            &interpreter.eval_expr(&args[0], frame)?,
            span,
        )?;
        let number = file_number_arg(
            effective_name,
            &interpreter.eval_expr(&args[1], frame)?,
            span,
        )?;
        return Ok(Some(Value::String(
            interpreter.input_file_chars(number, count, span)?,
        )));
    }

    if effective_name.eq_ignore_ascii_case("Shell") {
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "Shell expects 1 to 2 arguments",
                Some(span),
            ));
        }
        let command = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        let child = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", &command])
                .spawn()
        } else {
            std::process::Command::new("sh")
                .args(["-c", &command])
                .spawn()
        }
        .map_err(|error| {
            Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                format!("Shell failed to start '{}': {}", command, error),
                Some(span),
            )
        })?;
        return Ok(Some(Value::Int64(i64::from(child.id()))));
    }

    if effective_name.eq_ignore_ascii_case("Debug.Assert") {
        expect_arg_count(effective_name, args, 1, span)?;
        let condition = interpreter.eval_expr(&args[0], frame)?.is_truthy();
        if !condition {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "Assertion failed".to_string(),
                Some(args[0].span),
            ));
        }
        return Ok(Some(Value::Empty));
    }

    if effective_name.eq_ignore_ascii_case("Console.ReadLine") {
        let result = console::exec_console("ReadLine", &[], span)?;
        return Ok(Some(Value::String(result.unwrap_or_default())));
    }

    if effective_name.eq_ignore_ascii_case("CreateObject") {
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "CreateObject expects 1 to 2 arguments",
                Some(span),
            ));
        }
        let prog_id = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        if args.len() == 2 {
            let server = interpreter.eval_expr(&args[1], frame)?.to_output_string();
            if !server.is_empty() {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "CreateObject remote server activation is not supported yet",
                    Some(args[1].span),
                ));
            }
        }
        return Ok(Some(crate::runtime::com::create_object(&prog_id, span)?));
    }

    if effective_name.eq_ignore_ascii_case("GetObject") {
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "GetObject expects 1 to 2 arguments",
                Some(span),
            ));
        }
        let pathname = if !matches!(args[0].kind, ExprKind::Missing) {
            Some(interpreter.eval_expr(&args[0], frame)?.to_output_string())
        } else {
            None
        };
        let prog_id = if args.len() == 2 && !matches!(args[1].kind, ExprKind::Missing) {
            Some(interpreter.eval_expr(&args[1], frame)?.to_output_string())
        } else {
            None
        };
        return Ok(Some(crate::runtime::com::get_object(
            pathname.as_deref(),
            prog_id.as_deref(),
            span,
        )?));
    }

    if is_name_in(effective_name, FILE_FUNCTIONS) {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(interpreter.eval_expr(arg, frame)?);
        }
        return dispatch_file_function(interpreter, effective_name, &values, span);
    }

    if effective_name.eq_ignore_ascii_case("Environ") {
        expect_arg_count(effective_name, args, 1, span)?;
        let value = interpreter.eval_expr(&args[0], frame)?;
        return Ok(Some(Value::String(environ_value(
            effective_name,
            &value,
            span,
        )?)));
    }

    if effective_name.eq_ignore_ascii_case("GetSetting") {
        if args.len() < 3 || args.len() > 4 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "GetSetting expects 3 to 4 arguments",
                Some(span),
            ));
        }
        for arg in args.iter().take(3) {
            let _ = interpreter.eval_expr(arg, frame)?;
        }
        let default = if let Some(default) = args.get(3) {
            interpreter.eval_expr(default, frame)?.to_output_string()
        } else {
            String::new()
        };
        return Ok(Some(Value::String(default)));
    }

    if effective_name.eq_ignore_ascii_case("GetAllSettings") {
        expect_arg_count(effective_name, args, 2, span)?;
        for arg in args {
            let _ = interpreter.eval_expr(arg, frame)?;
        }
        return Ok(Some(empty_string_matrix()));
    }

    if effective_name.eq_ignore_ascii_case("IMEStatus") {
        expect_arg_count(effective_name, args, 0, span)?;
        return Ok(Some(Value::Int64(0)));
    }

    if effective_name.eq_ignore_ascii_case("MacID") {
        expect_arg_count(effective_name, args, 1, span)?;
        let text = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        let mut bytes = [b' '; 4];
        for (slot, byte) in bytes.iter_mut().zip(text.bytes()) {
            *slot = byte;
        }
        let value = bytes
            .iter()
            .fold(0_i64, |acc, byte| (acc << 8) | i64::from(*byte));
        return Ok(Some(Value::Int64(value)));
    }

    if effective_name.eq_ignore_ascii_case("MacScript") {
        expect_arg_count(effective_name, args, 1, span)?;
        let _ = interpreter.eval_expr(&args[0], frame)?;
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::GENERIC,
            "MacScript is not supported on this runtime",
            Some(span),
        ));
    }

    if is_name_in(effective_name, DATE_TIME_FUNCTIONS) {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(interpreter.eval_expr(arg, frame)?);
        }
        return dispatch_datetime_function(effective_name, &values, span);
    }

    if is_name_in(effective_name, FILE_SYSTEM_STATEMENT_FUNCTIONS) {
        expect_arg_count(effective_name, args, 1, span)?;
        let path = interpreter.eval_expr(&args[0], frame)?.to_output_string();
        match effective_name.to_ascii_lowercase().as_str() {
            "kill" => interpreter.kill_path(&path, span)?,
            "mkdir" => interpreter.mkdir_path(&path, span)?,
            "rmdir" => interpreter.rmdir_path(&path, span)?,
            "chdir" => interpreter.chdir_path(&path, span)?,
            _ => unreachable!(),
        }
        return Ok(Some(Value::Empty));
    }

    if !is_builtin_function(effective_name) {
        return Ok(None);
    }

    // Normal functions: evaluate all arguments first
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(interpreter.eval_expr(arg, frame)?);
    }

    if effective_name.eq_ignore_ascii_case("IsMissing") {
        if values.len() != 1 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "IsMissing expects exactly 1 argument",
                Some(span),
            ));
        }
        return Ok(Some(Value::Boolean(matches!(values[0], Value::Missing))));
    }

    if let Some(val) = types::eval_types(effective_name, &values, span)? {
        return Ok(Some(val));
    }
    if let Some(val) = arrays::eval_arrays(effective_name, &values, span)? {
        return Ok(Some(val));
    }
    if let Some(val) = strings::eval_strings(interpreter, effective_name, &values, span)? {
        return Ok(Some(val));
    }
    if let Some(val) = math::eval_math(interpreter, effective_name, &values, span)? {
        return Ok(Some(val));
    }
    if let Some(val) = eval_misc_value_function(effective_name, &values, span)? {
        return Ok(Some(val));
    }

    Ok(None)
}

fn dispatch_file_function(
    interpreter: &mut Interpreter,
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    match name.to_ascii_lowercase().as_str() {
        "freefile" => {
            if !args.is_empty() {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "FreeFile expects no arguments",
                    Some(span),
                ));
            }
            Ok(Some(Value::Int64(i64::from(
                interpreter.free_file_number(),
            ))))
        }
        "eof" => {
            expect_value_count(name, args, 1, span)?;
            let number = file_number_arg(name, &args[0], span)?;
            Ok(Some(Value::Boolean(interpreter.eof_file(number, span)?)))
        }
        "fileattr" => {
            expect_value_count(name, args, 2, span)?;
            let number = file_number_arg(name, &args[0], span)?;
            let attribute = integer_arg(name, &args[1], span)?;
            Ok(Some(Value::Int64(
                interpreter.file_attr(number, attribute, span)?,
            )))
        }
        "lof" => {
            expect_value_count(name, args, 1, span)?;
            let number = file_number_arg(name, &args[0], span)?;
            Ok(Some(Value::Int64(interpreter.lof_file(number, span)?)))
        }
        "loc" => {
            expect_value_count(name, args, 1, span)?;
            let number = file_number_arg(name, &args[0], span)?;
            Ok(Some(Value::Int64(interpreter.loc_file(number, span)?)))
        }
        "seek" => {
            expect_value_count(name, args, 1, span)?;
            let number = file_number_arg(name, &args[0], span)?;
            Ok(Some(Value::Int64(
                interpreter.seek_file_position(number, span)?,
            )))
        }
        "dir" => Ok(Some(Value::String(interpreter.dir(args, span)?))),
        "getattr" => {
            expect_value_count(name, args, 1, span)?;
            Ok(Some(Value::Int64(get_attr(
                &args[0].to_output_string(),
                span,
            )?)))
        }
        "filelen" => {
            expect_value_count(name, args, 1, span)?;
            let path = args[0].to_output_string();
            let len = std::fs::metadata(&path).map_err(|error| {
                Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    format!("Unable to get FileLen for '{}': {}", path, error),
                    Some(span),
                )
            })?;
            Ok(Some(Value::Int64(len.len() as i64)))
        }
        "filedatetime" => {
            expect_value_count(name, args, 1, span)?;
            let path = args[0].to_output_string();
            let modified = std::fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .map_err(|error| {
                    Diagnostic::new(
                        crate::runtime::DiagnosticCode::GENERIC,
                        format!("Unable to get FileDateTime for '{}': {}", path, error),
                        Some(span),
                    )
                })?;
            Ok(Some(Value::Date(system_time_to_vba_date(modified)?)))
        }
        "curdir" => {
            if args.len() > 1 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "CurDir expects 0 to 1 arguments",
                    Some(span),
                ));
            }
            let cwd = std::env::current_dir().map_err(|error| {
                Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    format!("Unable to get current directory: {}", error),
                    Some(span),
                )
            })?;
            Ok(Some(Value::String(cwd.display().to_string())))
        }
        _ => Ok(None),
    }
}

fn dispatch_datetime_function(
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    match name.to_ascii_lowercase().as_str() {
        "timer" => {
            expect_value_count(name, args, 0, span)?;
            Ok(Some(Value::Double(timer_seconds()?)))
        }
        "now" => {
            expect_value_count(name, args, 0, span)?;
            Ok(Some(Value::Date(system_time_to_vba_date(
                std::time::SystemTime::now(),
            )?)))
        }
        "date" => {
            expect_value_count(name, args, 0, span)?;
            let now = system_time_to_vba_date(std::time::SystemTime::now())?;
            Ok(Some(Value::Date(now.floor())))
        }
        "time" => {
            expect_value_count(name, args, 0, span)?;
            let now = system_time_to_vba_date(std::time::SystemTime::now())?;
            Ok(Some(Value::Date(now.fract())))
        }
        "dateserial" => {
            expect_value_count(name, args, 3, span)?;
            let year = integer_arg(name, &args[0], span)?;
            let month = integer_arg(name, &args[1], span)?;
            let day = integer_arg(name, &args[2], span)?;
            Ok(Some(Value::Date(date_serial(year, month, day))))
        }
        "timeserial" => {
            expect_value_count(name, args, 3, span)?;
            let hour = integer_arg(name, &args[0], span)?;
            let minute = integer_arg(name, &args[1], span)?;
            let second = integer_arg(name, &args[2], span)?;
            Ok(Some(Value::Date(time_serial(hour, minute, second))))
        }
        "datevalue" => {
            expect_value_count(name, args, 1, span)?;
            Ok(Some(Value::Date(parse_date_value(
                &args[0].to_output_string(),
                span,
            )?)))
        }
        "timevalue" => {
            expect_value_count(name, args, 1, span)?;
            Ok(Some(Value::Date(parse_time_value(
                &args[0].to_output_string(),
                span,
            )?)))
        }
        "dateadd" => {
            expect_value_count(name, args, 3, span)?;
            let interval = args[0].to_output_string();
            let number = integer_arg(name, &args[1], span)?;
            let date = date_arg(name, &args[2], span)?;
            Ok(Some(Value::Date(date_add(&interval, number, date, span)?)))
        }
        "datediff" => {
            if args.len() < 3 || args.len() > 5 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "DateDiff expects 3 to 5 arguments",
                    Some(span),
                ));
            }
            let interval = args[0].to_output_string();
            let start = date_arg(name, &args[1], span)?;
            let end = date_arg(name, &args[2], span)?;
            Ok(Some(Value::Int64(date_diff(&interval, start, end, span)?)))
        }
        "datepart" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "DatePart expects 2 to 4 arguments",
                    Some(span),
                ));
            }
            let interval = args[0].to_output_string();
            let date = date_arg(name, &args[1], span)?;
            Ok(Some(Value::Int64(date_part(&interval, date, span)?)))
        }
        "year" | "month" | "day" => {
            expect_value_count(name, args, 1, span)?;
            let serial = date_arg(name, &args[0], span)?;
            let (year, month, day) = civil_from_days(serial.floor() as i64 - UNIX_EPOCH_AS_VBA);
            let value = match name.to_ascii_lowercase().as_str() {
                "year" => year,
                "month" => i64::from(month),
                "day" => i64::from(day),
                _ => unreachable!(),
            };
            Ok(Some(Value::Int64(value)))
        }
        "hour" | "minute" | "second" => {
            expect_value_count(name, args, 1, span)?;
            let serial = date_arg(name, &args[0], span)?;
            let total = seconds_since_midnight(serial);
            let value = match name.to_ascii_lowercase().as_str() {
                "hour" => total / 3600,
                "minute" => (total % 3600) / 60,
                "second" => total % 60,
                _ => unreachable!(),
            };
            Ok(Some(Value::Int64(value)))
        }
        "weekday" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "Weekday expects 1 to 2 arguments",
                    Some(span),
                ));
            }
            let serial = date_arg(name, &args[0], span)?;
            let first_day = args
                .get(1)
                .and_then(crate::runtime::numeric::value_to_i64)
                .unwrap_or(1);
            Ok(Some(Value::Int64(weekday_value(serial, first_day))))
        }
        "monthname" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "MonthName expects 1 to 2 arguments",
                    Some(span),
                ));
            }
            let month = integer_arg(name, &args[0], span)?;
            let abbreviate = args.get(1).is_some_and(Value::is_truthy);
            Ok(Some(Value::String(month_name(month, abbreviate, span)?)))
        }
        "weekdayname" => {
            if args.is_empty() || args.len() > 3 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    "WeekdayName expects 1 to 3 arguments",
                    Some(span),
                ));
            }
            let weekday = integer_arg(name, &args[0], span)?;
            let abbreviate = args.get(1).is_some_and(Value::is_truthy);
            Ok(Some(Value::String(weekday_name(
                weekday, abbreviate, span,
            )?)))
        }
        _ => Ok(None),
    }
}

fn environ_value(
    name: &str,
    value: &Value,
    span: crate::runtime::Span,
) -> Result<String, Diagnostic> {
    match value {
        Value::String(key) => Ok(std::env::var(key).unwrap_or_default()),
        Value::Empty | Value::Missing => Ok(String::new()),
        _ => {
            let index = integer_arg(name, value, span)?;
            if index <= 0 {
                return Ok(String::new());
            }
            Ok(std::env::vars()
                .nth((index - 1) as usize)
                .map(|(key, value)| format!("{key}={value}"))
                .unwrap_or_default())
        }
    }
}

fn eval_misc_value_function(
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    match name.to_ascii_lowercase().as_str() {
        "rgb" => {
            expect_value_count(name, args, 3, span)?;
            let red = color_component(name, &args[0], span)?;
            let green = color_component(name, &args[1], span)?;
            let blue = color_component(name, &args[2], span)?;
            Ok(Some(Value::Int64(red | (green << 8) | (blue << 16))))
        }
        "qbcolor" => {
            expect_value_count(name, args, 1, span)?;
            let color = integer_arg(name, &args[0], span)?;
            const COLORS: [i64; 16] = [
                0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080, 0xC0C0C0,
                0x808080, 0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
            ];
            let Some(value) = COLORS.get(color as usize) else {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::TYPE_MISMATCH,
                    "QBColor color must be between 0 and 15",
                    Some(span),
                ));
            };
            Ok(Some(Value::Int64(*value)))
        }
        "spc" | "tab" => {
            expect_value_count(name, args, 1, span)?;
            let count = integer_arg(name, &args[0], span)?.max(0) as usize;
            Ok(Some(Value::String(" ".repeat(count))))
        }
        _ => Ok(None),
    }
}

fn color_component(
    name: &str,
    value: &Value,
    span: crate::runtime::Span,
) -> Result<i64, Diagnostic> {
    let value = integer_arg(name, value, span)?;
    Ok(value.clamp(0, 255))
}

fn get_attr(path: &str, span: crate::runtime::Span) -> Result<i64, Diagnostic> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        Diagnostic::new(
            crate::runtime::DiagnosticCode::GENERIC,
            format!("Unable to get attributes for '{}': {}", path, error),
            Some(span),
        )
    })?;
    let mut attr = if metadata.is_dir() { 16 } else { 0 };
    if metadata.permissions().readonly() {
        attr |= 1;
    }
    Ok(attr)
}

fn empty_string_matrix() -> Value {
    Value::Array(std::rc::Rc::new(crate::runtime::ArrayValue {
        element_type: crate::runtime::TypeName::Variant,
        elements: Vec::new(),
        bounds: vec![
            crate::runtime::ArrayBound {
                lower: 0,
                upper: -1,
            },
            crate::runtime::ArrayBound { lower: 0, upper: 1 },
        ],
        allocated: true,
        dynamic: true,
    }))
}

fn error_description(number: i64) -> String {
    match number {
        0 => String::new(),
        5 => "Invalid procedure call or argument".to_string(),
        6 => "Overflow".to_string(),
        7 => "Out of memory".to_string(),
        9 => "Subscript out of range".to_string(),
        11 => "Division by zero".to_string(),
        13 => "Type mismatch".to_string(),
        53 => "File not found".to_string(),
        76 => "Path not found".to_string(),
        _ => format!("Application-defined or object-defined error ({number})"),
    }
}

fn expect_value_count(
    name: &str,
    args: &[Value],
    expected: usize,
    span: crate::runtime::Span,
) -> Result<(), Diagnostic> {
    if args.len() == expected {
        Ok(())
    } else {
        let code = if args.len() < expected {
            crate::runtime::DiagnosticCode::ARGUMENT_NOT_OPTIONAL
        } else {
            crate::runtime::DiagnosticCode::GENERIC
        };
        Err(Diagnostic::new(
            code,
            format!("{name} expects exactly {expected} argument(s)"),
            Some(span),
        ))
    }
}

fn file_number_arg(
    name: &str,
    value: &Value,
    span: crate::runtime::Span,
) -> Result<i32, Diagnostic> {
    let number = crate::runtime::numeric::value_to_i64(value).ok_or_else(|| {
        Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} file number must be Integer"),
            Some(span),
        )
    })?;
    if !(1..=511).contains(&number) {
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "File number must be between 1 and 511",
            Some(span),
        ));
    }
    Ok(number as i32)
}

const UNIX_EPOCH_AS_VBA: i64 = 25_569;

fn system_time_to_vba_date(time: std::time::SystemTime) -> Result<f64, Diagnostic> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| {
            Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                format!("Date before Unix epoch is not supported: {}", error),
                None,
            )
        })?;
    Ok(UNIX_EPOCH_AS_VBA as f64 + duration.as_secs_f64() / 86_400.0)
}

fn timer_seconds() -> Result<f64, Diagnostic> {
    let now = system_time_to_vba_date(std::time::SystemTime::now())?;
    Ok((now.fract() * 86_400.0).rem_euclid(86_400.0))
}

fn integer_arg(name: &str, value: &Value, span: crate::runtime::Span) -> Result<i64, Diagnostic> {
    crate::runtime::numeric::value_to_i64(value).ok_or_else(|| {
        Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} argument must be Integer"),
            Some(span),
        )
    })
}

fn date_arg(name: &str, value: &Value, span: crate::runtime::Span) -> Result<f64, Diagnostic> {
    match value {
        Value::Date(value) | Value::Double(value) => Ok(*value),
        Value::Single(value) => Ok(f64::from(*value)),
        Value::Int16(value) => Ok(f64::from(*value)),
        Value::Int32(value) => Ok(f64::from(*value)),
        Value::Int64(value) => Ok(*value as f64),
        _ => Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} argument must be Date"),
            Some(span),
        )),
    }
}

fn date_serial(year: i64, month: i64, day: i64) -> f64 {
    let month_index = month - 1;
    let normalized_year = year + month_index.div_euclid(12);
    let normalized_month = month_index.rem_euclid(12) + 1;
    let days = days_from_civil(normalized_year, normalized_month as u32, 1) + day - 1;
    (days + UNIX_EPOCH_AS_VBA) as f64
}

fn time_serial(hour: i64, minute: i64, second: i64) -> f64 {
    let total = hour * 3600 + minute * 60 + second;
    total as f64 / 86_400.0
}

fn date_add(
    interval: &str,
    number: i64,
    serial: f64,
    span: crate::runtime::Span,
) -> Result<f64, Diagnostic> {
    let days = serial.floor() as i64 - UNIX_EPOCH_AS_VBA;
    let fraction = serial.fract();
    let (year, month, day) = civil_from_days(days);
    let result = match interval.to_ascii_lowercase().as_str() {
        "yyyy" => date_serial(year + number, i64::from(month), i64::from(day)) + fraction,
        "q" => date_serial(year, i64::from(month) + number * 3, i64::from(day)) + fraction,
        "m" => date_serial(year, i64::from(month) + number, i64::from(day)) + fraction,
        "y" | "d" | "w" => serial + number as f64,
        "ww" => serial + (number * 7) as f64,
        "h" => serial + number as f64 / 24.0,
        "n" => serial + number as f64 / 1_440.0,
        "s" => serial + number as f64 / 86_400.0,
        _ => return Err(invalid_interval(interval, span)),
    };
    Ok(result)
}

fn date_diff(
    interval: &str,
    start: f64,
    end: f64,
    span: crate::runtime::Span,
) -> Result<i64, Diagnostic> {
    let start_days = start.floor() as i64 - UNIX_EPOCH_AS_VBA;
    let end_days = end.floor() as i64 - UNIX_EPOCH_AS_VBA;
    let (start_year, start_month, _) = civil_from_days(start_days);
    let (end_year, end_month, _) = civil_from_days(end_days);
    let diff = match interval.to_ascii_lowercase().as_str() {
        "yyyy" => end_year - start_year,
        "q" => (end_year - start_year) * 4 + (i64::from(end_month) - i64::from(start_month)) / 3,
        "m" => (end_year - start_year) * 12 + i64::from(end_month) - i64::from(start_month),
        "y" | "d" | "w" => end_days - start_days,
        "ww" => (end_days - start_days) / 7,
        "h" => ((end - start) * 24.0).trunc() as i64,
        "n" => ((end - start) * 1_440.0).trunc() as i64,
        "s" => ((end - start) * 86_400.0).trunc() as i64,
        _ => return Err(invalid_interval(interval, span)),
    };
    Ok(diff)
}

fn date_part(interval: &str, serial: f64, span: crate::runtime::Span) -> Result<i64, Diagnostic> {
    let days = serial.floor() as i64 - UNIX_EPOCH_AS_VBA;
    let (year, month, day) = civil_from_days(days);
    let seconds = seconds_since_midnight(serial);
    let value = match interval.to_ascii_lowercase().as_str() {
        "yyyy" => year,
        "q" => ((month - 1) / 3 + 1).into(),
        "m" => month.into(),
        "y" => days - days_from_civil(year, 1, 1) + 1,
        "d" => day.into(),
        "w" => weekday_value(serial, 1),
        "ww" => ((days - days_from_civil(year, 1, 1)) / 7) + 1,
        "h" => seconds / 3600,
        "n" => (seconds % 3600) / 60,
        "s" => seconds % 60,
        _ => return Err(invalid_interval(interval, span)),
    };
    Ok(value)
}

fn invalid_interval(interval: &str, span: crate::runtime::Span) -> Diagnostic {
    Diagnostic::new(
        crate::runtime::DiagnosticCode::TYPE_MISMATCH,
        format!("Invalid date interval '{}'", interval),
        Some(span),
    )
}

pub(crate) fn parse_date_value(value: &str, span: crate::runtime::Span) -> Result<f64, Diagnostic> {
    let parts: Vec<_> = value
        .split(['-', '/'])
        .filter_map(|part| part.trim().parse::<i64>().ok())
        .collect();
    if parts.len() == 3 {
        let (year, month, day) = if parts[0] > 31 {
            (parts[0], parts[1], parts[2])
        } else {
            (parts[2], parts[0], parts[1])
        };
        Ok(date_serial(year, month, day))
    } else {
        Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "DateValue expects a date like yyyy-mm-dd or mm/dd/yyyy",
            Some(span),
        ))
    }
}

pub(crate) fn parse_time_value(value: &str, span: crate::runtime::Span) -> Result<f64, Diagnostic> {
    let parts: Vec<_> = value
        .split(':')
        .map(str::trim)
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            Diagnostic::new(
                crate::runtime::DiagnosticCode::TYPE_MISMATCH,
                "TimeValue expects a time like hh:mm:ss",
                Some(span),
            )
        })?;
    if !(2..=3).contains(&parts.len()) {
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "TimeValue expects a time like hh:mm:ss",
            Some(span),
        ));
    }
    Ok(time_serial(
        parts[0],
        parts[1],
        parts.get(2).copied().unwrap_or(0),
    ))
}

fn seconds_since_midnight(serial: f64) -> i64 {
    ((serial.fract().rem_euclid(1.0) * 86_400.0).round() as i64).rem_euclid(86_400)
}

fn weekday_value(serial: f64, first_day: i64) -> i64 {
    let days_since_unix = serial.floor() as i64 - UNIX_EPOCH_AS_VBA;
    let sunday_based = (days_since_unix + 4).rem_euclid(7) + 1;
    let first_day = if (1..=7).contains(&first_day) {
        first_day
    } else {
        1
    };
    (sunday_based - first_day).rem_euclid(7) + 1
}

fn month_name(
    month: i64,
    abbreviate: bool,
    span: crate::runtime::Span,
) -> Result<String, Diagnostic> {
    const FULL: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const SHORT: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if !(1..=12).contains(&month) {
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "MonthName month must be between 1 and 12",
            Some(span),
        ));
    }
    let names = if abbreviate { SHORT } else { FULL };
    Ok(names[month as usize - 1].to_string())
}

fn weekday_name(
    weekday: i64,
    abbreviate: bool,
    span: crate::runtime::Span,
) -> Result<String, Diagnostic> {
    const FULL: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    if !(1..=7).contains(&weekday) {
        return Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            "WeekdayName weekday must be between 1 and 7",
            Some(span),
        ));
    }
    let names = if abbreviate { SHORT } else { FULL };
    Ok(names[weekday as usize - 1].to_string())
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let month = month as i64;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + i64::from(month <= 2), month as u32, day as u32)
}

fn dispatch_callbyname(
    interpreter: &mut Interpreter,
    name: &str,
    args: &[Expr],
    frame: &mut Frame,
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    if name.eq_ignore_ascii_case("CallByName") {
        if args.len() < 3 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::GENERIC,
                "CallByName expects at least 3 arguments",
                Some(span),
            ));
        }
        let obj = interpreter.eval_expr(&args[0], frame)?;
        let member = interpreter.eval_expr(&args[1], frame)?.to_output_string();
        let call_type =
            interpreter.eval_integer_expr(&args[2], frame, "Call type must be Integer")?;

        let remaining_args = &args[3..];

        match call_type {
            1 => {
                // VbMethod
                // Try Function first to get a return value
                if let Ok(val) = interpreter.call_method_function(
                    obj.clone(),
                    &member,
                    remaining_args,
                    frame,
                    span,
                ) {
                    return Ok(Some(val));
                }

                // If function fails, try Sub
                interpreter.call_method_sub(obj, &member, remaining_args, frame, span)?;
                return Ok(Some(Value::Empty));
            }
            2 => {
                // VbGet
                return Ok(Some(interpreter.read_member(&obj, &member, frame, span)?));
            }
            4 | 8 => {
                // VbLet (4) or VbSet (8)
                if remaining_args.len() != 1 {
                    return Err(Diagnostic::new(
                        crate::runtime::DiagnosticCode::GENERIC,
                        "CallByName for Let/Set expects exactly one value argument",
                        Some(span),
                    ));
                }
                let value = interpreter.eval_expr(&remaining_args[0], frame)?;
                interpreter.assign_member_to_value(obj, &member, value, span)?;
                return Ok(Some(Value::Empty));
            }
            _ => {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::GENERIC,
                    format!("Invalid CallByName call type: {}", call_type),
                    Some(span),
                ));
            }
        }
    }
    Ok(None)
}

pub(crate) mod arrays;
pub(crate) mod console;
pub(crate) mod debug;
pub(crate) mod err;
pub(crate) mod math;
pub(crate) mod strings;
pub(crate) mod types;

pub(crate) fn expect_arg_count(
    name: &str,
    args: &[Expr],
    expected: usize,
    span: crate::runtime::Span,
) -> Result<(), Diagnostic> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::GENERIC,
            format!("{name} expects exactly {expected} argument(s)"),
            Some(span),
        ))
    }
}
