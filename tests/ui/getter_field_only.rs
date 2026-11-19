#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
#[getter(skip, base_name = "item")]
struct Item {
    value: u8,
}

fn main() {}
