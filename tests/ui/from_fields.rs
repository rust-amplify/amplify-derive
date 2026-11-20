#![allow(missing_docs)]

#[derive(amplify_derive::From)]
enum Value {
    #[from("u8")]
    First(u8),
    Second(#[from(u8, u16)] u32, #[from = 1] u64),
    #[from]
    Third(u8, u16),
}

fn main() {}
