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

use amplify_syn::{ArgValueReq, AttrReq, ValueClass};
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
        .arg("as_ref", ArgValueReq::with_default(""))
        .arg("as_mut", ArgValueReq::with_default("_mut"))
        .conflict("all", &["as_copy", "as_clone", "as_ref", "as_mut"])
        .conflict("as_copy", &["as_clone"]);
    if !field {
        return rules;
    }
    rules
        .arg("skip", ArgValueReq::Prohibited)
        .arg("base_name", ArgValueReq::required(ValueClass::Str))
}

/// The arguments of `#[getter(...)]`. Each method argument holds the suffix
/// of the method name.
struct Getter {
    prefix: Option<String>,
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
        let mut getter = Self {
            prefix: name("prefix"),
            skip: attr.has("skip"),
            base: name("base_name"),
            as_copy: name("as_copy"),
            as_clone: name("as_clone").or_else(|| all("")),
            as_ref: name("as_ref").or_else(|| all("_ref")),
            as_mut: name("as_mut").or_else(|| all("_mut")),
        };
        getter.normalize();
        Ok(getter)
    }

    /// Applies the defaults for one set of arguments: a plain borrowing getter
    /// is dropped when its name would clash with the copying or cloning one,
    /// and it is added when no method was requested at all.
    fn normalize(&mut self) {
        let owned = self.as_copy.is_some() || self.as_clone.is_some();
        if owned && self.as_ref.as_deref() == Some("") {
            self.as_ref = None;
        }
        if !owned && self.as_ref.is_none() && self.as_mut.is_none() {
            self.as_ref = Some(String::new());
        }
    }

    /// Puts the field-level arguments over the type-level ones.
    ///
    /// Field-level arguments replace type-level ones one by one. The copying,
    /// cloning and borrowing getters form one group: when the field-level set
    /// (after its own defaults are applied) names any of them, the whole group
    /// is taken from the field. `as_mut` is independent of that group.
    fn over(self, global: &Self) -> Self {
        let group = if self.as_copy.is_some() || self.as_clone.is_some() || self.as_ref.is_some() {
            (self.as_copy, self.as_clone, self.as_ref)
        } else {
            (global.as_copy.clone(), global.as_clone.clone(), global.as_ref.clone())
        };
        let mut getter = Self {
            prefix: self.prefix.or_else(|| global.prefix.clone()),
            skip: self.skip,
            base: self.base,
            as_copy: group.0,
            as_clone: group.1,
            as_ref: group.2,
            as_mut: self.as_mut.or_else(|| global.as_mut.clone()),
        };
        getter.normalize();
        getter
    }

    /// The methods to derive, each with the suffix of its name.
    fn methods(&self) -> impl Iterator<Item = (Method, &str)> {
        [
            (Method::Copy, &self.as_copy),
            (Method::Clone, &self.as_clone),
            (Method::AsRef, &self.as_ref),
            (Method::AsMut, &self.as_mut),
        ]
        .into_iter()
        .filter_map(|(method, suffix)| Some((method, suffix.as_deref()?)))
    }
}

/// Makes a method name out of `name`.
///
/// The name is emitted as a raw identifier. For an ordinary word that is the
/// same name, and for a word reserved in any edition it is the only spelling
/// which compiles.
fn method_ident(name: &str, span: Span) -> Result<Ident> {
    let mut ident = syn::parse_str::<Ident>(&format!("r#{name}"))
        .map_err(|_| Error::new(span, format!("`{name}` cannot be used as a method name")))?;
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

    let mut methods = Vec::with_capacity(fields.len());
    for field in fields {
        methods.extend(field_methods(field, struct_name, &global)?);
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #( #methods )*
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
            pub fn #fn_name(#receiver) -> #ret_ty {
                #ret_value
            }
        })
    });
    methods.collect()
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
        for name in
            ["self", "Self", "super", "crate", "_", "", "1st", "r#type", "get-value", "a b", "a(b)"]
        {
            let error = method_ident(name, Span::call_site()).expect_err(name);
            assert_eq!(error.to_string(), format!("`{name}` cannot be used as a method name"));
        }
    }
}
