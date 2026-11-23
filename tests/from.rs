#![allow(missing_docs)]

use amplify_derive::From;

mod no_implicit_prelude {
    #![no_implicit_prelude]

    #[derive(::amplify_derive::From, ::core::default::Default)]
    pub struct Named {
        #[from]
        pub x: u8,
        pub y: u8,
    }

    #[derive(::amplify_derive::From, ::core::default::Default)]
    #[from(u16)]
    pub struct Unit;

    #[derive(::amplify_derive::From, ::core::default::Default)]
    pub struct Tuple(#[from] pub u8, pub u16);

    #[derive(::amplify_derive::From)]
    pub enum Variants {
        #[from]
        Single(u8),
        #[from(u16)]
        Fields {
            a: u8,
            b: u8,
        },
        #[from(u32)]
        Unit,
        Mixed {
            #[from]
            value: i8,
            other: u8,
        },
        Positional(#[from] i16, u8),
    }
}

#[derive(From, Default, Debug, PartialEq)]
struct Number(u32);

#[derive(From, Default, Debug, PartialEq)]
struct Coordinates {
    #[from]
    x: u8,
    y: u8,
}

#[derive(From)]
enum ParseError {
    #[from]
    Io(std::io::Error),
    #[from]
    Integer(std::num::ParseIntError),
}

#[derive(From, Debug, PartialEq)]
enum FromValues {
    #[from(u8)]
    Named { first: u8, second: bool },
    #[from(u16)]
    Braced {},
    #[from(u32)]
    Tuple(),
    #[from(i8)]
    Pair(i16, bool),
}

#[derive(From, Debug, PartialEq)]
#[from(u8)]
struct EmptyTuple();

#[derive(From, Debug, PartialEq)]
#[from(u8)]
struct EmptyNamed {}

#[derive(From, Debug, PartialEq)]
struct SingleNamed {
    value: u8,
}

#[derive(From, Debug, PartialEq)]
#[from(u8)]
struct Widened {
    value: u16,
}

#[derive(From, Default, Debug, PartialEq)]
#[from(u8)]
struct TypeLevelNamed {
    first: u16,
    second: u16,
}

#[derive(From, Default, Debug, PartialEq)]
#[from(u8)]
struct TypeLevelTuple(u16, u16);

#[derive(From, Debug, PartialEq)]
struct Config {
    #[from]
    name: &'static str,
    retries: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "",
            retries: 3,
        }
    }
}

#[derive(From)]
#[allow(dead_code)]
union Either {
    #[from]
    int: u32,
    #[from]
    float: f32,
}

#[derive(From, Debug, PartialEq)]
enum NonPathTypes {
    #[from(&'static str)]
    Text,
    #[from([u8; 4])]
    Array,
    #[from((u8, u16))]
    Tuple,
    #[from(u64)]
    #[from(i64)]
    Number,
}

#[derive(From, Default, Debug, PartialEq)]
#[from(std::fmt::Error)]
struct TypeLevel;

#[derive(From)]
#[allow(dead_code)]
union UnionValue {
    #[from]
    value: u8,
}

mod hygiene {
    use amplify_derive::From;

    #[allow(non_upper_case_globals, dead_code)]
    const v: u8 = 0;

    #[derive(From, Debug, PartialEq)]
    pub struct Hygienic(pub u8);
}

#[test]
fn generates_struct_and_enum_conversions() {
    assert_eq!(Number::from(42), Number(42));
    assert_eq!(Coordinates::from(5), Coordinates { x: 5, y: 0 });

    let io_error = std::io::Error::other("failed");
    match ParseError::from(io_error) {
        ParseError::Io(error) => assert_eq!(error.to_string(), "failed"),
        ParseError::Integer(_) => panic!("expected an I/O error variant"),
    }

    let parse_error = "not a number".parse::<u8>().unwrap_err();
    match ParseError::from(parse_error) {
        ParseError::Integer(error) => assert!(!error.to_string().is_empty()),
        ParseError::Io(_) => panic!("expected an integer parse error variant"),
    }

    assert_eq!(FromValues::from(5u8), FromValues::Named {
        first: 0,
        second: false
    });
    assert!(matches!(FromValues::from(5u16), FromValues::Braced {}));
    assert!(matches!(FromValues::from(5u32), FromValues::Tuple()));
    let _union = UnionValue::from(7u8);

    assert_eq!(NonPathTypes::from("text"), NonPathTypes::Text);
    assert_eq!(NonPathTypes::from([0u8; 4]), NonPathTypes::Array);
    assert_eq!(NonPathTypes::from((1u8, 2u16)), NonPathTypes::Tuple);
    assert_eq!(NonPathTypes::from(1u64), NonPathTypes::Number);
    assert_eq!(NonPathTypes::from(1i64), NonPathTypes::Number);
    assert_eq!(TypeLevel::from(std::fmt::Error), TypeLevel);
    assert_eq!(hygiene::Hygienic::from(9u8), hygiene::Hygienic(9));
}

#[test]
fn converts_into_one_field_and_defaults_the_others() {
    assert_eq!(FromValues::from(5i8), FromValues::Pair(5, false));
    assert_eq!(EmptyTuple::from(1), EmptyTuple());
    assert_eq!(EmptyNamed::from(1), EmptyNamed {});
    assert_eq!(SingleNamed::from(1), SingleNamed { value: 1 });
    assert_eq!(Widened::from(1), Widened { value: 1 });
    assert_eq!(TypeLevelNamed::from(1), TypeLevelNamed::default());
    assert_eq!(TypeLevelTuple::from(1), TypeLevelTuple(1, 0));
    // A structure with named fields takes the other fields from its own `Default`.
    assert_eq!(Config::from("name"), Config {
        name: "name",
        retries: 3
    });
    let _int = Either::from(7u32);
    let _float = Either::from(0.5f32);
}

#[test]
fn generated_code_does_not_depend_on_the_prelude() {
    use no_implicit_prelude::{Named, Tuple, Unit, Variants};

    let named = Named::from(3u8);
    assert_eq!((named.x, named.y), (3, 0));
    let _unit: Unit = Unit::from(1u16);
    let tuple = Tuple::from(4u8);
    assert_eq!((tuple.0, tuple.1), (4, 0));

    assert!(matches!(Variants::from(1u8), Variants::Single(1)));
    assert!(matches!(Variants::from(1u16), Variants::Fields { a: 0, b: 0 }));
    assert!(matches!(Variants::from(1u32), Variants::Unit));
    assert!(matches!(Variants::from(2i8), Variants::Mixed { value: 2, other: 0 }));
    assert!(matches!(Variants::from(3i16), Variants::Positional(3, 0)));
}
