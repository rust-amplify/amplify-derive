#![allow(missing_docs)]

#[derive(amplify_derive::Display)]
#[display("constant")]
struct Tuple(#[display(separator = 5)] Vec<u8>);

#[derive(amplify_derive::Display)]
enum Variants {
    #[display("constant")]
    Named {
        #[display(seperator = ", ")]
        items: Vec<u8>,
    },
}

fn main() {}
