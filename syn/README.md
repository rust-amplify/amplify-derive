# Attribute syntax helper library

![Build](https://github.com/rust-amplify/amplify-derive/workflows/Build/badge.svg)
![Tests](https://github.com/rust-amplify/amplify-derive/workflows/Tests/badge.svg)
![Lints](https://github.com/rust-amplify/amplify-derive/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/rust-amplify/amplify-derive/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-amplify/rust-amplify)

[![crates.io](https://img.shields.io/crates/v/amplify_syn.svg)](https://crates.io/crates/amplify_syn)
[![Docs](https://docs.rs/amplify_syn/badge.svg)](https://docs.rs/amplify_syn)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/amplify_syn)](./LICENSE)

`amplify_syn` lets a derive macro state what its attributes may contain, and
reads the attributes through that statement.

- **Rules.** `AttrReq` declares the named arguments of an attribute and the
  value each of them holds (`ArgValueReq`, `ValueClass`), its unnamed
  arguments (`ListReq`) and the arguments which exclude each other.
- **Checked content.** Reading an attribute through its rules gives a
  `ParametrizedAttr`: only what the rules allow, with defaults filled in. An
  attribute is read once per item (`single`), merged over all its occurrences
  (`merged`) or occurrence by occurrence (`each`).
- **Errors.** Whatever breaks the rules is a `syn::Error` at the offending
  argument. All errors of an attribute are reported together, and no input
  makes the check panic.
- **Grammar.** Underneath, `Attr`, `Form`, `Arg`, `NamedArg` and `Value`
  parse `#[name]`, `#[name = value]` and `#[name(...)]` with literals, types
  and `name = value` arguments.

The crate is built on `syn` 3 and targets Rust 1.100.0 (edition 2024).

```rust,ignore
use amplify_syn::{ArgValueReq, AttrReq, ValueClass};

// `#[getter(prefix = "get_", as_mut, skip, base_name = "id")]`
let rules = AttrReq::new("getter")
    .arg("prefix", ArgValueReq::with_default(""))      // `prefix` alone is `prefix = ""`
    .arg("as_mut", ArgValueReq::with_default("_mut"))  // `as_mut` alone is `as_mut = "_mut"`
    .arg("skip", ArgValueReq::Prohibited)              // a flag
    .arg("base_name", ArgValueReq::required(ValueClass::Str))
    .conflict("skip", &["prefix", "as_mut", "base_name"]);

let getter = rules.merged(&field.attrs)?;
let skip = getter.has("skip");
let prefix = getter.str("prefix").map(|prefix| prefix.value());
```

Compared with 2.x, the rules are declared with a builder, their violations
carry spans, and the derive scaffolding (`DataType`, `DeriveInner`) is gone;
see the changelog for the complete list.
