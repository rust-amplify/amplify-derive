#[automatically_derived]
impl<T: Debug> ::core::error::Error for Failure<T> {}
const _: () = {
    extern crate alloc;
    #[automatically_derived]
    impl<T: Debug> ::core::convert::From<Failure<T>> for alloc::string::String {
        fn from(err: Failure<T>) -> Self {
            alloc::string::ToString::to_string(&err)
        }
    }
};
#[automatically_derived]
impl ::amplify::AsAny for Any {
    fn as_any(&self) -> &dyn ::core::any::Any {
        self as &dyn ::core::any::Any
    }
}
#[automatically_derived]
impl<T> renamed::AsAny for Renamed<T> {
    fn as_any(&self) -> &dyn ::core::any::Any {
        self as &dyn ::core::any::Any
    }
}
