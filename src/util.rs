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

use amplify_syn::{AttrReq, ListReq, Value, ValueClass};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;
use syn::{DeriveInput, Generics, Ident, Path, Result, WhereClause, parse_quote};

/// The bounds which a derived impl needs on top of those the type declares.
pub(crate) struct Bounds<'a> {
    generics: &'a Generics,
    clause: Option<WhereClause>,
}

impl<'a> Bounds<'a> {
    pub fn new(generics: &'a Generics) -> Self {
        Self {
            generics,
            clause: generics.where_clause.clone(),
        }
    }

    /// Requires `ty: bound`, where `ty` depends on a type parameter. Any
    /// other type either meets the bound or it does not, whatever the impl
    /// says.
    pub fn require(&mut self, ty: impl ToTokens, bound: impl ToTokens) {
        if self.is_generic(ty.to_token_stream()) {
            let clause = self.clause.get_or_insert_with(|| parse_quote!(where));
            let predicate = parse_quote!(#ty: #bound);
            if !clause.predicates.iter().any(|known| *known == predicate) {
                clause.predicates.push(predicate);
            }
        }
    }

    /// Returns the where clause of the impl.
    pub fn into_where_clause(self) -> Option<WhereClause> { self.clause }

    fn is_generic(&self, ty: TokenStream2) -> bool {
        let params = || self.generics.type_params();
        ty.into_iter().any(|token| match token {
            TokenTree::Ident(ident) if ident == "Self" => params().next().is_some(),
            TokenTree::Ident(ident) => params().any(|param| param.ident == ident),
            TokenTree::Group(group) => self.is_generic(group.stream()),
            _ => false,
        })
    }
}

/// Resolves `#[amplify_crate(path)]`, defaulting to `amplify`.
pub(crate) fn amplify_crate(input: &DeriveInput) -> Result<Path> {
    Ok(explicit_amplify_crate(input)?
        .unwrap_or_else(|| Path::from(Ident::new("amplify", input.ident.span()))))
}

/// Reads `#[amplify_crate(path)]`, if the attribute is present.
pub(crate) fn explicit_amplify_crate(input: &DeriveInput) -> Result<Option<Path>> {
    let rules = AttrReq::new("amplify_crate").unnamed(ListReq::one(ValueClass::Path));
    let attr = rules.single(&input.attrs)?;
    Ok(attr.and_then(|attr| attr.unnamed().first().and_then(Value::as_path).cloned()))
}
