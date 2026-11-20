#[automatically_derived]
impl ::core::fmt::Display for Point {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let Self { x: __amplify_field_0, y: __amplify_field_1, .. } = self;
        if f.alternate() {
            ::core::write!(f, "{x}:{y}", x = __amplify_field_0, y = __amplify_field_1)
        } else {
            ::core::write!(f, "({x}, {y})", x = __amplify_field_0, y = __amplify_field_1)
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Tuple {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let Self { 0: __amplify_field_0, .. } = self;
        if f.alternate() {
            ::core::write!(f, "{_0:#}", _0 = __amplify_field_0)
        } else {
            ::core::write!(f, "{_0}", _0 = __amplify_field_0)
        }
    }
}
#[automatically_derived]
impl<T> ::core::fmt::Display for Debugged<T>
where
    Self: ::core::fmt::Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(self, f)
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Shout {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Display::fmt(&Shout::print(self), f)
    }
}
#[automatically_derived]
impl<T, U> ::core::fmt::Display for Generic<T, U>
where
    U: ::core::fmt::Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let Self {
            items: __amplify_field_0,
            other: __amplify_field_1,
            width: __amplify_field_2,
            ..
        } = self;
        ::core::write!(
            f, "[{_0}] {_1:?} {width:>_2$}", _0 = __amplify_field_0.join(", "), _1 =
            __amplify_field_1, width = __amplify_field_2, _2 = __amplify_field_2
        )
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Documented {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::A { .. } => ::core::write!(f, "Letter A"),
            Self::Letter { 0: __amplify_field_0, .. } => {
                ::core::write!(f, "Letter {_0}", _0 = __amplify_field_0)
            }
            Self::Custom { name: __amplify_field_0, .. } => {
                if f.alternate() {
                    ::core::write!(f, "{name}", name = __amplify_field_0)
                } else {
                    ::core::write!(f, "custom {name}", name = __amplify_field_0)
                }
            }
            Self::Lower { 0: __amplify_field_0, 1: __amplify_field_1, .. } => {
                ::core::write!(
                    f, "lower({_0}, {_1})", _0 = __amplify_field_0, _1 =
                    __amplify_field_1
                )
            }
            Self::Inner { 0: __amplify_field_0, .. } => {
                if f.alternate() {
                    ::core::write!(f, "{_0:#}", _0 = __amplify_field_0)
                } else {
                    ::core::write!(f, "{_0}", _0 = __amplify_field_0)
                }
            }
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Plain {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Unit { .. } => ::core::write!(f, "Unit"),
            Self::Tuple { .. } => ::core::write!(f, "Tuple(..)"),
            Self::Named { .. } => ::core::write!(f, "Named {{ .. }}"),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Never {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match *self {}
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Bits {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::write!(f, "BITS")
    }
}
