# Rust Amplify Library: derive macros
![Build](https://github.com/rust-amplify/amplify-derive/workflows/Build/badge.svg)
![Tests](https://github.com/rust-amplify/amplify-derive/workflows/Tests/badge.svg)
![Lints](https://github.com/rust-amplify/amplify-derive/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/rust-amplify/amplify-derive/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-amplify/amplify-derive)

[![crates.io](https://img.shields.io/crates/v/amplify_derive.svg)](https://crates.io/crates/amplify_derive)
[![Docs](https://docs.rs/amplify_derive/badge.svg)](https://docs.rs/amplify_derive)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/amplify_derive)](./LICENSE)

Amplifying Rust language capabilities: multiple generic trait implementations, 
type wrappers, derive macros.

This is a part of Rust language amplification library providing required derive
macros.

Minimum supported Rust version (MSRV): 1.100.0. Rust edition: 2024.

## Overview

- [Display](#display-derive)
- [From](#from-derive)
- [Error](#error-derive)
- [Getters](#getters-derive)
- [Wrapper](#wrapper-derive)
- [AsAny](#asany-derive)

## Display derive 

1. Generate [`Display`] descriptions using other formatting trait:
   ```rust
    #[derive(Display, Debug)]
    #[display(Debug)]
    struct Some { /* ... */ }
   ```
2. Use existing function for displaying descriptions:
   ```rust
    #[derive(Display)]
    #[display(Int::print)]
    union Int { uint: u32, int: i32 };
   
    impl Int {
        pub fn print(&self) -> String {
            std::string::String::from("Integer representation")
        }
    }
   ```
   Formatting function must return [`String`] and take a single `self`
   argument (if you need formatting with streamed output, use one of
   existing formatting traits as shown in pt. 1).
3. Custom format string:
   ```rust
    #[derive(Display)]
    #[display("({x}, {y})")]
    struct Point { x: u32, y: u32 }
   ```
4. Use of doc comments for descrition representation. In this case doc
   comments may also contain formatting like in the case 3:
   ```rust
    #[macro_use] extern crate amplify;

    #[derive(Display)]
    #[display(doc_comments)]
    enum Variants {
        /// Letter A
        A,

        /// Letter B
        B,

        /// This comment is ignored
        #[display("Letter C")]
        C,

        /// Letter {0}
        Letter(String)
    };

    assert_eq!(format!("{}", Variants::C), "Letter C");
    assert_eq!(format!("{}", Variants::Letter(s!("K"))), "Letter K");
   ```
   You can also mix in this mode with other fors of display tags on a
   specific options; in this case doc comments are ignored

### Example

Advanced use with enums:
```rust
#[derive(Display)]
enum Test {
    Some,

    #[display("OtherName")]
    Other,

    Named {
        x: u8,
    },

    #[display("Custom{x}", alt = "this is alternative")]
    NamedCustom {
        x: u8,
    },

    #[display(inner)]
    Inner {
        a: String,
    },

    Unnamed(u16),

    // NB: Use indexes for tuple values
    #[display("Custom{0}")]
    UnnamedCustom(String),
}

assert_eq!(format!("{}", Test::Some), "Some");
assert_eq!(format!("{}", Test::Other), "OtherName");
assert_eq!(format!("{}", Test::Named { x: 1 }), "Named { .. }");
assert_eq!(format!("{}", Test::Unnamed(5)), "Unnamed(..)");
assert_eq!(format!("{}", Test::NamedCustom { x: 8 }), "Custom8");
assert_eq!(format!("{:#}", Test::NamedCustom { x: 8 }), "this is alternative");
assert_eq!(format!("{}", Test::UnnamedCustom("Test".to_string())), "CustomTest");
```

Format strings follow the rules of `format!`: fields are referenced by
index (`{0}`) and named fields by name as well, empty placeholders (`{}`) are
numbered in order, escaped braces (`{{` and `}}`) are printed literally, and
fields may be used as width or precision parameters (`{value:>width$}`).

On an enum, an attribute on the type applies to every variant which has no
attribute of its own, and a variant without any attribute prints its name.
`lowercase` and `uppercase` print the name of the type or of the variant with
its fields, `inner` prints the only field, and a field with
`#[display(separator = ", ")]` is joined with the separator.

When the type of a field depends on a type parameter, the implementation
requires the formatting trait which the placeholder of that field uses.

## Error derive

Error derive macro works to the full extend only when other derive macros
are used. With `#[derive(Display)]` and `[display(doc_comments)]` it uses
doc comments for generating error descriptions; with `#[derive(From)]` it
may automatically implement transofrations from other error types.

The macro implements `core::error::Error` and a conversion of the error into
`String`. It works in `no_std` crates which have the `alloc` crate.

```rust
#[derive(Debug, Display, Error)]
#[display(doc_comments)]
enum Error {
    /// I/O operation error
    Io,
    /// Math overflow
    Overflow,
    /// Zero division with {0}
    ZeroDivision(u16),
}
```

## From derive

Implements [`From`] trait for the whole entity and/or its separate fields.
Works well with `#[derive(Error)]` and, in many cases may require
[`Default`] implementation (for details, pls see Examples below)

Deriving `From` where there is nothing to convert from, or two conversions
from the same type, is an error.

### Examples

```rust
#[derive(From, Default)]
#[from(::std::io::Error)]
// Structure may contain no parameters
pub struct IoErrorUnit;

#[derive(From, Default)]
#[from(::std::io::Error)] // When no explicit binding is given, structure must implement `Default`
pub struct IoError {
    details: String,

    #[from]
    kind: IoErrorUnit,
}

#[derive(From)]
pub enum Error {
    // You can specify multiple conversions with separate attributes
    #[from(::std::io::Error)]
    #[from(IoError)]
    Io,

    #[from]
    Format(::std::fmt::Error),

    #[from]
    WithFields { details: ::std::str::Utf8Error },

    MultipleFields {
        // ...and you can also covert error type
        #[from(IoErrorUnit)]
        // rest of parameters must implement `Default`
        io: IoError,
        details: String,
    },
}

// A structure with a single field converts from the type of that field
// without any attribute
#[derive(From)]
pub struct Wrapper(u32);
```

## Wrapper derive

Creates a Rust newtype wrapping an existing type. It can be used in structures
containing multiple named or unnamed fields; in this case the field you'd
like to wrap must be marked with `#[wrap]` attribute.

The wrapper needs `From` for the wrapped type, which `#[derive(From)]`
provides. With multiple fields the wrapped one must be marked with `#[from]`
as well, and the other fields are filled with their defaults.

`#[derive(Wrapper)]` implements:
* `amplify::Wrapper`
* `AsRef` and `Borrow` for the wrapped type
* `From<Wrapper>` for the wrapped type

`#[derive(WrapperMut)]` implements:
* `amplify::WrapperMut`
* `AsMut` and `BorrowMut` for the wrapped type

Additional standard traits can be selected on stable Rust when they are
implemented by the wrapped type. `#[wrapper(...)]` supports formatting traits,
`FromStr`, `FromHex`, `Deref`, reference access, indexing, ranges, and value
operators. `#[wrapper_mut(...)]` supports `DerefMut`, mutable references,
mutable indexing, ranges, and assignment operators. Group options include
`Hex`, `Exp`, `NumberFmt`, `RangeOps`, `MathOps`, `BoolOps`, `BitOps`,
`RangeMut`, `MathAssign`, `BoolAssign`, and `BitAssign`. Use `NoRefs` to omit
the default reference impls; `#[wrapper(NoRefs, AsRef)]` keeps one of them.

A trait is derived once, however often it is named. When the wrapped type is
generic, a trait is derived for the wrappers of those types which implement
it.

Other traits, such as `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
`Hash` can be implemented using standard `#[derive]` attribute in the
same manner as `Default`, `Debug` and `From`

### Example

```rust
use std::collections::HashMap;
use std::marker::PhantomData;
use amplify::Wrapper as _;
use amplify_derive::{From, Wrapper};

#[derive(Clone, Wrapper, Default, From, Debug)]
struct Wrapped<T, U>(
    #[wrap]
    #[from]
    HashMap<usize, Vec<U>>,
    PhantomData<T>,
)
where
    U: Sized + Clone;

let w = Wrapped::<(), u8>::default();
assert_eq!(w.as_inner(), &HashMap::<usize, Vec<u8>>::default());
```

## Getters derive

Creates getter methods matching field names for all fields within a
structure (including public and private fields). Getters return reference
types, unless `#[getter(...)]` on the structure or on a field asks for
something else: `as_copy`, `as_clone`, `as_ref`, `as_mut`, `all`, `skip`,
`prefix = "..."` and `base_name = "..."`.

### Example

```rust
#[derive(Getters, Default)]
struct One {
    a: Vec<u8>,
    pub b: bool,
    pub(self) c: u8,
}

let one = One::default();
assert_eq!(one.a(), &Vec::<u8>::default());
assert_eq!(one.b(), &bool::default());
assert_eq!(one.c(), &u8::default());

#[derive(Getters, Default)]
#[getter(as_copy)]
struct Two {
    flag: bool,
    #[getter(as_ref, as_mut)]
    list: Vec<u8>,
    #[getter(skip)]
    secret: u8,
}

let mut two = Two::default();
assert_eq!(two.flag(), false);
two.list_mut().push(1);
assert_eq!(two.list(), &vec![1]);
```

## AsAny derive

Trait [`amplify::AsAny`] allows simple conversion of any type into a generic
"thick" pointer `&dyn Any` (see [`::core::any::Any`]), that can be later
converted back to the original type with a graceful failing for all other
conversions. `AsAny` derive macro allows to implement this trait for
arbitrary time without much hussle:

### Example

```rust
use amplify::AsAny as _;
use amplify_derive::AsAny;

#[derive(AsAny, Copy, Clone, PartialEq, Eq, Debug)]
struct Point {
    pub x: u64,
    pub y: u64,
}

#[derive(AsAny, PartialEq, Debug)]
struct Circle {
    pub radius: f64,
    pub center: Point,
}

let mut point = Point { x: 1, y: 2 };
let point_ptr = point.as_any();

let mut circle = Circle {
    radius: 18.,
    center: point,
};
let circle_ptr = circle.as_any();

assert_eq!(point_ptr.downcast_ref(), Some(&point));
assert_eq!(circle_ptr.downcast_ref(), Some(&circle));
assert_eq!(circle_ptr.downcast_ref::<Point>(), None);

let p = point_ptr.downcast_ref::<Point>().unwrap();
assert_eq!(p.x, 1)
```
