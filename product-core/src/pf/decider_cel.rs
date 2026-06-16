//! CEL evaluation for Decider logic — the standard expression layer (§4.4).
//!
//! Stage 2 lets guards and value assignments be CEL expressions over a small
//! environment (`state`, `command`, `event` maps). We build CEL values from our
//! own `Scalar` so integers stay `Int` (serde-sourcing would make them `UInt`
//! and break arithmetic), and convert results back. A value is a CEL expression
//! when it is a string with a leading `=`; otherwise it is a literal.

use std::collections::HashMap;
use std::sync::Arc;

use cel_interpreter::{Context, Program, Value};

use super::decider_logic::{Payload, Scalar};

/// Named environment maps (e.g. `("state", &state), ("command", &payload)`).
pub type Bindings<'a> = [(&'a str, &'a Payload)];

fn scalar_to_cel(s: &Scalar) -> Value {
    match s {
        Scalar::Bool(b) => Value::Bool(*b),
        Scalar::Int(i) => Value::Int(*i),
        Scalar::Str(s) => Value::String(Arc::new(s.clone())),
    }
}

fn map_to_cel(m: &Payload) -> Value {
    let hm: HashMap<String, Value> = m.iter().map(|(k, v)| (k.clone(), scalar_to_cel(v))).collect();
    Value::from(hm)
}

fn cel_to_scalar(v: &Value) -> Option<Scalar> {
    match v {
        Value::Bool(b) => Some(Scalar::Bool(*b)),
        Value::Int(i) => Some(Scalar::Int(*i)),
        Value::UInt(u) => i64::try_from(*u).ok().map(Scalar::Int),
        Value::String(s) => Some(Scalar::Str(s.to_string())),
        _ => None,
    }
}

fn context(bindings: &Bindings) -> Context<'static> {
    let mut ctx = Context::default();
    for (name, map) in bindings {
        ctx.add_variable_from_value(name.to_string(), map_to_cel(map));
    }
    ctx
}

/// Evaluate a CEL boolean guard. Total: a compile or execution error, or a
/// non-boolean result, is `false` (the guard fails, so the command is rejected).
pub fn eval_bool(expr: &str, bindings: &Bindings) -> bool {
    let Ok(program) = Program::compile(expr) else {
        return false;
    };
    matches!(program.execute(&context(bindings)), Ok(Value::Bool(true)))
}

/// Evaluate a CEL expression to a scalar value (for `set`/`with` assignments).
pub fn eval_scalar(expr: &str, bindings: &Bindings) -> Result<Scalar, String> {
    let program = Program::compile(expr).map_err(|e| format!("invalid expression '{expr}': {e}"))?;
    let value = program.execute(&context(bindings)).map_err(|e| format!("evaluating '{expr}': {e}"))?;
    cel_to_scalar(&value).ok_or_else(|| format!("expression '{expr}' produced an unsupported value"))
}

/// Resolve an assignment value: a string with a leading `=` is a CEL
/// expression, anything else is the literal scalar.
pub fn eval_value(v: &Scalar, bindings: &Bindings) -> Result<Scalar, String> {
    if let Scalar::Str(s) = v {
        if let Some(expr) = s.strip_prefix('=') {
            return eval_scalar(expr, bindings);
        }
    }
    Ok(v.clone())
}

/// Report whether a CEL expression compiles (used by the well-formedness pass).
pub fn compiles(expr: &str) -> Result<(), String> {
    Program::compile(expr).map(|_| ()).map_err(|e| format!("invalid expression '{expr}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, Scalar)]) -> Payload {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn bool_guard_over_state_and_command() {
        let state = map(&[("status", Scalar::Str("placed".into()))]);
        let command = map(&[("amount", Scalar::Int(100))]);
        let b = [("state", &state), ("command", &command)];
        assert!(eval_bool("state.status == 'placed' && command.amount > 0", &b));
        assert!(!eval_bool("state.status == 'paid'", &b));
    }

    #[test]
    fn int_arithmetic_works_not_uint() {
        let command = map(&[("amount", Scalar::Int(100))]);
        let b = [("command", &command)];
        assert_eq!(eval_scalar("command.amount + 1", &b).expect("eval"), Scalar::Int(101));
    }

    #[test]
    fn eval_value_distinguishes_literal_from_expression() {
        let command = map(&[("amount", Scalar::Int(7))]);
        let b = [("command", &command)];
        assert_eq!(eval_value(&Scalar::Str("placed".into()), &b).expect("lit"), Scalar::Str("placed".into()));
        assert_eq!(eval_value(&Scalar::Str("=command.amount".into()), &b).expect("cel"), Scalar::Int(7));
    }

    #[test]
    fn bad_expression_is_false_or_error() {
        let empty = map(&[]);
        let b = [("state", &empty)];
        assert!(!eval_bool("this is not cel", &b));
        assert!(eval_scalar("nope.field", &b).is_err());
    }
}
