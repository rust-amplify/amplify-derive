// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify_syn::{AttrReq, ListReq, Value};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Path, Result, Type};

use crate::util::amplify_crate;

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
    #[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
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
    #[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
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
    /// The marker which takes the reference traits off the list.
    const NO_REFS: Self;
    /// The reference traits, which are derived unless `NoRefs` is given.
    const REFS: [Self; 2];
    /// Adds the trait, or the traits a group name stands for, to the list.
    fn populate(self, list: &mut Vec<Self>);
}

impl Wrappers for Wrapper {
    const ATTR: &'static str = "wrapper";
    const NO_REFS: Self = Self::NoRefs;
    const REFS: [Self; 2] = [Self::AsRef, Self::Borrow];

    fn populate(self, list: &mut Vec<Self>) {
        let ext = match self {
            Wrapper::Hex => &[Wrapper::LowerHex, Wrapper::UpperHex, Wrapper::FromHex] as &[_],
            Wrapper::Exp => &[Wrapper::LowerExp, Wrapper::UpperExp] as &[_],
            Wrapper::NumberFmt => &[
                Wrapper::LowerHex,
                Wrapper::UpperHex,
                Wrapper::LowerExp,
                Wrapper::UpperExp,
                Wrapper::Octal,
            ] as &[_],
            Wrapper::RangeOps => &[
                Wrapper::IndexRange,
                Wrapper::IndexFrom,
                Wrapper::IndexTo,
                Wrapper::IndexInclusive,
                Wrapper::IndexToInclusive,
                Wrapper::IndexFull,
            ] as &[_],
            Wrapper::MathOps => &[
                Wrapper::Neg,
                Wrapper::Add,
                Wrapper::Sub,
                Wrapper::Mul,
                Wrapper::Div,
                Wrapper::Rem,
            ] as &[_],
            Wrapper::BoolOps => {
                &[Wrapper::Not, Wrapper::BitAnd, Wrapper::BitOr, Wrapper::BitXor] as &[_]
            }
            Wrapper::BitOps => &[
                Wrapper::Not,
                Wrapper::BitAnd,
                Wrapper::BitOr,
                Wrapper::BitXor,
                Wrapper::Shl,
                Wrapper::Shr,
            ] as &[_],
            x => {
                list.push(x);
                &[] as &[_]
            }
        };
        list.extend(ext);
    }
}

impl Wrapper {
    pub fn into_token_stream2(
        self,
        input: &DeriveInput,
        from: &Type,
        field: &TokenStream2,
        amplify_crate: &Path,
    ) -> TokenStream2 {
        if let Some(trait_name) = formatting_trait(self) {
            return formatting_impl(trait_name, input, field);
        }
        if let Some(index_type) = wrapper_index_type(self) {
            return index_impl(input, from, field, &index_type, false);
        }
        if let Some(operator) = wrapper_operator(self) {
            return operator_impl(input, field, operator);
        }

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident_name = &input.ident;

        match self {
            Wrapper::FromStr => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::str::FromStr for #ident_name #ty_generics #where_clause
                {
                    type Err = <<Self as #amplify_crate::Wrapper>::Inner as ::core::str::FromStr>::Err;

                    #[inline]
                    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                        <#from as ::core::str::FromStr>::from_str(s).map(<Self as ::core::convert::From<#from>>::from)
                    }
                }
            },
            Wrapper::FromHex => quote! {
                #[automatically_derived]
                impl #impl_generics #amplify_crate::hex::FromHex for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn from_byte_iter<I>(
                        iter: I,
                    ) -> ::core::result::Result<Self, #amplify_crate::hex::Error>
                    where
                        I: ::core::iter::Iterator<
                                Item = ::core::result::Result<u8, #amplify_crate::hex::Error>,
                            >
                            + ::core::iter::ExactSizeIterator
                            + ::core::iter::DoubleEndedIterator,
                    {
                        <#from as #amplify_crate::hex::FromHex>::from_byte_iter(iter).map(<Self as ::core::convert::From<#from>>::from)
                    }
                }
            },
            Wrapper::Deref => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::ops::Deref for #ident_name #ty_generics #where_clause
                {
                    type Target = #from;
                    #[inline]
                    fn deref(&self) -> &Self::Target {
                        &self.#field
                    }
                }
            },
            Wrapper::AsRef => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::AsRef<#from> for #ident_name #ty_generics #where_clause {
                    #[inline]
                    fn as_ref(&self) -> &#from {
                        &self.#field
                    }
                }
            },
            Wrapper::AsSlice => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::AsRef<[u8]> for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn as_ref(&self) -> &[u8] {
                        ::core::convert::AsRef::<[u8]>::as_ref(&self.#field)
                    }
                }
            },
            Wrapper::Borrow => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::borrow::Borrow<#from> for #ident_name #ty_generics #where_clause {
                    #[inline]
                    fn borrow(&self) -> &#from {
                        &self.#field
                    }
                }
            },
            Wrapper::BorrowSlice => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::borrow::Borrow<[u8]> for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn borrow(&self) -> &[u8] {
                        ::core::borrow::Borrow::<[u8]>::borrow(&self.#field)
                    }
                }
            },
            // Formatting, index and operator traits are generated from the tables below;
            // `NoRefs` and the group names are resolved before code generation.
            _ => TokenStream2::new(),
        }
    }
}

impl Wrappers for WrapperMut {
    const ATTR: &'static str = "wrapper_mut";
    const NO_REFS: Self = Self::NoRefs;
    const REFS: [Self; 2] = [Self::AsMut, Self::BorrowMut];

    fn populate(self, list: &mut Vec<Self>) {
        let exp = match self {
            WrapperMut::RangeMut => &[
                WrapperMut::IndexRangeMut,
                WrapperMut::IndexFromMut,
                WrapperMut::IndexToMut,
                WrapperMut::IndexInclusiveMut,
                WrapperMut::IndexToInclusiveMut,
                WrapperMut::IndexFullMut,
            ] as &[_],
            WrapperMut::MathAssign => &[
                WrapperMut::AddAssign,
                WrapperMut::SubAssign,
                WrapperMut::MulAssign,
                WrapperMut::DivAssign,
                WrapperMut::RemAssign,
            ] as &[_],
            WrapperMut::BoolAssign => {
                &[WrapperMut::BitAndAssign, WrapperMut::BitOrAssign, WrapperMut::BitXorAssign]
                    as &[_]
            }
            WrapperMut::BitAssign => &[
                WrapperMut::BitAndAssign,
                WrapperMut::BitOrAssign,
                WrapperMut::BitXorAssign,
                WrapperMut::ShlAssign,
                WrapperMut::ShrAssign,
            ],
            x => {
                list.push(x);
                &[] as &[_]
            }
        };
        list.extend(exp)
    }
}

impl WrapperMut {
    pub fn into_token_stream2(
        self,
        input: &DeriveInput,
        from: &Type,
        field: &TokenStream2,
        amplify_crate: &Path,
    ) -> TokenStream2 {
        if let Some(index_type) = wrapper_mut_index_type(self) {
            return index_impl(input, from, field, &index_type, true);
        }
        if let Some(operator) = wrapper_mut_operator(self) {
            return assignment_impl(input, field, operator);
        }

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident_name = &input.ident;

        match self {
            WrapperMut::DerefMut => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::ops::DerefMut for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.#field
                    }
                }
            },
            WrapperMut::AsMut => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::AsMut<<Self as #amplify_crate::Wrapper>::Inner> for #ident_name #ty_generics #where_clause {
                    #[inline]
                    fn as_mut(&mut self) -> &mut <Self as #amplify_crate::Wrapper>::Inner {
                        &mut self.#field
                    }
                }
            },
            WrapperMut::AsSliceMut => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::AsMut<[u8]> for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn as_mut(&mut self) -> &mut [u8] {
                        ::core::convert::AsMut::<[u8]>::as_mut(&mut self.#field)
                    }
                }
            },
            WrapperMut::BorrowMut => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::borrow::BorrowMut<<Self as #amplify_crate::Wrapper>::Inner> for #ident_name #ty_generics #where_clause {
                    #[inline]
                    fn borrow_mut(&mut self) -> &mut <Self as #amplify_crate::Wrapper>::Inner {
                        &mut self.#field
                    }
                }
            },
            WrapperMut::BorrowSliceMut => quote! {
                #[automatically_derived]
                impl #impl_generics ::core::borrow::BorrowMut<[u8]> for #ident_name #ty_generics #where_clause
                {
                    #[inline]
                    fn borrow_mut(&mut self) -> &mut [u8] {
                        ::core::borrow::BorrowMut::<[u8]>::borrow_mut(&mut self.#field)
                    }
                }
            },
            // Index and assignment traits are generated from the tables below;
            // `NoRefs` and the group names are resolved before code generation.
            _ => TokenStream2::new(),
        }
    }
}

fn formatting_trait(wrapper: Wrapper) -> Option<Ident> {
    let name = match wrapper {
        Wrapper::Display => "Display",
        Wrapper::Debug => "Debug",
        Wrapper::Octal => "Octal",
        Wrapper::LowerHex => "LowerHex",
        Wrapper::UpperHex => "UpperHex",
        Wrapper::LowerExp => "LowerExp",
        Wrapper::UpperExp => "UpperExp",
        _ => return None,
    };
    Some(Ident::new(name, Span::call_site()))
}

fn formatting_impl(trait_name: Ident, input: &DeriveInput, field: &TokenStream2) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::#trait_name for #ident #ty_generics #where_clause {
            #[inline]
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::#trait_name::fmt(&self.#field, formatter)
            }
        }
    }
}

fn wrapper_index_type(wrapper: Wrapper) -> Option<TokenStream2> {
    match wrapper {
        Wrapper::Index => Some(quote!(usize)),
        Wrapper::IndexRange => Some(quote!(::core::ops::Range<usize>)),
        Wrapper::IndexFull => Some(quote!(::core::ops::RangeFull)),
        Wrapper::IndexFrom => Some(quote!(::core::ops::RangeFrom<usize>)),
        Wrapper::IndexTo => Some(quote!(::core::ops::RangeTo<usize>)),
        Wrapper::IndexInclusive => Some(quote!(::core::ops::RangeInclusive<usize>)),
        Wrapper::IndexToInclusive => Some(quote!(::core::ops::RangeToInclusive<usize>)),
        _ => None,
    }
}

fn wrapper_mut_index_type(wrapper: WrapperMut) -> Option<TokenStream2> {
    match wrapper {
        WrapperMut::IndexMut => Some(quote!(usize)),
        WrapperMut::IndexRangeMut => Some(quote!(::core::ops::Range<usize>)),
        WrapperMut::IndexFullMut => Some(quote!(::core::ops::RangeFull)),
        WrapperMut::IndexFromMut => Some(quote!(::core::ops::RangeFrom<usize>)),
        WrapperMut::IndexToMut => Some(quote!(::core::ops::RangeTo<usize>)),
        WrapperMut::IndexInclusiveMut => Some(quote!(::core::ops::RangeInclusive<usize>)),
        WrapperMut::IndexToInclusiveMut => Some(quote!(::core::ops::RangeToInclusive<usize>)),
        _ => None,
    }
}

fn index_impl(
    input: &DeriveInput,
    from: &Type,
    field: &TokenStream2,
    index_type: &TokenStream2,
    mutable: bool,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    if mutable {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::ops::IndexMut<#index_type>
                for #ident #ty_generics #where_clause
            {
                #[inline]
                fn index_mut(&mut self, index: #index_type) -> &mut Self::Output {
                    <#from as ::core::ops::IndexMut<#index_type>>::index_mut(
                        &mut self.#field,
                        index,
                    )
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::ops::Index<#index_type>
                for #ident #ty_generics #where_clause
            {
                type Output = <#from as ::core::ops::Index<#index_type>>::Output;

                #[inline]
                fn index(&self, index: #index_type) -> &Self::Output {
                    <#from as ::core::ops::Index<#index_type>>::index(&self.#field, index)
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct OperatorSpec {
    trait_name: &'static str,
    method: &'static str,
    unary: bool,
}

fn wrapper_operator(wrapper: Wrapper) -> Option<OperatorSpec> {
    let (trait_name, method, unary) = match wrapper {
        Wrapper::Neg => ("Neg", "neg", true),
        Wrapper::Not => ("Not", "not", true),
        Wrapper::Add => ("Add", "add", false),
        Wrapper::Sub => ("Sub", "sub", false),
        Wrapper::Mul => ("Mul", "mul", false),
        Wrapper::Div => ("Div", "div", false),
        Wrapper::Rem => ("Rem", "rem", false),
        Wrapper::Shl => ("Shl", "shl", false),
        Wrapper::Shr => ("Shr", "shr", false),
        Wrapper::BitAnd => ("BitAnd", "bitand", false),
        Wrapper::BitOr => ("BitOr", "bitor", false),
        Wrapper::BitXor => ("BitXor", "bitxor", false),
        _ => return None,
    };
    Some(OperatorSpec {
        trait_name,
        method,
        unary,
    })
}

fn wrapper_mut_operator(wrapper: WrapperMut) -> Option<OperatorSpec> {
    let (trait_name, method) = match wrapper {
        WrapperMut::AddAssign => ("AddAssign", "add_assign"),
        WrapperMut::SubAssign => ("SubAssign", "sub_assign"),
        WrapperMut::MulAssign => ("MulAssign", "mul_assign"),
        WrapperMut::DivAssign => ("DivAssign", "div_assign"),
        WrapperMut::RemAssign => ("RemAssign", "rem_assign"),
        WrapperMut::ShlAssign => ("ShlAssign", "shl_assign"),
        WrapperMut::ShrAssign => ("ShrAssign", "shr_assign"),
        WrapperMut::BitAndAssign => ("BitAndAssign", "bitand_assign"),
        WrapperMut::BitOrAssign => ("BitOrAssign", "bitor_assign"),
        WrapperMut::BitXorAssign => ("BitXorAssign", "bitxor_assign"),
        _ => return None,
    };
    Some(OperatorSpec {
        trait_name,
        method,
        unary: false,
    })
}

fn operator_impl(
    input: &DeriveInput,
    field: &TokenStream2,
    operator: OperatorSpec,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let trait_name = Ident::new(operator.trait_name, Span::call_site());
    let method = Ident::new(operator.method, Span::call_site());
    let update = match &input.data {
        Data::Struct(data) if data.fields.len() > 1 => quote!(..self),
        _ => TokenStream2::new(),
    };
    if operator.unary {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::ops::#trait_name for #ident #ty_generics #where_clause {
                type Output = Self;

                #[inline]
                fn #method(self) -> Self {
                    Self {
                        #field: ::core::ops::#trait_name::#method(self.#field),
                        #update
                    }
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::ops::#trait_name for #ident #ty_generics #where_clause {
                type Output = Self;

                #[inline]
                fn #method(self, rhs: Self) -> Self {
                    Self {
                        #field: ::core::ops::#trait_name::#method(self.#field, rhs.#field),
                        #update
                    }
                }
            }
        }
    }
}

fn assignment_impl(
    input: &DeriveInput,
    field: &TokenStream2,
    operator: OperatorSpec,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let trait_name = Ident::new(operator.trait_name, Span::call_site());
    let method = Ident::new(operator.method, Span::call_site());
    quote! {
        #[automatically_derived]
        impl #impl_generics ::core::ops::#trait_name for #ident #ty_generics #where_clause {
            #[inline]
            fn #method(&mut self, rhs: Self) {
                ::core::ops::#trait_name::#method(&mut self.#field, rhs.#field)
            }
        }
    }
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let amplify_crate = amplify_crate(&input)?;

    let (field, from) = get_params(&input)?;
    let from_conversion = if is_bare_type_parameter(&from, &input.generics) {
        TokenStream2::new()
    } else {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#ident_name #ty_generics> for #from #where_clause {
                #[inline]
                fn from(wrapped: #ident_name #ty_generics) -> Self {
                    wrapped.#field
                }
            }
        }
    };

    let wrappers = get_wrappers::<Wrapper>(&input)?;
    let wrapper_derive = wrappers
        .iter()
        .map(|w| w.into_token_stream2(&input, &from, &field, &amplify_crate));

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #amplify_crate::Wrapper for #ident_name #ty_generics #where_clause {
            type Inner = #from;

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
        }

        #from_conversion

        #( #wrapper_derive )*
    })
}

fn is_bare_type_parameter(ty: &Type, generics: &syn::Generics) -> bool {
    let Type::Path(ty) = ty else {
        return false;
    };
    let Some(ident) = ty.path.get_ident() else {
        return false;
    };
    generics.type_params().any(|param| param.ident == *ident)
}

pub(crate) fn inner_mut(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let amplify_crate = amplify_crate(&input)?;

    let (field, from) = get_params(&input)?;

    let wrappers = get_wrappers::<WrapperMut>(&input)?;
    let wrapper_derive = wrappers
        .iter()
        .map(|w| w.into_token_stream2(&input, &from, &field, &amplify_crate));

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #amplify_crate::WrapperMut for #ident_name #ty_generics #where_clause {
            #[inline]
            fn as_inner_mut(&mut self) -> &mut <Self as #amplify_crate::Wrapper>::Inner {
                &mut self.#field
            }
        }

        #( #wrapper_derive )*
    })
}

/// Finds the wrapped field: the one marked with `#[wrap]`, or the only one.
fn get_params(input: &DeriveInput) -> Result<(TokenStream2, Type)> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) => {
            return Err(Error::new_spanned(input, "Deriving wrapper is not supported in enums"));
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(input, "Deriving wrapper is not supported in unions"));
        }
    };

    let rules = AttrReq::new("wrap");
    let mut wrapped = None;
    for (field, member) in fields.iter().zip(fields.members()) {
        if let Some(attr) = rules.single(&field.attrs)? {
            if wrapped.is_some() {
                return Err(attr.error("only a single field may be wrapped"));
            }
            wrapped = Some((field, member));
        }
    }

    let wrapped = match (wrapped, fields) {
        (Some(wrapped), _) => Some(wrapped),
        (None, Fields::Unit) => {
            return Err(Error::new_spanned(
                input,
                "Deriving wrapper is meaningless for unit structs",
            ));
        }
        (None, _) if fields.len() > 1 => {
            return Err(Error::new_spanned(
                fields,
                "When the structure has multiple fields you must point out the one you will wrap \
                 by using `#[wrap]` attribute",
            ));
        }
        (None, _) => fields.iter().zip(fields.members()).next(),
    };
    let Some((field, member)) = wrapped else {
        return Err(Error::new_spanned(input, "a wrapper must have at least one field"));
    };
    Ok((quote! { #member }, field.ty.clone()))
}

/// Reads the traits to derive from the attribute of `T`.
fn get_wrappers<T: Wrappers>(input: &DeriveInput) -> Result<Vec<T>> {
    let names = T::NAMES.iter().map(|(name, _)| *name);
    let rules = AttrReq::new(T::ATTR).unnamed(ListReq::any_of(names));
    let attr = rules.merged(&input.attrs)?;

    let mut wrappers = T::REFS.to_vec();
    for ident in attr.unnamed().iter().filter_map(Value::as_ident) {
        if let Some((_, wrapper)) = T::NAMES.iter().find(|(name, _)| ident == name) {
            wrapper.populate(&mut wrappers);
        }
    }
    if wrappers.contains(&T::NO_REFS) {
        wrappers.retain(|wrapper| *wrapper != T::NO_REFS && !T::REFS.contains(wrapper));
    }
    Ok(wrappers)
}
