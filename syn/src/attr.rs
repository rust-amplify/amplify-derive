// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2021 by
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

use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Ident, LitStr, Meta, Path, Result, Token};

use crate::{Arg, Value};

/// One occurrence of an attribute.
#[derive(Clone, Debug)]
pub struct Attr {
    /// The attribute path.
    pub path: Path,
    /// The attribute's syntactic form.
    pub form: Form,
}

impl Attr {
    /// Parses one Rust attribute into the supported attribute grammar.
    pub fn parse(attr: &Attribute) -> Result<Self> {
        match &attr.meta {
            Meta::Path(path) => Ok(Self {
                path: path.clone(),
                form: Form::Bare,
            }),
            Meta::NameValue(name_value) => Ok(Self {
                path: name_value.path.clone(),
                form: Form::Value(Value::from_expr(name_value.value.clone())),
            }),
            Meta::List(list) => {
                let parser = Punctuated::<Arg, Token![,]>::parse_terminated;
                let args = parser.parse2(list.tokens.clone())?;
                Ok(Self {
                    path: list.path.clone(),
                    form: Form::List(args),
                })
            }
        }
    }

    /// Parses every attribute named `name`, preserving source order.
    pub fn find_all<'a>(
        attrs: impl IntoIterator<Item = &'a Attribute>,
        name: &str,
    ) -> Result<Vec<Self>> {
        attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident(name))
            .map(Self::parse)
            .collect()
    }

    /// Returns the list arguments, or an empty iterator for other forms.
    pub fn args(&self) -> impl Iterator<Item = &Arg> {
        let args = match &self.form {
            Form::List(args) => Some(args),
            Form::Bare | Form::Value(_) => None,
        };
        args.into_iter().flat_map(|args| args.iter())
    }

    /// Creates an error attached to the attribute path.
    pub fn error(&self, message: impl Display) -> Error { Error::new_spanned(&self.path, message) }
}

impl ToTokens for Attr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.path;
        let form = &self.form;
        quote!(#path #form).to_tokens(tokens);
    }
}

/// The form of one attribute occurrence.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Form {
    /// A bare attribute, such as `#[name]`.
    Bare,
    /// An attribute with one value, such as `#[name = value]`.
    Value(Value),
    /// An attribute with a comma-separated argument list.
    List(Punctuated<Arg, Token![,]>),
}

impl ToTokens for Form {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Bare => {}
            Self::Value(value) => quote!(= #value).to_tokens(tokens),
            Self::List(args) => quote!((#args)).to_tokens(tokens),
        }
    }
}

/// The content of an attribute which has passed its [`AttrReq`].
///
/// It is produced by [`AttrReq::single`], [`AttrReq::merged`] and
/// [`AttrReq::each`], so every argument in it is one the rules allow, and
/// every value is of the kind the rules ask for.
///
/// # Panics
///
/// The methods which take the name of an argument panic when the rules do
/// not declare that argument. Asking for it is a fault of the derive macro,
/// and it would stay unnoticed if the answer were "not given".
///
/// [`AttrReq`]: crate::AttrReq
/// [`AttrReq::single`]: crate::AttrReq::single
/// [`AttrReq::merged`]: crate::AttrReq::merged
/// [`AttrReq::each`]: crate::AttrReq::each
#[derive(Clone, Debug)]
pub struct ParametrizedAttr {
    pub(crate) span: Span,
    /// The named arguments which the rules declare.
    pub(crate) declared: Vec<&'static str>,
    pub(crate) named: Vec<(Ident, Option<Value>)>,
    pub(crate) unnamed: Vec<Value>,
}

impl ParametrizedAttr {
    /// Returns the span of the attribute name. When the attribute is absent
    /// this is the span of the macro call.
    pub fn span(&self) -> Span { self.span }

    /// Tells whether the named argument is given.
    pub fn has(&self, name: &str) -> bool { self.position(name).is_some() }

    /// Returns the value of a named argument: the one written out, or the
    /// default from the rules when the argument is given as a bare name.
    pub fn value(&self, name: &str) -> Option<&Value> {
        self.named[self.position(name)?].1.as_ref()
    }

    /// Returns the value of a named argument which holds a string literal.
    pub fn str(&self, name: &str) -> Option<&LitStr> { self.value(name)?.as_str() }

    /// Returns the unnamed arguments in source order. The value of
    /// `#[name = value]` is the only one.
    pub fn unnamed(&self) -> &[Value] { &self.unnamed }

    /// Creates an error attached to the attribute name.
    pub fn error(&self, message: impl Display) -> Error { Error::new(self.span, message) }

    /// Creates an error attached to a named argument, or to the attribute
    /// name when the argument is absent.
    pub fn error_at(&self, name: &str, message: impl Display) -> Error {
        let span = self
            .position(name)
            .map_or(self.span, |index| self.named[index].0.span());
        Error::new(span, message)
    }

    pub(crate) fn position(&self, name: &str) -> Option<usize> {
        assert!(self.declared.contains(&name), "the rules declare no argument `{name}`");
        self.named.iter().position(|(ident, _)| ident == name)
    }
}
