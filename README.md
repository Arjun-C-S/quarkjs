# QuarkJS

> *A quark is the smallest fundamental building block. QuarkJS aims to be a minimal, embeddable JavaScript runtime — nothing unnecessary, everything load-bearing.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status: Pre-Alpha](https://img.shields.io/badge/Status-Pre--Alpha-orange.svg)]()
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

**QuarkJS is a lightweight embedded JavaScript runtime written in Rust, powered by the [QuickJS](https://bellard.org/quickjs/) engine.**

It fills a specific gap: deterministic, safe scripting for applications that need user-programmable behavior — without the complexity of Node.js or Deno. If your application needs to run user-supplied scripts safely, expose a plugin API, or allow custom business logic, QuarkJS is designed for that.

> **Current status: Pre-Alpha. No code exists yet.** The repository contains the architecture specification, roadmap, and project structure. Active development begins at Milestone 1.

---

## What it looks like

**Host side (Rust):**

```rust
let runtime = QuarkRuntime::new(QuarkConfig {
    memory_limit:      32 * 1024 * 1024,
    execution_timeout: Duration::from_millis(100),
    max_stack_size:    1 * 1024 * 1024,
    max_module_cache:  256,
});

runtime.register_function("log", log_fn)?;
runtime.eval_script("plugin.js")?;
runtime.call_export("onStart", &[])?;
```

**Script side (JavaScript):**

```js
import { log } from "host"

export function onStart() {
  log("QuarkJS working")
}
```

---

## Use cases

**Plugin Systems** — Expose extension APIs to user scripts.

**Automation Engines** — Let users define dynamic business logic without modifying application code.

**SaaS Customization** — Allow tenants to customize pricing, routing, or validation logic without backend deploys.

**IoT Device Behavior** — Control hardware behavior through lightweight scripts on constrained hosts.

---

## Design goals

| Goal | Description |
|---|---|
| **Lightweight** | Small footprint suitable for embedding inside applications and services |
| **Safe Execution** | Scripts run inside a sandbox with strict memory and CPU limits |
| **Easy Embedding** | Host applications expose functions and run scripts with minimal integration effort |
| **Deterministic** | Scripts cannot block or crash the host application |

---

## Non-goals

QuarkJS explicitly will not provide:

- Node.js compatibility
- npm ecosystem support
- High-performance JIT execution
- Direct filesystem access
- Direct networking APIs

Each of these would roughly double the project's complexity and introduce significant security surface area. QuickJS already solved the engine problem. The value of QuarkJS is the safe, clean bridge between Rust and JavaScript.

---

## Sandbox disclaimer

> ⚠️ **QuarkJS is a cooperative sandbox — not a security boundary.**

The sandbox prevents accidental crashes and runaway execution. It does not protect against malicious scripts. For multi-tenant products where scripts come from untrusted users, wrap the runtime in process-level isolation. For the intended use cases — plugin systems, automation engines, SaaS customization, IoT — the sandbox is appropriate and effective.

---

## Architecture overview

QuarkJS is three stacked systems:

```
┌─────────────────────────────────────┐
│  Layer 1 — Language Bindings        │
│  (C ABI, Rust API)                  │
└──────────────────┬──────────────────┘
                   │
┌──────────────────▼──────────────────┐
│  Layer 2 — Runtime                  │
│  (loader, event loop, APIs,         │
│   host bindings, sandbox)           │
└──────────────────┬──────────────────┘
                   │
┌──────────────────▼──────────────────┐
│  Layer 3 — Engine                   │
│  (QuickJS)                          │
└─────────────────────────────────────┘
```

The event loop is a **scheduler** owned by the Runtime Manager — not a pipeline stage scripts flow through. This distinction matters for correct deadlock analysis and lifecycle design.

For the full technical breakdown see [ARCHITECTURE.md](ARCHITECTURE.md).
For the build plan see [ROADMAP.md](ROADMAP.md).

---

## Project structure

```
quarkjs/
├── Cargo.toml                  ← workspace root
└── crates/
    ├── quarkjs-core/           ← the library
    └── quarkjs-cli/            ← thin binary for testing
```

---

## License

MIT — see [LICENSE](LICENSE).
