# QuarkJS — Roadmap

This document tracks the build plan for QuarkJS. Each milestone has a clear goal, a defined set of deliverables, and an explicit success criterion. A milestone is not complete until its criterion is met — partial implementation does not count.

---

## Current Status

**Pre-Alpha — No code yet.**

The architecture is finalized. The repository structure is in place. Active development starts at Milestone 1.

---

## Milestone 1 — MVP

**Goal:** Validate the core architecture and prove the safety boundaries hold.

A runtime that passes all five criterion tests below is architecturally correct. One that doesn't has a hole somewhere, regardless of how much else works.

### Deliverables

- [ ] Cargo workspace with `quarkjs-core` and `quarkjs-cli` crates
- [ ] QuickJS engine wrapper via `rquickjs`
- [ ] `QuarkRuntime::new(config)` initialization
- [ ] `register_function()` host binding
- [ ] `eval_script()` script execution
- [ ] `call_export()` export invocation
- [ ] `console.log` implementation
- [ ] Basic ESM module imports
- [ ] Root-jailed module resolver with `canonicalize()` verification
- [ ] Panic isolation via `catch_unwind` on all bindings
- [ ] `Result`-based error mapping to JS exceptions
- [ ] Execution timeout via interrupt handler
- [ ] Handle ownership model decided and implemented (`Arc<T>` or ID registry)

### Success Criterion — all five must pass

**1. Basic execution**
```js
import { log } from "host"
log("QuarkJS working")
// Expected: prints without error
```

**2. Panic safety**
```js
triggerPanic()
// Expected: JS receives an exception, host process stays alive
```

**3. Error mapping**
```js
try {
  failingHostFn()
} catch (e) {
  log(e.message)
  // Expected: Rust error message appears here
}
```

**4. Resolver jail**
```js
import { secret } from "../../etc/passwd"
// Expected: throws ResolveError before any file access
```

**5. Timeout**
```js
while (true) {}
// Expected: terminated after execution_timeout, host process survives
```

If all five pass, your sandbox is real. If any fail, your architecture has a hole.

---

## Milestone 2 — Sandbox + Async

**Goal:** Make the runtime safe enough for production embedding.

### Deliverables

- [ ] Memory limits enforced via `JS_SetMemoryLimit`
- [ ] Execution timeout hardened
- [ ] Event loop — minimal promise job scheduler (peer component under Runtime Manager, not pipeline stage)
- [ ] `setTimeout` implementation
- [ ] `setInterval` implementation
- [ ] Module caching with LRU eviction
- [ ] Bytecode compilation and caching (`compile_script` / `run_compiled` via `JS_WriteObject` / `JS_ReadObject`)
- [ ] Worker Thread pattern with Future-based host API (required if host uses Tokio or async Rust)
- [ ] Deadlock prevention — `run_pending_jobs()` internal to runtime tick, never exposed publicly

### Success Criterion

- A script using `setTimeout` resolves correctly without blocking the host
- A script allocating beyond `memory_limit` is terminated cleanly with a catchable JS error
- Module imports are served from cache on second load
- The host can call `call_export_async()` and await a `Future` without deadlocking
- A compiled bytecode file loads and executes faster than parsing the source equivalent

---

## Milestone 3 — Polish

**Goal:** Make QuarkJS a proper embeddable library usable from any language.

### Deliverables

- [ ] Async host functions
- [ ] Improved module resolver (configurable resolver interface)
- [ ] C API — stable ABI for embedding from C, C++, Python, Go, Zig
- [ ] Multiple context support (Model B) — one runtime, isolated plugin contexts
- [ ] `crates.io` publish of `quarkjs-core`
- [ ] Full API documentation (`rustdoc`)
- [ ] Usage guide and examples
- [ ] CI pipeline (GitHub Actions)

### Success Criterion

- A C program can embed QuarkJS using only the C API with no Rust toolchain installed
- A plugin system with multiple isolated script contexts runs without global scope leakage between plugins
- `cargo doc` produces complete, accurate API documentation

---

## Future / Unscheduled

These are not on the active roadmap. Recorded here so they are not forgotten and not accidentally pulled into an earlier milestone.

| Item | Reason deferred |
|---|---|
| Multiple runtime instances (Model C) | High memory cost; real use case not yet established |
| WASM compilation target | Significant additional complexity |
| Script hot-reloading | Requires context lifecycle work from Milestone 3 first |
| Metrics / tracing integration | Nice to have after the core is stable |
| npm-compatible module resolution | Explicitly a non-goal; reconsider only with compelling use case |
| CommonJS module support | ESM only is the correct scope for an embedded runtime |

---

## Design Constraints That Will Not Change

These decisions are final. They are not open for reconsideration at any milestone.

- **No Node.js compatibility** — not a goal at any milestone
- **No direct filesystem access from scripts** — host controls all I/O via bindings
- **No direct networking from scripts** — same reason
- **No JIT** — QuickJS does not provide one; adding a different engine is a different project
- **ESM only** — CommonJS and native C modules are out of scope
- **Panic isolation is mandatory** — `catch_unwind` on every binding, always, from Milestone 1 onward
- **Handles over serialization** — large data structures must be passed as opaque handles, not cloned into the JS heap
- **Cooperative sandbox only** — QuarkJS does not provide security-boundary isolation; process-level sandboxing is the embedder's responsibility for untrusted code
- **Event loop is a scheduler** — it is never a pipeline stage; this architectural distinction is permanent
