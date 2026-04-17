# Guardian Council v0.5 — The AmniShunt Vision
## "WebKit-Servo Amniscient Shunt & Beyond"

---

## The Proposal
Amni-Browse v0.4.0 achieves dual-backend (WebView + Servo-egui). The next leap:
1. Build a custom WebKit-Servo translation/shunt layer ("AmniShunt")
2. Use septidecimal (base-17) encoding for internal representations
3. Box the shunt for sandboxed safety
4. Eliminate all WebView/legacy dependencies
5. Integrate Amni-Ai capabilities
6. Progressively replace Servo internals with custom advanced implementations

---

## Guardian Assessments

### Tidus — Combat Pragmatist
**Stance: Start with what hits hardest NOW**

Priority focus: The rendering pipeline gap. Right now our Servo backend shows egui chrome + a "Loading..." placeholder. No actual web content renders.

Proposals:
- [P1] Build a minimal HTML parser in Rust (dom_parser.rs) — just enough for basic page structure
- [P2] CSS selector engine (style_resolver.rs) — property cascade, specificity, inheritance
- [P3] Layout engine (layout_engine.rs) — box model, flexbox basics, text flow
- [P4] Paint layer (paint.rs) — take layout tree → wgpu draw commands
- [P5] Network stack (net_stack.rs) — HTTP/2, TLS 1.3, connection pooling
- [P6] Each chunk is testable and shippable independently

Risk: "This is the hardest part of a browser. But Servo already did the hard work — we strip their crates, adapt, and accelerate."

### Lulu — Arcane Architect
**Stance: The shunt must be architecturally clean**

The AmniShunt is fundamentally a **bytecode interpreter layer** that sits between:
- Input: WebKit-style DOM/CSSOM APIs OR Servo's style/layout crates
- Output: Amni's own normalized intermediate representation (IR)

Proposals:
- [L1] Define AmniIR — a compact bytecode/AST for rendering instructions
  - Septidecimal-encoded opcodes (base-17 reduces collision space, harder to reverse-engineer)
  - Operations: CREATE_NODE, SET_ATTR, LAYOUT_BOX, PAINT_RECT, TEXT_RUN, etc.
- [L2] AmniShunt translator — two input adapters:
  - ServoAdapter: Converts Servo's layout/style trees → AmniIR
  - WebKitAdapter: Converts WebKit-compatible DOM events → AmniIR (future)
- [L3] BoxedRuntime — the shunt runs in a sandboxed execution context:
  - Memory isolation (own allocator, no shared heap with host)
  - Capability-based security (no network, no FS unless explicitly granted)
  - Fault isolation (crash in shunt doesn't crash browser)
- [L4] GPU-accelerated AmniIR executor — reads IR, emits wgpu commands directly
- [L5] Septidecimal encoding layer (base17.rs):
  - Internal wire format for IR opcodes + data
  - Makes binary dumps incomprehensible to standard hex editors
  - Additional degree of freedom in addressing (17 symbols vs 16)

Risk: "This is effectively building a mini VM. It's powerful but must be scoped carefully."

### Auron — Veteran Strategist
**Stance: Phased extraction, never break what works**

Proposals:
- [A1] Phase 1 (v0.5): AmniShunt skeleton + base17 codec + HTML tokenizer
  - Minimal viable: parse HTML → token stream → basic DOM tree
  - Tokenizer can run in shunt sandbox
  - Use Taichi/GPU for parallel tokenization of large documents
- [A2] Phase 2 (v0.6): CSS engine + basic layout
  - Cascade + specificity resolver (steal-and-rewrite from Servo's style crate)
  - Block/inline/flex layout
  - Text shaping (use font crate, GPU-accelerated glyph rasterization)
- [A3] Phase 3 (v0.7): Paint + compositing
  - Layer tree construction
  - wgpu render pipelines for each layer type
  - Compositor with GPU tiling
- [A4] Phase 4 (v0.8): JavaScript engine integration
  - Option A: Embed V8 via rusty_v8 (proven, massive)
  - Option B: Embed SpiderMonkey (Servo's approach)
  - Option C: Build minimal ES interpreter (heroic but years of work)
  - Recommendation: rusty_v8 initially, custom ES engine long-term
- [A5] Phase 5 (v1.0): Full independence
  - Remove all legacy Servo/WebView code paths
  - AmniShunt is the sole rendering pipeline
  - Amni-Ai integration for smart features

Risk: "Each phase ships a working browser. Never orphan functionality between releases."

### Wakka — Team Consensus Builder
**Stance: What do the users actually GET at each milestone?**

User-visible milestones:
- v0.5: Can render basic HTML pages (headings, paragraphs, links, images, lists)
- v0.6: CSS styled pages look reasonable, text wraps properly
- v0.7: Modern websites render (flexbox, images composite, scroll works)
- v0.8: JavaScript-powered sites work (SPAs, forms, AJAX)
- v0.9: Performance competitive with mainstream browsers
- v1.0: Amni-Ai features (smart summarization, privacy analysis, auto-fill intelligence)

Proposals:
- [W1] Build a benchmark suite from day 1 — Acid3, Speedometer, MotionMark
- [W2] Compatibility tracker — test top 100 websites each release
- [W3] User-facing changelog with visual diff screenshots

### Kimahri — Silent Guardian (Security)
**Stance: ...**

*Steps forward, draws blade, points at architecture diagram*

Proposals:
- [K1] The shunt box MUST be process-isolated (separate OS process, IPC only)
- [K2] Septidecimal + AEAD encryption on IPC between shunt and host
- [K3] Content Security Policy enforcement at the IR level, not DOM level
- [K4] Memory-safe IR executor — no raw pointers, stack-based VM
- [K5] Fuzzing harness for every parser input surface (HTML, CSS, JS, URL, headers)
- [K6] Sandboxed network — shunt cannot directly open sockets, must request through host
- [K7] Per-origin isolation — each origin gets its own shunt instance

*Nods. Sits back down.*

### Yuna — Summoner (Vision & Integration)
**Stance: This is where Amni-Ai transforms browsing**

The AmniShunt + Amni-Ai integration opportunities:
- [Y1] Smart Reader: AI extracts article content better than DOM heuristics
- [Y2] Privacy Sentinel: AI analyzes page scripts in real-time, flags trackers the blocklist missed
- [Y3] Smart Search: AI-powered address bar predictions, semantic search across history/bookmarks
- [Y4] Page Summarizer: One-click AI summary of any webpage
- [Y5] Code Assistant: Built-in AI for code pages (GitHub, Stack Overflow)
- [Y6] Translation: Real-time page translation without external APIs
- [Y7] Accessibility AI: Auto-generate alt text, improve contrast, screen reader optimization
- [Y8] Threat Detection: AI analysis of phishing pages, malware download warnings

---

## Majority Ruling: v0.5 Scope

Votes tallied across all guardians:

| Proposal | Votes | Decision |
|----------|-------|----------|
| AmniShunt skeleton (translator framework) | 6/6 | **YES** |
| base17.rs (septidecimal codec) | 5/6 | **YES** |
| HTML tokenizer (in-shunt) | 6/6 | **YES** |
| BoxedRuntime (sandbox foundation) | 5/6 | **YES** |
| AmniIR definition | 6/6 | **YES** |
| Basic DOM tree construction | 5/6 | **YES** |
| CSS engine | 2/6 | DEFER to v0.6 |
| Layout engine | 1/6 | DEFER to v0.6 |
| JavaScript engine | 0/6 | DEFER to v0.8 |
| Process isolation | 4/6 | **YES (foundation)** |
| Amni-Ai hooks | 3/6 | DEFER to v0.9+ |
| GPU-accelerated tokenizer | 4/6 | **YES (prototype)** |
| Benchmark suite | 5/6 | **YES** |

---

## v0.5 Architecture: AmniShunt

```
┌─────────────────────────────────────────────────────────────────┐
│                    AMNI-BROWSE HOST PROCESS                     │
│ ┌─────────┐  ┌───────────┐  ┌──────────────┐  ┌─────────────┐ │
│ │ chrome.rs│  │ app.rs    │  │ net_stack.rs │  │ ipc_shunt.rs│ │
│ │ (egui UI)│  │ (state)   │  │ (HTTP/TLS)  │  │ (host↔shunt)│ │
│ └────┬─────┘  └─────┬─────┘  └──────┬───────┘  └──────┬──────┘ │
│      │              │               │                  │        │
│      └──────────────┴───────────────┴──────────────────┘        │
│                             │ AmniIR                            │
│                             ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              AMNISHUNT SANDBOX (boxed process)           │   │
│  │  ┌────────────┐  ┌──────────┐  ┌─────────────────────┐  │   │
│  │  │ tokenizer  │  │ dom_tree │  │ ir_emitter (base17) │  │   │
│  │  │ (HTML→tok) │→ │ (tokens  │→ │ (DOM→AmniIR ops)    │  │   │
│  │  │ GPU-accel  │  │  →tree)  │  │                     │  │   │
│  │  └────────────┘  └──────────┘  └──────────┬──────────┘  │   │
│  │                                           │              │   │
│  │  Memory: isolated allocator               │              │   │
│  │  Capabilities: {parse} only               │              │   │
│  └───────────────────────────────────────────┘              │   │
│                             │ AmniIR (base17 encoded)       │   │
│                             ▼                               │   │
│  ┌──────────────────────────────────────────────────────┐   │   │
│  │            GPU RENDER PIPELINE (wgpu)                │   │   │
│  │  AmniIR → paint commands → render passes → present   │   │   │
│  └──────────────────────────────────────────────────────┘   │   │
└─────────────────────────────────────────────────────────────────┘
```

## New Files for v0.5

| File | Purpose | LOC Est |
|------|---------|---------|
| base17.rs | Septidecimal codec (encode/decode/stream) | ~150 |
| amni_ir.rs | IR opcode definitions + serialization | ~250 |
| tokenizer.rs | HTML5 tokenizer (spec-compliant) | ~800 |
| dom_tree.rs | Minimal DOM tree (Node, Element, Text, Attr) | ~400 |
| ir_emitter.rs | DOM tree → AmniIR instruction stream | ~300 |
| shunt_sandbox.rs | Process isolation + capability model | ~350 |
| ipc_shunt.rs | Host ↔ Shunt IPC (AmniIR transport) | ~200 |
| net_stack.rs | HTTP client (replaces reqwest for pages) | ~500 |
| paint_ir.rs | AmniIR → wgpu draw commands | ~400 |
| bench/ | Benchmark harness (Acid tests, perf) | ~200 |

Estimated: ~3,550 new lines → total ~9,500 LOC

## Septidecimal (Base-17) Encoding

Digits: 0123456789ABCDEFG (0-16)

Why base-17:
- Odd prime base — no clean bit-alignment, making binary analysis harder
- 17^2 = 289 values per 2-digit pair (vs 256 for hex)
- Extra degree of freedom for encoding metadata in the "surplus" space
- Wire format for AmniIR opcodes, addresses, and data payloads
- Hackers expecting hex/base64 dumps will find noise

Encoding scheme:
- Opcodes: 2 septidecimal digits (0-288, using 289 values → 256 opcodes + 33 metadata slots)
- Addresses: 4 digits (0-83,520 node address space)
- Data: Variable-length with length prefix

## Breaking Changes from v0.4.0
- None — v0.5 is additive. Dual backend preserved.
- New `amni-shunt` feature flag for the shunt pipeline
- Default remains `webview` for stability

---

*"We're building something nobody else has, ya know? Not Ladybird, not Servo, not Chrome. Our own thing!"* — Wakka

*"The septidecimal encoding alone will make security researchers do a double-take."* — Lulu

*"One phase at a time. We ship working software, not promises."* — Auron
