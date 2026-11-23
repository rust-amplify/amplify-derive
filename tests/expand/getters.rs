#[derive(Getters)]
#[getter(as_mut, prefix = "get_")]
struct One<T> {
    /// Contains byte representation of the data
    #[getter(all, base_name = "bytes")]
    vec: Vec<u8>,
    defaults: T,
    #[getter(as_copy)]
    pub flag: bool,
    #[getter(skip)]
    pub(self) field: u8,
    #[getter(as_copy, as_ref)]
    r#type: u8,
}

#[derive(Getters)]
#[getter(as_clone, crate)]
struct Two {
    name: String,
    #[getter(self, as_ref = "_borrowed", as_mut = "_changed")]
    list: Vec<u8>,
    #[getter(super, as_copy)]
    count: u8,
}
