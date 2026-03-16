use std::collections::HashMap;
use std::fmt;

use crate::parser::ast::{Param, Stmt};

#[derive(Debug, Clone)]
pub enum LingotValue {
    Text(String),
    Number(LingotNumber),
    Bool(bool),
    Void,
    List(Vec<LingotValue>),
    Map(HashMap<String, LingotValue>),
    Object {
        type_name: String,
        fields: HashMap<String, LingotValue>,
    },
    Func {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum LingotNumber {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub struct LingotResult {
    pub ok: bool,
    pub value: LingotValue,
    pub error: Option<String>,
}

impl LingotResult {
    pub fn ok(value: LingotValue) -> Self {
        LingotResult { ok: true, value, error: None }
    }

    pub fn fail(message: String) -> Self {
        LingotResult {
            ok: false,
            value: LingotValue::Void,
            error: Some(message),
        }
    }

    pub fn void() -> Self {
        LingotResult::ok(LingotValue::Void)
    }
}

impl LingotNumber {
    pub fn is_int(&self) -> bool {
        matches!(self, LingotNumber::Int(_))
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            LingotNumber::Int(i) => *i as f64,
            LingotNumber::Float(f) => *f,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            LingotNumber::Int(i) => *i,
            LingotNumber::Float(f) => *f as i64,
        }
    }

    pub fn from_f64(n: f64) -> Self {
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            LingotNumber::Int(n as i64)
        } else {
            LingotNumber::Float(n)
        }
    }
}

impl fmt::Display for LingotValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LingotValue::Text(s) => write!(f, "{}", s),
            LingotValue::Number(n) => write!(f, "{}", n),
            LingotValue::Bool(b) => write!(f, "{}", b),
            LingotValue::Void => write!(f, "Void"),
            LingotValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            LingotValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{} => {}", k, v)?;
                }
                write!(f, "}}")
            }
            LingotValue::Object { type_name, fields } => {
                write!(f, "{} {{", type_name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{} = {}", k, v)?;
                }
                write!(f, "}}")
            }
            LingotValue::Func { name, params, .. } => {
                write!(f, "Func {}(", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p.name)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for LingotNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LingotNumber::Int(i) => write!(f, "{}", i),
            LingotNumber::Float(fl) => {
                // Smart display: round to remove IEEE 754 noise
                let rounded = format!("{:.10}", fl);
                let trimmed = rounded.trim_end_matches('0').trim_end_matches('.');
                write!(f, "{}", trimmed)
            }
        }
    }
}
