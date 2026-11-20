#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Item {
    #[getter(as_cpoy)]
    first: u8,
    second: u8,
    #[getter(prefix = 5)]
    third: u8,
    #[getter(base_name = "self")]
    fourth: u8,
}

fn main() {}
