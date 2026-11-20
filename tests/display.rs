#![allow(missing_docs)]

use amplify_derive::Display;

mod no_implicit_prelude {
    #![no_implicit_prelude]

    #[derive(::amplify_derive::Display)]
    #[display("{x}", alt = "<{x}>")]
    pub struct Named {
        pub x: u8,
    }

    #[derive(::amplify_derive::Display)]
    #[display("{0}")]
    pub struct Tuple(pub u8);

    #[derive(::amplify_derive::Display)]
    #[display("unit")]
    pub struct Unit;

    #[derive(::amplify_derive::Display)]
    #[display("{items}")]
    pub struct Joined {
        #[display(separator = ", ")]
        pub items: ::std::vec::Vec<&'static str>,
    }

    #[derive(::amplify_derive::Display, ::core::fmt::Debug)]
    #[display(Debug)]
    #[allow(dead_code)]
    pub struct Debugged(pub u8);

    #[derive(::amplify_derive::Display)]
    pub enum Variants {
        #[display("{0}", alt = "{1}")]
        Tuple(u8, u8),
        #[display("{x}")]
        Named {
            x: u8,
        },
        #[display(inner)]
        Inner(u8),
        NoAttribute,
    }

    #[derive(::amplify_derive::Display)]
    #[display(lowercase)]
    pub enum Lower {
        Ready,
        Waiting(u8),
    }
}

#[derive(Display)]
#[display("({x}, {y})", alt = "{x}:{y}")]
struct Point {
    x: u8,
    y: u8,
}

#[derive(Display)]
#[display(inner)]
struct Label(&'static str);

#[derive(Display)]
#[display("items: {items}")]
struct Items {
    #[display(separator = ", ")]
    items: Vec<&'static str>,
}

#[derive(Display)]
#[display("number={0}")]
struct Number(u16);

#[derive(Display)]
#[display(lowercase)]
enum State {
    Ready,
    Waiting(u8),
}

#[derive(Display)]
#[display("unit")]
struct Unit;

#[derive(Display)]
#[allow(dead_code)]
enum VariantDisplay {
    #[display("{x}", alt = "{y}")]
    Named { x: u8, y: u8 },
    #[display("{1}")]
    Tuple(u8, u8),
    #[display("{f}")]
    Formatter { f: u8 },
    #[display("{0}", alt = "{1}")]
    TupleAlt(u8, u8),
}

#[derive(Display, Debug)]
#[display(Debug)]
#[allow(dead_code)]
struct DebugDisplay {
    value: u8,
}

#[derive(Display)]
#[display(Shout::print)]
struct Shout(&'static str);

impl Shout {
    fn print(&self) -> String { self.0.to_uppercase() }
}

#[derive(Display)]
#[display(doc_comments)]
enum Documented {
    /// Letter A
    A,
    /// Letter {0}
    Letter(char),
    /// This comment is ignored
    #[display("custom")]
    Custom,
}

#[derive(Display)]
#[display(uppercase)]
enum Loud {
    Ready,
    Waiting(u8),
}

#[derive(Display)]
#[display("{{0}}")]
struct Escaped;

#[derive(Display)]
#[display("{}")]
struct Implicit(u8);

#[derive(Display)]
#[display("{x:>w$}")]
struct Width {
    x: u8,
    w: usize,
}

#[derive(Display)]
#[display("{x }")]
struct TrailingWhitespace {
    x: u8,
}

#[derive(Display)]
#[display = "assigned {0}"]
struct Assigned(u8);

#[derive(Display)]
enum UnitAlt {
    #[display("plain", alt = "alternative")]
    Unit,
}

#[derive(Display)]
#[allow(dead_code)]
enum NoAttributes {
    Unit,
    Tuple(u8),
    Named { x: u8 },
}

#[derive(Display)]
enum Uninhabited {}

#[derive(Display)]
#[display("<{0}>", alt = "<<{0}>>")]
enum TypeLevelFormat {
    Number(u8),
    Text {
        text: &'static str,
    },
    #[display("nothing")]
    Nothing,
}

struct Binary;

impl Binary {
    fn show(flags: &Flags) -> String { format!("0b{:b}", flags.0) }
}

#[derive(Display)]
#[display(Binary::show)]
struct Flags(u8);

#[derive(Display)]
#[display("[{0}]")]
struct TupleItems(#[display(separator = "; ")] Vec<&'static str>);

#[derive(Display)]
enum VariantItems {
    #[display("({0})")]
    Tuple(#[display(separator = "+")] Vec<&'static str>),
    #[display("({items})", alt = "{items}")]
    Named {
        #[display(separator = "-")]
        items: Vec<&'static str>,
    },
}

#[derive(Display)]
#[display(lowercase)]
struct LowerPoint {
    x: u8,
    r#type: u8,
}

#[derive(Display)]
#[display(uppercase)]
struct UpperPair(u8, u8);

#[derive(Display)]
#[display(uppercase)]
struct UpperUnit;

#[derive(Display)]
#[display(lowercase)]
#[allow(dead_code)]
union LowerUnion {
    value: u8,
}

#[derive(Display)]
#[display(inner)]
struct InnerUnit;

#[derive(Display)]
#[display("{0}/{1:?}/{2:x}")]
struct Generic<T, U, X>(T, Option<U>, X);

#[derive(Display, Debug)]
#[display(Debug)]
struct GenericDebug<T>(T);

#[derive(Display)]
#[display(inner)]
enum GenericVariants<T> {
    Value(T),
    #[display("{code:02}: {reason}")]
    Coded {
        code: u8,
        reason: T,
    },
}

#[derive(Display)]
#[display("{0}")]
struct Unsized<T: ?Sized>(Box<T>);

const UNIT: &str = "km";

#[derive(Display)]
#[display("{type} {0}{UNIT} {_1}")]
struct Distance {
    r#type: u8,
    value: u8,
}

#[test]
fn formats_structs_tuples_units_and_variants() {
    let point = Point { x: 2, y: 7 };
    assert_eq!(format!("{point}"), "(2, 7)");
    assert_eq!(format!("{point:#}"), "2:7");
    assert_eq!(format!("{}", Label("hello")), "hello");
    assert_eq!(
        format!("{}", Items {
            items: vec!["a", "b"]
        }),
        "items: a, b"
    );
    assert_eq!(format!("{}", Number(12)), "number=12");
    assert_eq!(format!("{}", State::Ready), "ready");
    assert_eq!(format!("{}", State::Waiting(3)), "waiting(3)");
    assert_eq!(format!("{}", Unit), "unit");
    assert_eq!(format!("{}", VariantDisplay::Named { x: 1, y: 2 }), "1");
    assert_eq!(format!("{:#}", VariantDisplay::Named { x: 1, y: 2 }), "2");
    assert_eq!(format!("{}", VariantDisplay::Tuple(11, 22)), "22");
    assert_eq!(format!("{}", VariantDisplay::Formatter { f: 8 }), "8");
    assert_eq!(format!("{}", VariantDisplay::TupleAlt(1, 2)), "1");
    assert_eq!(format!("{:#}", VariantDisplay::TupleAlt(1, 2)), "2");
    assert_eq!(format!("{}", DebugDisplay { value: 1 }), "DebugDisplay { value: 1 }");
    assert_eq!(format!("{}", Shout("hello")), "HELLO");
    assert_eq!(format!("{}", Documented::A), "Letter A");
    assert_eq!(format!("{}", Documented::Letter('K')), "Letter K");
    assert_eq!(format!("{}", Documented::Custom), "custom");
    assert_eq!(format!("{}", Loud::Ready), "READY");
    assert_eq!(format!("{}", Loud::Waiting(3)), "WAITING(3)");
    assert_eq!(format!("{}", Escaped), "{0}");
    assert_eq!(format!("{}", Implicit(9)), "9");
    assert_eq!(format!("{}", Width { x: 7, w: 3 }), "  7");
    assert_eq!(format!("{}", TrailingWhitespace { x: 5 }), "5");
    assert_eq!(format!("{}", Assigned(4)), "assigned 4");
}

#[test]
fn alternative_format_applies_to_unit_variants() {
    assert_eq!(format!("{}", UnitAlt::Unit), "plain");
    assert_eq!(format!("{:#}", UnitAlt::Unit), "alternative");
}

#[test]
fn enums_without_attributes_print_variant_names() {
    assert_eq!(format!("{}", NoAttributes::Unit), "Unit");
    assert_eq!(format!("{}", NoAttributes::Tuple(1)), "Tuple(..)");
    assert_eq!(format!("{}", NoAttributes::Named { x: 1 }), "Named { .. }");
    let _: fn(&Uninhabited) -> String = ToString::to_string;
}

#[test]
fn type_level_format_is_applied_to_each_variant() {
    assert_eq!(format!("{}", TypeLevelFormat::Number(1)), "<1>");
    assert_eq!(format!("{:#}", TypeLevelFormat::Number(1)), "<<1>>");
    assert_eq!(format!("{}", TypeLevelFormat::Text { text: "a" }), "<a>");
    assert_eq!(format!("{}", TypeLevelFormat::Nothing), "nothing");
    assert_eq!(format!("{:#}", TypeLevelFormat::Nothing), "nothing");
}

#[test]
fn a_path_named_like_a_formatting_trait_is_a_function() {
    assert_eq!(format!("{}", Flags(5)), "0b101");
}

#[test]
fn separators_join_tuple_fields() {
    assert_eq!(format!("{}", TupleItems(vec!["a", "b"])), "[a; b]");
    assert_eq!(format!("{}", VariantItems::Tuple(vec!["a", "b"])), "(a+b)");
    let named = VariantItems::Named {
        items: vec!["a", "b"],
    };
    assert_eq!(format!("{named}"), "(a-b)");
    assert_eq!(format!("{named:#}"), "a-b");
}

#[test]
fn letter_case_applies_to_structs_and_unions() {
    assert_eq!(format!("{}", LowerPoint { x: 1, r#type: 2 }), "lowerpoint { x: 1, type: 2 }");
    assert_eq!(format!("{}", UpperPair(1, 2)), "UPPERPAIR(1, 2)");
    assert_eq!(format!("{}", UpperUnit), "UPPERUNIT");
    assert_eq!(format!("{}", LowerUnion { value: 1 }), "lowerunion");
    assert_eq!(format!("{}", InnerUnit), "InnerUnit");
}

#[test]
fn placeholders_name_fields_by_name_or_position_and_other_items_in_scope() {
    assert_eq!(
        format!("{}", Distance {
            r#type: 1,
            value: 2
        }),
        "1 1km 2"
    );
}

#[test]
fn generic_fields_get_the_bounds_of_their_placeholders() {
    assert_eq!(format!("{}", Generic(1u8, Some("a"), 255u8)), "1/Some(\"a\")/ff");
    assert_eq!(format!("{}", GenericDebug("a")), "GenericDebug(\"a\")");
    assert_eq!(format!("{}", GenericVariants::Value("a")), "a");
    let coded = GenericVariants::Coded {
        code: 4,
        reason: "b",
    };
    assert_eq!(format!("{coded}"), "04: b");
    assert_eq!(format!("{}", Unsized::<str>("a".into())), "a");
}

#[test]
fn generated_code_does_not_depend_on_the_prelude() {
    use no_implicit_prelude::{Debugged, Joined, Lower, Named, Tuple, Unit, Variants};

    assert_eq!(format!("{}", Named { x: 1 }), "1");
    assert_eq!(format!("{:#}", Named { x: 1 }), "<1>");
    assert_eq!(format!("{}", Tuple(2)), "2");
    assert_eq!(format!("{}", Unit), "unit");
    assert_eq!(
        format!("{}", Joined {
            items: vec!["a", "b"]
        }),
        "a, b"
    );
    assert_eq!(format!("{}", Debugged(3)), "Debugged(3)");
    assert_eq!(format!("{}", Variants::Tuple(4, 5)), "4");
    assert_eq!(format!("{:#}", Variants::Tuple(4, 5)), "5");
    assert_eq!(format!("{}", Variants::Named { x: 6 }), "6");
    assert_eq!(format!("{}", Variants::Inner(7)), "7");
    assert_eq!(format!("{}", Variants::NoAttribute), "NoAttribute");
    assert_eq!(format!("{}", Lower::Ready), "ready");
    assert_eq!(format!("{}", Lower::Waiting(8)), "waiting(8)");
}
