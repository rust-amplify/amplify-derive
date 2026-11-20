Change Log
==========

3.0.0
-----
- Rebuilt the crate on `syn` 3, Rust 1.100.0, and edition 2024.
- New attribute grammar: `Attr`, `Form`, `Arg`, `NamedArg` and `Value`. It
  reads types which are not paths (`&'static str`, `[u8; 4]`, `!`) and
  expression values, keeps repeated attributes apart, and its errors preserve
  token spans.
- The attribute rules are rebuilt on that grammar:
  - `AttrReq` is declared with a builder (`arg`, `mandatory`, `conflict`,
    `unnamed`, `name_value`) and reads attributes itself: once per item
    (`single`), merged over all occurrences (`merged`) or occurrence by
    occurrence (`each`).
  - `ArgValueReq` keeps `Prohibited`, `Optional` and `Required` with an
    optional default. It no longer decides whether the argument must be given;
    `AttrReq::mandatory` does.
  - `ListReq` describes all unnamed arguments at once: their `ValueClass`es,
    their number and, with `any_of`, the identifiers accepted.
  - `ParametrizedAttr` is the checked content of an attribute and is read
    through `has`, `value`, `str` and `unnamed`.
  - Every violation is a `syn::Error` at the offending argument, all violations
    of an attribute are reported together, and no input panics.
  - `collect_all` gathers the errors of several attributes, such as those of
    all the fields of a structure, into one.
  - A name which the rules do not declare is a fault of the derive macro and
    panics: in `AttrReq::conflict` and in the `ParametrizedAttr` methods which
    take the name of an argument.
- Removed `ExtractAttr`, `SingularAttr`, `ArgValue`, `ValueReq`, `LiteralClass`,
  `TypeClass`, the `MetaArg` parsers, `DataType` with the derive scaffolding
  around it, the custom `Error` type, and the `ident!` and `path!` macros.

1.1.0
-----
- Custom parsers allowing attribute values to take any form, not only literals
- Convenience methods for ArgValue conversion from/to idents & paths
- `ident!` macros for simplified `syn::Ident` creation

1.0.1
-----
- Fixing sample code URL in README
- Renaming `ArgReq` to `ArgValReq` (since the old name)
- Yanking 1.0.0 version (since the rename is breaking backward compatibility)
