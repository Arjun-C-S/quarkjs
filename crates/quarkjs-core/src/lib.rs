use crate::api::console;
use rquickjs::{Context, Error, Runtime};

pub mod api;

pub fn run_quark(js_code: &str) {
    let runtime = Runtime::new().expect("Failed to create JSRuntime");
    let context = Context::full(&runtime).expect("Failed to create JSContext");

    let response = console::init_console(&context);

    if let Err(e) = response {
        eprintln!("Failed to initialize console: {}", e);
        return;
    }

    context.with(|ctx| {
        let _: Result<(), Error> = ctx.eval(js_code);
    });
}
