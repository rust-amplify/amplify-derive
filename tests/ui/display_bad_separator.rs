#![allow(missing_docs)]

#[derive(amplify_derive::Display)]
#[display("items: {items}")]
struct Item {
    #[display(separator = a b)]
    items: Vec<u8>,
}

fn main() {}