use quarkjs_core::Engine;

fn main() {
    let engine = Engine::new().unwrap();

    let result = engine.eval("1 * 2").unwrap();

    let result2 = engine.eval("console.log('Hello from console')").unwrap();

    let result3 = engine.eval("console.log({ a: 1, b: { c: 0 } })").unwrap();

    println!("{}", result);

    println!("{}", result2);

    println!("{}", result3);
}
