#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
#[getter(prefix = 5)]
struct Item {
    value: u8,
}

fn main() {}