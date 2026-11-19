#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Item {
    #[getter(as_cpoy)]
    value: u8,
}

fn main() {}