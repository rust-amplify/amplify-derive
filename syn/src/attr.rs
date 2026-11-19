// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2021 by
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
/// [`AttrReq`]: crate::AttrReq
/// [`AttrReq::single`]: crate::AttrReq::single
/// [`AttrReq::merged`]: crate::AttrReq::merged
/// [`AttrReq::each`]: crate::AttrReq::each
#[derive(Clone, Debug)]
pub struct ParametrizedAttr {
    pub(crate) span: Span,
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
        self.named.iter().position(|(ident, _)| ident == name)
    }
}
