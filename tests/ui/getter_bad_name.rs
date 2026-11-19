#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Reserved {
    #[getter(base_name = "self")]
    value: u8,
}

#[derive(amplify_derive::Getters)]
#[getter(prefix = "get-")]
struct Malformed {
    value: u8,
}

fn main() {}
