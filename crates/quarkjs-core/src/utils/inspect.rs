use rquickjs::{Ctx, Object, Value};

const MAX_DEPTH: usize = 3;

pub fn inspect_value(ctx: &Ctx, value: &Value, depth: usize) -> String {
    if depth > MAX_DEPTH {
        return "...".into();
    }

    if value.is_object() {
        return inspect_object(ctx, value, depth);
    }

    if let Some(s) = value.as_string() {
        return s.to_string().unwrap_or_default();
    }

    if let Some(n) = value.as_number() {
        return n.to_string();
    }

    if let Some(b) = value.as_bool() {
        return b.to_string();
    }

    if value.is_null() {
        return "null".into();
    }

    if value.is_undefined() {
        return "undefined".into();
    }

    "[unknown]".into()
}

fn inspect_object(ctx: &Ctx, value: &Value, depth: usize) -> String {
    let obj = match Object::from_value(value.clone()) {
        Ok(o) => o,
        Err(_) => return "[object Object]".into(),
    };

    let mut parts = Vec::new();

    for k in obj.keys() {
        let key: String = match k {
            Ok(k) => k,
            Err(_) => continue,
        };

        let val: Value = match obj.get(&key) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let formatted = inspect_value(ctx, &val, depth + 1);

        parts.push(format!("{}: {}", key, formatted));
    }

    format!("{{ {} }}", parts.join(", "))
}
