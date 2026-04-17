# AmniShunt Technical Design — v0.5
## The WebKit-Servo Amniscient Rendering Shunt

---

## 1. Base-17 (Septidecimal) Codec

### Encoding Table
```
Value:  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16
Digit:  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F  G
```

### Wire Format
All AmniIR data uses septidecimal on the wire between shunt and host:
- **Byte encoding**: Each byte (0-255) → 2 sept digits + 1 metadata flag
  - e.g., 0xFF (255) → "F0" with overflow bit = 1
  - Surplus space (256-288) encodes 33 metadata tags (frame markers, checksums, etc.)
- **Varint**: Length-prefixed, base-17 digit stream
- **Checksums**: Running base-17 digit sum modulo 17 (single-digit check)

### Performance
- Encode: ~2 sept digits per byte (expansion factor 1.21x vs raw, 0.97x vs hex)
- GPU batch encoding: process 1M bytes → sept digits in parallel via Taichi/compute shader
- Decode: lookup table, O(1) per digit

---

## 2. AmniIR Opcode Set

### Category 0x0_: Document Structure (opcodes 00-0G)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 00 | DOC_START | width, height | Begin document |
| 01 | DOC_END | — | End document |
| 02 | NODE_CREATE | node_id, tag_hash | Create DOM node |
| 03 | NODE_DESTROY | node_id | Remove node |
| 04 | NODE_APPEND | parent_id, child_id | Append child |
| 05 | NODE_INSERT | parent_id, child_id, index | Insert at position |
| 06 | NODE_REMOVE | parent_id, child_id | Remove child |
| 07 | ATTR_SET | node_id, key_hash, value_ptr | Set attribute |
| 08 | ATTR_DEL | node_id, key_hash | Delete attribute |
| 09 | TEXT_SET | node_id, text_ptr | Set text content |
| 0A | TEXT_APPEND | node_id, text_ptr | Append text |

### Category 1x: Layout Instructions (10-1G)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 10 | BOX_CREATE | node_id, x, y, w, h | Create layout box |
| 11 | BOX_UPDATE | node_id, x, y, w, h | Update position/size |
| 12 | BOX_MARGIN | node_id, t, r, b, l | Set margins |
| 13 | BOX_PADDING | node_id, t, r, b, l | Set padding |
| 14 | BOX_BORDER | node_id, t, r, b, l | Set border widths |
| 15 | BOX_DISPLAY | node_id, mode | block/inline/flex/grid/none |
| 16 | BOX_OVERFLOW | node_id, mode | visible/hidden/scroll/auto |
| 17 | FLEX_CONFIG | node_id, dir, wrap, justify, align | Flex container props |
| 18 | TEXT_LAYOUT | node_id, font_id, size, line_h | Text layout params |
| 19 | TEXT_RUN | node_id, glyph_ptr, count | Positioned glyph run |

### Category 2x: Paint Instructions (20-2G)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 20 | PAINT_RECT | x, y, w, h, color | Fill rectangle |
| 21 | PAINT_BORDER | x, y, w, h, widths, colors, radii | Draw border |
| 22 | PAINT_TEXT | x, y, glyph_ptr, count, color | Render glyphs |
| 23 | PAINT_IMAGE | x, y, w, h, texture_id | Draw image |
| 24 | PAINT_SHADOW | x, y, w, h, blur, spread, color | Box shadow |
| 25 | PAINT_CLIP | x, y, w, h | Set clip rectangle |
| 26 | PAINT_UNCLIP | — | Restore clip |
| 27 | PAINT_OPACITY | value | Set layer opacity |
| 28 | PAINT_TRANSFORM | matrix_ptr | Set transform matrix |
| 29 | PAINT_GRADIENT | x, y, w, h, type, stops_ptr | Linear/radial gradient |
| 2A | LAYER_PUSH | layer_id | Push compositing layer |
| 2B | LAYER_POP | — | Pop compositing layer |

### Category 3x: Event & Control (30-3G)
| Opcode | Mnemonic | Operands | Description |
|--------|----------|----------|-------------|
| 30 | SCROLL_TO | x, y | Scroll viewport |
| 31 | SCROLL_BY | dx, dy | Scroll relative |
| 32 | NAVIGATE | url_ptr | Navigate to URL |
| 33 | RESOURCE_REQ | url_ptr, type | Request resource (img, css, js, font) |
| 34 | RESOURCE_RESP | req_id, data_ptr, len | Resource response |
| 35 | FRAME_BEGIN | frame_num, timestamp | Begin render frame |
| 36 | FRAME_END | — | End render frame |
| 37 | HIT_TEST | x, y | Find node at coordinates |
| 38 | HIT_RESULT | node_id, x, y | Hit test response |

### Metadata Slots (surplus 256-288, encoded as special 2-digit sept pairs)
| Slot | Purpose |
|------|---------|
| 256 | STREAM_START marker |
| 257 | STREAM_END marker |
| 258 | CHECKSUM (next digit is base-17 sum) |
| 259 | HEARTBEAT (shunt alive ping) |
| 260 | ERROR (followed by error code) |
| 261-288 | Reserved for future use |

---

## 3. HTML Tokenizer Architecture

### States (HTML5 spec subset for v0.5)
- Data, TagOpen, EndTagOpen, TagName, BeforeAttrName
- AttrName, BeforeAttrValue, AttrValueDoubleQuoted, AttrValueSingleQuoted
- AttrValueUnquoted, SelfClosingStartTag, BogusComment
- MarkupDeclarationOpen, CommentStart, Comment, CommentEnd
- DOCTYPE, CDATASection

### Token Types
```
StartTag { name, attrs: Vec<(String, String)>, self_closing: bool }
EndTag { name }
Character { data: char }
Comment { data: String }
DOCTYPE { name, public_id, system_id }
EOF
```

### GPU Acceleration Strategy
- Chunk input into 4KB blocks
- Parallel scan for '<' '>' '/' '=' '"' boundaries
- Build boundary index on GPU
- Sequential state machine on CPU uses boundary index for O(1) skips
- Hybrid approach: ~3-5x speedup on large documents

---

## 4. DOM Tree (Minimal v0.5)

### Node Types
```
Document { children }
Element { tag, attrs, children, namespace }
Text { data }
Comment { data }
DocumentType { name, public_id, system_id }
```

### Node Storage
- Arena allocator (typed_arena or custom slab)
- Node IDs are 32-bit indices into arena
- Parent/child/sibling pointers via IDs (no Rc/RefCell)
- O(1) node access, cache-friendly layout

### Supported Tags (v0.5 minimum)
html, head, title, meta, link, style, body, div, span, p, a, img,
h1-h6, ul, ol, li, table, tr, td, th, form, input, button, textarea,
select, option, br, hr, pre, code, blockquote, strong, em, b, i,
section, article, nav, header, footer, main, aside, figure, figcaption

---

## 5. Shunt Sandbox Model

### Process Isolation
```
┌──────────────────────┐    IPC (pipes)    ┌──────────────────────┐
│    HOST PROCESS      │◄─────────────────►│   SHUNT PROCESS      │
│                      │  base17-encoded   │                      │
│  Browser chrome      │  AmniIR stream    │  HTML tokenizer      │
│  Network stack       │                   │  DOM builder         │
│  App state           │  Resource req/    │  IR emitter          │
│  GPU compositor      │  resp channel     │                      │
│                      │                   │  No network access   │
│                      │                   │  No filesystem       │
│                      │                   │  No syscalls (WASI)  │
└──────────────────────┘                   └──────────────────────┘
```

### Capability Model
- PARSE: Can process HTML/CSS input → tokens → DOM → IR
- EMIT: Can output AmniIR instructions
- ALLOC: Can allocate within sandbox memory budget (default 256MB)
- No other capabilities by default

### Fault Tolerance
- Shunt crash → host detects broken pipe → show "page crashed" message → restart shunt
- Memory limit exceeded → OOM signal → graceful degradation
- Infinite loop → watchdog timer (10s) → kill + restart

---

## 6. Implementation Order

### Sprint 1: Foundation
1. base17.rs — encode/decode/stream with tests
2. amni_ir.rs — opcode enum, serialization, base17 wire format
3. Cargo.toml — add `amni-shunt` feature flag

### Sprint 2: Parser
4. tokenizer.rs — HTML5 tokenizer (state machine)
5. dom_tree.rs — arena-based DOM tree
6. Tests: Parse 10 real HTML pages, verify DOM structure

### Sprint 3: IR Pipeline
7. ir_emitter.rs — DOM → AmniIR instruction stream
8. paint_ir.rs — AmniIR → wgpu draw commands (basic: rects, text, images)
9. Integration: tokenize → DOM → IR → paint → screen

### Sprint 4: Sandbox
10. shunt_sandbox.rs — process isolation framework
11. ipc_shunt.rs — host↔shunt IPC with base17 transport
12. Wire it all together: HTML arrives at host → sent to shunt → IR returns → paint

### Sprint 5: Network & Polish
13. net_stack.rs — async HTTP client for page fetching
14. Replace content placeholder in servo_backend.rs with real paint output
15. Benchmark suite, compatibility tests

---

## 7. Ladybird Competitive Analysis

| Aspect | Ladybird | Amni-Browse |
|--------|----------|-------------|
| Language | C++ → Rust migration (2026) | Pure Rust from day 1 |
| Engine | LibWeb (custom) | AmniShunt (custom, from Servo base) |
| JS Engine | LibJS (custom) | TBD (V8/SpiderMonkey/custom) |
| Rendering | Custom paint | wgpu + GPU compositing |
| Privacy | Standard | Zero-telemetry, built-in ad block, DoH |
| Security | Standard sandboxing | Septidecimal IR + process isolation + capability model |
| AI | None | Amni-Ai integration planned |
| Platforms | Linux/macOS/Windows | Same + mobile planned |

Key differentiators:
- **Septidecimal IR** — unique wire format invisible to standard tools
- **GPU-first architecture** — Taichi/wgpu for parsing and rendering
- **AI-native** — Amni-Ai for smart features
- **Privacy-first DNA** — not bolted on, built in

---

*"Take what we can from Servo, burn the rest, start fresh — that's the plan, and drec ec ruf fa tu ed!"* — Rikku
