# QuarkJS — Architecture

This document is the full technical specification for QuarkJS. It covers system design, component responsibilities, implementation constraints, and the decisions behind them.

---

## Table of Contents

1. [Three-Layer System Model](#1-three-layer-system-model)
2. [Global Integration Architecture](#2-global-integration-architecture)
3. [Runtime Execution Architecture](#3-runtime-execution-architecture)
4. [QuickJS Core Model](#4-quickjs-core-model)
5. [Context vs Runtime vs Isolate](#5-context-vs-runtime-vs-isolate)
6. [Garbage Collection Behavior](#6-garbage-collection-behavior)
7. [Core Components](#7-core-components)
8. [Host-Facing Runtime API](#8-host-facing-runtime-api)
9. [Script Lifecycle Model](#9-script-lifecycle-model)
10. [Critical Implementation Details](#10-critical-implementation-details)
11. [Project Structure](#11-project-structure)
12. [Configuration System](#12-configuration-system)
13. [Observability & Debugging](#13-observability--debugging)
14. [Key Decisions & Rationale](#14-key-decisions--rationale)

---

## 1. Three-Layer System Model

QuarkJS is three stacked systems, not one. Conflating them is how most embedded runtimes fail.

```
┌─────────────────────────────────────┐
│  Layer 1 — Language Bindings        │
│  (C ABI, Rust API)                  │
│  How external programs embed QuarkJS│
└──────────────────┬──────────────────┘
                   │
┌──────────────────▼──────────────────┐
│  Layer 2 — Runtime                  │
│  (loader, event loop, APIs,         │
│   host bindings, sandbox)           │
│  How JS executes internally         │
└──────────────────┬──────────────────┘
                   │
┌──────────────────▼──────────────────┐
│  Layer 3 — Engine                   │
│  (QuickJS)                          │
│  The actual JS VM                   │
└─────────────────────────────────────┘
```

**Layer 1 (C ABI / Rust API)** is the front door. It is not part of the execution pipeline. It sits outside the runtime and acts as a gateway for host applications in any language.

**Layer 2 (Runtime)** is the internal execution architecture — the pipeline a JS script travels through when it runs.

**Layer 3 (Engine)** is QuickJS. This layer is not owned by QuarkJS.

---

## 2. Global Integration Architecture

This shows how external programs talk to QuarkJS — the embedding concern, separate from how JS executes internally.

```
      Host Applications
  ┌────────┬────────┬────────┐
  │ Rust   │  C     │  C++   │
  │ Python │  Go    │  Zig   │
  └────────┴───┬────┴────────┘
               │
               ▼
         C API (Stable ABI)         ← Layer 1
               │
               ▼
         Runtime Manager            ← Layer 2 entry point
       ┌───────┼────────┐
       │       │        │
       ▼       ▼        ▼
  Module   Event Loop  APIs
  Loader
       │
       ▼
  Host Bindings
       │
       ▼
    Sandbox
       │
       ▼
  Engine Wrapper                    ← Layer 2 / Layer 3 boundary
       │
       ▼
    QuickJS                         ← Layer 3
```

Example host program in C using the C API (Milestone 3):

```c
QuarkRuntime* rt = quark_new();
quark_register_function(rt, "fan_on", fan_on);
quark_load_script(rt, "rules.js");
quark_call(rt, "control");
quark_free(rt);
```

The C API is Milestone 3. It is documented here so the runtime is designed with it in mind from the start — not retrofitted onto a Rust-only architecture.

---

## 3. Runtime Execution Architecture

The internal pipeline — how a JS script executes once it enters the runtime.

```
  User Scripts
       │
  Module Loader         ← import resolution + caching
       │
  Runtime APIs          ← console, timers
       │
  Host Bindings         ← Rust ↔ JS bridge  [HIGH RISK]
       │
  Event Loop            ← promise resolution, async tasks
       │
  Engine Wrapper        ← QuickJS Rust interface
       │
  QuickJS Engine        ← C engine (stable, proven)
```

**Host Bindings is marked HIGH RISK** because it is where the vast majority of memory safety and error-handling bugs will originate. This layer requires the most careful design.

Layer dependency order matters: Runtime APIs depend on bindings, bindings depend on the engine, and the event loop interacts with engine promise jobs.

---

## 4. QuickJS Core Model

QuarkJS is built on four fundamental QuickJS types. Understanding these is required before touching the engine wrapper.

```
JSRuntime   → global VM instance, owns the GC heap
JSContext   → execution environment (analogous to a JS realm)
JSValue     → represents all JavaScript values
JSAtom      → internal string deduplication handle
```

Key implications:

- `JSRuntime` holds the GC heap — one per QuarkJS instance
- `JSContext` isolates global scope — multiple contexts can share one runtime
- `JSRuntime` is **not thread-safe** — one runtime must stay on one thread, always
- Module loading is context-scoped; sandbox limits are set on the runtime

---

## 5. Context vs Runtime vs Isolate

Different engines organize execution environments differently:

| Engine | Top Level | Script Environment |
|---|---|---|
| QuickJS | Runtime | Context |
| V8 | Isolate | Context |
| Lua | State | Coroutine/Stack |

### How QuickJS is structured

```
JSRuntime
   │
   ├── JSContext  ← script A's global environment
   ├── JSContext  ← script B's global environment
   └── JSContext  ← script C's global environment
```

`JSRuntime` owns: garbage collector, memory allocator, atom table, job queue, global limits.

`JSContext` owns: `globalThis`, built-in objects, module registry.

Scripts in separate contexts cannot see each other's globals, but they **share the GC, memory pool, and atom table**. This is lightweight isolation, not full sandboxing.

### How V8 differs

```
Isolate  ← fully independent VM, separate heap, separate GC
   │
   ├── Context
   └── Context
```

V8 Isolates are fully independent. QuickJS Contexts are not — they share the runtime's memory. QuarkJS cannot offer V8-level isolation without running multiple `JSRuntime` instances.

### The three models available to QuarkJS

**Model A — One Runtime, One Context** *(chosen for MVP)*

```
JSRuntime
   └── JSContext
```

Simplest implementation. Correct for embedded single-script use cases.

**Model B — One Runtime, Multiple Contexts**

```
JSRuntime
   ├── JSContext  ← plugin A
   ├── JSContext  ← plugin B
   └── JSContext  ← plugin C
```

Script isolation with shared memory pool. Suitable for multi-plugin hosts.

**Model C — Multiple Runtimes**

```
Runtime A → JSRuntime → JSContext
Runtime B → JSRuntime → JSContext
```

Strongest isolation. Required for true multi-tenant sandboxing. Higher memory cost.

### The decision for QuarkJS

**Start with Model A: one JSRuntime, one JSContext.**

QuarkJS is an embedded scripting runtime, not a multi-tenant cloud runtime. The complexity explosion from adding multiple contexts too early — shared objects, async job queues, module cache invalidation — makes the runtime unmaintainable before it is functional. Promote to Model B when a real host application requires plugin isolation.

---

## 6. Garbage Collection Behavior

QuickJS uses **reference counting with cycle detection**, not a tracing GC.

- Objects are freed immediately when their reference count reaches zero
- Cyclic references are detected and collected during GC passes
- Host objects wrapped as JS values must integrate via finalizers — if they don't, they leak permanently
- `JS_SetMemoryLimit` triggers a GC pass before throwing an out-of-memory error

> **Critical:** Every Rust object handed to QuickJS must have a registered finalizer. The GC is the only mechanism that will ever call your cleanup code. Missing a single `Drop` implementation for a `JSValue` handle means the host's RAM will bleed until the process is killed.

---

## 7. Core Components

### 7.1 Runtime Manager — `src/runtime/runtime.rs`

The central orchestrator. All other components are owned and driven by the Runtime Manager. This is the entry point for both the Rust API and (in Milestone 3) the C API. It is not a layer in the execution pipeline — it is the system that runs the pipeline.

Responsibilities:
- Initialize and own the QuickJS `JSRuntime` and `JSContext`
- Coordinate component startup and shutdown order
- Expose the host-facing API (`register_function`, `eval_script`, `call_export`)
- Drive the event loop tick internally

### 7.2 Engine Wrapper — `src/engine/quickjs_wrapper.rs`

Wraps the QuickJS C engine and provides a safe Rust interface. **Use `rquickjs` — do not write raw FFI bindings from scratch.**

Responsibilities:
- Create and destroy runtime instances
- Execute scripts
- Manage JavaScript values and reference counts
- Run garbage collection on demand
- Enforce memory limits via `JS_SetMemoryLimit`
- Stop runaway scripts via `JS_SetInterruptHandler`

### 7.3 Module Loader — `src/modules/`

Handles JavaScript module resolution and loading. Module sources are resolved through a Resolver interface — never directly from the filesystem.

Responsibilities:
- Resolve import paths against a configurable root
- Load module source through the resolver interface
- Cache loaded modules (LRU, bounded by `max_module_cache`)
- Prevent path traversal and unauthorized access

```js
import { helper } from "./utils.js"
```

**Security rule — sandbox escape prevention:**

All module paths must resolve inside the configured root. The following must be rejected at the resolver level, not by convention:

```
import "../../etc/passwd"   // must hard-fail
import "/absolute/path"     // must hard-fail
```

**Root-jail implementation:**

Use `canonicalize()` to resolve the final path, then assert it starts with the configured root prefix. `canonicalize()` has known behavior differences on Windows vs Linux — test both explicitly. Do not rely on string prefix matching alone before canonicalization.

```rust
let resolved = canonicalize(root.join(import_path))?;
if !resolved.starts_with(&root) {
    return Err(ResolveError::PathEscapesRoot);
}
```

### 7.4 Host Binding System — `src/bindings/`

The bridge between the Rust host application and the JavaScript runtime. **Highest-risk component in the project.**

```rust
// Host side (Rust)
runtime.register_function("log", log_function);
```

```js
// Script side (JavaScript)
log("hello world")
```

**Data serialization strategy:**

The silent performance killer in embedded runtimes is moving large data structures between Rust and JS. Cloning a 10 MB JSON payload into the QuickJS heap can exceed the memory limit or spike CPU enough to trigger the interrupt handler prematurely.

**Rule:** Favor passing **handles** over full data serialization. Use the `Class<T>` trait to wrap Rust objects as opaque JS handles.

```rust
// Avoid: serializing large data into the JS heap
runtime.register_function("getOrder", |args| {
    Ok(serialize_entire_order_to_jsvalue(order))  // dangerous at scale
});

// Prefer: pass an opaque handle
runtime.register_function("getOrder", |args| {
    Ok(QuarkClass::new(OrderHandle { id: order_id }))  // safe
});
```

### 7.5 Sandbox System — `src/sandbox/`

Ensures scripts cannot crash or block the host. All limits are enforced at the engine level, not by convention.

Enforced limits:
- Memory limit (configurable, default 32 MB)
- Execution timeout via interrupt handler (default 100 ms)
- Module cache size (configurable, default 256 entries)
- Restricted API surface — only explicitly registered functions are available

### 7.6 Interrupt Handler — `src/sandbox/interrupt.rs`

The interrupt handler is how execution timeouts are enforced. QuickJS calls the registered handler periodically during script execution.

```rust
JS_SetInterruptHandler(rt, |opaque| {
    let state = opaque as *mut RuntimeState;
    if (*state).start_time.elapsed() > (*state).timeout {
        return 1;  // non-zero = interrupt execution
    }
    0
}, state_ptr);
```

Must be registered before any script executes.

**Performance note:** QuickJS calls the interrupt handler frequently — on every bytecode instruction boundary in some builds. The handler body must be as cheap as possible: a single elapsed time check. No allocations, no locks, no system calls inside the handler.

### 7.7 Event Loop — `src/event_loop/` *(Milestone 2)*

QuickJS supports promises but ships without a native event loop. The runtime implements a minimal loop that polls async tasks, runs pending promise jobs, and resolves completed futures.

**Deadlock risk with Worker Thread Pattern:**

If Option B (Worker Thread) is chosen for thread safety, the event loop **must** reside on the same worker thread as the runtime. If the host tries to synchronously await a script result from the main thread while the worker is blocked, the threads deadlock.

The fix: `QuarkRuntime` must either:
1. Run `run_pending_jobs()` internally as part of its own tick — the host never calls it directly
2. Return a thread-safe `Future` to the host that resolves when the job completes

```rust
// Safe pattern: host gets a Future, never blocks the worker
let result: Future<JsValue> = runtime.call_export_async("handleOrder", args)?;
```

Never expose a blocking `await_result()` that holds a lock across the channel boundary.

### 7.8 Runtime APIs — `src/api/`

Minimal built-in APIs implemented using host bindings:
- `console.log`
- `setTimeout` *(Milestone 2)*
- `setInterval` *(Milestone 2)*

---

## 8. Host-Facing Runtime API

```rust
let runtime = QuarkRuntime::new(QuarkConfig {
    memory_limit:      32 * 1024 * 1024,
    execution_timeout: Duration::from_millis(100),
    max_stack_size:    1 * 1024 * 1024,
    max_module_cache:  256,
});

runtime.register_function("log", log_fn)?;
runtime.register_function("applyDiscount", discount_fn)?;

runtime.eval_script("script.js")?;
runtime.call_export("onStart", &[])?;
```

Scripts are loaded once, exports are called repeatedly. The host controls the entire lifecycle.

---

## 9. Script Lifecycle Model

```
1. Runtime initialization     → QuarkRuntime::new(config)
2. Host API registration      → register_function(...)
3. Script loading             → eval_script("script.js")
4. Export invocation          → call_export("onStart", args)  [repeated]
5. Event loop processing      → handled internally per tick   [Milestone 2]
6. Runtime shutdown           → drop(runtime)
```

Scripts are **loaded once** and their exports are **called repeatedly**. A script is not re-parsed on every invocation. `run_pending_jobs()` is not exposed as a public host API — it is called internally on each tick.

---

## 10. Critical Implementation Details

> These four issues will require a full rewrite of `src/engine/` and `src/bindings/` if not addressed before writing code.

### 10.1 The Finalizer — Memory Safety for Host Objects

**Risk:** Without a finalizer, Rust objects exposed to JavaScript live forever on the heap. Missing a single `Drop` implementation means the host's RAM bleeds until the process is killed.

Rules:
- Every Rust object exposed to JS must be stored as `Box<T>` or `Arc<T>` in the `opaque` field of a QuickJS class object
- `src/bindings/object.rs` must implement a **JS Class Finalizer** for every exposed type
- `rquickjs` provides the `Class<T>` trait for this — study it before writing any bindings code

### 10.2 The Sync Gatekeeper — Thread Safety

**Risk:** QuickJS is strictly single-threaded. Calling the runtime from a different thread causes a segfault or memory corruption — intermittently, under load, and very painful to diagnose.

**Option A — Compiler-enforced single-thread:**

```rust
pub struct QuarkRuntime {
    _not_send: PhantomData<*mut ()>,  // makes !Send + !Sync
}
```

**Option B — Worker Thread Pattern:**
`QuarkRuntime` lives permanently on a dedicated thread. The host communicates via `mpsc` channels. Required for multi-threaded host applications. If choosing this, re-read Section 7.7 on the event loop deadlock risk before implementing.

### 10.3 Host-to-JS Error Mapping — Exception Strategy

**Risk:** If a Rust host function fails with no error strategy, scripts silently succeed with undefined behavior.

```rust
runtime.register_function("log", |args| {
    let msg = args.get_string(0)?;
    host_log(msg).map_err(JsError::from)?;
    Ok(JsValue::undefined())
});
```

Use `anyhow::Error` or a project-specific error enum throughout. **Never use `unwrap()` in the bindings layer.**

### 10.4 Panic Isolation — Preventing Host Crashes

**Risk:** A Rust panic inside a host binding unwinds through QuickJS C code. Unwinding across FFI is undefined behavior. One panicking plugin takes down the entire host.

```rust
runtime.register_function("log", |args| {
    let result = std::panic::catch_unwind(|| {
        // binding logic here
    });
    match result {
        Ok(val) => val,
        Err(_)  => Err(JsError::new("host function panicked")),
    }
});
```

**Rule:** All host bindings must be panic-safe. No exceptions.

---

## 11. Project Structure

```
quarkjs/
├── Cargo.toml                  ← workspace root
├── README.md
├── ARCHITECTURE.md
├── ROADMAP.md
├── LICENSE
│
└── crates/
    ├── quarkjs-core/           ← the library (lib.rs)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── runtime/
    │       │   ├── mod.rs
    │       │   ├── runtime.rs  ← Runtime Manager lives here
    │       │   └── config.rs
    │       ├── engine/
    │       │   ├── mod.rs
    │       │   └── quickjs_wrapper.rs
    │       ├── bindings/
    │       │   ├── mod.rs
    │       │   ├── function.rs
    │       │   └── object.rs      ← JS Class Finalizer lives here
    │       ├── modules/
    │       │   ├── mod.rs
    │       │   ├── loader.rs
    │       │   └── resolver.rs
    │       ├── sandbox/
    │       │   ├── mod.rs
    │       │   ├── limits.rs
    │       │   └── interrupt.rs   ← interrupt handler lives here
    │       ├── event_loop/        ← Milestone 2
    │       │   ├── mod.rs
    │       │   └── scheduler.rs
    │       ├── api/
    │       │   ├── mod.rs
    │       │   ├── console.rs
    │       │   └── timers.rs      ← Milestone 2
    │       └── utils/
    │           └── mod.rs
    │
    └── quarkjs-cli/            ← thin binary for testing
        ├── Cargo.toml
        └── src/
            └── main.rs
```

---

## 12. Configuration System

```rust
QuarkConfig {
    memory_limit:      32 MB,    // enforced by JS_SetMemoryLimit
    execution_timeout: 100 ms,   // enforced by interrupt handler
    max_stack_size:    1 MB,
    max_module_cache:  256,       // LRU eviction beyond this limit
}
```

Configuration is passed at runtime initialization. Scripts have no mechanism to modify or inspect their own limits.

---

## 13. Observability & Debugging

Even at MVP, the following must be tracked:

- **Execution time per script call** — duration of every `eval_script` and `call_export`
- **Script error logging** — all JS exceptions must surface with message and location
- **Stack traces** — QuickJS provides stack information on errors; capture and expose it
- **Host function call logging** — optional debug mode that logs every binding invocation

These are the responsibility of the engine wrapper and binding system to emit. Without them, debugging script failures in embedded contexts is nearly impossible.

---

## 14. Key Decisions & Rationale

**Use `rquickjs`, not raw FFI**
rquickjs is actively maintained and has already solved reference counting and finalizer patterns in Rust. Raw FFI from scratch would triple the time to Milestone 1 and introduce preventable bugs.

**Workspace over single binary crate**
`quarkjs-core` (lib) and `quarkjs-cli` (binary) are separate crates in a workspace. This correctly separates the embeddable API surface from test scaffolding.

**Model A context model for MVP**
One `JSRuntime` with one `JSContext`. The complexity explosion from multiple contexts too early makes the runtime unmaintainable before it is functional. Promote to Model B when a real use case requires it.

**No filesystem / network APIs**
The host application controls all I/O through registered bindings. Scripts can only access capabilities the host explicitly grants.

**Event loop deferred to Milestone 2**
A correct event loop requires working bindings and sandbox limits first. Building it second avoids designing the loop before understanding the constraints and deadlock risks.

**Panic isolation is mandatory from day one**
Panics crossing FFI into QuickJS C code are undefined behavior. This must be addressed in the first binding written, not added later.

**Handles over serialization**
Passing opaque Rust handles via `Class<T>` instead of serializing full data structures into the JS heap prevents memory limit violations and spurious interrupt handler triggers on large payloads.

**C API designed for from day one**
The three-layer model must be respected in the architecture from the start. A Rust-only design that retrofits C ABI later requires a full public API redesign.
