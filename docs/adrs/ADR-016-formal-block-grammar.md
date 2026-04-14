---
id: ADR-016
title: Formal Block Grammar
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:a9fa10429010ef3c48bed80f8e4f63c0b92e6ae9d91e794c206dadb83e9bbb53
---

**Status:** Accepted

**Context:** ADR-011 defines the AISP-influenced formal block notation for test criteria files. It specifies the block types and symbol subset but defers the question of how blocks are parsed. Without a defined grammar, two implementations of the parser may accept different inputs, the error messages for malformed blocks will be inconsistent, and the Rust type model for parsed formal blocks will be ambiguous.

This ADR defines the grammar, the Rust type model, and the error behaviour for malformed blocks.

**Decision:** Formal blocks are parsed as structured text using a hand-written recursive descent parser over the minimal symbol subset defined in ADR-011. The parser produces a typed AST. Blocks that fail to parse are reported as E001 parse errors with line-level precision. Blocks that are syntactically valid but semantically meaningless (e.g., an empty `⟦Γ:Invariants⟧{}` block) produce W004 warnings.

---

### Grammar (informal BNF)

```
formal-section   ::= block*
block            ::= "⟦" block-type "⟧" "{" block-body "}"
                   | evidence-block
block-type       ::= "Σ:Types" | "Γ:Invariants" | "Λ:Scenario"
                   | "Λ:ExitCriteria" | "Λ:Benchmark"
block-body       ::= statement ( "\n" statement )*
statement        ::= type-def | invariant | scenario-field | exit-field | benchmark-field

benchmark-field  ::= "baseline" "≜" "condition" "(" ident ")"
                   | "target"   "≜" "condition" "(" ident ")"
                   | "scorer"   "≜" "rubric_llm" "(" scorer-params ")"
                   | "pass"     "≜" expr
scorer-params    ::= ident ":" literal ("," ident ":" literal)*

type-def         ::= ident "≜" type-expr
type-expr        ::= ident | union-type | tuple-type | list-type | func-type
union-type       ::= type-expr "|" type-expr
tuple-type       ::= "⟨" type-expr ("," type-expr)* "⟩"
list-type        ::= type-expr "+"       (* one or more *)
                   | type-expr "*"       (* zero or more *)
func-type        ::= type-expr "→" type-expr

invariant        ::= quantifier | comparison
quantifier       ::= ("∀" | "∃") binding ":" expr
binding          ::= ident | ident "∈" ident
expr             ::= ident | literal | func-call | infix | set-expr
infix            ::= expr ("=" | "≠" | "<" | ">" | "≤" | "≥" | "∧" | "∨") expr
set-expr         ::= "|" "{" expr "|" expr "}" "|"   (* set cardinality *)
func-call        ::= ident "(" expr ("," expr)* ")"
comparison       ::= expr ("=" | "≠" | "<" | ">") expr

scenario-field   ::= ("given" | "when" | "then") "≜" expr
exit-field       ::= ident comparison

evidence-block   ::= "⟦Ε⟧" "⟨" evidence-fields "⟩"
evidence-fields  ::= evidence-field (";" evidence-field)*
evidence-field   ::= "δ≜" float | "φ≜" integer | "τ≜" stability
stability        ::= "◊⁺" | "◊⁻" | "◊?"

ident            ::= [A-Za-z_][A-Za-z0-9_]*
literal          ::= integer | float | string | duration
integer          ::= [0-9]+
float            ::= [0-9]+ "." [0-9]+
string           ::= '"' [^"]* '"'
duration         ::= integer ("s" | "ms" | "min" | "h")
```

The grammar is intentionally permissive on `expr` — the goal is structural validation and AST construction, not full formal verification. An expression that parses but cannot be evaluated is not an error; it is simply stored as a string in the AST leaf.

---

### Rust Type Model

```rust
pub enum FormalBlock {
    Types(Vec<TypeDef>),
    Invariants(Vec<Invariant>),
    Scenario(ScenarioBlock),
    ExitCriteria(Vec<ExitField>),
    Benchmark(BenchmarkBlock),
    Evidence(EvidenceBlock),
}

pub struct TypeDef {
    pub name: String,
    pub expr: TypeExpr,
}

pub enum TypeExpr {
    Named(String),
    Union(Box<TypeExpr>, Box<TypeExpr>),
    Tuple(Vec<TypeExpr>),
    List(Box<TypeExpr>, Multiplicity),
    Func(Box<TypeExpr>, Box<TypeExpr>),
}

pub struct ScenarioBlock {
    pub given: Option<String>,   // stored as raw expression string
    pub when: Option<String>,
    pub then: Option<String>,
}

pub struct BenchmarkBlock {
    pub baseline:    String,          // condition name, e.g. "none"
    pub target:      String,          // condition name, e.g. "product"
    pub scorer:      ScorerConfig,
    pub pass:        String,          // raw pass expression, stored verbatim
}

pub struct ScorerConfig {
    pub kind:        String,          // e.g. "rubric_llm"
    pub params:      Vec<(String, String)>,
}

pub struct EvidenceBlock {
    pub delta: f64,              // δ — confidence [0.0, 1.0]
    pub phi: u8,                 // φ — coverage [0, 100]
    pub tau: Stability,
}

pub enum Stability { Stable, Unstable, Unknown }

pub struct Invariant {
    pub raw: String,             // stored verbatim for context bundle output
    pub quantifier: Option<Quantifier>,
}
```

`Invariant.raw` stores the original text verbatim. The AST is used for validation; the raw string is used for context bundle output. This ensures the bundle output matches exactly what the author wrote, without any round-trip formatting changes.

---

### Parse Error Behaviour

Block delimiter errors (unclosed `⟦`, unrecognised block type) are E001 errors — the file cannot be processed further.

Malformed content inside a block (invalid expression, missing `≜`) is E001 on the specific line. The rest of the block is skipped; subsequent blocks in the same file are still parsed.

An empty block body (`⟦Γ:Invariants⟧{}`) is W004 — syntactically valid but semantically meaningless.

An evidence block with `δ` outside [0.0, 1.0] or `φ` outside [0, 100] is E001.

---

### Opaque Storage for Context Bundles

Formal blocks are stored as both a parsed AST (for validation) and the original raw text (for bundle output). The raw text is extracted between the outer `{...}` delimiters and preserved byte-for-byte. This means whitespace, comments, and any content that parses but is not fully modelled in the AST is round-tripped faithfully.

**Rationale:**
- Hand-written recursive descent is the right tool for a small, well-defined grammar with good error recovery requirements. Parser combinator libraries (nom, pest) add compile-time complexity for a grammar this size without meaningful benefit.
- Storing `Invariant.raw` verbatim rather than pretty-printing from the AST ensures the context bundle output matches the author's intent exactly — Product is a context assembly tool, not a formatter.
- The grammar is intentionally permissive on expressions. Full semantic validation of formal expressions is not Product's job — that belongs to the agent or tool consuming the bundle.

**Rejected alternatives:**
- **Treat formal blocks as opaque strings (no parsing)** — simplest implementation. Rejected because it removes the ability to validate evidence block ranges, detect empty blocks (W004), or surface parse errors with line precision. The grammar provides validation without requiring full semantic analysis.
- **Pest PEG parser** — clean grammar definition, good error messages. Rejected because it adds a build-time dependency and a `.pest` file to maintain. For a grammar this small, the overhead is not justified.
- **Regex-based extraction** — extract block content with regex patterns. Rejected because nested `⟨⟩` and `{}` delimiters cannot be correctly parsed with regex. A recursive descent parser is required for correct delimiter matching.