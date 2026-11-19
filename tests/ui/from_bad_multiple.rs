#![allow(missing_docs)]

#[derive(amplify_derive::From)]
#[from(u8, u16)]
struct Item(u8);

fn main() {}