#![allow(missing_docs)]

#[derive(amplify_derive::Display)]
#[display(inner)]
union Value {
    field: u32,
}

fn main() {}
