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

use amplify_syn::{ArgValueReq, AttrReq, ValueClass, collect_all};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Field, Fields, LitStr, Result};

/// The content of `#[getter(...)]` on a structure and, with `skip` and
/// `base_name` added, on a field.
fn rules(field: bool) -> AttrReq {
    let rules = AttrReq::new("getter")
        .arg("prefix", ArgValueReq::with_default(""))
        .arg("all", ArgValueReq::Prohibited)
        .arg("as_copy", ArgValueReq::with_default(""))
        .arg("as_clone", ArgValueReq::with_default(""))
        .arg("as_ref", ArgValueReq::Optional(ValueClass::Str))
        .arg("as_mut", ArgValueReq::with_default("_mut"))
        .arg("crate", ArgValueReq::Prohibited)
        .arg("self", ArgValueReq::Prohibited)
        .arg("super", ArgValueReq::Prohibited)
        .conflict("all", &["as_copy", "as_clone", "as_ref", "as_mut"])
        .conflict("as_copy", &["as_clone"])
        .conflict("crate", &["self", "super"])
        .conflict("self", &["super"]);
    if !field {
        return rules;
    }
    rules
        .arg("skip", ArgValueReq::Prohibited)
        .arg("base_name", ArgValueReq::required(ValueClass::Str))
}

#[derive(Copy, Clone)]
enum Visibility {
    Crate,
    SelfMod,
    Super,
}

/// The arguments of `#[getter(...)]`. Each method argument holds the suffix
/// of the method name.
struct Getter {
    prefix: Option<String>,
    vis: Option<Visibility>,
    skip: bool,
    base: Option<String>,
    as_copy: Option<String>,
    as_clone: Option<String>,
    as_ref: Option<String>,
    as_mut: Option<String>,
}

#[derive(Copy, Clone)]
enum Method {
    Copy,
    Clone,
    AsRef,
    AsMut,
}

impl Getter {
    fn read(attrs: &[Attribute], field: bool) -> Result<Self> {
        let attr = rules(field).merged(attrs)?;
        let name = |arg: &str| attr.str(arg).map(LitStr::value);
        let all = |suffix: &str| attr.has("all").then(|| suffix.to_owned());
        let as_copy = name("as_copy");
        let as_clone = name("as_clone").or_else(|| all(""));
        // The borrowing getter has the plain name, unless the copying or the
        // cloning one takes it.
        let plain = as_copy.is_none() && as_clone.is_none();
        let as_ref = (attr.has("as_ref") || attr.has("all"))
            .then(|| name("as_ref").unwrap_or_else(|| if plain { "" } else { "_ref" }.to_owned()));
        let vis = if attr.has("crate") {
            Some(Visibility::Crate)
        } else if attr.has("self") {
            Some(Visibility::SelfMod)
        } else if attr.has("super") {
            Some(Visibility::Super)
        } else {
            None
        };
        Ok(Self {
            prefix: name("prefix"),
            vis,
            skip: field && attr.has("skip"),
            base: field.then(|| name("base_name")).flatten(),
            as_copy,
            as_clone,
            as_ref,
            as_mut: name("as_mut").or_else(|| all("_mut")),
        })
    }

    /// Puts the field-level arguments over the type-level ones.
    ///
    /// The copying, cloning and borrowing getters form one set: a field which
    /// names any of them replaces the whole set of the type. `prefix`,
    /// visibility and `as_mut` are replaced on their own.
    fn over(self, global: &Self) -> Self {
        let own = self.as_copy.is_some() || self.as_clone.is_some() || self.as_ref.is_some();
        let set = |field: Option<String>, global: &Option<String>| {
            if own { field } else { global.clone() }
        };
        Self {
            prefix: self.prefix.or_else(|| global.prefix.clone()),
            vis: self.vis.or(global.vis),
            skip: self.skip,
            base: self.base,
            as_copy: set(self.as_copy, &global.as_copy),
            as_clone: set(self.as_clone, &global.as_clone),
            as_ref: set(self.as_ref, &global.as_ref),
            as_mut: self.as_mut.or_else(|| global.as_mut.clone()),
        }
    }

    /// The methods to derive, each with the suffix of its name. Where no
    /// copying, cloning or borrowing getter is asked for, the borrowing one
    /// is derived.
    fn methods(&self) -> impl Iterator<Item = (Method, &str)> {
        let none = self.as_copy.is_none() && self.as_clone.is_none() && self.as_ref.is_none();
        [
            (Method::Copy, self.as_copy.as_deref()),
            (Method::Clone, self.as_clone.as_deref()),
            (Method::AsRef, self.as_ref.as_deref().or(none.then_some(""))),
            (Method::AsMut, self.as_mut.as_deref()),
        ]
        .into_iter()
        .filter_map(|(method, suffix)| Some((method, suffix?)))
    }
}

/// Makes a method name out of `name`.
///
/// The name is emitted as a raw identifier. For an ordinary word that is the
/// same name, and for a word reserved in any edition it is the only spelling
/// which compiles.
fn method_ident(name: &str, span: Span) -> Result<Ident> {
    let mut ident = syn::parse_str::<Ident>(&format!("r#{name}"))
        .ok()
        .filter(|ident| ident.unraw() == name)
        .ok_or_else(|| Error::new(span, format!("`{name}` cannot be used as a method name")))?;
    ident.set_span(span);
    Ok(ident)
}

pub(crate) fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let struct_name = &input.ident;
    let global = Getter::read(&input.attrs, false)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return Err(Error::new(
                    Span::call_site(),
                    "Deriving getters is not supported for tuple-bases structs",
                ));
            }
            Fields::Unit => {
                return Err(Error::new(
                    Span::call_site(),
                    "Deriving getters is meaningless for unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(Error::new_spanned(&input, "Deriving getters is not supported in enums"));
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(&input, "Deriving getters is not supported in unions"));
        }
    };

    let methods = fields
        .iter()
        .map(|field| field_methods(field, struct_name, &global));
    let methods = collect_all(methods)?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #( #( #methods )* )*
        }
    })
}

fn field_methods(field: &Field, struct_name: &Ident, global: &Getter) -> Result<Vec<TokenStream2>> {
    let getter = Getter::read(&field.attrs, true)?.over(global);
    let Some(field_name) = field.ident.as_ref().filter(|_| !getter.skip) else {
        return Ok(Vec::new());
    };

    let span = field.span();
    let ty = &field.ty;
    let docs = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"));
    let prefix = getter.prefix.as_deref().unwrap_or_default();
    let plain_name = field_name.unraw();
    let base = getter
        .base
        .clone()
        .unwrap_or_else(|| plain_name.to_string());
    let vis = match getter.vis {
        None => quote!(pub),
        Some(Visibility::Crate) => quote!(pub(crate)),
        Some(Visibility::SelfMod) => quote!(pub(self)),
        Some(Visibility::Super) => quote!(pub(super)),
    };

    let methods = getter.methods().map(|(method, suffix)| {
        let fn_name = method_ident(&format!("{prefix}{base}{suffix}"), span)
            .map_err(|error| Error::new(field_name.span(), error))?;
        let (doc, receiver, ret_ty, ret_value) = match method {
            Method::Copy => (
                "returning copy of",
                quote!(&self),
                quote!(#ty),
                quote_spanned!(span => self.#field_name),
            ),
            Method::Clone => (
                "cloning",
                quote!(&self),
                quote!(#ty),
                quote_spanned!(span => ::core::clone::Clone::clone(&self.#field_name)),
            ),
            Method::AsRef => (
                "borrowing",
                quote!(&self),
                quote!(&#ty),
                quote_spanned!(span => &self.#field_name),
            ),
            Method::AsMut => (
                "returning mutable borrow of",
                quote!(&mut self),
                quote!(&mut #ty),
                quote_spanned!(span => &mut self.#field_name),
            ),
        };
        let fn_doc = format!("Method {doc} [`{struct_name}::{plain_name}`] field.\n");
        let docs = docs.clone();

        Ok(quote_spanned! { span =>
            #[doc = #fn_doc]
            #( #docs )*
            #[inline]
            #vis fn #fn_name(#receiver) -> #ret_ty {
                #ret_value
            }
        })
    });
    collect_all(methods)
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::parse_quote;

    use super::{derive, method_ident};

    #[test]
    fn copies_every_field_doc_line_to_the_getter() {
        let output = derive(parse_quote! {
            struct Item {
                /// First line.
                /// Second line.
                value: u8,
            }
        })
        .expect("getters are derived")
        .to_string();

        assert!(output.contains("First line."), "first doc line is missing: {output}");
        assert!(output.contains("Second line."), "second doc line is missing: {output}");
    }

    #[test]
    fn composes_getter_names_from_unraw_field_names() {
        let output = derive(parse_quote! {
            #[getter(prefix = "get_", as_mut)]
            struct Item {
                r#type: u8,
            }
        })
        .expect("getters are derived")
        .to_string();

        assert!(output.contains("fn r#get_type_mut"), "unexpected getter name: {output}");
        assert!(output.contains("[`Item::type`]"), "unexpected link to the field: {output}");
    }

    #[test]
    fn method_names_are_raw_identifiers_or_errors() {
        for name in ["value", "type", "try", "yield", "gen", "__"] {
            let ident = method_ident(name, Span::call_site());
            let ident = ident.map(|ident| ident.to_string());
            assert_eq!(ident.ok(), Some(format!("r#{name}")));
        }
        for name in [
            "self",
            "Self",
            "super",
            "crate",
            "_",
            "",
            "1st",
            "r#type",
            "get-value",
            "a b",
            "a(b)",
            "foo ",
            "foo/*x*/",
        ] {
            let error = method_ident(name, Span::call_site()).expect_err(name);
            assert_eq!(error.to_string(), format!("`{name}` cannot be used as a method name"));
        }
    }
}
