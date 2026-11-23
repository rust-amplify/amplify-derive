#![allow(missing_docs)]

mod inner {
    #[derive(amplify_derive::Getters, Default)]
    pub struct Secret {
        #[getter(self, as_copy)]
        value: u8,
    }
}

#[derive(amplify_derive::Getters)]
#[getter(crate, self)]
struct ConflictingStruct {
    value: u8,
}

#[derive(amplify_derive::Getters)]
struct ConflictingField {
    #[getter(self, super, crate = true)]
    value: u8,
}

fn main() {
    let _ = inner::Secret::default().value();
}
