#[automatically_derived]
impl<T: Debug> ::core::error::Error for Failure<T>
where
    Self: ::core::fmt::Debug + ::core::fmt::Display,
{}
const _: () = {
    extern crate alloc;
    #[automatically_derived]
    impl<T: Debug> ::core::convert::From<Failure<T>> for alloc::string::String
    where
        Failure<T>: ::core::fmt::Display,
    {
        fn from(__amplify_err: Failure<T>) -> Self {
            alloc::string::ToString::to_string(&__amplify_err)
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
impl<T> renamed::AsAny for Renamed<T>
where
    Self: 'static,
{
    fn as_any(&self) -> &dyn ::core::any::Any {
        self as &dyn ::core::any::Any
    }
}
