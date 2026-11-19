#![allow(missing_docs)]

#[derive(amplify_derive::Getters)]
struct Flags {
    #[getter(all, as_mut)]
    #[getter(skip = true)]
    value: u8,
}

#[derive(amplify_derive::Getters)]
struct Values {
    #[getter(base_name, "name")]
    value: u8,
}

fn main() {}
