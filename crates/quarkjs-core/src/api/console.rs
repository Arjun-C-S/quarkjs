//! Console API implementation for the QuarkJS runtime.
//!
//! Provides `console.log`, `console.warn`, `console.error`, and `console.debug`.
//!
//! NOTE:
//! This implementation uses a simplified value formatter and does NOT fully
//! implement JavaScript's ToString coercion or object inspection.

use std::sync::Arc;

use rquickjs::{Context, Ctx, Function, Object, Result, Value, function::Rest};

use crate::utils::inspect::{InspectOptions, inspect_value};

/// Log levels supported by the console.
#[derive(Clone, Copy)]
pub enum LogLevel {
    Log,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    fn prefix(self) -> &'static str {
        match self {
            LogLevel::Log => "[LOG]",
            LogLevel::Warn => "[WARN]",
            LogLevel::Error => "[ERROR]",
            LogLevel::Debug => "[DEBUG]",
        }
    }
}

/// Output sink abstraction for console logging.
///
/// This allows decoupling the runtime from stdout and enables:
/// - Testing (mock sink)
/// - File logging
/// - Embedding into host applications
pub trait ConsoleSink: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

/// Default stdout sink.
pub struct StdoutSink;

impl ConsoleSink for StdoutSink {
    fn log(&self, level: LogLevel, message: &str) {
        println!("{} {}", level.prefix(), message);
    }
}

/// Core logging function with reduced allocations.
fn build_log_line<'js>(args: &[Value<'js>]) -> String {
    let mut line = String::new();

    let opts = InspectOptions::default();

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }

        // CRITICAL: The cycle detection stack MUST be scoped to the individual argument.
        // Reusing a single `seen` Vec across different arguments will cause false circular triggers.
        let mut seen = Vec::new();

        // Pass the required mutable references
        let formatted = inspect_value(arg, 0, &mut seen, &opts);
        line.push_str(&formatted);
    }

    line
}

/// Factory for console methods.
fn make_logger<'js>(
    ctx: Ctx<'js>,
    level: LogLevel,
    sink: Arc<dyn ConsoleSink>,
) -> Result<Function<'js>> {
    Function::new(ctx.clone(), move |args: Rest<Value<'js>>| -> Result<()> {
        let line = build_log_line(&args.0);
        sink.log(level, &line);
        Ok(())
    })
}

/// Registers the `console` object on the global scope.
///
/// This version is:
/// - Decoupled from stdout
/// - Extensible via `ConsoleSink`
/// - Structurally aligned with runtime design practices
pub fn init_console(ctx: &Context) -> Result<()> {
    let sink: Arc<dyn ConsoleSink> = Arc::new(StdoutSink);

    ctx.with(|ctx| {
        let global = ctx.globals();
        let console = Object::new(ctx.clone())?;

        console.set(
            "log",
            make_logger(ctx.clone(), LogLevel::Log, sink.clone())?,
        )?;
        console.set(
            "warn",
            make_logger(ctx.clone(), LogLevel::Warn, sink.clone())?,
        )?;
        console.set(
            "error",
            make_logger(ctx.clone(), LogLevel::Error, sink.clone())?,
        )?;
        console.set(
            "debug",
            make_logger(ctx.clone(), LogLevel::Debug, sink.clone())?,
        )?;

        global.set("console", console)?;

        Ok(())
    })
}
