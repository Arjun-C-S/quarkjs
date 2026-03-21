use super::js_value::JsValue;

pub fn format_js_value(value: &JsValue) -> String {
    match value {
        JsValue::Undefined => "undefined".into(),
        JsValue::Null => "null".into(),
        JsValue::Bool(b) => b.to_string(),
        JsValue::Number(n) => n.to_string(),
        JsValue::String(s) => s.clone(),

        JsValue::Object => "[object Object]".into(),
        JsValue::Array => "[object Array]".into(),
        JsValue::Function => "[function]".into(),
    }
}
