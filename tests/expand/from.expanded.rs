#[automatically_derived]
impl ::core::convert::From<::std::io::Error> for Unit {
    fn from(v: ::std::io::Error) -> Self {
        Self {}
    }
}
#[automatically_derived]
impl ::core::convert::From<u32> for Single {
    fn from(v: u32) -> Self {
        Self {
            0: ::core::convert::Into::into(v),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<u8> for Named {
    fn from(v: u8) -> Self {
        Self {
            ..::core::default::Default::default()
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<String> for Named {
    fn from(v: String) -> Self {
        Self {
            name: ::core::convert::Into::into(v),
            ..::core::default::Default::default()
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<u16> for Named {
    fn from(v: u16) -> Self {
        Self {
            count: ::core::convert::Into::into(v),
            ..::core::default::Default::default()
        }
    }
}
#[automatically_derived]
impl<T> ::core::convert::From<i8> for Tuple<T> {
    fn from(v: i8) -> Self {
        Self {
            0: ::core::convert::Into::into(v),
            1: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl<T> ::core::convert::From<T> for Tuple<T> {
    fn from(v: T) -> Self {
        Self {
            0: ::core::default::Default::default(),
            1: ::core::convert::Into::into(v),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<::std::io::Error> for Error {
    fn from(v: ::std::io::Error) -> Self {
        Self::Io {}
    }
}
#[automatically_derived]
impl ::core::convert::From<Unit> for Error {
    fn from(v: Unit) -> Self {
        Self::Io {}
    }
}
#[automatically_derived]
impl ::core::convert::From<::std::fmt::Error> for Error {
    fn from(v: ::std::fmt::Error) -> Self {
        Self::Format {
            0: ::core::convert::Into::into(v),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<::std::str::Utf8Error> for Error {
    fn from(v: ::std::str::Utf8Error) -> Self {
        Self::WithFields {
            details: ::core::convert::Into::into(v),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<Single> for Error {
    fn from(v: Single) -> Self {
        Self::MultipleFields {
            io: ::core::convert::Into::into(v),
            details: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<!> for Error {
    fn from(v: !) -> Self {
        Self::Never {}
    }
}
#[automatically_derived]
impl ::core::convert::From<u64> for Error {
    fn from(v: u64) -> Self {
        Self::Empty {}
    }
}
#[automatically_derived]
impl ::core::convert::From<i64> for Error {
    fn from(v: i64) -> Self {
        Self::Pair {
            0: ::core::convert::Into::into(v),
            1: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<u32> for Either {
    fn from(v: u32) -> Self {
        Self {
            int: ::core::convert::Into::into(v),
        }
    }
}
#[automatically_derived]
impl ::core::convert::From<f32> for Either {
    fn from(v: f32) -> Self {
        Self {
            float: ::core::convert::Into::into(v),
        }
    }
}
