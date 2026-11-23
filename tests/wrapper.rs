#![allow(missing_docs)]

use amplify::{Wrapper as WrapperTrait, WrapperMut as WrapperMutTrait};
use amplify_derive::{From, Wrapper, WrapperMut};

mod no_implicit_prelude {
    #![no_implicit_prelude]

    #[derive(::amplify_derive::Wrapper, ::amplify_derive::WrapperMut, ::amplify_derive::From)]
    #[amplify_crate(::amplify)]
    #[wrapper(FromStr, Display, Debug, NumberFmt, Deref, MathOps, BitOps)]
    #[wrapper_mut(DerefMut, MathAssign, BitAssign)]
    pub struct Number(pub i64);

    #[derive(::amplify_derive::Wrapper, ::amplify_derive::WrapperMut, ::amplify_derive::From)]
    #[wrapper(FromHex, Deref, AsSlice, BorrowSlice, Index, RangeOps)]
    #[wrapper_mut(DerefMut, AsSliceMut, BorrowSliceMut, IndexMut, RangeMut)]
    pub struct Bytes(pub ::std::vec::Vec<u8>);
}

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(
    FromStr, Display, Debug, Octal, LowerHex, UpperHex, LowerExp, UpperExp, Deref, Neg, Add, Sub,
    Mul, Div, Rem, Not, Shl, Shr, BitAnd, BitOr, BitXor
)]
#[wrapper_mut(
    DerefMut,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    ShlAssign,
    ShrAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign
)]
struct Number(i64);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(
    AsSlice,
    BorrowSlice,
    Deref,
    Index,
    IndexRange,
    IndexFull,
    IndexFrom,
    IndexTo,
    IndexInclusive,
    IndexToInclusive
)]
#[wrapper_mut(
    BorrowSliceMut,
    IndexMut,
    IndexRangeMut,
    IndexFullMut,
    IndexFromMut,
    IndexToMut,
    IndexInclusiveMut,
    IndexToInclusiveMut
)]
struct Bytes(Vec<u8>);

#[derive(Wrapper, From)]
#[wrapper(FromHex)]
struct HexByte(Vec<u8>);

#[derive(Wrapper, From)]
#[wrapper(Hex)]
struct HexNumber(HexData);

#[derive(Wrapper, From)]
#[wrapper(Exp)]
struct Exponential(f64);

#[derive(Wrapper, From)]
#[wrapper(NumberFmt)]
struct NumberFormat(u64);

#[derive(Wrapper, From)]
#[wrapper(RangeOps)]
struct RangeBytes(Vec<u8>);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(RangeOps, Deref)]
#[wrapper_mut(RangeMut)]
struct MutableRange(Vec<u8>);

#[derive(Wrapper, From)]
#[wrapper(MathOps)]
struct MathNumber(i32);

#[derive(Wrapper, From)]
#[wrapper(BoolOps)]
struct BoolNumber(u8);

#[derive(Wrapper, From)]
#[wrapper(BitOps)]
struct BitNumber(u8);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper_mut(MathAssign)]
struct MathAssigned(i32);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper_mut(BoolAssign)]
struct BoolAssigned(u8);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper_mut(BitAssign)]
struct BitAssigned(u8);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(AsSlice, BorrowSlice)]
#[wrapper_mut(AsSliceMut, BorrowSliceMut)]
struct SliceAccess(Vec<u8>);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(NoRefs)]
#[wrapper_mut(NoRefs)]
struct NoRefNumber(Vec<u8>);

#[derive(Wrapper, WrapperMut, From)]
struct Bounded<T: Clone>(T);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(Index)]
#[wrapper_mut(IndexMut)]
struct Defaulted<T = u8>(Vec<T>);

struct DifferentRange(Vec<u8>);

impl std::ops::Index<std::ops::RangeInclusive<usize>> for DifferentRange {
    type Output = [u8];

    fn index(&self, range: std::ops::RangeInclusive<usize>) -> &Self::Output { &self.0[range] }
}

impl std::ops::Index<std::ops::RangeToInclusive<usize>> for DifferentRange {
    type Output = u8;

    fn index(&self, range: std::ops::RangeToInclusive<usize>) -> &Self::Output {
        &self.0[range.end]
    }
}

#[derive(Wrapper, From)]
#[wrapper(IndexToInclusive)]
struct Inclusive(DifferentRange);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(Index)]
#[wrapper_mut(AsSliceMut, IndexMut)]
struct MutableBytes(Vec<u8>);

#[derive(Wrapper, From, Default)]
#[wrapper(Add)]
struct MultiField {
    #[wrap]
    #[from]
    inner: i32,
    other: i32,
}

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(AsRef, Borrow)]
#[wrapper_mut(AsMut, BorrowMut)]
struct NamedDefaults(u8);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(NoRefs, AsRef)]
#[wrapper_mut(NoRefs, AsMut)]
struct OneOfEachPair(u8);

// The derives leave these out, or the impls would conflict.
impl std::borrow::Borrow<u8> for OneOfEachPair {
    fn borrow(&self) -> &u8 { &0 }
}

impl std::borrow::BorrowMut<u8> for OneOfEachPair {
    fn borrow_mut(&mut self) -> &mut u8 { &mut self.0 }
}

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(BoolOps, BitOps, Not, Exp, NumberFmt, Add)]
#[wrapper_mut(BoolAssign, BitAssign, ShlAssign, AddAssign)]
struct Overlapping(u8);

#[derive(Wrapper, From)]
struct Boxed<T>(Box<T>);

#[derive(Wrapper, From)]
struct Borrowed<'a, T>(&'a T);

#[derive(Wrapper, WrapperMut, From)]
struct BorrowedMut<'a, T>(&'a mut T);

#[derive(Wrapper, From)]
struct Pinned<T>(std::pin::Pin<Box<T>>);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(FromStr, Display, Debug, LowerHex, Neg, Add, Shl)]
#[wrapper_mut(AddAssign)]
struct GenericNumber<T>(T);

#[derive(Wrapper, WrapperMut, From)]
#[wrapper(Deref, AsSlice, BorrowSlice, Index, RangeOps)]
#[wrapper_mut(DerefMut, AsSliceMut, BorrowSliceMut, IndexMut, RangeMut)]
struct GenericBytes<T>(T);

#[derive(Wrapper, From)]
#[wrapper(FromHex)]
struct GenericHex<I>(I);

mod hygiene {
    use amplify_derive::{From, Wrapper, WrapperMut};

    #[allow(dead_code)]
    mod amplify {}

    #[allow(non_upper_case_globals, dead_code)]
    const inner: i32 = 0;
    #[allow(non_upper_case_globals, dead_code)]
    const wrapped: i32 = 0;
    #[allow(non_upper_case_globals, dead_code)]
    const s: &str = "";
    #[allow(non_upper_case_globals, dead_code)]
    const iter: u8 = 0;
    #[allow(non_upper_case_globals, dead_code)]
    const formatter: u8 = 0;
    #[allow(non_upper_case_globals, dead_code)]
    const index: usize = 0;
    #[allow(non_upper_case_globals, dead_code)]
    const rhs: i32 = 0;

    #[derive(Wrapper, WrapperMut, From)]
    #[wrapper(FromStr, Display, Debug, Add)]
    #[wrapper_mut(AddAssign)]
    pub struct CleanNum(pub i32);

    #[derive(Wrapper, WrapperMut, From)]
    #[wrapper(FromHex, Index)]
    #[wrapper_mut(IndexMut)]
    pub struct CleanBytes(pub Vec<u8>);
}

mod result_alias {
    #[allow(dead_code)]
    type Result<T, E> = std::result::Result<T, (E, ())>;

    #[derive(amplify_derive::Wrapper, amplify_derive::From)]
    #[wrapper(FromStr)]
    pub struct Number(pub i32);
}

#[test]
fn generated_code_does_not_depend_on_the_prelude() {
    use std::str::FromStr;

    use no_implicit_prelude::{Bytes, Number};

    let mut number = Number::from_str("6").unwrap();
    number += Number::from(2);
    assert_eq!((number + Number::from(1)).0, 9);
    assert_eq!(format!("{:x}", Number::from(255)), "ff");

    let mut bytes =
        <Bytes as amplify::hex::FromHex>::from_byte_iter(vec![Ok(1), Ok(2)].into_iter()).unwrap();
    bytes[0] = 3;
    bytes[1..].copy_from_slice(&[4]);
    assert_eq!(&bytes[..], &[3, 4]);
    assert_eq!(std::convert::AsRef::<[u8]>::as_ref(&bytes), &[3, 4]);
    assert_eq!(Bytes::from(vec![5]).into_inner(), vec![5]);
}

#[derive(Default)]
struct HexData(Vec<u8>);

impl std::fmt::LowerHex for HexData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl std::fmt::UpperHex for HexData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl amplify::hex::FromHex for HexData {
    fn from_byte_iter<I>(iter: I) -> std::result::Result<Self, amplify::hex::Error>
    where I: Iterator<Item = std::result::Result<u8, amplify::hex::Error>>
            + ExactSizeIterator
            + DoubleEndedIterator {
        <Vec<u8> as amplify::hex::FromHex>::from_byte_iter(iter).map(Self)
    }
}

#[test]
fn wraps_formatting_references_ranges_and_operators() {
    use std::str::FromStr;

    let number = Number::from(42);
    assert_eq!(number.as_inner(), &42);
    assert_eq!(format!("{number}"), "42");
    assert_eq!(format!("{number:?}"), "42");
    assert_eq!(format!("{number:o}"), "52");
    assert_eq!(format!("{number:x}"), "2a");
    assert_eq!(format!("{number:X}"), "2A");
    assert_eq!(format!("{number:e}"), "4.2e1");
    assert_eq!(format!("{number:E}"), "4.2E1");
    assert_eq!(Number::from_str("9").unwrap().0, 9);
    assert_eq!(result_alias::Number::from_str("9").unwrap().0, 9);
    assert_eq!(std::convert::AsRef::<i64>::as_ref(&number), &42);
    assert_eq!(std::borrow::Borrow::<i64>::borrow(&number), &42);
    assert_eq!((-Number::from(3)).0, -3);
    assert_eq!((Number::from(3) + Number::from(4)).0, 7);
    assert_eq!((Number::from(9) - Number::from(4)).0, 5);
    assert_eq!((Number::from(3) * Number::from(4)).0, 12);
    assert_eq!((Number::from(12) / Number::from(3)).0, 4);
    assert_eq!((Number::from(13) % Number::from(5)).0, 3);
    assert_eq!((!Number::from(0)).0, !0);
    assert_eq!((Number::from(1) << Number::from(2)).0, 4);
    assert_eq!((Number::from(8) >> Number::from(2)).0, 2);
    assert_eq!((Number::from(6) & Number::from(3)).0, 2);
    assert_eq!((Number::from(4) | Number::from(1)).0, 5);
    assert_eq!((Number::from(6) ^ Number::from(3)).0, 5);

    let mut number = Number::from(20);
    number += Number::from(2);
    number -= Number::from(1);
    number *= Number::from(3);
    number /= Number::from(3);
    number %= Number::from(7);
    number <<= Number::from(1);
    number >>= Number::from(1);
    number &= Number::from(7);
    number |= Number::from(8);
    number ^= Number::from(1);
    assert_eq!(number.0, 9);
    assert_eq!(*number, 9);
    *std::convert::AsMut::<i64>::as_mut(&mut number) = 7;
    *std::borrow::BorrowMut::<i64>::borrow_mut(&mut number) = 8;
    *number.as_inner_mut() = 9;
    *number = 10;
    assert_eq!(number.0, 10);

    let mut bytes = Bytes::from(vec![1, 2, 3, 4]);
    assert_eq!(bytes[1], 2);
    assert_eq!(&bytes[1..3], &[2, 3]);
    assert_eq!(&bytes[..2], &[1, 2]);
    assert_eq!(&bytes[2..], &[3, 4]);
    assert_eq!(&bytes[..], &[1, 2, 3, 4]);
    assert_eq!(&bytes[1..=2], &[2, 3]);
    assert_eq!(std::convert::AsRef::<Vec<u8>>::as_ref(&bytes), &vec![1, 2, 3, 4]);
    assert_eq!(bytes.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(std::borrow::Borrow::<[u8]>::borrow(&bytes), &[1, 2, 3, 4]);
    bytes[1] = 7;
    bytes[1..2].copy_from_slice(&[8]);
    bytes[..1].copy_from_slice(&[9]);
    bytes[1..].copy_from_slice(&[3, 4, 5]);
    bytes[..].copy_from_slice(&[1, 2, 3, 4]);
    bytes[0..=0].copy_from_slice(&[6]);
    bytes[1..=2].copy_from_slice(&[7, 8]);
    assert_eq!(&*bytes, &[6, 7, 8, 4]);

    assert_eq!(format!("{:x}", HexNumber::from(HexData(vec![255]))), "ff");
    assert_eq!(format!("{:e}", Exponential::from(2.5)), "2.5e0");
    assert_eq!(format!("{:o}", NumberFormat::from(8)), "10");
    let hex_byte =
        <HexByte as amplify::hex::FromHex>::from_byte_iter(std::iter::once(Ok(255))).unwrap();
    assert_eq!(hex_byte.0, vec![255]);
    let hex_group =
        <HexNumber as amplify::hex::FromHex>::from_byte_iter(vec![Ok(0xab), Ok(0xcd)].into_iter())
            .unwrap();
    assert_eq!(format!("{hex_group:x}"), "abcd");

    let range = RangeBytes::from(vec![2, 4, 6]);
    assert_eq!(&range[1..], &[4, 6]);
    let mut mutable = MutableRange::from(vec![1, 2, 3]);
    mutable[1..].copy_from_slice(&[8, 9]);
    assert_eq!(&*mutable, &[1, 8, 9]);

    assert_eq!((MathNumber::from(2) + MathNumber::from(3)).0, 5);
    assert_eq!((!BoolNumber::from(0)).0, !0);
    assert_eq!((BitNumber::from(1) << BitNumber::from(2)).0, 4);
    let mut math = MathAssigned::from(10);
    math += MathAssigned::from(2);
    math -= MathAssigned::from(1);
    math *= MathAssigned::from(3);
    math /= MathAssigned::from(3);
    math %= MathAssigned::from(5);
    assert_eq!(math.0, 1);
    let mut boolean = BoolAssigned::from(7);
    boolean &= BoolAssigned::from(3);
    boolean |= BoolAssigned::from(8);
    boolean ^= BoolAssigned::from(2);
    assert_eq!(boolean.0, 9);
    let mut bits = BitAssigned::from(1);
    bits &= BitAssigned::from(3);
    bits |= BitAssigned::from(4);
    bits ^= BitAssigned::from(2);
    bits <<= BitAssigned::from(1);
    bits >>= BitAssigned::from(1);
    assert_eq!(bits.0, 7);

    let mut slice_access = SliceAccess::from(vec![1, 2]);
    assert_eq!(std::convert::AsRef::<[u8]>::as_ref(&slice_access), &[1, 2]);
    assert_eq!(std::borrow::Borrow::<[u8]>::borrow(&slice_access), &[1, 2]);
    std::convert::AsMut::<[u8]>::as_mut(&mut slice_access)[0] = 3;
    std::borrow::BorrowMut::<[u8]>::borrow_mut(&mut slice_access)[1] = 4;
    assert_eq!(slice_access.0, vec![3, 4]);

    let mut bounded = Bounded::from(String::from("value"));
    bounded.as_inner_mut().push('!');
    assert_eq!(bounded.as_inner(), "value!");

    let mut defaulted = Defaulted::from(vec![2u8, 3]);
    assert_eq!(defaulted[0], 2);
    defaulted[1] = 4;
    assert_eq!(defaulted[1], 4);

    let inclusive = Inclusive::from(DifferentRange(vec![4, 5, 6]));
    assert_eq!(inclusive[..=2], 6);

    let mut mutable_bytes = MutableBytes::from(vec![1, 2]);
    mutable_bytes[0] = 9;
    assert_eq!(mutable_bytes.0, vec![9, 2]);

    let left = MultiField { inner: 4, other: 7 };
    let sum = left + MultiField { inner: 3, other: 9 };
    assert_eq!(sum.inner, 7);
    assert_eq!(sum.other, 7);

    let mut no_refs = NoRefNumber::from(vec![5, 6]);
    no_refs.as_inner_mut()[0] = 7;
    assert_eq!(no_refs.as_inner(), &vec![7, 6]);
}

#[test]
fn default_traits_can_be_named_and_overlapping_names_derive_once() {
    let mut named = NamedDefaults::from(1);
    assert_eq!(std::convert::AsRef::<u8>::as_ref(&named), &1);
    assert_eq!(std::borrow::Borrow::<u8>::borrow(&named), &1);
    *std::convert::AsMut::<u8>::as_mut(&mut named) = 2;
    *std::borrow::BorrowMut::<u8>::borrow_mut(&mut named) = 3;
    assert_eq!(named.0, 3);

    let mut one = OneOfEachPair::from(1);
    assert_eq!(std::convert::AsRef::<u8>::as_ref(&one), &1);
    assert_eq!(std::borrow::Borrow::<u8>::borrow(&one), &0);
    *std::convert::AsMut::<u8>::as_mut(&mut one) = 2;
    assert_eq!(one.0, 2);

    let mut overlapping = !Overlapping::from(0b1010) & Overlapping::from(0b1111);
    overlapping <<= Overlapping::from(1);
    overlapping |= Overlapping::from(1);
    overlapping += Overlapping::from(1);
    assert_eq!((overlapping + Overlapping::from(1)).0, 0b1101);
    assert_eq!(format!("{:x} {:e}", Overlapping::from(255), Overlapping::from(2)), "ff 2e0");
}

#[test]
fn wraps_type_parameters_behind_fundamental_types() {
    assert_eq!(*Boxed::from(Box::new(1)).into_inner(), 1);
    assert_eq!(Borrowed::from(&1).into_inner(), &1);
    let mut value = 1;
    let mut borrowed = BorrowedMut::from(&mut value);
    **borrowed.as_inner_mut() = 2;
    assert_eq!(value, 2);
    assert_eq!(*Pinned::from(Box::pin(1)).into_inner(), 1);
}

#[test]
fn derives_traits_for_generic_inner_types() {
    use std::str::FromStr;

    let mut number = GenericNumber::<i32>::from_str("6").unwrap();
    number += GenericNumber::from(1);
    let number = -(number + GenericNumber::from(1)) << GenericNumber::from(1);
    assert_eq!(format!("{number} {number:?} {:x}", GenericNumber::from(255u8)), "-16 -16 ff");
    // A wrapper of a type without the trait stays usable.
    assert!(GenericNumber::from(()).as_inner() == &());

    let mut bytes = GenericBytes::from(vec![1u8, 2, 3]);
    bytes[0] = 4;
    bytes[1..].copy_from_slice(&[5, 6]);
    bytes.push(7);
    std::convert::AsMut::<[u8]>::as_mut(&mut bytes)[3] = 8;
    std::borrow::BorrowMut::<[u8]>::borrow_mut(&mut bytes)[3] += 1;
    assert_eq!(std::convert::AsRef::<[u8]>::as_ref(&bytes), &[4, 5, 6, 9]);
    assert_eq!(std::borrow::Borrow::<[u8]>::borrow(&bytes), &bytes[..]);
    assert_eq!(bytes.len(), 4);

    let hex = <GenericHex<Vec<u8>> as amplify::hex::FromHex>::from_byte_iter(
        vec![Ok(0xab), Ok(0xcd)].into_iter(),
    );
    assert_eq!(hex.unwrap().0, vec![0xab, 0xcd]);

    let mut clean_num = hygiene::CleanNum::from_str("5").unwrap();
    clean_num += hygiene::CleanNum::from(2);
    assert_eq!(format!("{}", clean_num + hygiene::CleanNum::from(1)), "8");
    let mut clean_bytes = <hygiene::CleanBytes as amplify::hex::FromHex>::from_byte_iter(
        vec![Ok(0x10), Ok(0x20)].into_iter(),
    )
    .unwrap();
    clean_bytes[0] = 0x30;
    assert_eq!(clean_bytes[0], 0x30);
}
