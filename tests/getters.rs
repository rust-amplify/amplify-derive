#![allow(missing_docs)]

use amplify_derive::Getters;

mod no_implicit_prelude {
    #![no_implicit_prelude]

    #[derive(::amplify_derive::Getters, ::core::default::Default)]
    #[getter(as_mut)]
    pub struct Item {
        #[getter(as_clone)]
        name: ::std::string::String,
        #[getter(as_copy)]
        flag: bool,
        #[getter(all)]
        bytes: ::std::vec::Vec<u8>,
    }
}

#[derive(Getters, Default)]
struct Basic {
    vec: Vec<u8>,
    defaults: String,
    #[getter(as_copy)]
    flag: bool,
    #[getter(as_copy)]
    field: u8,
}

#[derive(Getters, Default)]
#[getter(as_copy)]
struct Overrides {
    #[getter(as_ref)]
    vec: Vec<u8>,
    #[getter(as_clone)]
    defaults: String,
    flag: bool,
    field: u8,
}

#[derive(Getters, Default)]
#[getter(as_mut, prefix = "get_")]
struct Advanced {
    /// Contains byte representation of the data
    #[getter(all, base_name = "bytes")]
    vec: Vec<u8>,
    defaults: String,
    #[getter(as_copy)]
    flag: bool,
    #[getter(skip)]
    field: u8,
}

#[derive(Getters, Default)]
struct MutOnly {
    #[getter(as_mut)]
    value: u8,
}

#[derive(Getters, Default)]
#[getter(as_copy)]
struct CopyWithMut {
    #[getter(as_mut)]
    value: u8,
    #[getter(prefix = "get_")]
    other: u8,
}

#[derive(Getters, Default)]
struct Suffixes {
    #[getter(as_copy = "_copy", as_ref = "_ref", as_mut = "_mutable")]
    value: u8,
    #[getter(as_clone = "_cloned")]
    #[getter(prefix = "the_")]
    name: String,
}

#[derive(Getters)]
#[getter(as_copy)]
struct Keywords {
    #[getter(as_copy)]
    r#type: bool,
    #[getter(as_copy, as_mut)]
    r#try: u8,
    #[getter(as_copy)]
    r#yield: u8,
    #[getter(as_copy)]
    r#gen: u8,
    #[getter(as_copy, prefix = "f", base_name = "n")]
    composed: u8,
    #[getter(as_copy, base_name = "match")]
    renamed: u8,
}

/// A getter fixture used to ensure every field documentation attribute is copied.
pub mod documented {
    #![deny(missing_docs)]

    use amplify_derive::Getters;

    /// A documented getter fixture.
    #[derive(Getters)]
    pub struct Item {
        /// First documentation line.
        /// Second documentation line.
        pub value: u8,
    }
}

#[test]
fn generates_documented_getter_variants_and_overrides() {
    let basic = Basic::default();
    assert_eq!(basic.vec(), &Vec::<u8>::new());
    assert_eq!(basic.defaults(), "");
    assert!(!basic.flag());
    assert_eq!(basic.field(), 0);

    let overrides = Overrides::default();
    assert_eq!(overrides.vec(), &Vec::<u8>::new());
    assert_eq!(overrides.defaults(), String::new());
    assert!(!overrides.flag());
    // A field without its own `#[getter]` gets the default borrowing getter;
    // the type-level `as_copy` does not reach it.
    assert_eq!(overrides.flag(), &false);
    assert_eq!(overrides.field(), &0);

    let mut advanced = Advanced::default();
    assert_eq!(advanced.get_bytes_ref(), &Vec::<u8>::new());
    *advanced.get_bytes_mut() = vec![0, 1, 2];
    assert_eq!(advanced.get_defaults(), "");
    advanced.get_defaults_mut().push('x');
    assert_eq!(advanced.get_defaults(), "x");
    assert_eq!(advanced.get_bytes(), vec![0, 1, 2]);
    assert!(!advanced.get_flag());
    *advanced.get_flag_mut() = true;
    assert!(advanced.get_flag());
    assert_eq!(advanced.flag, advanced.get_flag());
    assert_eq!(advanced.field, 0);

    // `as_mut` alone keeps the default borrowing getter.
    let mut mut_only = MutOnly::default();
    assert_eq!(mut_only.value(), &0);
    *mut_only.value_mut() = 1;
    assert_eq!(mut_only.value(), &1);

    // A field naming only `as_mut` keeps the type-level `as_copy`.
    let mut copy_with_mut = CopyWithMut::default();
    let copied: u8 = copy_with_mut.value();
    assert_eq!(copied, 0);
    *copy_with_mut.value_mut() = 2;
    assert_eq!(copy_with_mut.value(), 2);
    assert_eq!(copy_with_mut.get_other(), &0);

    // A value of a method argument replaces the suffix of the method name,
    // and the arguments of several attributes add up.
    let mut suffixes = Suffixes::default();
    *suffixes.value_mutable() = 7;
    assert_eq!(suffixes.value_copy(), 7);
    assert_eq!(suffixes.value_ref(), &7);
    assert_eq!(suffixes.the_name_cloned(), String::new());

    // Getter names which are reserved words, in this or in another edition.
    let mut keywords = Keywords {
        r#type: true,
        r#try: 1,
        r#yield: 2,
        r#gen: 3,
        composed: 4,
        renamed: 5,
    };
    assert!(keywords.r#type());
    *keywords.try_mut() += 10;
    assert_eq!(keywords.r#try(), 11);
    assert_eq!(keywords.r#yield(), 2);
    assert_eq!(keywords.r#gen(), 3);
    assert_eq!(keywords.r#fn(), 4);
    assert_eq!(keywords.r#match(), 5);
    assert_eq!(documented::Item { value: 3 }.value(), &3);
}

#[test]
fn generated_code_does_not_depend_on_the_prelude() {
    let mut item = no_implicit_prelude::Item::default();
    item.name_mut().push('x');
    assert_eq!(item.name(), "x");
    *item.flag_mut() = true;
    assert!(item.flag());
    item.bytes_mut().push(1);
    assert_eq!(item.bytes(), vec![1]);
    assert_eq!(item.bytes_ref(), &vec![1]);
}
