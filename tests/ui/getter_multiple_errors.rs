#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Item {
    #[getter(as_cpoy, as_mut = 5)]
    value: u8,
}

fn main() {}