#![allow(missing_docs)]

use amplify as renamed_amplify;
use amplify::AsAny as AsAnyTrait;
use amplify_derive::{AsAny, Display, Error};

mod no_implicit_prelude {
    #![no_implicit_prelude]

    #[derive(::core::fmt::Debug, ::amplify_derive::Error)]
    pub struct ErrorType;

    impl ::core::fmt::Display for ErrorType {
        fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            formatter.write_str("error")
        }
    }
}

#[derive(AsAny, Clone, Debug, PartialEq)]
struct Point {
    x: u8,
    y: u8,
}

#[derive(AsAny)]
#[amplify_crate(renamed_amplify)]
struct RenamedCrate;

#[derive(Debug, Display, Error)]
#[display("invalid input")]
struct InputError;

#[test]
fn derives_error_and_as_any_behaviors() {
    let point = Point { x: 1, y: 2 };
    let erased = point.as_any();
    assert_eq!(erased.downcast_ref(), Some(&point));
    assert!(erased.downcast_ref::<InputError>().is_none());
    assert!(RenamedCrate.as_any().is::<RenamedCrate>());

    let error = InputError;
    assert_eq!(error.to_string(), "invalid input");
    let message: String = error.into();
    assert_eq!(message, "invalid input");
    let error = InputError;
    let _: &dyn std::error::Error = &error;
    assert_eq!(std::string::ToString::to_string(&no_implicit_prelude::ErrorType), "error");
}
