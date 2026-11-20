#[derive(Wrapper, WrapperMut)]
#[wrapper(FromStr, Display, Deref, Index, IndexRange, Neg, Add)]
#[wrapper_mut(DerefMut, IndexMut, AddAssign)]
struct Number(i64);

#[derive(Wrapper, WrapperMut)]
#[amplify_crate(::renamed)]
#[wrapper(NoRefs, AsRef, AsSlice, BorrowSlice, FromHex, LowerHex, BitAnd)]
#[wrapper_mut(NoRefs, AsSliceMut, BorrowSliceMut, ShlAssign)]
struct Generic<T, U>
where U: Copy
{
    #[wrap]
    inner: Vec<T>,
    other: U,
}

#[derive(Wrapper)]
struct Param<'a, T>(&'a T);
