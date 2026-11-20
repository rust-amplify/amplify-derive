#![allow(missing_docs)]

#[derive(amplify_derive::Display)]
#[display("{0}")]
enum Value {
    Number(u8),
    Nothing,
}

#[derive(amplify_derive::Display)]
#[display("{x}: {1:>2$}")]
struct Point {
    x: u8,
    y: u8,
}

fn main() {}
