#![allow(missing_docs, dead_code)]

extern crate amplify as renamed;

mod display_expanded {
    use amplify_derive::Display;

    include!("expand/display.rs");

    impl Shout {
        fn print(&self) -> String { self.0.to_owned() }
    }

    pub fn render_shout() -> String { Shout("ok").to_string() }
}

mod error_as_any_expanded {
    use core::fmt::Debug;

    use amplify_derive::{AsAny, Error};

    include!("expand/error_as_any.rs");
}

mod from_expanded {
    use amplify_derive::From;

    include!("expand/from.rs");

    impl Default for Named {
        fn default() -> Self {
            Self {
                name: String::new(),
                count: 1,
            }
        }
    }

    impl From<Single> for Named {
        fn from(value: Single) -> Self {
            Self {
                name: String::new(),
                count: value.0,
            }
        }
    }
}

mod getters_expanded {
    pub mod inner {
        use amplify_derive::Getters;

        include!("expand/getters.rs");

        pub fn check() -> u8 {
            let one = One::<u8> {
                vec: vec![1],
                defaults: 0,
                flag: true,
                field: 2,
                r#type: 3,
            };
            let mut two = Two {
                name: String::from("a"),
                list: vec![4],
                count: 5,
            };
            two.list_changed().push(6);
            *one.get_type_ref() + two.list_borrowed()[0] + two.count()
        }
    }
}

mod wrapper_expanded {
    use amplify_derive::{Wrapper, WrapperMut};

    include!("expand/wrapper.rs");

    impl From<i64> for Number {
        fn from(value: i64) -> Self { Self(value) }
    }

    impl From<Vec<u8>> for Indexed {
        fn from(value: Vec<u8>) -> Self { Self(value) }
    }

    impl<T, U: Copy + Default> From<Vec<T>> for Generic<T, U> {
        fn from(inner: Vec<T>) -> Self {
            Self {
                inner,
                other: U::default(),
            }
        }
    }

    impl<'a, T> From<&'a T> for Param<'a, T> {
        fn from(value: &'a T) -> Self { Self(value) }
    }
}

#[test]
fn expansion_fixtures_typecheck_and_run() {
    assert_eq!(display_expanded::render_shout(), "ok");
    assert_eq!(getters_expanded::inner::check(), 12);
}
