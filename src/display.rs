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

use amplify_syn::{ArgValueReq, Attr, AttrReq, Form, ListReq, Value, ValueClass, collect_all};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Field, Fields, Ident, Lit, LitStr, Path, Result};

use crate::format::{ArgRef, Format, scan};
use crate::util::Bounds;

const NAME: &str = "display";
const EXAMPLE: &str = r#"#[display("format {} string" | Trait | Type::function)]"#;

/// The traits of `core::fmt` a type can be displayed through.
const TRAITS: [&str; 8] =
    ["Debug", "Octal", "Binary", "Pointer", "LowerHex", "UpperHex", "LowerExp", "UpperExp"];

/// The content of `#[display(...)]` on a type or a variant: a format string,
/// which may come with an alternative one, or the name of a formatting
/// trait, of a function or of a built-in technique.
fn rules() -> AttrReq {
    AttrReq::new(NAME)
        .unnamed(ListReq::one(ValueClass::Str).or(ValueClass::Path))
        .name_value(ValueClass::Str)
        .arg("alt", ArgValueReq::required(ValueClass::Str))
}

/// The content of `#[display(...)]` on a field.
fn field_rules() -> AttrReq {
    AttrReq::new(NAME).mandatory("separator", ArgValueReq::required(ValueClass::Str))
}

/// How a type or a variant is displayed.
enum Technique {
    /// Through another formatting trait of the type, as `#[display(Debug)]`.
    Trait(Ident),
    /// Through a function which takes the value by reference.
    Function(Path),
    /// With a format string, and with another one for `{:#}`.
    Format(LitStr, Option<LitStr>),
    /// With the doc comments as the format string.
    DocComments,
    /// As the only field.
    Inner,
    /// As the name in lower case, followed by the fields.
    Lowercase,
    /// As the name in upper case, followed by the fields.
    Uppercase,
}

impl Technique {
    fn read(attrs: &[Attribute]) -> Result<Option<Self>> {
        let Some(attr) = rules().single(attrs)? else {
            return Ok(None);
        };
        let Some(first) = attr.unnamed().first() else {
            return Ok(None);
        };
        if let Some(format) = first.as_str() {
            return Ok(Some(Self::Format(format.clone(), attr.str("alt").cloned())));
        }
        let Some(path) = first.as_path().filter(|_| !attr.has("alt")) else {
            return Err(attr
                .error_at("alt", "alternative formatting can be given only with a format string"));
        };
        Ok(Some(match path.get_ident() {
            Some(name) if name == "doc_comments" || name == "docs" => Self::DocComments,
            Some(name) if name == "inner" => Self::Inner,
            Some(name) if name == "lowercase" => Self::Lowercase,
            Some(name) if name == "uppercase" => Self::Uppercase,
            Some(name) if TRAITS.iter().any(|known| name == known) => Self::Trait(name.clone()),
            _ => Self::Function(path.clone()),
        }))
    }
}

/// What one `write!` displays: a structure, a variant or a union.
///
/// The fields of a union can't be read, so a union comes without fields.
struct Item<'a> {
    name: String,
    fields: &'a Fields,
    /// The separator which joins the items of each field, if it has one.
    separators: Vec<Option<LitStr>>,
    attrs: &'a [Attribute],
    span: Span,
}

impl<'a> Item<'a> {
    fn new(name: &Ident, fields: &'a Fields, attrs: &'a [Attribute], span: Span) -> Result<Self> {
        let separator = |field: &Field| {
            let attr = field_rules().single(&field.attrs)?;
            Ok(attr.and_then(|attr| attr.str("separator").cloned()))
        };
        Ok(Self {
            name: name.unraw().to_string(),
            fields,
            separators: collect_all(fields.iter().map(separator))?,
            attrs,
            span,
        })
    }

    /// Generates the code which displays the item.
    ///
    /// Returns the fields which the code uses, as the content of a pattern
    /// which binds them, and the code.
    fn display(
        &self,
        technique: Option<&Technique>,
        bounds: &mut Bounds<'_>,
    ) -> Result<(TokenStream2, TokenStream2)> {
        let Self { fields, span, .. } = *self;
        let f = formatter();
        let (plain, alt) = match technique {
            Some(Technique::Trait(name)) => {
                bounds.require(quote!(Self), quote!(::core::fmt::#name));
                let body = quote_spanned! { span => ::core::fmt::#name::fmt(self, #f) };
                return Ok((TokenStream2::new(), body));
            }
            Some(Technique::Function(function)) => {
                let body =
                    quote_spanned! { span => ::core::fmt::Display::fmt(&#function(self), #f) };
                return Ok((TokenStream2::new(), body));
            }
            Some(Technique::Format(plain, alt)) => (plain.clone(), alt.clone()),
            Some(Technique::DocComments) => (self.docs()?, None),
            Some(Technique::Inner) if !fields.is_empty() => {
                if matches!(fields, Fields::Named(_)) && fields.len() != 1 {
                    return Err(Error::new(
                        fields.span(),
                        "display(inner) requires only single field in the structure",
                    ));
                }
                (LitStr::new("{0}", span), Some(LitStr::new("{0:#}", span)))
            }
            Some(Technique::Lowercase) => (self.with_fields(self.name.to_lowercase()), None),
            Some(Technique::Uppercase) => (self.with_fields(self.name.to_uppercase()), None),
            None | Some(Technique::Inner) => {
                let name = &self.name;
                let text = match fields {
                    Fields::Named(_) => format!("{name} {{{{ .. }}}}"),
                    Fields::Unnamed(_) => format!("{name}(..)"),
                    Fields::Unit => name.clone(),
                };
                (LitStr::new(&text, span), None)
            }
        };

        let mut used = Vec::new();
        let plain = self.write(&plain, &mut used, bounds)?;
        let alt = alt.map(|alt| self.write(&alt, &mut used, bounds));
        let body = match alt.transpose()? {
            Some(alt) if alt.to_string() != plain.to_string() => {
                quote! { if #f.alternate() { #alt } else { #plain } }
            }
            _ => plain,
        };
        let members = fields.members().enumerate();
        let members = members.filter(|(index, _)| used.contains(index));
        let bindings = members.map(|(index, member)| {
            let binding = binding(index);
            quote! { #member: #binding }
        });
        Ok((quote! { #( #bindings, )* }, body))
    }

    /// Generates the `write!` of a format string. The fields which it
    /// refers to are added to `used`, and the bounds which they need to
    /// `bounds`.
    fn write(
        &self,
        format: &LitStr,
        used: &mut Vec<usize>,
        bounds: &mut Bounds<'_>,
    ) -> Result<TokenStream2> {
        let Format { text, args, traits } =
            scan(&format.value()).map_err(|message| Error::new(format.span(), message))?;
        for arg in &args {
            if let ArgRef::Index(index) = arg &&
                *index >= self.fields.len()
            {
                let message = format!("`{}` has no field `{index}`", self.name);
                return Err(Error::new(format.span(), message));
            }
        }
        for (arg, name) in traits {
            if let Some((index, _)) = self.field(&arg) &&
                self.separators[index].is_none()
            {
                let name = Ident::new(name, self.span);
                let field = self.fields.iter().nth(index).expect("the index of a field");
                bounds.require(&field.ty, quote!(::core::fmt::#name));
            }
        }
        let args = args.iter().filter_map(|arg| {
            let (index, key) = self.field(arg)?;
            if !used.contains(&index) {
                used.push(index);
            }
            let value = binding(index);
            Some(match &self.separators[index] {
                Some(separator) => quote! { #key = #value.join(#separator) },
                None => quote! { #key = #value },
            })
        });
        let f = formatter();
        Ok(quote_spanned! { self.span => ::core::write!(#f, #text #(, #args)*) })
    }

    /// Finds the field which a format argument refers to, and makes the name
    /// which the argument has in the scanned format string.
    ///
    /// A field is referred to by its position, as `{0}` or `{_0}`, and a
    /// named field by its name as well. A format string names the field
    /// `r#type` as `{type}`.
    fn field(&self, arg: &ArgRef) -> Option<(usize, Ident)> {
        let fields = self.fields;
        let position =
            |index: usize| (index < fields.len()).then(|| (index, format_ident!("_{}", index)));
        match arg {
            ArgRef::Index(index) => position(*index),
            ArgRef::Name(name) => {
                let named = |(index, field): (usize, &Field)| {
                    Some((index, field.ident.clone().filter(|ident| ident.unraw() == name)?))
                };
                let mut fields = fields.iter().enumerate();
                fields.find_map(named).or_else(|| {
                    position(name.strip_prefix('_')?.parse().ok()?).filter(|(_, key)| key == name)
                })
            }
        }
    }

    /// Makes the format string out of the doc comments.
    fn docs(&self) -> Result<LitStr> {
        let mut text = String::new();
        let mut span = self.span;
        for attr in self.attrs.iter().filter(|attr| attr.path().is_ident("doc")) {
            let Form::Value(Value::Lit(Lit::Str(literal))) = Attr::parse(attr)?.form else {
                continue;
            };
            span = literal.span();
            let fragment = literal.value().trim().replace("\\n", "\n");
            if fragment.is_empty() || fragment == "\n" {
                text.push('\n');
            } else {
                text.push_str(&fragment);
                text.push(' ');
            }
        }
        Ok(LitStr::new(&text.trim().replace(" \n", "\n"), span))
    }

    /// Makes the format string which prints `name` followed by the fields,
    /// in the way the fields are written in the source.
    fn with_fields(&self, name: String) -> LitStr {
        let text = match self.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().enumerate().map(|(index, field)| {
                    let name = field.ident.as_ref().expect("a named field").unraw();
                    format!("{name}: {{{index}}}")
                });
                format!("{name} {{{{ {} }}}}", fields.collect::<Vec<_>>().join(", "))
            }
            Fields::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len()).map(|index| format!("{{{index}}}"));
                format!("{name}({})", fields.collect::<Vec<_>>().join(", "))
            }
            Fields::Unit => name,
        };
        LitStr::new(&text, self.span)
    }
}

/// The variable which a field is bound to.
fn binding(index: usize) -> Ident {
    format_ident!("__amplify_field_{}", index, span = Span::mixed_site())
}

/// The formatter, which a field named `f` must not shadow.
fn formatter() -> Ident { Ident::new("f", Span::mixed_site()) }

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let f = formatter();
    let mut bounds = Bounds::new(&input.generics);
    let missing = || {
        let message =
            format!("Deriving `Display`: required attribute `{NAME}` is missing.\n{EXAMPLE}");
        Error::new(input.span(), message)
    };

    let content = match &input.data {
        Data::Struct(data) => {
            let technique = Technique::read(&input.attrs)?.ok_or_else(missing)?;
            let item = Item::new(ident_name, &data.fields, &input.attrs, input.span())?;
            let (bindings, body) = item.display(Some(&technique), &mut bounds)?;
            if bindings.is_empty() {
                body
            } else {
                quote! {
                    let Self { #bindings .. } = self;
                    #body
                }
            }
        }
        Data::Enum(data) => {
            let global = Technique::read(&input.attrs)?;
            let arms = data.variants.iter().map(|variant| {
                let local = Technique::read(&variant.attrs)?;
                let technique = local.as_ref().or(global.as_ref());
                let name = &variant.ident;
                let item = Item::new(name, &variant.fields, &variant.attrs, variant.span())?;
                let (bindings, body) = item.display(technique, &mut bounds)?;
                Ok(quote_spanned! { variant.span() => Self::#name { #bindings .. } => #body, })
            });
            let arms = collect_all(arms)?;
            // A reference to an enum without variants still needs an arm.
            let value = if arms.is_empty() {
                quote!(*self)
            } else {
                quote!(self)
            };
            quote! { match #value { #( #arms )* } }
        }
        Data::Union(_) => {
            let technique = Technique::read(&input.attrs)?.ok_or_else(missing)?;
            if matches!(technique, Technique::Inner) {
                return Err(Error::new_spanned(
                    ident_name,
                    "display(inner) can't be used with a union: reading its field is unsafe",
                ));
            }
            let item = Item::new(ident_name, &Fields::Unit, &input.attrs, input.span())?;
            item.display(Some(&technique), &mut bounds)?.1
        }
    };

    let where_clause = bounds.into_where_clause();
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ident_name #ty_generics #where_clause {
            fn fmt(&self, #f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #content
            }
        }
    })
}
