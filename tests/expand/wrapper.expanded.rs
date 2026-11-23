#[automatically_derived]
impl ::amplify::Wrapper for Number {
    type Inner = i64;
    #[inline]
    fn from_inner(__amplify_inner: Self::Inner) -> Self {
        <Self as ::core::convert::From<Self::Inner>>::from(__amplify_inner)
    }
    #[inline]
    fn as_inner(&self) -> &Self::Inner {
        &self.0
    }
    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
#[automatically_derived]
impl ::core::convert::From<Number> for i64 {
    #[inline]
    fn from(__amplify_wrapped: Number) -> Self {
        __amplify_wrapped.0
    }
}
#[automatically_derived]
impl ::core::convert::AsRef<i64> for Number {
    #[inline]
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}
#[automatically_derived]
impl ::core::borrow::Borrow<i64> for Number {
    #[inline]
    fn borrow(&self) -> &i64 {
        &self.0
    }
}
#[automatically_derived]
impl ::core::str::FromStr for Number {
    type Err = <i64 as ::core::str::FromStr>::Err;
    #[inline]
    fn from_str(__amplify_s: &str) -> ::core::result::Result<Self, Self::Err> {
        <i64 as ::core::str::FromStr>::from_str(__amplify_s)
            .map(<Self as ::core::convert::From<i64>>::from)
    }
}
#[automatically_derived]
impl ::core::fmt::Display for Number {
    #[inline]
    fn fmt(
        &self,
        __amplify_formatter: &mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        ::core::fmt::Display::fmt(&self.0, __amplify_formatter)
    }
}
#[automatically_derived]
impl ::core::ops::Deref for Number {
    type Target = i64;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[automatically_derived]
impl ::core::ops::Neg for Number {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            0: ::core::ops::Neg::neg(self.0),
        }
    }
}
#[automatically_derived]
impl ::core::ops::Add for Number {
    type Output = Self;
    #[inline]
    fn add(self, __amplify_rhs: Self) -> Self {
        Self {
            0: ::core::ops::Add::add(self.0, __amplify_rhs.0),
        }
    }
}
#[automatically_derived]
impl ::amplify::WrapperMut for Number {
    #[inline]
    fn as_inner_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::convert::AsMut<i64> for Number {
    #[inline]
    fn as_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::borrow::BorrowMut<i64> for Number {
    #[inline]
    fn borrow_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::ops::DerefMut for Number {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::ops::AddAssign for Number {
    #[inline]
    fn add_assign(&mut self, __amplify_rhs: Self) {
        ::core::ops::AddAssign::add_assign(&mut self.0, __amplify_rhs.0)
    }
}
#[automatically_derived]
impl ::amplify::Wrapper for Indexed {
    type Inner = Vec<u8>;
    #[inline]
    fn from_inner(__amplify_inner: Self::Inner) -> Self {
        <Self as ::core::convert::From<Self::Inner>>::from(__amplify_inner)
    }
    #[inline]
    fn as_inner(&self) -> &Self::Inner {
        &self.0
    }
    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
#[automatically_derived]
impl ::core::convert::From<Indexed> for Vec<u8> {
    #[inline]
    fn from(__amplify_wrapped: Indexed) -> Self {
        __amplify_wrapped.0
    }
}
#[automatically_derived]
impl ::core::convert::AsRef<Vec<u8>> for Indexed {
    #[inline]
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}
#[automatically_derived]
impl ::core::borrow::Borrow<Vec<u8>> for Indexed {
    #[inline]
    fn borrow(&self) -> &Vec<u8> {
        &self.0
    }
}
#[automatically_derived]
impl ::core::ops::Index<usize> for Indexed {
    type Output = <Vec<u8> as ::core::ops::Index<usize>>::Output;
    #[inline]
    fn index(&self, __amplify_index: usize) -> &Self::Output {
        <Vec<u8> as ::core::ops::Index<usize>>::index(&self.0, __amplify_index)
    }
}
#[automatically_derived]
impl ::core::ops::Index<::core::ops::Range<usize>> for Indexed {
    type Output = <Vec<u8> as ::core::ops::Index<::core::ops::Range<usize>>>::Output;
    #[inline]
    fn index(&self, __amplify_index: ::core::ops::Range<usize>) -> &Self::Output {
        <Vec<
            u8,
        > as ::core::ops::Index<
            ::core::ops::Range<usize>,
        >>::index(&self.0, __amplify_index)
    }
}
#[automatically_derived]
impl ::amplify::WrapperMut for Indexed {
    #[inline]
    fn as_inner_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::convert::AsMut<Vec<u8>> for Indexed {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::borrow::BorrowMut<Vec<u8>> for Indexed {
    #[inline]
    fn borrow_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
#[automatically_derived]
impl ::core::ops::IndexMut<usize> for Indexed {
    #[inline]
    fn index_mut(&mut self, __amplify_index: usize) -> &mut Self::Output {
        <Vec<
            u8,
        > as ::core::ops::IndexMut<usize>>::index_mut(&mut self.0, __amplify_index)
    }
}
#[automatically_derived]
impl<T, U> ::renamed::Wrapper for Generic<T, U>
where
    U: Copy + Default,
{
    type Inner = Vec<T>;
    #[inline]
    fn from_inner(__amplify_inner: Self::Inner) -> Self {
        <Self as ::core::convert::From<Self::Inner>>::from(__amplify_inner)
    }
    #[inline]
    fn as_inner(&self) -> &Self::Inner {
        &self.inner
    }
    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.inner
    }
}
#[automatically_derived]
impl<T, U> ::core::convert::From<Generic<T, U>> for Vec<T>
where
    U: Copy + Default,
{
    #[inline]
    fn from(__amplify_wrapped: Generic<T, U>) -> Self {
        __amplify_wrapped.inner
    }
}
#[automatically_derived]
impl<T, U> ::core::convert::AsRef<Vec<T>> for Generic<T, U>
where
    U: Copy + Default,
{
    #[inline]
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}
#[automatically_derived]
impl<T, U> ::core::convert::AsRef<[u8]> for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::convert::AsRef<[u8]>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        ::core::convert::AsRef::<[u8]>::as_ref(&self.inner)
    }
}
#[automatically_derived]
impl<T, U> ::core::borrow::Borrow<[u8]> for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::borrow::Borrow<[u8]>,
{
    #[inline]
    fn borrow(&self) -> &[u8] {
        ::core::borrow::Borrow::<[u8]>::borrow(&self.inner)
    }
}
#[automatically_derived]
impl<T, U> ::renamed::hex::FromHex for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::renamed::hex::FromHex,
{
    #[inline]
    fn from_byte_iter<__AmplifyIter>(
        __amplify_iter: __AmplifyIter,
    ) -> ::core::result::Result<Self, ::renamed::hex::Error>
    where
        __AmplifyIter: ::core::iter::Iterator<
                Item = ::core::result::Result<u8, ::renamed::hex::Error>,
            > + ::core::iter::ExactSizeIterator + ::core::iter::DoubleEndedIterator,
    {
        <Vec<T> as ::renamed::hex::FromHex>::from_byte_iter(__amplify_iter)
            .map(<Self as ::core::convert::From<Vec<T>>>::from)
    }
}
#[automatically_derived]
impl<T, U> ::core::fmt::LowerHex for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::fmt::LowerHex,
{
    #[inline]
    fn fmt(
        &self,
        __amplify_formatter: &mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        ::core::fmt::LowerHex::fmt(&self.inner, __amplify_formatter)
    }
}
#[automatically_derived]
impl<T, U> ::core::ops::BitAnd for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::ops::BitAnd<Output = Vec<T>>,
{
    type Output = Self;
    #[inline]
    fn bitand(self, __amplify_rhs: Self) -> Self {
        Self {
            inner: ::core::ops::BitAnd::bitand(self.inner, __amplify_rhs.inner),
            ..self
        }
    }
}
#[automatically_derived]
impl<T, U> ::renamed::WrapperMut for Generic<T, U>
where
    U: Copy + Default,
{
    #[inline]
    fn as_inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }
}
#[automatically_derived]
impl<T, U> ::core::convert::AsMut<[u8]> for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::convert::AsMut<[u8]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        ::core::convert::AsMut::<[u8]>::as_mut(&mut self.inner)
    }
}
#[automatically_derived]
impl<T, U> ::core::borrow::BorrowMut<[u8]> for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::borrow::BorrowMut<[u8]>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        ::core::borrow::BorrowMut::<[u8]>::borrow_mut(&mut self.inner)
    }
}
#[automatically_derived]
impl<T, U> ::core::ops::ShlAssign for Generic<T, U>
where
    U: Copy + Default,
    Vec<T>: ::core::ops::ShlAssign,
{
    #[inline]
    fn shl_assign(&mut self, __amplify_rhs: Self) {
        ::core::ops::ShlAssign::shl_assign(&mut self.inner, __amplify_rhs.inner)
    }
}
#[automatically_derived]
impl<'a, T> ::amplify::Wrapper for Param<'a, T> {
    type Inner = &'a T;
    #[inline]
    fn from_inner(__amplify_inner: Self::Inner) -> Self {
        <Self as ::core::convert::From<Self::Inner>>::from(__amplify_inner)
    }
    #[inline]
    fn as_inner(&self) -> &Self::Inner {
        &self.0
    }
    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
#[automatically_derived]
impl<'a, T> ::core::convert::AsRef<&'a T> for Param<'a, T> {
    #[inline]
    fn as_ref(&self) -> &&'a T {
        &self.0
    }
}
#[automatically_derived]
impl<'a, T> ::core::borrow::Borrow<&'a T> for Param<'a, T> {
    #[inline]
    fn borrow(&self) -> &&'a T {
        &self.0
    }
}
