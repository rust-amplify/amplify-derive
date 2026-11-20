#![allow(missing_docs)]

#[derive(amplify_derive::From)]
struct Pair(u32, i16);

#[derive(amplify_derive::From)]
enum Variants {
    Unit,
    Tuple(u8),
}

fn main() {}
