use super::super::Interpreter;
use crate::runtime::numeric::{value_to_f64, value_to_i64};
use crate::runtime::{Diagnostic, Value};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;

pub(crate) fn eval_math(
    interpreter: &mut Interpreter,
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    if name.eq_ignore_ascii_case("Abs") {
        let num = number_arg(name, &args[0], span)?;
        return Ok(Some(Value::Double(num.abs())));
    }
    if name.eq_ignore_ascii_case("Atn") {
        return Ok(Some(Value::Double(
            number_arg(name, &args[0], span)?.atan(),
        )));
    }
    if name.eq_ignore_ascii_case("Cos") {
        return Ok(Some(Value::Double(number_arg(name, &args[0], span)?.cos())));
    }
    if name.eq_ignore_ascii_case("Exp") {
        return Ok(Some(Value::Double(number_arg(name, &args[0], span)?.exp())));
    }
    if name.eq_ignore_ascii_case("Log") {
        return Ok(Some(Value::Double(number_arg(name, &args[0], span)?.ln())));
    }
    if name.eq_ignore_ascii_case("Sin") {
        return Ok(Some(Value::Double(number_arg(name, &args[0], span)?.sin())));
    }
    if name.eq_ignore_ascii_case("Sqr") {
        return Ok(Some(Value::Double(
            number_arg(name, &args[0], span)?.sqrt(),
        )));
    }
    if name.eq_ignore_ascii_case("Tan") {
        return Ok(Some(Value::Double(number_arg(name, &args[0], span)?.tan())));
    }
    if name.eq_ignore_ascii_case("Round") {
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::ARGUMENT_COUNT,
                "Round expects 1 to 2 arguments",
                Some(span),
            ));
        }
        let value = number_arg(name, &args[0], span)?;
        let places = args
            .get(1)
            .map(|value| integer_arg(name, value, span))
            .transpose()?
            .unwrap_or(0);
        let factor = 10_f64.powi(places as i32);
        return Ok(Some(Value::Double((value * factor).round() / factor)));
    }
    if name.eq_ignore_ascii_case("Sgn") {
        let num = number_arg(name, &args[0], span)?;
        return Ok(Some(Value::Int16(if num > 0.0 {
            1
        } else if num < 0.0 {
            -1
        } else {
            0
        })));
    }
    if name.eq_ignore_ascii_case("Int") {
        let num = number_arg(name, &args[0], span)?;
        return Ok(Some(Value::Int64(num.floor() as i64)));
    }
    if name.eq_ignore_ascii_case("Fix") {
        let num = number_arg(name, &args[0], span)?;
        return Ok(Some(Value::Int64(num.trunc() as i64)));
    }
    if name.eq_ignore_ascii_case("Randomize") {
        if args.len() > 1 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::ARGUMENT_COUNT,
                "Randomize expects at most 1 argument",
                Some(span),
            ));
        }
        let seed = if args.is_empty() {
            rand::thread_rng().r#gen::<u64>()
        } else {
            value_to_i64(&args[0]).ok_or_else(|| {
                Diagnostic::new(
                    crate::runtime::DiagnosticCode::TYPE_MISMATCH,
                    "Randomize seed must be Integer",
                    Some(span),
                )
            })? as u64
        };
        interpreter.rng = Pcg64::seed_from_u64(seed);
        return Ok(Some(Value::Empty));
    }
    if name.eq_ignore_ascii_case("Rnd") {
        if args.len() > 1 {
            return Err(Diagnostic::new(
                crate::runtime::DiagnosticCode::ARGUMENT_COUNT,
                "Rnd expects at most 1 argument",
                Some(span),
            ));
        }
        return Ok(Some(Value::Double(interpreter.rng.r#gen::<f64>())));
    }
    if let Some(value) = eval_financial(name, args, span)? {
        return Ok(Some(value));
    }

    Ok(None)
}

fn eval_financial(
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Option<Value>, Diagnostic> {
    match name.to_ascii_lowercase().as_str() {
        "sln" => {
            expect_value_count(name, args, 3, span)?;
            let cost = number_arg(name, &args[0], span)?;
            let salvage = number_arg(name, &args[1], span)?;
            let life = number_arg(name, &args[2], span)?;
            Ok(Some(Value::Double((cost - salvage) / life)))
        }
        "syd" => {
            expect_value_count(name, args, 4, span)?;
            let cost = number_arg(name, &args[0], span)?;
            let salvage = number_arg(name, &args[1], span)?;
            let life = number_arg(name, &args[2], span)?;
            let period = number_arg(name, &args[3], span)?;
            Ok(Some(Value::Double(
                (cost - salvage) * (life - period + 1.0) * 2.0 / (life * (life + 1.0)),
            )))
        }
        "ddb" => {
            if args.len() < 4 || args.len() > 5 {
                return Err(Diagnostic::new(
                    crate::runtime::DiagnosticCode::ARGUMENT_COUNT,
                    "DDB expects 4 to 5 arguments",
                    Some(span),
                ));
            }
            let cost = number_arg(name, &args[0], span)?;
            let salvage = number_arg(name, &args[1], span)?;
            let life = number_arg(name, &args[2], span)?;
            let period = integer_arg(name, &args[3], span)?;
            let factor = args
                .get(4)
                .map(|value| number_arg(name, value, span))
                .transpose()?
                .unwrap_or(2.0);
            let mut book = cost;
            let mut depreciation = 0.0;
            for _ in 1..=period {
                depreciation = (book * factor / life).min(book - salvage).max(0.0);
                book -= depreciation;
            }
            Ok(Some(Value::Double(depreciation)))
        }
        "fv" | "pv" | "pmt" | "nper" | "ipmt" | "ppmt" | "rate" | "npv" | "irr" | "mirr" => {
            eval_cashflow(name, args, span).map(Some)
        }
        _ => Ok(None),
    }
}

fn eval_cashflow(
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<Value, Diagnostic> {
    match name.to_ascii_lowercase().as_str() {
        "fv" => {
            let (rate, nper, pmt, pv, typ) = annuity_args(name, args, span)?;
            Ok(Value::Double(fv(rate, nper, pmt, pv, typ)))
        }
        "pv" => {
            let (rate, nper, pmt, fv_value, typ) = annuity_args(name, args, span)?;
            Ok(Value::Double(pv(rate, nper, pmt, fv_value, typ)))
        }
        "pmt" => {
            if args.len() < 3 || args.len() > 5 {
                return Err(arg_range(name, 3, 5, span));
            }
            let rate = number_arg(name, &args[0], span)?;
            let nper = number_arg(name, &args[1], span)?;
            let pv = number_arg(name, &args[2], span)?;
            let fv_value = optional_number(name, args.get(3), span, 0.0)?;
            let typ = optional_number(name, args.get(4), span, 0.0)?;
            Ok(Value::Double(pmt(rate, nper, pv, fv_value, typ)))
        }
        "nper" => {
            if args.len() < 3 || args.len() > 5 {
                return Err(arg_range(name, 3, 5, span));
            }
            let rate = number_arg(name, &args[0], span)?;
            let pmt = number_arg(name, &args[1], span)?;
            let pv = number_arg(name, &args[2], span)?;
            let fv_value = optional_number(name, args.get(3), span, 0.0)?;
            let typ = optional_number(name, args.get(4), span, 0.0)?;
            if rate == 0.0 {
                return Ok(Value::Double(-(pv + fv_value) / pmt));
            }
            let numerator = pmt * (1.0 + rate * typ) - fv_value * rate;
            let denominator = pv * rate + pmt * (1.0 + rate * typ);
            Ok(Value::Double(
                (numerator / denominator).ln() / (1.0 + rate).ln(),
            ))
        }
        "ipmt" | "ppmt" => {
            if args.len() < 4 || args.len() > 6 {
                return Err(arg_range(name, 4, 6, span));
            }
            let rate = number_arg(name, &args[0], span)?;
            let per = integer_arg(name, &args[1], span)?;
            let nper = number_arg(name, &args[2], span)?;
            let pv_value = number_arg(name, &args[3], span)?;
            let fv_value = optional_number(name, args.get(4), span, 0.0)?;
            let typ = optional_number(name, args.get(5), span, 0.0)?;
            let payment = pmt(rate, nper, pv_value, fv_value, typ);
            let mut balance = pv_value;
            let mut interest = 0.0;
            for period in 1..=per {
                interest = if typ != 0.0 && period == 1 {
                    0.0
                } else {
                    balance * rate
                };
                let principal = payment - interest;
                if period < per {
                    balance += principal;
                }
            }
            Ok(Value::Double(if name.eq_ignore_ascii_case("IPmt") {
                interest
            } else {
                payment - interest
            }))
        }
        "rate" => {
            if args.len() < 3 || args.len() > 6 {
                return Err(arg_range(name, 3, 6, span));
            }
            let nper = number_arg(name, &args[0], span)?;
            let pmt_value = number_arg(name, &args[1], span)?;
            let pv_value = number_arg(name, &args[2], span)?;
            let fv_value = optional_number(name, args.get(3), span, 0.0)?;
            let typ = optional_number(name, args.get(4), span, 0.0)?;
            let mut rate = optional_number(name, args.get(5), span, 0.1)?;
            for _ in 0..50 {
                let f = fv(rate, nper, pmt_value, pv_value, typ) + fv_value;
                let delta = 1e-6;
                let df = (fv(rate + delta, nper, pmt_value, pv_value, typ) + fv_value - f) / delta;
                if df.abs() < 1e-12 {
                    break;
                }
                let next = rate - f / df;
                if (next - rate).abs() < 1e-10 {
                    rate = next;
                    break;
                }
                rate = next;
            }
            Ok(Value::Double(rate))
        }
        "npv" => {
            if args.len() < 2 {
                return Err(arg_range(name, 2, usize::MAX, span));
            }
            let rate = number_arg(name, &args[0], span)?;
            let values = flatten_values(&args[1..], span)?;
            let total = values
                .iter()
                .enumerate()
                .map(|(index, value)| value / (1.0 + rate).powi(index as i32 + 1))
                .sum();
            Ok(Value::Double(total))
        }
        "irr" => {
            if args.is_empty() || args.len() > 2 {
                return Err(arg_range(name, 1, 2, span));
            }
            let values = flatten_values(&args[0..1], span)?;
            ensure_mixed_cashflows("IRR", &values, span)?;
            let guess = optional_number(name, args.get(1), span, 0.1)?;
            Ok(Value::Double(solve_rate(
                name,
                &values,
                guess,
                span,
                |values, rate| {
                    values
                        .iter()
                        .enumerate()
                        .map(|(index, value)| value / (1.0 + rate).powi(index as i32))
                        .sum()
                },
            )?))
        }
        "mirr" => {
            expect_value_count(name, args, 3, span)?;
            let values = flatten_values(&args[0..1], span)?;
            ensure_mixed_cashflows("MIRR", &values, span)?;
            let finance_rate = number_arg(name, &args[1], span)?;
            let reinvest_rate = number_arg(name, &args[2], span)?;
            let periods = values.len() as i32 - 1;
            let positive: f64 = values
                .iter()
                .enumerate()
                .filter(|(_, value)| **value > 0.0)
                .map(|(index, value)| value * (1.0 + reinvest_rate).powi(periods - index as i32))
                .sum();
            let negative: f64 = values
                .iter()
                .enumerate()
                .filter(|(_, value)| **value < 0.0)
                .map(|(index, value)| value / (1.0 + finance_rate).powi(index as i32))
                .sum();
            Ok(Value::Double(
                (positive / -negative).powf(1.0 / periods as f64) - 1.0,
            ))
        }
        _ => Ok(Value::Empty),
    }
}

fn annuity_args(
    name: &str,
    args: &[Value],
    span: crate::runtime::Span,
) -> Result<(f64, f64, f64, f64, f64), Diagnostic> {
    if args.len() < 3 || args.len() > 5 {
        return Err(arg_range(name, 3, 5, span));
    }
    Ok((
        number_arg(name, &args[0], span)?,
        number_arg(name, &args[1], span)?,
        number_arg(name, &args[2], span)?,
        optional_number(name, args.get(3), span, 0.0)?,
        optional_number(name, args.get(4), span, 0.0)?,
    ))
}

fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        -(pv + pmt * nper)
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(pv * factor + pmt * (1.0 + rate * typ) * (factor - 1.0) / rate)
    }
}

fn pv(rate: f64, nper: f64, pmt: f64, fv_value: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        -(fv_value + pmt * nper)
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(fv_value + pmt * (1.0 + rate * typ) * (factor - 1.0) / rate) / factor
    }
}

fn pmt(rate: f64, nper: f64, pv: f64, fv_value: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        -(pv + fv_value) / nper
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(fv_value + pv * factor) * rate / ((1.0 + rate * typ) * (factor - 1.0))
    }
}

fn solve_rate(
    name: &str,
    values: &[f64],
    guess: f64,
    span: crate::runtime::Span,
    f: impl Fn(&[f64], f64) -> f64,
) -> Result<f64, Diagnostic> {
    let mut rate = guess;
    for _ in 0..50 {
        let value = f(values, rate);
        if value.abs() < 1e-10 {
            return Ok(rate);
        }
        let delta = 1e-6;
        let derivative = (f(values, rate + delta) - value) / delta;
        if derivative.abs() < 1e-12 {
            break;
        }
        let next = rate - value / derivative;
        if next.is_finite() && next > -1.0 && (next - rate).abs() < 1e-10 {
            return Ok(next);
        }
        if !next.is_finite() || next <= -1.0 {
            break;
        }
        rate = next;
    }

    let mut previous_rate = -0.999_999;
    let mut previous_value = f(values, previous_rate);
    for step in 1..=20_000 {
        let current_rate = -0.999_999 + step as f64 * 0.001;
        let current_value = f(values, current_rate);
        if previous_value.signum() != current_value.signum() {
            let mut low = previous_rate;
            let mut high = current_rate;
            for _ in 0..100 {
                let mid = (low + high) / 2.0;
                let mid_value = f(values, mid);
                if mid_value.abs() < 1e-10 {
                    return Ok(mid);
                }
                if f(values, low).signum() == mid_value.signum() {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            return Ok((low + high) / 2.0);
        }
        previous_rate = current_rate;
        previous_value = current_value;
    }

    Err(Diagnostic::new(
        crate::runtime::DiagnosticCode::TYPE_MISMATCH,
        format!("{name} could not find a convergent rate for the supplied cash flows"),
        Some(span),
    ))
}

fn ensure_mixed_cashflows(
    name: &str,
    values: &[f64],
    span: crate::runtime::Span,
) -> Result<(), Diagnostic> {
    if values.iter().any(|value| *value > 0.0) && values.iter().any(|value| *value < 0.0) {
        Ok(())
    } else {
        Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} requires at least one positive and one negative cash flow"),
            Some(span),
        ))
    }
}

fn flatten_values(values: &[Value], span: crate::runtime::Span) -> Result<Vec<f64>, Diagnostic> {
    let mut out = Vec::new();
    for value in values {
        if let Value::Array(array) = value {
            for element in &array.elements {
                out.push(number_arg("financial", element, span)?);
            }
        } else {
            out.push(number_arg("financial", value, span)?);
        }
    }
    Ok(out)
}

fn number_arg(name: &str, value: &Value, span: crate::runtime::Span) -> Result<f64, Diagnostic> {
    value_to_f64(value).ok_or_else(|| {
        Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} requires a numeric argument"),
            Some(span),
        )
    })
}

fn integer_arg(name: &str, value: &Value, span: crate::runtime::Span) -> Result<i64, Diagnostic> {
    value_to_i64(value).ok_or_else(|| {
        Diagnostic::new(
            crate::runtime::DiagnosticCode::TYPE_MISMATCH,
            format!("{name} argument must be Integer"),
            Some(span),
        )
    })
}

fn optional_number(
    name: &str,
    value: Option<&Value>,
    span: crate::runtime::Span,
    default: f64,
) -> Result<f64, Diagnostic> {
    match value {
        Some(Value::Missing) | None => Ok(default),
        Some(value) => number_arg(name, value, span),
    }
}

fn arg_range(name: &str, min: usize, max: usize, span: crate::runtime::Span) -> Diagnostic {
    Diagnostic::new(
        crate::runtime::DiagnosticCode::GENERIC,
        if max == usize::MAX {
            format!("{name} expects at least {min} arguments")
        } else {
            format!("{name} expects {min} to {max} arguments")
        },
        Some(span),
    )
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
        Err(Diagnostic::new(
            crate::runtime::DiagnosticCode::ARGUMENT_COUNT,
            format!("{name} expects exactly {expected} argument(s)"),
            Some(span),
        ))
    }
}
