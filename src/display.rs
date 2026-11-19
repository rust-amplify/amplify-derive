// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Elichai Turkel <elichai.turkel@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify_syn::{ArgValueReq, Attr, AttrReq, Form, ListReq, Value, ValueClass};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Error, Fields, Ident, Index,
    Lit, LitStr, Path, Result,
};

use crate::format::{ArgRef, Format, scan};

const NAME: &str = "display";
const EXAMPLE: &str = r#"#[display("format {} string" | Trait | Type::function)]"#;
const INNER_ERR: &str = "display(inner) requires only single field in the structure";

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum FormattingTrait {
    Debug,
    Octal,
    Binary,
    Pointer,
    LowerHex,
    UpperHex,
    LowerExp,
    UpperExp,
}

impl FormattingTrait {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.segments.first()?.ident.to_string().as_str() {
            "Debug" => Some(FormattingTrait::Debug),
            "Octal" => Some(FormattingTrait::Octal),
            "Binary" => Some(FormattingTrait::Binary),
            "Pointer" => Some(FormattingTrait::Pointer),
            "LowerHex" => Some(FormattingTrait::LowerHex),
            "UpperHex" => Some(FormattingTrait::UpperHex),
            "LowerExp" => Some(FormattingTrait::LowerExp),
            "UpperExp" => Some(FormattingTrait::UpperExp),
            _ => None,
        }
    }

    pub fn to_fmt(self, alt: bool) -> TokenStream2 {
        let mut fmt = match self {
            FormattingTrait::Debug => "{:?}",
            FormattingTrait::Octal => "{:o}",
            FormattingTrait::Binary => "{:b}",
            FormattingTrait::Pointer => "{:p}",
            FormattingTrait::LowerHex => "{:x}",
            FormattingTrait::UpperHex => "{:X}",
            FormattingTrait::LowerExp => "{:e}",
            FormattingTrait::UpperExp => "{:E}",
        }
        .to_owned();
        if alt {
            fmt = fmt.replace(':', ":#");
        }
        quote! { #fmt }
    }

    pub fn into_token_stream2(self, span: Span) -> TokenStream2 {
        let formatter = formatter_ident();
        match self {
            FormattingTrait::Debug => quote_spanned! { span =>
                ::core::fmt::Debug::fmt(&self, #formatter)
            },
            FormattingTrait::Octal => quote_spanned! { span =>
                ::core::fmt::Octal::fmt(&self, #formatter)
            },
            FormattingTrait::Binary => quote_spanned! { span =>
                ::core::fmt::Binary::fmt(&self, #formatter)
            },
            FormattingTrait::Pointer => quote_spanned! { span =>
                ::core::fmt::Pointer::fmt(&self, #formatter)
            },
            FormattingTrait::LowerHex => quote_spanned! { span =>
                ::core::fmt::LowerHex::fmt(&self, #formatter)
            },
            FormattingTrait::UpperHex => quote_spanned! { span =>
                ::core::fmt::UpperHex::fmt(&self, #formatter)
            },
            FormattingTrait::LowerExp => quote_spanned! { span =>
                ::core::fmt::LowerExp::fmt(&self, #formatter)
            },
            FormattingTrait::UpperExp => quote_spanned! { span =>
                ::core::fmt::UpperExp::fmt(&self, #formatter)
            },
        }
    }
}

#[derive(Clone)]
struct FormatValue {
    text: String,
    args: Vec<ArgRef>,
    span: Span,
}

impl FormatValue {
    fn parse(text: &str, span: Span) -> Result<Self> {
        let Format { text, args } = scan(text).map_err(|message| Error::new(span, message))?;
        Ok(Self { text, args, span })
    }

    fn of(literal: &LitStr) -> Result<Self> { Self::parse(&literal.value(), literal.span()) }

    fn empty(span: Span) -> Self {
        Self {
            text: String::new(),
            args: Vec::new(),
            span,
        }
    }
}

#[derive(Clone)]
enum Technique {
    FromTrait(FormattingTrait),
    FromMethod(Path),
    WithFormat(FormatValue, Option<FormatValue>),
    DocComments(FormatValue),
    Inner,
    Lowercase(FormatValue),
    Uppercase(FormatValue),
}

impl Technique {
    pub fn from_attrs<'a>(
        attrs: impl IntoIterator<Item = &'a Attribute> + Clone,
    ) -> Result<Option<Self>> {
        let Some(attr) = rules().single(attrs.clone())? else {
            return Ok(None);
        };
        let Some(first) = attr.unnamed().first() else {
            return Ok(None);
        };

        let mut res = if let Some(format) = first.as_str() {
            let alt = attr.str("alt").map(FormatValue::of).transpose()?;
            Technique::WithFormat(FormatValue::of(format)?, alt)
        } else if let Some(path) = first.as_path().filter(|_| !attr.has("alt")) {
            match path.get_ident().map(Ident::to_string).as_deref() {
                Some("doc_comments" | "docs") => {
                    Technique::DocComments(FormatValue::empty(first.span()))
                }
                Some("inner") => Technique::Inner,
                Some("lowercase") => Technique::Lowercase(FormatValue::empty(first.span())),
                Some("uppercase") => Technique::Uppercase(FormatValue::empty(first.span())),
                _ => FormattingTrait::from_path(path)
                    .map_or(Technique::FromMethod(path.clone()), Technique::FromTrait),
            }
        } else {
            return Err(attr
                .error_at("alt", "alternative formatting can be given only with a format string"));
        };

        res.apply_docs(attrs)?;
        Ok(Some(res))
    }

    pub fn to_fmt(&self, alt: bool) -> TokenStream2 {
        match self {
            Technique::FromTrait(fmt) => fmt.to_fmt(alt),
            Technique::FromMethod(_) => quote! { "{}" },
            Technique::WithFormat(fmt, fmt_alt) => {
                if alt && fmt_alt.is_some() {
                    let text = &fmt_alt.as_ref().unwrap().text;
                    quote! {#text}
                } else {
                    let text = &fmt.text;
                    quote! {#text}
                }
            }
            Technique::DocComments(doc) => {
                let text = &doc.text;
                quote! { #text }
            }
            Technique::Inner => {
                if alt {
                    quote! { "{_0:#}" }
                } else {
                    quote! { "{_0}" }
                }
            }
            Technique::Lowercase(fields_fmt) | Technique::Uppercase(fields_fmt) => {
                let text = &fields_fmt.text;
                quote! { #text }
            }
        }
    }

    pub fn into_token_stream2(
        self,
        fields: &Fields,
        span: Span,
        alt: bool,
    ) -> Result<TokenStream2> {
        match self {
            Technique::FromTrait(fmt) => Ok(fmt.into_token_stream2(span)),
            Technique::FromMethod(path) => {
                let formatter = formatter_ident();
                Ok(quote_spanned! { span =>
                    ::core::fmt::Display::fmt(&#path(self), #formatter)
                })
            }
            Technique::WithFormat(fmt, fmt_alt) => {
                let format = if alt {
                    fmt_alt.as_ref().unwrap_or(&fmt)
                } else {
                    &fmt
                };
                Self::impl_format(fields, format, span)
            }
            Technique::DocComments(format) |
            Technique::Lowercase(format) |
            Technique::Uppercase(format) => Self::impl_format(fields, &format, span),
            Technique::Inner => {
                let format = if alt { "{_0:#}" } else { "{_0}" };
                let format = FormatValue {
                    text: format.to_owned(),
                    args: vec![ArgRef::Name("_0".to_owned())],
                    span,
                };
                Self::impl_format(fields, &format, span)
            }
        }
    }

    fn format_value(&self, alt: bool) -> Option<&FormatValue> {
        match self {
            Technique::WithFormat(format, format_alt) => Some(if alt {
                format_alt.as_ref().unwrap_or(format)
            } else {
                format
            }),
            Technique::DocComments(format) |
            Technique::Lowercase(format) |
            Technique::Uppercase(format) => Some(format),
            _ => None,
        }
    }

    fn format_args(&self, alt: bool) -> Vec<ArgRef> {
        match self {
            Technique::Inner => vec![ArgRef::Name("_0".to_owned())],
            _ => self
                .format_value(alt)
                .map_or_else(Vec::new, |format| format.args.clone()),
        }
    }

    fn impl_format(fields: &Fields, format: &FormatValue, span: Span) -> Result<TokenStream2> {
        let formatter = formatter_ident();
        let text = &format.text;
        match fields {
            Fields::Named(fields) => {
                let args = named_format_args(fields, &format.args, span)?;
                Ok(quote_spanned! { span =>
                    ::core::write!(#formatter, #text, #( #args, )*)
                })
            }
            Fields::Unnamed(fields) => {
                let args = tuple_format_args(fields, &format.args, span);
                Ok(quote_spanned! { span =>
                    ::core::write!(#formatter, #text, #( #args, )*)
                })
            }
            Fields::Unit => Ok(quote_spanned! { span => ::core::write!(#formatter, #text) }),
        }
    }

    fn apply_docs<'a>(&mut self, attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<()> {
        if let Technique::DocComments(format) = self {
            let mut text = String::new();
            let mut span = format.span;
            for attr in attrs.into_iter().filter(|attr| attr.path().is_ident("doc")) {
                let parsed = Attr::parse(attr)?;
                let Form::Value(Value::Lit(Lit::Str(literal))) = parsed.form else {
                    continue;
                };
                span = literal.span();
                let fragment = literal.value().trim().replace("\\n", "\n");
                if fragment.is_empty() || fragment == "\n" {
                    text.push('\n');
                } else {
                    text.push_str(&fragment);
                    text.push(' ');
                };
            }
            format.text = text.trim().replace(" \n", "\n");
            let scanned = FormatValue::parse(&format.text, span)?;
            *format = scanned;
        }
        Ok(())
    }

    fn apply_case(&mut self, type_str: &str, fields: &Fields) -> Result<()> {
        let (type_str_cased, format) = match self {
            Technique::Lowercase(format) => (type_str.to_lowercase(), format),
            Technique::Uppercase(format) => (type_str.to_uppercase(), format),
            _ => return Ok(()),
        };
        let text = match fields {
            Fields::Named(f) => {
                let idents = f
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                let inner = idents
                    .iter()
                    .map(|ident| format!("{}: {{{0}}}", ident.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{{{ {} }}}}", type_str_cased, inner)
            }
            Fields::Unnamed(f) => {
                let inner = (0..f.unnamed.len())
                    .map(|i| format!("{{_{}}}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", type_str_cased, inner)
            }
            Fields::Unit => type_str_cased,
        };
        *format = FormatValue::parse(&text, format.span)?;
        Ok(())
    }
}

fn formatter_ident() -> Ident { Ident::new("f", Span::mixed_site()) }

fn named_format_args(
    fields: &syn::FieldsNamed,
    args: &[ArgRef],
    span: Span,
) -> Result<Vec<TokenStream2>> {
    let mut formatted = Vec::new();
    for arg in args {
        let (key, field) = match arg {
            ArgRef::Index(index) => {
                let Some(field) = fields.named.iter().nth(*index) else {
                    continue;
                };
                (Ident::new(&format!("_{index}"), span), field)
            }
            ArgRef::Name(name) => {
                let field = fields
                    .named
                    .iter()
                    .find(|field| {
                        field
                            .ident
                            .as_ref()
                            .is_some_and(|ident| ident == name.as_str())
                    })
                    .or_else(|| {
                        (name == "_0" && fields.named.len() == 1).then(|| &fields.named[0])
                    });
                let Some(field) = field else {
                    continue;
                };
                let key = field
                    .ident
                    .as_ref()
                    .filter(|ident| ident.to_string() == *name)
                    .cloned()
                    .unwrap_or_else(|| Ident::new(name, span));
                (key, field)
            }
        };
        let Some(field_ident) = field.ident.as_ref() else {
            continue;
        };
        formatted.push(field_format_arg(
            field,
            &key,
            quote_spanned! { span => self.#field_ident },
        )?);
    }
    Ok(formatted)
}

fn tuple_format_args(
    fields: &syn::FieldsUnnamed,
    args: &[ArgRef],
    span: Span,
) -> Vec<TokenStream2> {
    args.iter()
        .filter_map(|arg| {
            let index = match arg {
                ArgRef::Index(index) => *index,
                ArgRef::Name(name) => name.strip_prefix('_')?.parse().ok()?,
            };
            if index >= fields.unnamed.len() {
                return None;
            }
            let key = Ident::new(&format!("_{index}"), span);
            let field = Index::from(index);
            Some(quote_spanned! { span => #key = self.#field })
        })
        .collect()
}

fn named_field_index(fields: &syn::FieldsNamed, arg: &ArgRef) -> Option<usize> {
    match arg {
        ArgRef::Index(index) => (*index < fields.named.len()).then_some(*index),
        ArgRef::Name(name) => fields
            .named
            .iter()
            .position(|field| {
                field
                    .ident
                    .as_ref()
                    .is_some_and(|ident| ident == name.as_str())
            })
            .or_else(|| (name == "_0" && fields.named.len() == 1).then_some(0)),
    }
}

fn tuple_field_index(fields: &syn::FieldsUnnamed, arg: &ArgRef) -> Option<usize> {
    let index = match arg {
        ArgRef::Index(index) => *index,
        ArgRef::Name(name) => name.strip_prefix('_')?.parse().ok()?,
    };
    (index < fields.unnamed.len()).then_some(index)
}

fn named_enum_format_args(
    fields: &syn::FieldsNamed,
    args: &[ArgRef],
    bindings: &[(Ident, Ident)],
    span: Span,
) -> Result<Vec<TokenStream2>> {
    let mut formatted = Vec::new();
    for arg in args {
        let Some(index) = named_field_index(fields, arg) else {
            continue;
        };
        let field = &fields.named[index];
        let Some(field_ident) = field.ident.as_ref() else {
            continue;
        };
        let Some((_, binding)) = bindings.iter().find(|(name, _)| name == field_ident) else {
            continue;
        };
        let key = match arg {
            ArgRef::Index(index) => Ident::new(&format!("_{index}"), span),
            ArgRef::Name(name) if field_ident == name.as_str() => field_ident.clone(),
            ArgRef::Name(name) => Ident::new(name, span),
        };
        formatted.push(field_format_arg(field, &key, quote!(#binding))?);
    }
    Ok(formatted)
}

fn tuple_enum_bindings(
    fields: &syn::FieldsUnnamed,
    args: &[ArgRef],
    span: Span,
) -> (Vec<TokenStream2>, Vec<(usize, Ident)>) {
    let mut patterns = Vec::with_capacity(fields.unnamed.len());
    let mut bindings = Vec::new();
    for index in 0..fields.unnamed.len() {
        if args
            .iter()
            .any(|arg| tuple_field_index(fields, arg) == Some(index))
        {
            let binding = Ident::new(&format!("__amplify_field_{index}"), span);
            patterns.push(quote!(#binding));
            bindings.push((index, binding));
        } else {
            patterns.push(quote!(_));
        }
    }
    (patterns, bindings)
}

fn tuple_enum_format_args(
    fields: &syn::FieldsUnnamed,
    args: &[ArgRef],
    bindings: &[(usize, Ident)],
    span: Span,
) -> Vec<TokenStream2> {
    args.iter()
        .filter_map(|arg| {
            let index = tuple_field_index(fields, arg)?;
            let (_, binding) = bindings
                .iter()
                .find(|(field_index, _)| *field_index == index)?;
            let key = Ident::new(&format!("_{index}"), span);
            Some(quote!(#key = #binding))
        })
        .collect()
}

fn field_format_arg(field: &syn::Field, key: &Ident, value: TokenStream2) -> Result<TokenStream2> {
    let attr = field_rules().single(&field.attrs)?;
    Ok(match attr.as_ref().and_then(|attr| attr.str("separator")) {
        Some(separator) => quote! { #key = #value.join(#separator) },
        None => quote! { #key = #value },
    })
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        Data::Struct(ref data) => inner_struct(&input, data),
        Data::Enum(ref data) => inner_enum(&input, data),
        Data::Union(ref data) => inner_union(&input, data),
    }
}

fn inner_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let technique = Technique::from_attrs(&input.attrs)?.ok_or_else(|| {
        Error::new(
            input.span(),
            format!("Deriving `Display`: required attribute `{}` is missing.\n{}", NAME, EXAMPLE),
        )
    })?;
    if let (Fields::Named(fields), Technique::Inner) = (&data.fields, &technique) &&
        fields.named.len() != 1
    {
        return Err(Error::new(fields.span(), INNER_ERR));
    }

    let tokens_fmt = technique.to_fmt(false);
    let tokens_alt = technique.to_fmt(true);
    let format = technique
        .clone()
        .into_token_stream2(&data.fields, input.span(), false)?;
    let format_alt = technique.into_token_stream2(&data.fields, input.span(), true)?;
    let formatter = formatter_ident();
    let display = if tokens_fmt.to_string() == tokens_alt.to_string() {
        format
    } else {
        quote! {
            if #formatter.alternate() {
                #format_alt
            } else {
                #format
            }
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ident_name #ty_generics #where_clause {
            fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #display
            }
        }
    })
}

fn inner_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let mut display = TokenStream2::new();
    let formatter = formatter_ident();

    let global = Technique::from_attrs(&input.attrs)?;
    let mut use_global = !matches!(
        global.as_ref(),
        Some(Technique::Inner | Technique::Lowercase(_) | Technique::Uppercase(_))
    );

    for v in &data.variants {
        let type_name = &v.ident;
        let type_str = format!("{}", type_name);

        let mut local = Technique::from_attrs(&v.attrs)?;
        let mut parent = global.clone();
        let current = local.as_mut().or(parent.as_mut());
        let mut current = current.map(|technique| technique.clone());

        if local.is_some() {
            use_global = false;
        }

        if let Some(technique) = current.as_mut() {
            match technique {
                Technique::DocComments(_) => technique.apply_docs(&v.attrs)?,
                Technique::Lowercase(_) | Technique::Uppercase(_) => {
                    technique.apply_case(&type_str, &v.fields)?;
                }
                _ => {}
            }
        }

        if matches!(
            current,
            Some(Technique::DocComments(_) | Technique::Lowercase(_) | Technique::Uppercase(_))
        ) {
            use_global = false;
        }

        if matches!(current.as_ref(), Some(Technique::FromTrait(_) | Technique::FromMethod(_))) {
            let stream = current
                .clone()
                .unwrap()
                .into_token_stream2(&v.fields, v.span(), false)?;
            let arm = match &v.fields {
                Fields::Named(_) => {
                    quote_spanned! { v.span() => Self::#type_name { .. } => #stream, }
                }
                Fields::Unnamed(_) => {
                    quote_spanned! { v.span() => Self::#type_name ( .. ) => #stream, }
                }
                Fields::Unit => quote_spanned! { v.span() => Self::#type_name => #stream, },
            };
            display.extend(arm);
            continue;
        }

        let tokens_fmt = current.as_ref().map(|technique| technique.to_fmt(false));
        let tokens_alt = current.as_ref().map(|technique| technique.to_fmt(true));
        match (&v.fields, current.as_ref(), &tokens_fmt, &tokens_alt) {
            (Fields::Named(_), None, _, _) => {
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name { .. } => #formatter.write_str(concat!(#type_str, " { .. }")),
                });
            }
            (Fields::Unnamed(_), None, _, _) => {
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name(..) => #formatter.write_str(concat!(#type_str, "(..)")),
                });
            }
            (Fields::Unit, None, _, _) => {
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name => #formatter.write_str(#type_str),
                });
            }
            (Fields::Named(fields), Some(technique), Some(tokens_fmt), Some(tokens_alt)) => {
                if matches!(technique, Technique::Inner) && fields.named.len() != 1 {
                    return Err(Error::new(fields.span(), INNER_ERR));
                }
                let args_fmt = technique.format_args(false);
                let args_alt = technique.format_args(true);
                let mut all_args = args_fmt.clone();
                all_args.extend(args_alt.clone());
                let mut pattern_fields = Vec::new();
                let mut bindings = Vec::new();
                for (index, field) in fields.named.iter().enumerate() {
                    let Some(field_ident) = field.ident.as_ref() else {
                        continue;
                    };
                    if all_args
                        .iter()
                        .any(|arg| named_field_index(fields, arg) == Some(index))
                    {
                        let binding =
                            Ident::new(&format!("__amplify_field_{index}"), Span::mixed_site());
                        pattern_fields.push(quote!(#field_ident: #binding));
                        bindings.push((field_ident.clone(), binding));
                    }
                }
                let format_args = named_enum_format_args(fields, &args_fmt, &bindings, v.span())?;
                let format_args_alt =
                    named_enum_format_args(fields, &args_alt, &bindings, v.span())?;
                let write_fmt = quote_spanned! { v.span() =>
                    ::core::write!(#formatter, #tokens_fmt, #( #format_args, )*)
                };
                let write_alt = quote_spanned! { v.span() =>
                    ::core::write!(#formatter, #tokens_alt, #( #format_args_alt, )*)
                };
                let body = if tokens_fmt.to_string() == tokens_alt.to_string() {
                    write_fmt
                } else {
                    quote! {
                        if #formatter.alternate() { #write_alt } else { #write_fmt }
                    }
                };
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name { #( #pattern_fields, )* .. } => { #body },
                });
            }
            (Fields::Unnamed(fields), Some(technique), Some(tokens_fmt), Some(tokens_alt)) => {
                let args_fmt = technique.format_args(false);
                let args_alt = technique.format_args(true);
                let mut all_args = args_fmt.clone();
                all_args.extend(args_alt.clone());
                let (pattern_fields, bindings) =
                    tuple_enum_bindings(fields, &all_args, Span::mixed_site());
                let format_args = tuple_enum_format_args(fields, &args_fmt, &bindings, v.span());
                let format_args_alt =
                    tuple_enum_format_args(fields, &args_alt, &bindings, v.span());
                let write_fmt = quote_spanned! { v.span() =>
                    ::core::write!(#formatter, #tokens_fmt, #( #format_args, )*)
                };
                let write_alt = quote_spanned! { v.span() =>
                    ::core::write!(#formatter, #tokens_alt, #( #format_args_alt, )*)
                };
                let body = if tokens_fmt.to_string() == tokens_alt.to_string() {
                    write_fmt
                } else {
                    quote! {
                        if #formatter.alternate() { #write_alt } else { #write_fmt }
                    }
                };
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name ( #( #pattern_fields, )* ) => { #body },
                });
            }
            (Fields::Unit, Some(Technique::Inner), _, _) => {
                display.extend(quote_spanned! { v.span() =>
                    Self::#type_name => #formatter.write_str(#type_str),
                });
            }
            (Fields::Unit, Some(technique), _, _) => {
                let format = technique
                    .clone()
                    .into_token_stream2(&v.fields, v.span(), false)?;
                display.extend(quote_spanned! { v.span() => Self::#type_name => #format, });
            }
            _ => unreachable!(),
        }
    }

    let content = match (use_global, global) {
        (false, _) => quote! {
            match self {
                #display
            }
        },
        (true, Some(tenchique)) => {
            let format_str =
                tenchique
                    .clone()
                    .into_token_stream2(&Fields::Unit, input.span(), false)?;
            let format_alt = tenchique.into_token_stream2(&Fields::Unit, input.span(), true)?;
            if format_str.to_string() != format_alt.to_string() {
                quote! {
                    if #formatter.alternate() {
                        #format_alt
                    } else {
                        #format_str
                    }
                }
            } else {
                quote! { #format_str }
            }
        }
        _ => unreachable!(),
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ident_name #ty_generics #where_clause {
            #![allow(clippy::if_same_then_else)]
            fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #content
            }
        }
    })
}

fn inner_union(input: &DeriveInput, _data: &DataUnion) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;
    let formatter = formatter_ident();

    let global = Technique::from_attrs(&input.attrs)?.ok_or_else(|| {
        Error::new(
            input.span(),
            format!("Deriving `Display` for a union requires a type-level `{NAME}` attribute"),
        )
    })?;

    let format_str = global
        .clone()
        .into_token_stream2(&Fields::Unit, input.span(), false)?;
    let format_alt = global.into_token_stream2(&Fields::Unit, input.span(), true)?;
    let content = if format_str.to_string() != format_alt.to_string() {
        quote! {
            if #formatter.alternate() {
                #format_alt
            } else {
                #format_str
            }
        }
    } else {
        format_str
    };
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ident_name #ty_generics #where_clause {
            #![allow(clippy::if_same_then_else)]
            fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #content
            }
        }
    })
}
