use rquickjs::{Context, Result, Runtime, Value};

use crate::api::console::{self};
use crate::utils::js_value::{from_qjs, JsValue};

/// Core JavaScript execution engine.
///
/// Responsible for:
/// - Owning the QuickJS runtime
/// - Managing the execution context
/// - Evaluating JavaScript code
pub struct Engine {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
}

impl Engine {
    /// Create a new JavaScript engine instance.
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        // Inject built-in APIs
        console::init_console(&context)?;

        Ok(Self { runtime, context })
    }

    /// Evaluate JavaScript code and return the result.
    pub fn eval(&self, code: &str) -> Result<JsValue> {
        self.context.with(|ctx| {
            let value: Value = ctx.eval(code)?;
            Ok(from_qjs(&value))
        })
    }
}
