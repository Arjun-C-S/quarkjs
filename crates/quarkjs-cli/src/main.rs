fn main() {
    println!("QuarkJS CLI");

    let js_code = format!(
        "{}{}{}{}{}{}{}",
        "console.log('Hello from the CLI string');",
        "console.log(null);",
        "console.log(42);",
        "console.log(true);",
        "console.log({});",
        "console.log([]);",
        "console.log(undefined);"
    );

    quarkjs_core::run_quark(&js_code);

    println!("QuarkJS CLI finished");
}
