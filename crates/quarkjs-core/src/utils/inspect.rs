use rquickjs::{Object, Value};

/// For controlling inspection behavior.
#[derive(Clone)]
pub struct InspectOptions {
    pub max_depth: usize,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self { max_depth: 2 }
    }
}

// Use a Vec as an ancestor stack instead of a global HashSet.
// This is drastically faster for shallow depths and fixes the sibling bug.
type Seen<'js> = Vec<Object<'js>>;

pub fn inspect_value<'js>(
    value: &Value<'js>,
    depth: usize,
    seen: &mut Seen<'js>,
    opts: &InspectOptions,
) -> String {
    if depth >= opts.max_depth {
        return "...".into();
    }

    if let Some(obj) = value.as_object() {
        return inspect_object(obj, depth, seen, opts);
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

fn inspect_object<'js>(
    obj: &Object<'js>,
    depth: usize,
    seen: &mut Seen<'js>,
    opts: &InspectOptions,
) -> String {
    if seen.contains(obj) {
        return "[Circular]".into();
    }

    seen.push(obj.clone());

    let mut parts = Vec::new();

    for prop in obj.props::<String, Value<'js>>() {
        if let Ok((key, val)) = prop {
            let formatted = inspect_value(&val, depth + 1, seen, opts);
            parts.push(format!("{}: {}", key, formatted));
        }
    }

    seen.pop();

    if parts.is_empty() {
        return "{}".into();
    }

    format!("{{ {} }}", parts.join(", "))
}
