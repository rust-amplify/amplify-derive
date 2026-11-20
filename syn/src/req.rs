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

use std::fmt::{self, Display, Formatter};
use std::slice;

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Ident, Lit, Result};

use crate::{Arg, Attr, Form, ParametrizedAttr, Value};

/// The kind of value an argument or an attribute may hold.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ValueClass {
    /// Any literal.
    Lit,
    /// A string literal.
    Str,
    /// A byte string literal.
    ByteStr,
    /// A character literal.
    Char,
    /// An integer literal.
    Int,
    /// A floating-point literal.
    Float,
    /// A boolean literal.
    Bool,
    /// Any type.
    Type,
    /// A type which is a plain path, such as `Debug` or `Type::function`.
    Path,
    /// A path of a single segment, such as `Debug`.
    Ident,
    /// Any literal, type or expression.
    Any,
}

impl ValueClass {
    /// Tells whether `value` belongs to the class.
    pub fn accepts(self, value: &Value) -> bool {
        match (self, value) {
            (Self::Any, _) | (Self::Lit, Value::Lit(_)) | (Self::Type, Value::Type(_)) => true,
            (Self::Str, Value::Lit(Lit::Str(_))) |
            (Self::ByteStr, Value::Lit(Lit::ByteStr(_))) |
            (Self::Char, Value::Lit(Lit::Char(_))) |
            (Self::Int, Value::Lit(Lit::Int(_))) |
            (Self::Float, Value::Lit(Lit::Float(_))) |
            (Self::Bool, Value::Lit(Lit::Bool(_))) => true,
            (Self::Path, value) => value.as_path().is_some(),
            (Self::Ident, value) => value.as_ident().is_some(),
            _ => false,
        }
    }

    /// Returns the class of `value`: the kind of a literal, [`Self::Type`]
    /// for a type and [`Self::Any`] for an expression.
    pub fn of(value: &Value) -> Self {
        match value {
            Value::Lit(Lit::Str(_)) => Self::Str,
            Value::Lit(Lit::ByteStr(_)) => Self::ByteStr,
            Value::Lit(Lit::Char(_)) => Self::Char,
            Value::Lit(Lit::Int(_)) => Self::Int,
            Value::Lit(Lit::Float(_)) => Self::Float,
            Value::Lit(Lit::Bool(_)) => Self::Bool,
            Value::Lit(_) => Self::Lit,
            Value::Type(_) => Self::Type,
            Value::Expr(_) => Self::Any,
        }
    }
}

impl Display for ValueClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lit => "a literal",
            Self::Str => "a string literal",
            Self::ByteStr => "a byte string literal",
            Self::Char => "a character literal",
            Self::Int => "an integer literal",
            Self::Float => "a floating-point literal",
            Self::Bool => "a boolean literal",
            Self::Type => "a type",
            Self::Path => "a path",
            Self::Ident => "an identifier",
            Self::Any => "a value",
        })
    }
}

/// Whether a named argument holds a value, and of which kind.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ArgValueReq {
    /// The argument is a flag and takes no value, as `skip` in
    /// `#[getter(skip)]`.
    Prohibited,

    /// The argument may be given with a value of the class or without one.
    Optional(ValueClass),

    /// The argument holds a value of the class.
    Required {
        /// The kind of the value.
        class: ValueClass,
        /// The value the argument holds when it is given as a bare name.
        /// Without a default the value has to be written out.
        default: Option<Value>,
    },
}

impl ArgValueReq {
    /// An argument whose value has to be written out, as in `name = "value"`.
    pub fn required(class: ValueClass) -> Self {
        Self::Required {
            class,
            default: None,
        }
    }

    /// An argument which holds `default` when it is given as a bare name. A
    /// value written out must be of the same kind as the default.
    pub fn with_default(default: impl Into<Value>) -> Self {
        let default = default.into();
        Self::Required {
            class: ValueClass::of(&default),
            default: Some(default),
        }
    }
}

/// The unnamed arguments an attribute takes: their kind and their number.
#[derive(Clone, Debug)]
pub struct ListReq {
    classes: Vec<ValueClass>,
    names: Option<Vec<&'static str>>,
    min: usize,
    max: Option<usize>,
}

impl ListReq {
    fn new(classes: Vec<ValueClass>, min: usize, max: Option<usize>) -> Self {
        Self {
            classes,
            names: None,
            min,
            max,
        }
    }

    /// No unnamed arguments.
    pub fn deny() -> Self { Self::new(Vec::new(), 0, Some(0)) }

    /// Exactly one argument of the class.
    pub fn one(class: ValueClass) -> Self { Self::new(vec![class], 1, Some(1)) }

    /// One argument of the class, or none.
    pub fn maybe_one(class: ValueClass) -> Self { Self::new(vec![class], 0, Some(1)) }

    /// Any number of arguments of the class.
    pub fn any(class: ValueClass) -> Self { Self::new(vec![class], 0, None) }

    /// Any number of identifiers out of `names`.
    pub fn any_of(names: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            names: Some(names.into_iter().collect()),
            ..Self::any(ValueClass::Ident)
        }
    }

    /// Accepts arguments of one more class.
    #[must_use]
    pub fn or(mut self, class: ValueClass) -> Self {
        self.classes.push(class);
        self
    }

    fn accepts(&self, value: &Value) -> bool {
        self.classes.iter().any(|class| class.accepts(value)) &&
            self.names.as_ref().is_none_or(|names| {
                value
                    .as_ident()
                    .is_some_and(|ident| names.iter().any(|name| ident == name))
            })
    }

    fn expected(&self) -> String {
        let classes = self.classes.iter().map(ValueClass::to_string);
        classes.collect::<Vec<_>>().join(" or ")
    }
}

#[derive(Clone, Debug)]
struct ArgReq {
    name: &'static str,
    value: ArgValueReq,
    mandatory: bool,
}

/// The rules for the content of an attribute: which arguments it takes, of
/// which kind, and which of them exclude each other.
///
/// An attribute is read as a list of arguments. `#[name]` is the empty list,
/// and `#[name = value]` is the list of that one value where
/// [`AttrReq::name_value`] allows the form. An argument is *named* when it is
/// written as `name = value`, or when it is a bare identifier declared with
/// [`AttrReq::arg`]. Any other literal or type is *unnamed*.
///
/// Whatever the rules do not allow is an error at the offending argument, and
/// all errors found in an attribute are reported together.
///
/// ```
/// use amplify_syn::{ArgValueReq, AttrReq, ListReq, ValueClass};
/// use syn::{DeriveInput, parse_quote};
///
/// let rules = AttrReq::new("getter")
///     .arg("prefix", ArgValueReq::with_default(""))
///     .arg("as_mut", ArgValueReq::with_default("_mut"))
///     .arg("skip", ArgValueReq::Prohibited)
///     .conflict("skip", &["prefix", "as_mut"])
///     .unnamed(ListReq::maybe_one(ValueClass::Type));
///
/// let input: DeriveInput = parse_quote! {
///     #[getter(u8, prefix = "get_")]
///     #[getter(as_mut)]
///     struct Item;
/// };
/// let getter = rules.merged(&input.attrs)?;
/// assert_eq!(getter.str("prefix").map(|prefix| prefix.value()), Some("get_".to_owned()));
/// assert_eq!(getter.str("as_mut").map(|suffix| suffix.value()), Some("_mut".to_owned()));
/// assert!(!getter.has("skip"));
/// assert!(getter.unnamed()[0].as_ident().is_some_and(|ty| ty == "u8"));
///
/// let input: DeriveInput = parse_quote! {
///     #[getter(skip, as_mut, sufix = "_ref")]
///     struct Item;
/// };
/// let errors = rules.merged(&input.attrs).unwrap_err();
/// let messages = errors.into_iter().map(|error| error.to_string());
/// assert_eq!(messages.collect::<Vec<_>>(), [
///     "unknown `getter` argument `sufix`",
///     "`skip` and `as_mut` cannot be used together"
/// ]);
/// # Ok::<_, syn::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct AttrReq {
    name: &'static str,
    args: Vec<ArgReq>,
    conflicts: Vec<(&'static str, &'static str)>,
    unnamed: ListReq,
    name_value: Option<ValueClass>,
}

impl AttrReq {
    /// Starts the rules for the attribute `name`. The attribute takes no
    /// arguments until they are declared.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            args: Vec::new(),
            conflicts: Vec::new(),
            unnamed: ListReq::deny(),
            name_value: None,
        }
    }

    /// Declares a named argument, which may be left out.
    #[must_use]
    pub fn arg(mut self, name: &'static str, value: ArgValueReq) -> Self {
        self.args.push(ArgReq {
            name,
            value,
            mandatory: false,
        });
        self
    }

    /// Declares a named argument which must be given whenever the attribute
    /// is.
    #[must_use]
    pub fn mandatory(mut self, name: &'static str, value: ArgValueReq) -> Self {
        self.args.push(ArgReq {
            name,
            value,
            mandatory: true,
        });
        self
    }

    /// Declares that the argument `name` cannot be used together with any of
    /// `others`.
    ///
    /// # Panics
    ///
    /// Panics if one of the arguments is not declared yet.
    #[must_use]
    pub fn conflict(mut self, name: &'static str, others: &[&'static str]) -> Self {
        for name in others.iter().chain([&name]) {
            let declared = self.args.iter().any(|arg| arg.name == *name);
            assert!(declared, "the rules declare no argument `{name}`");
        }
        let pairs = others.iter().map(|other| (name, *other));
        self.conflicts.extend(pairs);
        self
    }

    /// Declares the unnamed arguments.
    #[must_use]
    pub fn unnamed(mut self, list: ListReq) -> Self {
        self.unnamed = list;
        self
    }

    /// Allows the `#[name = value]` form for a value of the class. It reads
    /// as `#[name(value)]`.
    #[must_use]
    pub fn name_value(mut self, class: ValueClass) -> Self {
        self.name_value = Some(class);
        self
    }

    /// Reads an attribute which may be given once. Returns `None` when it is
    /// absent.
    pub fn single<'a>(
        &self,
        attrs: impl IntoIterator<Item = &'a Attribute>,
    ) -> Result<Option<ParametrizedAttr>> {
        let found = Attr::find_all(attrs, self.name)?;
        if let Some(repeated) = found.get(1) {
            return Err(repeated.error(format!("only one `{}` attribute is allowed", self.name)));
        }
        found
            .first()
            .map(|attr| self.check(slice::from_ref(attr)))
            .transpose()
    }

    /// Reads all occurrences of an attribute as one list of arguments. The
    /// content is empty when the attribute is absent.
    pub fn merged<'a>(
        &self,
        attrs: impl IntoIterator<Item = &'a Attribute>,
    ) -> Result<ParametrizedAttr> {
        self.check(&Attr::find_all(attrs, self.name)?)
    }

    /// Reads every occurrence of an attribute on its own, in source order.
    pub fn each<'a>(
        &self,
        attrs: impl IntoIterator<Item = &'a Attribute>,
    ) -> Result<Vec<ParametrizedAttr>> {
        let found = Attr::find_all(attrs, self.name)?;
        collect_all(found.iter().map(|attr| self.check(slice::from_ref(attr))))
    }

    /// Checks parsed occurrences of the attribute, read as one list of
    /// arguments.
    pub fn check(&self, occurrences: &[Attr]) -> Result<ParametrizedAttr> {
        let mut check = Check {
            rules: self,
            errors: Errors::default(),
            content: ParametrizedAttr {
                span: occurrences
                    .first()
                    .map_or_else(Span::call_site, |attr| attr.path.span()),
                declared: self.args.iter().map(|arg| arg.name).collect(),
                named: Vec::new(),
                unnamed: Vec::new(),
            },
        };
        for attr in occurrences {
            match &attr.form {
                Form::Bare => {}
                Form::Value(value) => check.name_value(attr, value),
                Form::List(args) => args.iter().for_each(|arg| check.arg(arg)),
            }
        }
        if !occurrences.is_empty() {
            check.totals();
        }
        check.errors.finish()?;
        Ok(check.content)
    }

    fn arg_req(&self, name: &Ident) -> Option<&ArgReq> {
        self.args.iter().find(|arg| name == arg.name)
    }
}

/// One pass of an attribute through its rules.
struct Check<'rules> {
    rules: &'rules AttrReq,
    content: ParametrizedAttr,
    errors: Errors,
}

impl Check<'_> {
    fn name_value(&mut self, attr: &Attr, value: &Value) {
        let AttrReq {
            name,
            args,
            unnamed: list,
            name_value,
            ..
        } = self.rules;
        match name_value {
            Some(class) if class.accepts(value) => return self.unnamed(value.clone()),
            Some(class) => self.error(value, format!("expected {class}")),
            None if args.is_empty() && list.classes.is_empty() => {
                self.error(value, format!("`{name}` takes no value"))
            }
            None => self
                .errors
                .push(attr.error(format!("expected `#[{name}(...)]`, found `#[{name} = ...]`"))),
        }
        self.content.unnamed.push(value.clone());
    }

    fn arg(&mut self, arg: &Arg) {
        let rules = self.rules;
        match arg {
            Arg::Named(arg) => self.named(&arg.name, Some(arg.value.clone())),
            Arg::Lit(lit) => self.unnamed(Value::Lit(lit.clone())),
            Arg::Type(ty) => match arg.as_ident().filter(|name| rules.arg_req(name).is_some()) {
                Some(name) => self.named(name, None),
                None => self.unnamed(Value::Type(ty.clone())),
            },
        }
    }

    fn named(&mut self, name: &Ident, value: Option<Value>) {
        let rules = self.rules;
        let attr = rules.name;
        let Some(arg) = rules.arg_req(name) else {
            return self.error(name, format!("unknown `{attr}` argument `{name}`"));
        };
        if self.content.has(arg.name) {
            return self.error(name, format!("repeated `{attr}` argument `{name}`"));
        }
        let value = match (&arg.value, value) {
            (ArgValueReq::Prohibited, Some(value)) => {
                self.error(&value, format!("`{name}` takes no value"));
                None
            }
            (ArgValueReq::Optional(class) | ArgValueReq::Required { class, .. }, Some(value)) => {
                if !class.accepts(&value) {
                    self.error(&value, format!("expected {class}"));
                }
                Some(value)
            }
            (
                ArgValueReq::Required {
                    class,
                    default: None,
                },
                None,
            ) => {
                self.error(name, format!("`{name}` requires a value: expected {class}"));
                None
            }
            (
                ArgValueReq::Required {
                    default: Some(default),
                    ..
                },
                None,
            ) => Some(default.clone().at(name.span())),
            (ArgValueReq::Prohibited | ArgValueReq::Optional(_), None) => None,
        };
        self.content.named.push((name.clone(), value));
    }

    fn unnamed(&mut self, value: Value) {
        let AttrReq {
            name,
            args,
            unnamed: list,
            ..
        } = self.rules;
        if !list.accepts(&value) {
            let message = match value.as_ident() {
                Some(ident) => format!("unknown `{name}` argument `{ident}`"),
                None if !list.classes.is_empty() => format!("expected {}", list.expected()),
                None if args.is_empty() => format!("`{name}` takes no arguments"),
                None => format!("`{name}` takes only named arguments"),
            };
            self.error(&value, message);
        } else if let Some(max) = list.max.filter(|max| self.content.unnamed.len() >= *max) {
            let limit = match max {
                1 => "a single unnamed argument".to_owned(),
                max => format!("at most {max} unnamed arguments"),
            };
            self.error(&value, format!("unexpected argument: `{name}` takes {limit}"));
        }
        self.content.unnamed.push(value);
    }

    /// Checks the rules which concern the attribute as a whole.
    fn totals(&mut self) {
        let AttrReq {
            name,
            args,
            conflicts,
            unnamed: list,
            ..
        } = self.rules;
        let Self {
            content, errors, ..
        } = self;
        if content.unnamed.len() < list.min {
            let number = match list.min {
                1 => "an argument".to_owned(),
                min => format!("at least {min} arguments"),
            };
            let expected = list.expected();
            errors.push(content.error(format!("`{name}` requires {number}: expected {expected}")));
        }
        for arg in args.iter().filter(|arg| arg.mandatory) {
            if !content.has(arg.name) {
                let arg = arg.name;
                errors.push(content.error(format!("`{name}` requires the `{arg}` argument")));
            }
        }
        for (first, second) in conflicts {
            if let (Some(first), Some(second)) = (content.position(first), content.position(second))
            {
                let earlier = &content.named[first.min(second)].0;
                let later = &content.named[first.max(second)].0;
                errors.push(Error::new(
                    later.span(),
                    format!("`{earlier}` and `{later}` cannot be used together"),
                ));
            }
        }
    }

    fn error(&mut self, at: &impl ToTokens, message: String) {
        self.errors.push(Error::new_spanned(at, message));
    }
}

/// Collects the values of `results`, or all of their errors as one.
///
/// A derive macro reads the attributes of fields and variants through this
/// function, so that it reports every faulty one, not just the first.
///
/// ```
/// use amplify_syn::{AttrReq, collect_all};
/// use syn::{DeriveInput, parse_quote};
///
/// let input: DeriveInput = parse_quote! {
///     struct Item(#[wrap(x)] u8, #[wrap] u8, #[wrap = 1] u8);
/// };
/// let syn::Data::Struct(data) = &input.data else { unreachable!() };
/// let rules = AttrReq::new("wrap");
/// let errors = collect_all(data.fields.iter().map(|field| rules.single(&field.attrs)));
/// assert_eq!(errors.unwrap_err().into_iter().count(), 2);
/// ```
pub fn collect_all<T>(results: impl IntoIterator<Item = Result<T>>) -> Result<Vec<T>> {
    let mut errors = Errors::default();
    let mut values = Vec::new();
    for result in results {
        match result {
            Ok(value) => values.push(value),
            Err(error) => errors.push(error),
        }
    }
    errors.finish()?;
    Ok(values)
}

/// Collects independent errors and reports them together.
#[derive(Default)]
struct Errors(Option<Error>);

impl Errors {
    fn push(&mut self, error: Error) {
        match &mut self.0 {
            Some(errors) => errors.combine(error),
            None => self.0 = Some(error),
        }
    }

    fn finish(self) -> Result<()> { self.0.map_or(Ok(()), Err) }
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, DeriveInput, parse_str};

    use super::{ArgValueReq, AttrReq, ListReq, ValueClass};
    use crate::ParametrizedAttr;

    fn attrs(source: &str) -> Vec<Attribute> {
        let input: DeriveInput = parse_str(&format!("{source} struct Item;")).expect(source);
        input.attrs
    }

    fn messages<T>(result: syn::Result<T>) -> Vec<String> {
        let errors = result.err().expect("the rules must reject the attribute");
        errors.into_iter().map(|error| error.to_string()).collect()
    }

    fn str(attr: &ParametrizedAttr, name: &str) -> Option<String> {
        attr.str(name).map(|value| value.value())
    }

    fn getter() -> AttrReq {
        AttrReq::new("getter")
            .arg("prefix", ArgValueReq::with_default(""))
            .arg("all", ArgValueReq::Prohibited)
            .arg("as_copy", ArgValueReq::with_default(""))
            .arg("as_clone", ArgValueReq::with_default(""))
            .arg("as_mut", ArgValueReq::with_default("_mut"))
            .arg("base_name", ArgValueReq::required(ValueClass::Str))
            .arg("limit", ArgValueReq::Optional(ValueClass::Int))
            .conflict("all", &["as_copy", "as_clone", "as_mut"])
            .conflict("as_copy", &["as_clone"])
    }

    #[test]
    fn named_arguments_hold_written_values_and_defaults() {
        let rules = getter();
        let attr = rules
            .merged(&attrs(r#"#[getter(prefix = "get_", as_mut, limit, base_name = "id")]"#))
            .expect("the rules accept the attribute");
        assert_eq!(str(&attr, "prefix").as_deref(), Some("get_"));
        assert_eq!(str(&attr, "as_mut").as_deref(), Some("_mut"));
        assert_eq!(str(&attr, "base_name").as_deref(), Some("id"));
        assert!(attr.has("limit") && attr.value("limit").is_none());
        assert!(!attr.has("all") && !attr.has("as_copy") && attr.value("as_copy").is_none());
        assert!(attr.unnamed().is_empty());

        let attr = rules
            .merged(&attrs("#[getter(all, limit = 5, prefix)]"))
            .expect("the rules accept the attribute");
        assert!(attr.has("all") && attr.value("all").is_none());
        assert!(attr.value("limit").is_some() && attr.str("limit").is_none());
        assert_eq!(str(&attr, "prefix").as_deref(), Some(""));
    }

    #[test]
    fn named_argument_violations_are_reported_together() {
        let rules = getter();
        assert_eq!(
            messages(rules.merged(&attrs(
                r#"#[getter(as_cpoy, sufix = "x", all = true, prefix = 5, base_name, limit = "x")]"#
            ))),
            [
                "unknown `getter` argument `as_cpoy`",
                "unknown `getter` argument `sufix`",
                "`all` takes no value",
                "expected a string literal",
                "`base_name` requires a value: expected a string literal",
                "expected an integer literal",
            ]
        );
        assert_eq!(messages(rules.merged(&attrs("#[getter(as_mut, as_mut)]"))), [
            "repeated `getter` argument `as_mut`"
        ]);
        assert_eq!(messages(rules.merged(&attrs("#[getter(as_clone, all, as_copy)]"))), [
            "`all` and `as_copy` cannot be used together",
            "`as_clone` and `all` cannot be used together",
            "`as_clone` and `as_copy` cannot be used together",
        ]);
        assert_eq!(messages(rules.merged(&attrs(r#"#[getter("x", u8, [u8; 2])]"#))), [
            "`getter` takes only named arguments",
            "unknown `getter` argument `u8`",
            "`getter` takes only named arguments",
        ]);
        assert_eq!(messages(rules.merged(&attrs("#[getter = 1]"))), [
            "expected `#[getter(...)]`, found `#[getter = ...]`"
        ]);
    }

    #[test]
    fn mandatory_arguments_are_required_only_when_the_attribute_is_given() {
        let rules =
            AttrReq::new("display").mandatory("separator", ArgValueReq::required(ValueClass::Str));
        assert!(rules.single(&attrs("")).expect("absent").is_none());
        assert!(
            rules
                .merged(&attrs(""))
                .expect("absent")
                .value("separator")
                .is_none()
        );
        for source in ["#[display]", "#[display()]"] {
            assert_eq!(messages(rules.single(&attrs(source))), [
                "`display` requires the `separator` argument"
            ]);
        }
        let attr = rules
            .single(&attrs(r#"#[display(separator = ", ")]"#))
            .expect("the rules accept the attribute")
            .expect("the attribute is present");
        assert_eq!(str(&attr, "separator").as_deref(), Some(", "));
    }

    #[test]
    fn occurrences_are_read_once_merged_or_one_by_one() {
        let rules = getter();
        let repeated = attrs(r#"#[getter(prefix = "a")] #[other] #[getter(as_mut)]"#);

        assert_eq!(messages(rules.single(&repeated)), ["only one `getter` attribute is allowed"]);
        assert!(
            rules
                .single(&repeated[..2])
                .expect("one occurrence")
                .is_some()
        );

        let merged = rules.merged(&repeated).expect("the occurrences add up");
        assert!(merged.has("prefix") && merged.has("as_mut"));
        assert_eq!(
            messages(rules.merged(&attrs(r#"#[getter(prefix = "a")] #[getter(prefix = "b")]"#))),
            ["repeated `getter` argument `prefix`"]
        );
        assert_eq!(messages(rules.merged(&attrs("#[getter(as_copy)] #[getter(as_clone)]"))), [
            "`as_copy` and `as_clone` cannot be used together"
        ]);

        let each = rules.each(&repeated).expect("each occurrence is valid");
        assert_eq!(each.len(), 2);
        assert!(each[0].has("prefix") && !each[0].has("as_mut"));
        assert!(each[1].has("as_mut") && !each[1].has("prefix"));
        let each = rules.each(&attrs(r#"#[getter(prefix = "a")] #[getter(prefix = "b")]"#));
        assert_eq!(
            each.expect("no argument repeats within an occurrence")
                .len(),
            2
        );
        assert_eq!(messages(rules.each(&attrs("#[getter(x)] #[getter(as_mut)] #[getter(y)]"))), [
            "unknown `getter` argument `x`",
            "unknown `getter` argument `y`",
        ]);
        assert!(rules.each(&attrs("")).expect("absent").is_empty());
    }

    #[test]
    fn unnamed_arguments_are_checked_for_kind_and_number() {
        let display = AttrReq::new("display")
            .unnamed(ListReq::one(ValueClass::Str).or(ValueClass::Path))
            .name_value(ValueClass::Str)
            .arg("alt", ArgValueReq::required(ValueClass::Str));
        for source in [r#"#[display("{}")]"#, r#"#[display = "{}"]"#] {
            let attr = display.merged(&attrs(source)).expect(source);
            let [format] = attr.unnamed() else {
                panic!("one unnamed argument is expected for {source}");
            };
            assert_eq!(format.as_str().map(|format| format.value()).as_deref(), Some("{}"));
        }
        let attr = display
            .merged(&attrs(r#"#[display(Type::function, alt = "x")]"#))
            .expect("a path is accepted");
        assert!(attr.unnamed()[0].as_path().is_some() && attr.unnamed()[0].as_ident().is_none());
        assert_eq!(str(&attr, "alt").as_deref(), Some("x"));

        assert_eq!(messages(display.merged(&attrs("#[display]"))), ["`display` requires an \
                                                                     argument: expected a \
                                                                     string literal or a path"]);
        assert_eq!(messages(display.merged(&attrs(r#"#[display(alt = "x")]"#))), [
            "`display` requires an argument: expected a string literal or a path"
        ]);
        assert_eq!(messages(display.merged(&attrs(r#"#[display("a", "b", 5)]"#))), [
            "unexpected argument: `display` takes a single unnamed argument",
            "expected a string literal or a path",
        ]);
        for source in ["#[display = 5]", "#[display = Debug]"] {
            assert_eq!(messages(display.merged(&attrs(source))), ["expected a string literal"]);
        }
        assert_eq!(messages(display.merged(&attrs("#[display(alt)]"))), [
            "`alt` requires a value: expected a string literal",
            "`display` requires an argument: expected a string literal or a path",
        ]);

        let from = AttrReq::new("from").unnamed(ListReq::maybe_one(ValueClass::Type));
        for source in ["#[from]", "#[from()]", "#[from(&'static str)]", "#[from(!)]"] {
            let attr = from.merged(&attrs(source)).expect(source);
            assert_eq!(attr.unnamed().len(), usize::from(source.len() > 9), "{source}");
        }
        assert_eq!(messages(from.merged(&attrs(r#"#[from("u8")]"#))), ["expected a type"]);
        assert_eq!(messages(from.merged(&attrs("#[from(u8, u16)]"))), ["unexpected argument: \
                                                                        `from` takes a single \
                                                                        unnamed argument"]);
        let path = AttrReq::new("amplify_crate").unnamed(ListReq::one(ValueClass::Path));
        assert_eq!(messages(path.single(&attrs(r#"#[amplify_crate = "x"]"#))), ["expected `#\
                                                                                 [amplify_crate(.\
                                                                                 ..)]`, found \
                                                                                 `#[amplify_crate \
                                                                                 = ...]`"]);

        let wrap = AttrReq::new("wrap");
        assert!(wrap.single(&attrs("#[wrap]")).expect("bare").is_some());
        assert_eq!(
            messages(wrap.single(&attrs("#[wrap(yes)]"))),
            ["unknown `wrap` argument `yes`"]
        );
        assert_eq!(messages(wrap.single(&attrs("#[wrap(1)]"))), ["`wrap` takes no arguments"]);
        assert_eq!(messages(wrap.single(&attrs("#[wrap(yes = 1)]"))), [
            "unknown `wrap` argument `yes`"
        ]);
        assert_eq!(messages(wrap.single(&attrs("#[wrap = 1]"))), ["`wrap` takes no value"]);

        let pair = AttrReq::new("pair").unnamed(ListReq::any(ValueClass::Int));
        assert_eq!(
            pair.merged(&attrs("#[pair(1, 2)] #[pair(3)]"))
                .expect("integers")
                .unnamed()
                .len(),
            3
        );
    }

    #[test]
    fn unnamed_identifiers_are_limited_to_the_listed_names() {
        let wrapper = AttrReq::new("wrapper").unnamed(ListReq::any_of(["Add", "Sub", "NoRefs"]));
        let attr = wrapper
            .merged(&attrs("#[wrapper(Add, NoRefs)] #[wrapper] #[wrapper(Sub, Add)]"))
            .expect("listed names are accepted");
        let names = attr.unnamed().iter().filter_map(|value| value.as_ident());
        assert_eq!(names.map(ToString::to_string).collect::<Vec<_>>(), [
            "Add", "NoRefs", "Sub", "Add"
        ]);
        assert_eq!(messages(wrapper.merged(&attrs(r#"#[wrapper(Bogus, "x", ops::Add, Add)]"#))), [
            "unknown `wrapper` argument `Bogus`",
            "expected an identifier",
            "expected an identifier",
        ]);
    }

    #[test]
    #[should_panic(expected = "the rules declare no argument `skp`")]
    fn asking_for_an_undeclared_argument_panics() {
        let attr = getter().merged(&attrs("#[getter(all)]"));
        attr.expect("the rules accept the attribute").has("skp");
    }

    #[test]
    #[should_panic(expected = "the rules declare no argument `as_cpoy`")]
    fn a_conflict_with_an_undeclared_argument_panics() {
        let _ = getter().conflict("all", &["as_cpoy"]);
    }

    #[test]
    fn value_classes_accept_their_values_only() {
        let classes = [
            (ValueClass::Lit, r#""x" b"x" 'c' 1 1.5 true"#),
            (ValueClass::Str, r#""x""#),
            (ValueClass::ByteStr, r#"b"x""#),
            (ValueClass::Char, "'c'"),
            (ValueClass::Int, "1"),
            (ValueClass::Float, "1.5"),
            (ValueClass::Bool, "true"),
            (ValueClass::Type, "Debug a::B <T>::C [u8;4]"),
            (ValueClass::Path, "Debug a::B"),
            (ValueClass::Ident, "Debug"),
            (ValueClass::Any, r#""x" b"x" 'c' 1 1.5 true Debug a::B <T>::C [u8;4] 1+1"#),
        ];
        let values = r#""x" b"x" 'c' 1 1.5 true Debug a::B <T>::C [u8;4] 1+1"#;
        for (class, accepted) in classes {
            let rules = AttrReq::new("attr").arg("arg", ArgValueReq::required(class));
            for value in values.split(' ') {
                let result = rules.merged(&attrs(&format!("#[attr(arg = {value})]")));
                let expected = accepted.split(' ').any(|accepted| accepted == value);
                assert_eq!(result.is_ok(), expected, "{value} as {class}");
            }
        }
    }
}
