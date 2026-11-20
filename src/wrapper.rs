// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at <http://www.apache.org/licenses/LICENSE-2.0>
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use amplify_syn::{AttrReq, ListReq, Value, collect_all};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, GenericArgument, Generics, Member, Path, PathArguments,
    Result, Type,
};

use crate::util::{Bounds, amplify_crate};

/// The variants of an enum by their names.
trait Names: Sized + 'static {
    const NAMES: &'static [(&'static str, Self)];
}

/// Declares an enum together with its [`Names`].
macro_rules! names {
    ($(#[$meta:meta])* enum $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        enum $name { $($variant),+ }

        impl Names for $name {
            const NAMES: &'static [(&'static str, Self)] =
                &[$((stringify!($variant), Self::$variant)),+];
        }
    };
}

names! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum Wrapper {
        NoRefs,
        // Formatting
        FromStr,
        Display,
        Debug,
        Octal,
        FromHex,
        LowerHex,
        UpperHex,
        LowerExp,
        UpperExp,
        // References
        Deref,
        AsRef,
        AsSlice,
        Borrow,
        BorrowSlice,
        // Indexes
        Index,
        IndexRange,
        IndexFull,
        IndexFrom,
        IndexTo,
        IndexInclusive,
        IndexToInclusive,
        // Arithmetics
        Neg,
        Add,
        Sub,
        Mul,
        Div,
        Rem,
        // Booleans
        Not,
        Shl,
        Shr,
        BitAnd,
        BitOr,
        BitXor,
        // Group operations
        Hex,
        Exp,
        NumberFmt,
        RangeOps,
        MathOps,
        BoolOps,
        BitOps,
    }
}

names! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum WrapperMut {
        NoRefs,
        // References
        DerefMut,
        AsMut,
        AsSliceMut,
        BorrowMut,
        BorrowSliceMut,
        // Indexes
        IndexMut,
        IndexRangeMut,
        IndexFullMut,
        IndexFromMut,
        IndexToMut,
        IndexInclusiveMut,
        IndexToInclusiveMut,
        // Arithmetics
        AddAssign,
        SubAssign,
        MulAssign,
        DivAssign,
        RemAssign,
        ShlAssign,
        ShrAssign,
        // Booleans
        BitAndAssign,
        BitOrAssign,
        BitXorAssign,
        // Group operations
        RangeMut,
        MathAssign,
        BoolAssign,
        BitAssign,
    }
}

/// The traits one of the wrapper attributes derives.
///
/// The attribute takes the names of the enum variants, in any number and
/// order. A group name stands for several traits.
trait Wrappers: Names + Copy + PartialEq {
    /// The attribute which lists the traits.
    const ATTR: &'static str;
    /// The marker which takes the default traits off the list.
    const NO_REFS: Self;
    /// The traits which are derived unless `NoRefs` is given.
    const REFS: [Self; 2];

    /// Returns the traits a group name stands for.
    fn group(self) -> &'static [Self];

    /// Generates the implementation of the trait the attribute is named
    /// after, with what comes along with it.
    fn base(wrapped: &Wrapped<'_>) -> TokenStream2;

    /// Generates the implementation of the trait.
    fn implement(self, wrapped: &Wrapped<'_>) -> TokenStream2;

    /// Reads the traits to derive from the attribute. A trait is derived
    /// once, however often it is named.
    fn read(input: &DeriveInput) -> Result<Vec<Self>> {
        let names = Self::NAMES.iter().map(|(name, _)| *name);
        let rules = AttrReq::new(Self::ATTR).unnamed(ListReq::any_of(names));
        let attr = rules.merged(&input.attrs)?;

        let mut named = Vec::new();
        for ident in attr.unnamed().iter().filter_map(Value::as_ident) {
            match Self::NAMES.iter().find(|(name, _)| ident == name) {
                Some((_, wrapper)) if wrapper.group().is_empty() => named.push(*wrapper),
                Some((_, wrapper)) => named.extend(wrapper.group()),
                None => {}
            }
        }
        let mut wrappers = Vec::new();
        if !named.contains(&Self::NO_REFS) {
            wrappers.extend(Self::REFS);
        }
        for wrapper in named {
            if wrapper != Self::NO_REFS && !wrappers.contains(&wrapper) {
                wrappers.push(wrapper);
            }
        }
        Ok(wrappers)
    }

    /// Returns the name of the variant, which is the name of the trait.
    fn name(self) -> &'static str {
        let mut names = Self::NAMES.iter();
        names
            .find(|(_, variant)| *variant == self)
            .map_or("", |(name, _)| name)
    }
}

impl Wrappers for Wrapper {
    const ATTR: &'static str = "wrapper";
    const NO_REFS: Self = Self::NoRefs;
    const REFS: [Self; 2] = [Self::AsRef, Self::Borrow];

    fn group(self) -> &'static [Self] {
        match self {
            Self::Hex => &[Self::LowerHex, Self::UpperHex, Self::FromHex],
            Self::Exp => &[Self::LowerExp, Self::UpperExp],
            Self::NumberFmt => {
                &[Self::LowerHex, Self::UpperHex, Self::LowerExp, Self::UpperExp, Self::Octal]
            }
            Self::RangeOps => &[
                Self::IndexRange,
                Self::IndexFrom,
                Self::IndexTo,
                Self::IndexInclusive,
                Self::IndexToInclusive,
                Self::IndexFull,
            ],
            Self::MathOps => &[Self::Neg, Self::Add, Self::Sub, Self::Mul, Self::Div, Self::Rem],
            Self::BoolOps => &[Self::Not, Self::BitAnd, Self::BitOr, Self::BitXor],
            Self::BitOps => {
                &[Self::Not, Self::BitAnd, Self::BitOr, Self::BitXor, Self::Shl, Self::Shr]
            }
            _ => &[],
        }
    }

    fn base(wrapped: &Wrapped<'_>) -> TokenStream2 {
        let Wrapped {
            input,
            field,
            inner,
            amplify_crate,
        } = wrapped;
        let wrapper = wrapped.implement(&quote!(#amplify_crate::Wrapper), None, quote! {
            type Inner = #inner;

            #[inline]
            fn from_inner(inner: Self::Inner) -> Self {
                <Self as ::core::convert::From<Self::Inner>>::from(inner)
            }

            #[inline]
            fn as_inner(&self) -> &Self::Inner {
                &self.#field
            }

            #[inline]
            fn into_inner(self) -> Self::Inner {
                self.#field
            }
        });
        if is_uncovered(inner, &input.generics) {
            return wrapper;
        }
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident = &input.ident;
        quote! {
            #wrapper

            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#ident #ty_generics> for #inner #where_clause {
                #[inline]
                fn from(wrapped: #ident #ty_generics) -> Self {
                    wrapped.#field
                }
            }
        }
    }

    fn implement(self, wrapped: &Wrapped<'_>) -> TokenStream2 {
        let Wrapped {
            field,
            inner,
            amplify_crate,
            ..
        } = wrapped;
        match self {
            Self::FromStr => {
                let from_str = quote!(::core::str::FromStr);
                wrapped.implement(&from_str, Some(&from_str), quote! {
                    type Err = <#inner as #from_str>::Err;

                    #[inline]
                    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                        <#inner as #from_str>::from_str(s)
                            .map(<Self as ::core::convert::From<#inner>>::from)
                    }
                })
            }
            Self::FromHex => {
                let hex = quote!(#amplify_crate::hex);
                let from_hex = quote!(#hex::FromHex);
                wrapped.implement(&from_hex, Some(&from_hex), quote! {
                    #[inline]
                    fn from_byte_iter<I>(iter: I) -> ::core::result::Result<Self, #hex::Error>
                    where
                        I: ::core::iter::Iterator<Item = ::core::result::Result<u8, #hex::Error>>
                            + ::core::iter::ExactSizeIterator
                            + ::core::iter::DoubleEndedIterator,
                    {
                        <#inner as #from_hex>::from_byte_iter(iter)
                            .map(<Self as ::core::convert::From<#inner>>::from)
                    }
                })
            }
            Self::Display |
            Self::Debug |
            Self::Octal |
            Self::LowerHex |
            Self::UpperHex |
            Self::LowerExp |
            Self::UpperExp => {
                let name = Ident::new(self.name(), Span::call_site());
                let fmt = quote!(::core::fmt::#name);
                wrapped.implement(&fmt, Some(&fmt), quote! {
                    #[inline]
                    fn fmt(
                        &self,
                        formatter: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        #fmt::fmt(&self.#field, formatter)
                    }
                })
            }
            Self::Deref => wrapped.implement(&quote!(::core::ops::Deref), None, quote! {
                type Target = #inner;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.#field
                }
            }),
            Self::AsRef => wrapped.reference(quote!(::core::convert::AsRef), "as_ref", false),
            Self::AsSlice => wrapped.reference(quote!(::core::convert::AsRef), "as_ref", true),
            Self::Borrow => wrapped.reference(quote!(::core::borrow::Borrow), "borrow", false),
            Self::BorrowSlice => wrapped.reference(quote!(::core::borrow::Borrow), "borrow", true),
            Self::Index |
            Self::IndexRange |
            Self::IndexFull |
            Self::IndexFrom |
            Self::IndexTo |
            Self::IndexInclusive |
            Self::IndexToInclusive => {
                let by = index_type(self.name());
                let index = quote!(::core::ops::Index<#by>);
                wrapped.implement(&index, Some(&index), quote! {
                    type Output = <#inner as #index>::Output;

                    #[inline]
                    fn index(&self, index: #by) -> &Self::Output {
                        <#inner as #index>::index(&self.#field, index)
                    }
                })
            }
            Self::Neg | Self::Not => {
                let (operator, method, rest) = wrapped.operator(self.name());
                let bound = quote!(#operator<Output = #inner>);
                wrapped.implement(&operator, Some(&bound), quote! {
                    type Output = Self;

                    #[inline]
                    fn #method(self) -> Self {
                        Self { #field: #operator::#method(self.#field), #rest }
                    }
                })
            }
            Self::Add |
            Self::Sub |
            Self::Mul |
            Self::Div |
            Self::Rem |
            Self::Shl |
            Self::Shr |
            Self::BitAnd |
            Self::BitOr |
            Self::BitXor => {
                let (operator, method, rest) = wrapped.operator(self.name());
                let bound = quote!(#operator<Output = #inner>);
                wrapped.implement(&operator, Some(&bound), quote! {
                    type Output = Self;

                    #[inline]
                    fn #method(self, rhs: Self) -> Self {
                        Self { #field: #operator::#method(self.#field, rhs.#field), #rest }
                    }
                })
            }
            // `NoRefs` and the group names never get here: `read` resolves them.
            Self::NoRefs |
            Self::Hex |
            Self::Exp |
            Self::NumberFmt |
            Self::RangeOps |
            Self::MathOps |
            Self::BoolOps |
            Self::BitOps => TokenStream2::new(),
        }
    }
}

impl Wrappers for WrapperMut {
    const ATTR: &'static str = "wrapper_mut";
    const NO_REFS: Self = Self::NoRefs;
    const REFS: [Self; 2] = [Self::AsMut, Self::BorrowMut];

    fn group(self) -> &'static [Self] {
        match self {
            Self::RangeMut => &[
                Self::IndexRangeMut,
                Self::IndexFromMut,
                Self::IndexToMut,
                Self::IndexInclusiveMut,
                Self::IndexToInclusiveMut,
                Self::IndexFullMut,
            ],
            Self::MathAssign => &[
                Self::AddAssign,
                Self::SubAssign,
                Self::MulAssign,
                Self::DivAssign,
                Self::RemAssign,
            ],
            Self::BoolAssign => &[Self::BitAndAssign, Self::BitOrAssign, Self::BitXorAssign],
            Self::BitAssign => &[
                Self::BitAndAssign,
                Self::BitOrAssign,
                Self::BitXorAssign,
                Self::ShlAssign,
                Self::ShrAssign,
            ],
            _ => &[],
        }
    }

    fn base(wrapped: &Wrapped<'_>) -> TokenStream2 {
        let Wrapped {
            field,
            inner,
            amplify_crate,
            ..
        } = wrapped;
        wrapped.implement(&quote!(#amplify_crate::WrapperMut), None, quote! {
            #[inline]
            fn as_inner_mut(&mut self) -> &mut #inner {
                &mut self.#field
            }
        })
    }

    fn implement(self, wrapped: &Wrapped<'_>) -> TokenStream2 {
        let Wrapped { field, inner, .. } = wrapped;
        match self {
            Self::DerefMut => wrapped.implement(&quote!(::core::ops::DerefMut), None, quote! {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#field
                }
            }),
            Self::AsMut => wrapped.reference(quote!(::core::convert::AsMut), "as_mut", false),
            Self::AsSliceMut => wrapped.reference(quote!(::core::convert::AsMut), "as_mut", true),
            Self::BorrowMut => {
                wrapped.reference(quote!(::core::borrow::BorrowMut), "borrow_mut", false)
            }
            Self::BorrowSliceMut => {
                wrapped.reference(quote!(::core::borrow::BorrowMut), "borrow_mut", true)
            }
            Self::IndexMut |
            Self::IndexRangeMut |
            Self::IndexFullMut |
            Self::IndexFromMut |
            Self::IndexToMut |
            Self::IndexInclusiveMut |
            Self::IndexToInclusiveMut => {
                let by = index_type(self.name());
                let index_mut = quote!(::core::ops::IndexMut<#by>);
                wrapped.implement(&index_mut, Some(&index_mut), quote! {
                    #[inline]
                    fn index_mut(&mut self, index: #by) -> &mut Self::Output {
                        <#inner as #index_mut>::index_mut(&mut self.#field, index)
                    }
                })
            }
            Self::AddAssign |
            Self::SubAssign |
            Self::MulAssign |
            Self::DivAssign |
            Self::RemAssign |
            Self::ShlAssign |
            Self::ShrAssign |
            Self::BitAndAssign |
            Self::BitOrAssign |
            Self::BitXorAssign => {
                let (operator, method, _) = wrapped.operator(self.name());
                wrapped.implement(&operator, Some(&operator), quote! {
                    #[inline]
                    fn #method(&mut self, rhs: Self) {
                        #operator::#method(&mut self.#field, rhs.#field)
                    }
                })
            }
            // `NoRefs` and the group names never get here: `read` resolves them.
            Self::NoRefs |
            Self::RangeMut |
            Self::MathAssign |
            Self::BoolAssign |
            Self::BitAssign => TokenStream2::new(),
        }
    }
}

/// Returns the type which the index trait `name` indexes the wrapper by.
fn index_type(name: &str) -> TokenStream2 {
    match name.trim_end_matches("Mut") {
        "IndexRange" => quote!(::core::ops::Range<usize>),
        "IndexFull" => quote!(::core::ops::RangeFull),
        "IndexFrom" => quote!(::core::ops::RangeFrom<usize>),
        "IndexTo" => quote!(::core::ops::RangeTo<usize>),
        "IndexInclusive" => quote!(::core::ops::RangeInclusive<usize>),
        "IndexToInclusive" => quote!(::core::ops::RangeToInclusive<usize>),
        _ => quote!(usize),
    }
}

/// A wrapper: the structure, the field it wraps and the type of that field.
struct Wrapped<'a> {
    input: &'a DeriveInput,
    field: Member,
    inner: &'a Type,
    amplify_crate: Path,
}

impl<'a> Wrapped<'a> {
    /// Finds the wrapped field: the one marked with `#[wrap]`, or the only
    /// one.
    fn read(input: &'a DeriveInput) -> Result<Self> {
        let amplify_crate = amplify_crate(input)?;
        let fields = match &input.data {
            Data::Struct(data) => &data.fields,
            Data::Enum(_) => {
                return Err(Error::new_spanned(
                    input,
                    "Deriving wrapper is not supported in enums",
                ));
            }
            Data::Union(_) => {
                return Err(Error::new_spanned(
                    input,
                    "Deriving wrapper is not supported in unions",
                ));
            }
        };

        let rules = AttrReq::new("wrap");
        let marks = collect_all(fields.iter().map(|field| rules.single(&field.attrs)))?;
        let mut candidates = fields.iter().zip(fields.members());
        let marked = candidates.clone().zip(marks);
        let mut marked = marked.filter_map(|(field, mark)| Some((field, mark?)));
        let wrapped = match (marked.next(), marked.next()) {
            (_, Some((_, mark))) => {
                return Err(mark.error("only a single field may be wrapped"));
            }
            (Some((field, _)), None) => Some(field),
            (None, _) if matches!(fields, Fields::Unit) => {
                return Err(Error::new_spanned(
                    input,
                    "Deriving wrapper is meaningless for unit structs",
                ));
            }
            (None, _) if fields.len() > 1 => {
                return Err(Error::new_spanned(
                    fields,
                    "When the structure has multiple fields you must point out the one you will \
                     wrap by using `#[wrap]` attribute",
                ));
            }
            (None, _) => candidates.next(),
        };
        let Some((field, member)) = wrapped else {
            return Err(Error::new_spanned(input, "a wrapper must have at least one field"));
        };
        Ok(Self {
            input,
            field: member,
            inner: &field.ty,
            amplify_crate,
        })
    }

    /// Generates the implementation of a trait for the wrapper out of its
    /// items. Where the inner type is generic the impl gets `bound` on it,
    /// which is what the items need.
    fn implement(
        &self,
        trait_path: &TokenStream2,
        bound: Option<&TokenStream2>,
        items: TokenStream2,
    ) -> TokenStream2 {
        let (impl_generics, ty_generics, _) = self.input.generics.split_for_impl();
        let ident = &self.input.ident;
        let mut bounds = Bounds::new(&self.input.generics);
        if let Some(bound) = bound {
            bounds.require(self.inner, bound);
        }
        let where_clause = bounds.into_where_clause();
        quote! {
            #[automatically_derived]
            impl #impl_generics #trait_path for #ident #ty_generics #where_clause {
                #items
            }
        }
    }

    /// Generates the implementation of a trait which gives a reference to
    /// the inner type through `method`. With `bytes` the reference is to
    /// `[u8]`, which the inner type gives through the same trait.
    fn reference(&self, trait_path: TokenStream2, method: &str, bytes: bool) -> TokenStream2 {
        let Self { field, inner, .. } = self;
        let mutability = method.ends_with("mut").then(|| quote!(mut));
        let method = Ident::new(method, Span::call_site());
        if bytes {
            let of_bytes = quote!(#trait_path<[u8]>);
            self.implement(&of_bytes, Some(&of_bytes), quote! {
                #[inline]
                fn #method(&#mutability self) -> &#mutability [u8] {
                    #trait_path::<[u8]>::#method(&#mutability self.#field)
                }
            })
        } else {
            self.implement(&quote!(#trait_path<#inner>), None, quote! {
                #[inline]
                fn #method(&#mutability self) -> &#mutability #inner {
                    &#mutability self.#field
                }
            })
        }
    }

    /// Returns the path of the operator trait `name` and its method, which
    /// is named as the trait in lower case, with `_assign` set apart. The
    /// third item fills the fields which the operator leaves as they are.
    fn operator(&self, name: &str) -> (TokenStream2, Ident, Option<TokenStream2>) {
        let operator = Ident::new(name, Span::call_site());
        let method = match name.strip_suffix("Assign") {
            Some(name) => format_ident!("{}_assign", name.to_lowercase()),
            None => format_ident!("{}", name.to_lowercase()),
        };
        let rest = matches!(&self.input.data, Data::Struct(data) if data.fields.len() > 1);
        (quote!(::core::ops::#operator), method, rest.then(|| quote!(..self)))
    }
}

/// Tells whether `ty` is a type parameter which nothing covers, also behind
/// the fundamental `&`, `&mut`, `Box` and `Pin`. A foreign trait can't be
/// implemented for such a type: the orphan rules forbid it.
fn is_uncovered(ty: &Type, generics: &Generics) -> bool {
    match ty {
        Type::Reference(reference) => is_uncovered(&reference.elem, generics),
        Type::Path(path) if path.qself.is_none() => {
            let Some(last) = path.path.segments.last() else {
                return false;
            };
            match &last.arguments {
                PathArguments::None => generics
                    .type_params()
                    .any(|param| path.path.is_ident(&param.ident)),
                PathArguments::AngleBracketed(args)
                    if last.ident == "Box" || last.ident == "Pin" =>
                {
                    matches!(args.args.first(), Some(GenericArgument::Type(ty)) if is_uncovered(ty, generics))
                }
                _ => false,
            }
        }
        _ => false,
    }
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> { derive::<Wrapper>(&input) }

pub(crate) fn inner_mut(input: DeriveInput) -> Result<TokenStream2> { derive::<WrapperMut>(&input) }

fn derive<T: Wrappers>(input: &DeriveInput) -> Result<TokenStream2> {
    let wrapped = Wrapped::read(input)?;
    let base = T::base(&wrapped);
    let traits = T::read(input)?
        .into_iter()
        .map(|wrapper| wrapper.implement(&wrapped));
    Ok(quote! {
        #base

        #( #traits )*
    })
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::{Wrapper, WrapperMut, Wrappers};

    #[test]
    fn a_trait_is_derived_once_and_no_refs_takes_only_the_defaults_away() {
        let read = |input: DeriveInput| Wrapper::read(&input).expect("known names");
        assert_eq!(
            read(parse_quote!(
                struct Item(u8);
            )),
            [Wrapper::AsRef, Wrapper::Borrow]
        );
        assert_eq!(
            read(parse_quote! {
                #[wrapper(Add, Add, BoolOps, Borrow, BitOps)]
                #[wrapper(Add)]
                struct Item(u8);
            }),
            [
                Wrapper::AsRef,
                Wrapper::Borrow,
                Wrapper::Add,
                Wrapper::Not,
                Wrapper::BitAnd,
                Wrapper::BitOr,
                Wrapper::BitXor,
                Wrapper::Shl,
                Wrapper::Shr,
            ]
        );
        assert_eq!(
            read(parse_quote!(
                #[wrapper(Borrow, NoRefs)]
                struct Item(u8);
            )),
            [Wrapper::Borrow]
        );
        assert_eq!(
            read(parse_quote!(
                #[wrapper(NoRefs, NoRefs)]
                struct Item(u8);
            )),
            []
        );

        let read = |input: DeriveInput| WrapperMut::read(&input).expect("known names");
        assert_eq!(
            read(parse_quote!(
                #[wrapper_mut(NoRefs, AsMut, AsMut)]
                struct Item(u8);
            )),
            [WrapperMut::AsMut]
        );
    }

    #[test]
    fn operator_methods_are_named_after_their_traits() {
        let input: DeriveInput = parse_quote!(
            struct Item(u8);
        );
        let wrapped = super::Wrapped::read(&input).expect("a wrapper");
        for (name, method) in [("Neg", "neg"), ("BitAnd", "bitand"), ("ShlAssign", "shl_assign")] {
            assert_eq!(wrapped.operator(name).1, method);
        }
    }
}
