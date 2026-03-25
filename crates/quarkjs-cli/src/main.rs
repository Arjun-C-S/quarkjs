use quarkjs_core::Engine;

fn main() {
    let engine = Engine::new().unwrap();

    // 1. Direct self-reference
    engine
        .eval(
            r#"
        const a = {};
        a.self = a;
        console.log("TEST 1:", a);
    "#,
        )
        .unwrap();

    // 2. Mutual reference (a <-> b)
    engine
        .eval(
            r#"
        const a = {};
        const b = { a };
        a.b = b;
        console.log("TEST 2:", a);
    "#,
        )
        .unwrap();

    // 3. Deep circular chain
    engine
        .eval(
            r#"
        const a = { level: 1 };
        const b = { level: 2 };
        const c = { level: 3 };

        a.next = b;
        b.next = c;
        c.next = a;

        console.log("TEST 3:", a);
    "#,
        )
        .unwrap();

    // 4. Circular inside nested structure
    engine
        .eval(
            r#"
        const obj = {
            user: {
                profile: {}
            }
        };

        obj.user.profile.parent = obj;

        console.log("TEST 4:", obj);
    "#,
        )
        .unwrap();

    // 5. Array circular reference (you probably fail this right now)
    engine
        .eval(
            r#"
        const arr = [1, 2];
        arr.push(arr);
        console.log("TEST 5:", arr);
    "#,
        )
        .unwrap();

    // 6. Multiple references to same object (NOT circular)
    engine
        .eval(
            r#"
        const shared = { value: 42 };
        const obj = {
            a: shared,
            b: shared
        };

        console.log("TEST 6:", obj);
    "#,
        )
        .unwrap();

    // 7. Mixed primitives + circular
    engine
        .eval(
            r#"
        const a = {
            name: "test",
            count: 10
        };

        a.self = a;

        console.log("TEST 7:", a);
    "#,
        )
        .unwrap();
}
