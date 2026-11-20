#![allow(missing_docs)]

#[derive(amplify_derive::Display)]
enum Value {
    #[display("{0}", alt = 5)]
    First(u8),
    Second,
    #[display("{", alt = "}")]
    Third,
    #[display("{0}")]
    Fourth,
    #[display("{items}")]
    Fifth {
        #[display(separator)]
        items: Vec<u8>,
        #[display(sep = ", ")]
        other: Vec<u8>,
    },
}

fn main() {}
