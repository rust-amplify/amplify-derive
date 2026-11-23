// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Elichai Turkel <elichai.turkel@gmail.com>
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

use amplify_syn::{AttrReq, ListReq, Value, ValueClass, collect_all};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Fields, Result, Type};

const NAME: &str = "from";

/// The content of `#[from]`: the type to convert from, which may be left out
/// where a single field gives it.
fn rules() -> AttrReq { AttrReq::new(NAME).unnamed(ListReq::maybe_one(ValueClass::Type)) }

/// One `From` implementation.
struct Conversion {
    from: Type,
    /// The expression which makes `Self` out of the value `v`.
    value: TokenStream2,
    /// Where the conversion is asked for.
    span: Span,
}

/// What a conversion makes: a structure, a variant of an enum or a union.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kind {
    Struct,
    Variant,
    Union,
}

/// Reads the conversions which the attributes of a structure, a variant or a
/// union and of its fields ask for. `path` is `Self` or `Self::Variant`.
///
/// A structure or a union with a single field converts from the type of the
/// field without being asked.
fn conversions(
    kind: Kind,
    path: TokenStream2,
    fields: &Fields,
    attrs: &[Attribute],
) -> Result<Vec<Conversion>> {
    let convert = |from: &Type, target: Option<usize>, span: Span| Conversion {
        from: from.clone(),
        value: construct(kind, &path, fields, target),
        span,
    };
    let single = fields.iter().next().filter(|_| fields.len() == 1);
    let mut conversions = Vec::new();
    // The attributes of the item itself, and then of each of its fields.
    let places = [attrs]
        .into_iter()
        .chain(fields.iter().map(|field| &field.attrs[..]));
    let mut asked = collect_all(places.map(|attrs| rules().each(attrs)))?.into_iter();

    // A value given to the item as a whole goes into the only field, or into
    // the first one where the fields have no names. Other items ignore the
    // value; multi-field unions must specify the target field explicitly.
    let positional = matches!(fields, Fields::Unnamed(_));
    let target = (single.is_some() || positional && !fields.is_empty()).then_some(0);
    for attr in asked.next().unwrap_or_default() {
        if kind == Kind::Union && fields.len() > 1 {
            return Err(attr.error(
                "type-level `#[from]` is not allowed on multi-field unions; specify `#[from]` \
                 right ahead of the target field",
            ));
        }
        let explicit = attr.unnamed().first().and_then(Value::as_type);
        let Some(from) = explicit.or(single.map(|field| &field.ty)) else {
            return Err(attr.error(
                "empty attribute is allowed only for entities with a single field; for \
                 multi-field entities specify the attribute right ahead of the target field",
            ));
        };
        conversions.push(convert(from, target, attr.span()));
    }
    for (index, (field, asked)) in fields.iter().zip(asked).enumerate() {
        for attr in asked {
            let explicit = attr.unnamed().first().and_then(Value::as_type);
            conversions.push(convert(explicit.unwrap_or(&field.ty), Some(index), attr.span()));
        }
    }
    if let Some(field) = single.filter(|_| conversions.is_empty() && kind != Kind::Variant) {
        conversions.push(convert(&field.ty, Some(0), field.span()));
    }
    Ok(conversions)
}

fn from_param() -> Ident { Ident::new("__amplify_v", Span::mixed_site()) }

/// Makes the expression which puts the value `v` into the `target` field.
///
/// The other fields take their defaults. A structure with named fields takes
/// them from its own `Default`, and a union has no other fields to fill.
fn construct(
    kind: Kind,
    path: &TokenStream2,
    fields: &Fields,
    target: Option<usize>,
) -> TokenStream2 {
    let v = from_param();
    let own_default = kind == Kind::Struct && matches!(fields, Fields::Named(_));
    let field_wise = !own_default && kind != Kind::Union;
    let members = fields.members().enumerate();
    let members = members.filter(|(index, _)| field_wise || target == Some(*index));
    let members = members.map(|(index, member)| {
        if target == Some(index) {
            quote! { #member: ::core::convert::Into::into(#v), }
        } else {
            quote! { #member: ::core::default::Default::default(), }
        }
    });
    let rest = (own_default && fields.len() > target.iter().len())
        .then(|| quote! { ..::core::default::Default::default() });
    quote! { #path { #( #members )* #rest } }
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;

    let found = match &input.data {
        Data::Struct(data) => conversions(Kind::Struct, quote!(Self), &data.fields, &input.attrs)?,
        Data::Enum(data) => {
            // Do not let top-level `from` on enums
            if let Some(attr) = input.attrs.iter().find(|attr| attr.path().is_ident(NAME)) {
                return Err(Error::new_spanned(
                    attr.path(),
                    "top-level attribute is not allowed, use it for specific fields or variants",
                ));
            }
            let found = data.variants.iter().map(|variant| {
                let path = &variant.ident;
                conversions(Kind::Variant, quote!(Self::#path), &variant.fields, &variant.attrs)
            });
            collect_all(found)?.into_iter().flatten().collect()
        }
        Data::Union(data) => {
            let fields = Fields::Named(data.fields.clone());
            conversions(Kind::Union, quote!(Self), &fields, &input.attrs)?
        }
    };

    if found.is_empty() {
        return Err(Error::new_spanned(
            ident_name,
            "Deriving `From`: there is nothing to convert from, as no `#[from]` attribute is given",
        ));
    }
    for (index, conversion) in found.iter().enumerate() {
        let from = &conversion.from;
        if found[..index].iter().any(|known| known.from == *from) {
            let message =
                format!("Attribute `#[{NAME}]`: repeated use of type `{}`", quote!(#from));
            return Err(Error::new(conversion.span, message));
        }
    }

    let v = from_param();
    let impls = found.iter().map(|Conversion { from, value, .. }| {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#from> for #ident_name #ty_generics #where_clause {
                fn from(#v: #from) -> Self {
                    #value
                }
            }
        }
    });
    Ok(quote! { #( #impls )* })
}
