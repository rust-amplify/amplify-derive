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

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Result, Type, parse_quote};

use crate::util::Bounds;

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let err = Ident::new("__amplify_err", Span::mixed_site());

    let self_ty: Type = parse_quote!(Self);
    let mut error_bounds = Bounds::new(&input.generics);
    error_bounds.require(&self_ty, quote!(::core::fmt::Debug + ::core::fmt::Display));
    let error_where = error_bounds.into_where_clause();

    let target_ty: Type = parse_quote!(#ident_name #ty_generics);
    let mut from_bounds = Bounds::new(&input.generics);
    from_bounds.require(&target_ty, quote!(::core::fmt::Display));
    let from_where = from_bounds.into_where_clause();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for #ident_name #ty_generics #error_where {
        }

        // `String` has no path which works both with `std` and without it,
        // so the `alloc` crate is brought in where the path is used.
        const _: () = {
            extern crate alloc;

            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#ident_name #ty_generics> for alloc::string::String #from_where {
                fn from(#err: #ident_name #ty_generics) -> Self {
                    alloc::string::ToString::to_string(&#err)
                }
            }
        };
    })
}
