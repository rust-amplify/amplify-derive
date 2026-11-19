#![allow(missing_docs)]

#[derive(amplify_derive::Wrapper)]
struct Item(#[wrap(yes)] u8);

fn main() {}