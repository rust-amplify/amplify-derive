#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Item {
    #[getter(as_copy, as_clone)]
    value: u8,
}

fn main() {}