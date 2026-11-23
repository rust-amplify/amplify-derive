// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// SPDX-License-Identifier: Apache-2.0
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
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Expr, Ident, Lit, LitBool, LitStr, Path, Result, Token, Type, parse2};

/// One argument in an attribute list.
#[derive(Clone, Debug)]
pub enum Arg {
    /// A literal argument.
    Lit(Lit),
    /// A type argument.
    Type(Type),
    /// A named argument of the form `name = value`.
    Named(NamedArg),
}

impl Arg {
    /// Returns the identifier for a single-segment path argument.
    pub fn as_ident(&self) -> Option<&Ident> { self.as_path()?.get_ident() }

    /// Returns the path for a path-type argument.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Type(ty) => plain_path(ty),
            _ => None,
        }
    }

    /// Creates an error attached to this argument.
    pub fn error(&self, message: impl Display) -> Error { Error::new_spanned(self, message) }
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Ident::peek_any) &&
            !input.peek(LitBool) &&
            !input.peek(Token![_]) &&
            input.peek2(Token![=])
        {
            let name = input.call(Ident::parse_any)?;
            let eq_token = input.parse()?;
            let value = input.parse()?;
            return Ok(Self::Named(NamedArg {
                name,
                eq_token,
                value,
            }));
        }

        let fork = input.fork();
        if let Ok(literal) = fork.parse::<Lit>() &&
            is_argument_end(&fork)
        {
            let _: Lit = input.parse()?;
            return Ok(Self::Lit(literal));
        }

        let fork = input.fork();
        if let Ok(ty) = fork.parse::<Type>() &&
            is_argument_end(&fork)
        {
            let _: Type = input.parse()?;
            return Ok(Self::Type(ty));
        }

        Err(input.error("expected a literal, type, or named argument"))
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Type(ty) => ty.to_tokens(tokens),
            Self::Named(named) => named.to_tokens(tokens),
        }
    }
}

/// A named attribute argument, `name = value`.
#[derive(Clone, Debug)]
pub struct NamedArg {
    /// The argument name.
    pub name: Ident,
    /// The equals token.
    pub eq_token: Token![=],
    /// The argument value.
    pub value: Value,
}

impl ToTokens for NamedArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            eq_token,
            value,
        } = self;
        quote!(#name #eq_token #value).to_tokens(tokens);
    }
}

/// A value accepted by a named attribute argument or `#[name = value]`.
#[derive(Clone, Debug)]
pub enum Value {
    /// A literal value.
    Lit(Lit),
    /// A type value.
    Type(Type),
    /// An expression value.
    Expr(Expr),
}

impl Value {
    /// Returns the value as a string literal, or an error at the value span.
    pub fn lit_str(&self) -> Result<&LitStr> {
        self.as_str()
            .ok_or_else(|| self.error("expected a string literal"))
    }

    /// Returns the string literal, if the value is one.
    pub fn as_str(&self) -> Option<&LitStr> {
        match self {
            Self::Lit(Lit::Str(lit)) => Some(lit),
            _ => None,
        }
    }

    /// Returns the type, if the value is one.
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            Self::Type(ty) => Some(ty),
            _ => None,
        }
    }

    /// Returns the path, if the value is a plain path such as
    /// `Type::function`.
    pub fn as_path(&self) -> Option<&Path> { plain_path(self.as_type()?) }

    /// Returns the identifier, if the value is a single-segment path such as
    /// `Debug`.
    pub fn as_ident(&self) -> Option<&Ident> { self.as_path()?.get_ident() }

    /// Creates an error attached to this value.
    pub fn error(&self, message: impl Display) -> Error { Error::new_spanned(self, message) }

    /// Moves a literal to `span`, so that errors about a default value point
    /// at the argument which took it.
    pub(crate) fn at(mut self, span: Span) -> Self {
        if let Self::Lit(lit) = &mut self {
            lit.set_span(span);
        }
        self
    }

    pub(crate) fn from_expr(expr: Expr) -> Self {
        let tokens = expr.to_token_stream();
        if let Ok(lit) = parse2::<Lit>(tokens.clone()) {
            return Self::Lit(lit);
        }
        if let Ok(ty) = parse2::<Type>(tokens) {
            return Self::Type(ty);
        }
        Self::Expr(expr)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self { Self::Lit(Lit::Str(LitStr::new(value, Span::call_site()))) }
}

impl From<Lit> for Value {
    fn from(lit: Lit) -> Self { Self::Lit(lit) }
}

impl From<Type> for Value {
    fn from(ty: Type) -> Self { Self::Type(ty) }
}

impl Parse for Value {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<Lit>() &&
            is_argument_end(&fork)
        {
            let _: Lit = input.parse()?;
            return Ok(Self::Lit(lit));
        }

        let fork = input.fork();
        if let Ok(ty) = fork.parse::<Type>() &&
            is_argument_end(&fork)
        {
            let _: Type = input.parse()?;
            return Ok(Self::Type(ty));
        }

        input.parse::<Expr>().map(Self::Expr)
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Type(ty) => ty.to_tokens(tokens),
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

fn is_argument_end(input: ParseStream<'_>) -> bool { input.is_empty() || input.peek(Token![,]) }

fn plain_path(ty: &Type) -> Option<&Path> {
    match ty {
        Type::Path(ty) if ty.qself.is_none() => Some(&ty.path),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, DeriveInput, Lit, Type, parse_str};

    use super::{Arg, Value};
    use crate::{Attr, Form};

    fn attribute(source: &str) -> Attribute {
        let input: DeriveInput = parse_str(source).expect("valid item with an attribute");
        input.attrs.into_iter().next().expect("attribute present")
    }

    #[test]
    fn grammar_table_classifies_all_34_inputs() {
        let cases = [
            ("\"text\"", "literal"),
            ("'c'", "literal"),
            ("b'b'", "literal"),
            ("b\"bytes\"", "literal"),
            ("10", "literal"),
            ("10u8", "literal"),
            ("1.5", "literal"),
            ("true", "literal"),
            ("false", "literal"),
            ("u8", "type"),
            ("std::io::Error", "type"),
            ("&'static str", "type"),
            ("[u8; 4]", "type"),
            ("(u8, u16)", "type"),
            ("!", "type"),
            ("Vec::<u8>", "type"),
            ("impl Into<String>", "type"),
            ("name = \"text\"", "named-literal"),
            ("bytes = b\"bytes\"", "named-literal"),
            ("flag = true", "named-literal"),
            ("count = 2 + 2", "named-expression"),
            ("max = u16::MAX as usize", "named-expression"),
            ("member = a.b", "named-expression"),
            ("block = { let x = 1; x + 1 }", "named-expression"),
            ("call = Vec::<u8>::new()", "named-expression"),
            ("ty = u8", "named-type"),
            ("path = std::io::Error", "named-type"),
            ("reference = &'static str", "named-type"),
            ("array = [u8; 4]", "named-type"),
            ("tuple = (u8, u16)", "named-type"),
            ("never = !", "named-type"),
            ("generic = Vec::<u8>", "named-type"),
            ("impl_trait = impl Into<String>", "named-type"),
            ("arithmetic = 1 - 2", "named-expression"),
        ];

        assert_eq!(cases.len(), 34);
        for (argument, expected) in cases {
            let attr = Attr::parse(&attribute(&format!("#[attr({argument})] struct Item;")))
                .expect(argument);
            let Form::List(args) = attr.form else {
                panic!("expected list form for {argument}");
            };
            let arg = args.first().expect("argument parsed");
            let actual = match arg {
                Arg::Lit(_) => "literal",
                Arg::Type(_) => "type",
                Arg::Named(named) => match &named.value {
                    Value::Lit(_) => "named-literal",
                    Value::Type(_) => "named-type",
                    Value::Expr(_) => "named-expression",
                },
            };
            assert_eq!(actual, expected, "classification of `{argument}`");
        }
    }

    #[test]
    fn parses_bare_value_and_list_forms_and_preserves_order() {
        let attrs = [
            attribute("#[setting] struct Item;"),
            attribute("#[setting = 5] struct Item;"),
            attribute("#[setting(path::Type, name = \"value\")] struct Item;"),
        ];
        let parsed = Attr::find_all(&attrs, "setting").expect("all attributes parse");
        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0].form, Form::Bare));
        assert!(matches!(parsed[1].form, Form::Value(Value::Lit(_))));
        assert!(matches!(parsed[2].form, Form::List(_)));
        assert_eq!(parsed[2].args().count(), 2);
    }

    #[test]
    fn path_and_string_value_helpers_are_typed() {
        let path_attr =
            Attr::parse(&attribute("#[attr(std::io::Error)] struct Item;")).expect("path parses");
        let Form::List(args) = path_attr.form else {
            panic!("expected list form");
        };
        let path = args
            .first()
            .expect("path argument")
            .as_path()
            .expect("path type");
        assert_eq!(quote::quote!(#path).to_string(), "std :: io :: Error");

        let value_attr = Attr::parse(&attribute("#[attr(name = \"value\")] struct Item;"))
            .expect("named value parses");
        let Form::List(args) = value_attr.form else {
            panic!("expected list form");
        };
        let Arg::Named(named) = args.first().expect("named argument") else {
            panic!("expected named argument");
        };
        assert_eq!(named.value.lit_str().expect("string literal").value(), "value");
    }

    #[test]
    fn rejects_malformed_arguments() {
        for source in [
            "#[attr(x = )] struct Item;",
            "#[attr(x y)] struct Item;",
            "#[attr(x = a b)] struct Item;",
            "#[attr(+)] struct Item;",
        ] {
            let parsed = attribute(source);
            assert!(Attr::parse(&parsed).is_err(), "accepted malformed `{source}`");
        }
    }

    #[test]
    fn calls_are_expressions_even_when_they_also_read_as_types() {
        for source in ["foo()", "Foo::new()", "Vec::<u8>::new()", "Foo::new(1, 2)", "a.b()"] {
            let value: Value = parse_str(source).expect(source);
            assert!(matches!(value, Value::Expr(_)), "`{source}` must be an expression");
        }
        for source in ["fn(u8) -> u8", "dyn Fn(u8) -> u8", "Foo", "Vec<u8>"] {
            let value: Value = parse_str(source).expect(source);
            assert!(matches!(value, Value::Type(_)), "`{source}` must be a type");
        }

        let attr =
            Attr::parse(&attribute("#[attr = Foo::new()] struct Item;")).expect("call parses");
        assert!(matches!(attr.form, Form::Value(Value::Expr(_))));
        let attr = Attr::parse(&attribute("#[attr(x = foo())] struct Item;")).expect("call parses");
        let Some(Arg::Named(named)) = attr.args().next() else {
            panic!("expected named argument");
        };
        assert!(matches!(named.value, Value::Expr(_)));
    }

    #[test]
    fn named_values_round_trip_as_literals_types_and_expressions() {
        let value: Value = syn::parse_str("Vec::<u8>::new()").expect("expression parses");
        assert!(matches!(value, Value::Expr(_)));
        let ty: Type = syn::parse_str("[u8; 4]").expect("array type parses");
        assert_eq!(quote::quote!(#ty).to_string(), "[u8 ; 4]");
        let lit: Lit = syn::parse_str("\"hello\"").expect("literal parses");
        assert_eq!(quote::quote!(#lit).to_string(), "\"hello\"");
    }

    #[test]
    fn parses_keyword_named_and_path_arguments() {
        let attr = Attr::parse(&attribute(
            r#"#[attr(type = "kind", ref = true, crate = "local", crate, self, super)] struct Item;"#,
        ))
        .expect("keyword arguments parse");
        let args = attr.args().collect::<Vec<_>>();
        assert_eq!(args.len(), 6);
        assert!(matches!(args[0], Arg::Named(named) if named.name == "type"));
        assert!(matches!(args[1], Arg::Named(named) if named.name == "ref"));
        assert!(matches!(args[2], Arg::Named(named) if named.name == "crate"));
        assert!(args[3].as_ident().is_some_and(|ident| ident == "crate"));
        assert!(args[4].as_ident().is_some_and(|ident| ident == "self"));
        assert!(args[5].as_ident().is_some_and(|ident| ident == "super"));
    }
}
