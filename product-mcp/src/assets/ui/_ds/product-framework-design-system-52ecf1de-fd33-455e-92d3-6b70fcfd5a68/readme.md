# The Product Framework — Design System

A design system for **Product / The Product Framework** — an open standard, and
the reference Rust CLI + MCP tooling, for specifying software as a verifiable
**What / How / Delivery** graph so engineers can build it (increasingly with
LLMs) in a *trustworthy, traceable* way. This system exists so design agents can
produce on-brand interfaces, diagrams, decks, and docs that communicate the
framework's abstract concepts clearly to a technical, subject-matter audience.

## What the product is

The Product Framework describes a software product as one connected,
machine-readable graph:

- **What** — the domain model (entities, relations, invariants, reference data)
  and the event model (commands, events, read-models/views, UI steps typed
  against *Abstract Interaction Objects*, systems, triggers, Deciders,
  Projectors). Owned by product & design.
- **How** — decisions, contracts, the screen-composition / reification model,
  repository layout, delivery slices. Owned by engineering.
- **Delivery** — features and releases as graph partitions; "done" is a
  *computed predicate*, not a judgement.

The spine of the brand is one chain — *reproducibility → measurement →
improvement* — and one daily loop, the **funnel: Intake → What → How → Build**,
where hard thinking moves upstream and behaviour is *simulated before any code*.

Two product surfaces ship today, and both are recreated as UI kits here:

1. **What Graph — Live View** (`product mcp --http`): a dark Event-Modeling
   swimlane timeline of the live What-graph.
2. **What → Preview — Generic AIO Renderer**: a light, schematic "blueprint"
   that renders a derived *render contract* with no design system coupled — proof
   the contract carries enough to build against.

## Sources used (explore these to go deeper)

This design system was reverse-engineered from two repositories. The reader is
encouraged to browse them to build richer, more accurate designs:

- **Framework / open standard** — https://github.com/Hafeok/product-framework
  - `README.md`, `GUIDE.md` (the Intake→What→How→Build loop), `docs/product-framework.md` (the full spec, §-numbered)
  - `preview/renderer.html` — **the blueprint renderer**, basis for the AIO Renderer UI kit
  - `preview/render-contract.schema.md`, `preview/build-seam/*` — contract + work-unit schemas
- **CLI + MCP tooling** — https://github.com/Hafeok/product-cli
  - `README.md`, `CLAUDE.md` — product description, architecture, vocabulary
  - `product-mcp/src/assets/view.html` — **the live What-graph view**, basis for the What Graph UI kit

No bundled brand assets (logo, fonts, icon set) exist in either repo; the logo,
typeface choice, and icon guidance below are deliberate, documented decisions.

---

## CONTENT FUNDAMENTALS — how this brand writes

The voice is that of a **precise specification, made human**. It is confident,
declarative, and economical; it explains *why* before *how*; it never hypes.

- **Tone:** rigorous but plain-spoken. Abstract ideas are stated, then made
  concrete with a single worked example (the recurring "checkout" / bookstore
  domain). Sentences carry weight: *"A failure reads both ways: the data is
  wrong, or the spec has gone stale."*
- **Person:** mostly **imperative second person** in guides ("Locate it on the
  graph", "Simulate the Decider before you write code") and **impersonal
  third person** in the spec ("the Decider derives its structure from the
  model"). First person is essentially absent.
- **Casing:** **sentence case** for prose and headings. **Defined terms are
  capitalised** as proper nouns — What, How, Delivery, Decider, Projector,
  Trigger, View, Command, Event, Abstract Interaction Object. UI/AIO identifiers
  and commands are **lowercase kebab/mono** — `ui-review-cart`, `cmd-begin-payment`,
  `product domain validate --strict`. Spec cross-refs are **§-numbered** (§3.3, §4.5).
- **Signature devices:** the em-dash aside; the "X, not Y" reframe ("shapes and
  rules, not any particular product"); the cheap-gate maxim ("a behaviour defect
  caught here costs a sentence"); cumulative ladders (Described → Realised →
  Verified → Delivered).
- **Emoji:** **none.** Never. Emphasis is carried by **bold**, `mono`, and
  tables/ladders. The occasional callout uses a `>` blockquote.
- **Vibe:** an exacting senior engineer who has thought the problem all the way
  down and wants you to catch the bug while it still costs a sentence.

> **Worked-example rule:** when illustrating a concept, reuse the canonical
> checkout/bookstore domain (`ev-item-added`, `rm-cart-summary`,
> `cmd-begin-payment`). Don't invent a fresh fictional product per example.

---

## VISUAL FOUNDATIONS — the motifs

Two coupled surfaces define the look. They share one palette and one typeface
and are intentionally different in mood.

- **The blueprint (light).** Drafting paper (`--paper #f3f7fb`), navy ink, a
  drafting-blue accent, **monospace as body text**. Reads like a technical
  drawing: 1.5px drawn rules, **dashed construction lines**, uppercase tracked
  labels, and a **hard, un-blurred offset shadow** (`--shadow-draft: 6px 6px 0`)
  — ink offset on paper, never a soft glow. This is the voice of *specification*.
- **The graph (dark).** A slate canvas (`--slate-900`), luminous semantic
  nodes, soft elevation shadows. This is the voice of the *live tool*.

**Colour.** The load-bearing system is the **Event-Modeling semantic palette** —
every construct has one fixed hue and it is never reassigned:
command = blue `#2563eb`, view/read-model = green `#16a34a`, event = amber
`#f59e0b`, trigger = violet `#7c3aed`, UI step = neutral dashed, bridge
(What→How link) = magenta `#db2777`. Phase colours (What blue / How amber /
Build green) come straight from the tool. Neutrals are a slate ramp; the light
surface adds a paper/ink/drafting-line set. Avoid: purple-blue gradients, emoji
cards, soft pastel UI — none of that is in this product.

**Type.** IBM Plex superfamily. **Plex Mono is the dominant voice** (labels,
identifiers, code, schematic specimens, eyebrows); Plex Sans carries UI and
display; Plex Serif is the long-form spec voice. The signature treatment is the
**uppercase, wide-tracked mono eyebrow** (`.t-label`, tracking `.12em`).
*(Substitution — see note below.)*

**Backgrounds.** Flat fills only — paper or slate. No photography, no
illustration, no gradient washes, no texture. Where structure is implied, it's
via lane tints (`rgba(...,.05–.07)`), 1px rules, and dashed dividers.

**Borders, radius, cards.** Thin (1px hair / 1.5px standard / 2px emphasis),
often **dashed** for placeholders, AIO slots, flow dividers, and What→How
bridges. Corners are **tight** (2–10px; default 5px — the graph node radius).
Cards come two ways: *elevation* (hairline + soft shadow) for app UI, *draft*
(ink border + hard offset shadow) for the blueprint.

**Shadows.** Two systems: **drafting** (hard offset, accent-wash, no blur) and
**elevation** (soft neutral). Never mix them on one surface.

**Motion.** Restrained and mechanical. Fades and brightness shifts (`filter:
brightness(1.08)` on node hover), a 0.8s linear spinner for loading, a dim/glow
for phase state. **Press = `translateY(1px)`**, never a scale-bounce. Easing is
`cubic-bezier(.2,0,0,1)`; durations 120–320ms. Cross-links fade in on hover.

**Hover / press states.** Hover brightens (graph) or darkens slightly
(`brightness(.96)`, buttons); selection is a **2px focus ring** offset from the
background; active press translates down 1px.

**Layout.** Dense and orderly: fixed left **gutter** (188px) for lane labels,
fixed **columns** (~210px) per Event-Modeling slice, horizontal scroll for wide
graphs. The blueprint reads at `--content-max 1180px` in a two-pane grid
(contract | surface). Transparency/blur is essentially unused — this brand
prefers crisp opaque rules over glassmorphism.

---

## ICONOGRAPHY

The source product **ships no icon font and no SVG icon set.** It communicates
with **bare Unicode glyphs** and **programmatically-drawn SVG primitives**:

- Glyphs in use: `→` (flow), `›` (phase separator), `▼` (funnel), `●`/`○`
  (selected / option), `↺` (reset/start-over), `×` (close), `⟨ ⟩` (missing
  content key). These are rendered in mono, in the accent colour.
- Diagram primitives: nodes are plain `rect`s (rx 5) with text; edges are
  `path`s; arrowheads are a single SVG `marker`. The graph *is* the iconography.
- The **brand mark** is geometric, not pictorial: a three-band **funnel**
  (What→How→Build) in the three phase colours. See `assets/logo-*.svg`.
- **Emoji are never used.**

**Substitution (flagged).** When a UI genuinely needs richer icons (a settings
gear, a search glyph), substitute **[Lucide](https://lucide.dev)** via CDN — its
1.5px stroke and geometric construction match the drafting aesthetic. Keep them
monochrome in `--text-muted` or `--accent`. This is a *documented substitution*,
not something present in the source. ⚠️

> ⚠️ **Typeface substitution.** The repos use the OS `system-ui` stack + a
> generic monospace. **IBM Plex** (Sans / Mono / Serif) is a deliberate brand
> choice loaded from Google Fonts — it keeps the mono-forward technical feel
> while giving the system a stable, ownable face. If you have a different
> licensed typeface in mind, swap it in `tokens/fonts.css` + `tokens/typography.css`.

---

## INDEX — what's in this folder

**Foundations**
- `styles.css` — the single entry point consumers link (import list only).
- `tokens/` — `fonts.css`, `colors.css`, `typography.css`, `spacing.css`, `effects.css`, `base.css`.
- `guidelines/cards/` — foundation specimen cards (Colors, Type, Spacing, Brand) shown in the Design System tab.
- `assets/` — `logo-mark.svg`, `logo-wordmark.svg`, `logo-wordmark-light.svg`.

**Components** (`window.ProductFrameworkDesignSystem_52ecf1.*`)
- `components/core/` — `Button`, `Tag`, `StatePill`, `Card`, `ConformanceBadge`
- `components/graph/` — `EMNode` (the signature Event-Modeling node), `PhaseStepper`

**UI kits** (full-screen recreations)
- `ui_kits/what-graph/` — the live Event-Modeling view (dark). `index.html` is interactive.
- `ui_kits/aio-renderer/` — the generic blueprint renderer (light). `index.html` is interactive.

**Other**
- `SKILL.md` — Agent-Skill front-matter so this system is usable from Claude Code.

> Build something? Start from the matching UI kit, lift the components, and obey
> the two rules that carry this brand: **never recolour an Event-Modeling
> construct**, and **never reach for emoji**.
