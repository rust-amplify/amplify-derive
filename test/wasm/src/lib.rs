//! A `no_std` crate which uses every derive macro of `amplify_derive`.
//!
//! Generated code which names `std` does not compile here, so the crate shows
//! that the macros need only `core` and `alloc`. Its tests run natively and,
//! through `wasm-bindgen-test`, on `wasm32-unknown-unknown`.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use amplify_derive::{AsAny, Display, Error, From, Getters, Wrapper, WrapperMut};

/// An error which is displayed through its doc comments.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Failure {
    /// value {0} is out of range
    #[from]
    OutOfRange(u8),

    /// {reason} at {position}
    Parse {
        /// Where the input is wrong.
        position: usize,
        /// What is wrong with it.
        #[from]
        reason: String,
    },

    /// unknown failure
    #[from(())]
    Unknown,
}

/// A structure with getters.
#[derive(Clone, PartialEq, Eq, Default, Debug, Display, From, Getters, AsAny)]
#[display("({x}, {y})", alt = "{x}:{y}")]
#[getter(as_copy)]
pub struct Point {
    #[from]
    x: i32,
    #[getter(as_mut)]
    y: i32,
}

/// A wrapper around a number.
#[derive(Wrapper, WrapperMut, Clone, Copy, PartialEq, Eq, Debug, Display, From)]
#[display(inner)]
#[wrapper(FromStr, LowerHex, Add, Not)]
#[wrapper_mut(AddAssign)]
pub struct Number(u32);

/// A wrapper around a generic collection.
#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, Debug, Display, From)]
#[display("{0:?}")]
#[wrapper(Deref, Index, RangeOps)]
#[wrapper_mut(DerefMut, IndexMut)]
pub struct List<T>(Vec<T>);

/// A structure with a joined field.
#[derive(Clone, PartialEq, Eq, Debug, Display, From)]
#[display("{name}: {items}")]
pub struct Named {
    name: &'static str,
    #[from]
    #[display(separator = ", ")]
    items: Vec<&'static str>,
}

impl Default for Named {
    fn default() -> Self {
        Self {
            name: "items",
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::{format, vec};
    use core::str::FromStr;

    use amplify::{AsAny, Wrapper, WrapperMut};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn errors_are_displayed_and_converted() {
        assert_eq!(Failure::from(7).to_string(), "value 7 is out of range");
        assert_eq!(Failure::from(String::from("no digit")).to_string(), "no digit at 0");
        assert_eq!(String::from(Failure::from(())), "unknown failure");
        let _: &dyn core::error::Error = &Failure::Unknown;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn structures_get_display_getters_and_conversions() {
        let mut point = Point::from(1);
        *point.y_mut() = 2;
        assert_eq!((point.x(), point.y()), (1, 2));
        assert_eq!(format!("{point} {point:#}"), "(1, 2) 1:2");
        assert_eq!(point.as_any().downcast_ref::<Point>(), Some(&point));
        assert_eq!(Named::from(vec!["a", "b"]).to_string(), "items: a, b");
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn wrappers_pass_traits_through() {
        let mut number = Number::from_str("254").expect("a number");
        number += Number::from(1);
        assert_eq!(format!("{number} {number:x}"), "255 ff");
        assert_eq!((!number + Number::from(1)).into_inner(), u32::MAX - 254);
        *number.as_inner_mut() = 0;
        assert_eq!(number.as_inner(), &0);

        let mut list = List::from(vec!["a", "b", "c"]);
        list[0] = "d";
        list.push("e");
        assert_eq!(list[1..3], ["b", "c"]);
        assert_eq!(list.to_string(), r#"["d", "b", "c", "e"]"#);
    }
}
