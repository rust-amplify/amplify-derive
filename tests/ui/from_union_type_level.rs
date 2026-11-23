#![allow(missing_docs)]

#[derive(amplify_derive::From)]
#[from(u32)]
union Either {
    int: u32,
    float: f32,
}

fn main() {}
