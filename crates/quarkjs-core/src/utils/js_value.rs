use std::fmt;

use rquickjs::Value;

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),

    Object,
    Array,
    Function,
}

impl fmt::Display for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Null => write!(f, "null"),
            JsValue::Bool(b) => write!(f, "{}", b),
            JsValue::Number(n) => write!(f, "{}", n),
            JsValue::String(s) => write!(f, "{}", s),

            JsValue::Object => write!(f, "[object Object]"),
            JsValue::Array => write!(f, "[object Array]"),
            JsValue::Function => write!(f, "[function]"),
        }
    }
}

/// Convert QuickJS value → owned Rust representation
pub fn from_qjs(value: &Value) -> JsValue {
    if value.is_undefined() {
        return JsValue::Undefined;
    }

    if value.is_null() {
        return JsValue::Null;
    }

    if let Some(b) = value.as_bool() {
        return JsValue::Bool(b);
    }

    if let Some(n) = value.as_number() {
        return JsValue::Number(n);
    }

    if let Some(s) = value.as_string() {
        return JsValue::String(s.to_string().unwrap_or_else(|_| "[string error]".into()));
    }

    if value.is_array() {
        return JsValue::Array;
    }

    if value.is_function() {
        return JsValue::Function;
    }

    if value.is_object() {
        return JsValue::Object;
    }

    JsValue::Undefined
}
