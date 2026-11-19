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

use amplify_syn::{AttrReq, ListReq, Value, ValueClass};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Error, Field, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Result, Type,
};

const NAME: &str = "from";

/// The content of `#[from]`: the type to convert from, which may be left out
/// where a single field gives it.
fn rules() -> AttrReq { AttrReq::new(NAME).unnamed(ListReq::maybe_one(ValueClass::Type)) }

#[derive(Clone, PartialEq, Eq, Debug)]
enum InstructionEntity {
    Default,
    DefaultEnumFields {
        variant: Ident,
        fields: Vec<Ident>,
    },
    Unit {
        variant: Option<Ident>,
    },
    EmptyNamed {
        variant: Ident,
    },
    EmptyUnnamed {
        variant: Ident,
    },
    Named {
        variant: Option<Ident>,
        field: Ident,
        other: Vec<Ident>,
        union: bool,
    },
    Unnamed {
        variant: Option<Ident>,
        index: usize,
        total: usize,
    },
}

impl InstructionEntity {
    pub fn with_fields(fields: &Fields, variant: Option<Ident>, union: bool) -> Result<Self> {
        let res = match (fields.len(), variant, fields.clone(), fields.iter().next().cloned()) {
            (0, Some(v), Fields::Named(_), ..) => InstructionEntity::EmptyNamed { variant: v },
            (0, Some(v), Fields::Unnamed(_), ..) => InstructionEntity::EmptyUnnamed { variant: v },
            (0, Some(v), Fields::Unit, ..) => InstructionEntity::Unit { variant: Some(v) },
            (_, variant, Fields::Unit, ..) => InstructionEntity::Unit { variant },
            (1, variant, Fields::Named(f), Some(Field { ident: Some(i), .. })) => {
                InstructionEntity::Named {
                    variant,
                    field: i.clone(),
                    other: f
                        .named
                        .iter()
                        .filter_map(|f| f.ident.clone())
                        .filter(|ident| ident != &i)
                        .collect(),
                    union,
                }
            }
            (1, _, Fields::Named(_), ..) => {
                unreachable!("If we have named field, it will match previous option")
            }
            (_, Some(variant), Fields::Named(f), ..) => InstructionEntity::DefaultEnumFields {
                variant,
                fields: f.named.iter().filter_map(|f| f.ident.clone()).collect(),
            },
            (_, None, Fields::Named(f), ..) if union => InstructionEntity::Named {
                variant: None,
                field: f
                    .named
                    .first()
                    .and_then(|field| field.ident.clone())
                    .expect("a union has at least one field"),
                other: Vec::new(),
                union: true,
            },
            (len, variant, Fields::Unnamed(_), ..) => InstructionEntity::Unnamed {
                variant,
                index: 0,
                total: len,
            },
            (_, None, ..) => InstructionEntity::Default,
        };
        Ok(res)
    }

    pub fn with_field(
        index: usize,
        total: usize,
        field: &Field,
        fields: &Fields,
        variant: Option<Ident>,
        union: bool,
    ) -> Self {
        if let Some(ref ident) = field.ident {
            InstructionEntity::Named {
                variant,
                field: ident.clone(),
                other: fields
                    .iter()
                    .filter_map(|f| f.ident.clone())
                    .filter(|i| ident != i)
                    .collect(),
                union,
            }
        } else {
            InstructionEntity::Unnamed {
                variant,
                index,
                total,
            }
        }
    }

    pub fn into_token_stream2(self) -> TokenStream2 {
        match self {
            InstructionEntity::Default => quote! {
                <Self as ::core::default::Default>::default()
            },
            InstructionEntity::Unit { variant } => {
                let var = variant.map_or(quote! {}, |v| quote! {:: #v});
                quote! { Self #var }
            }
            InstructionEntity::EmptyNamed { variant } => quote! { Self::#variant {} },
            InstructionEntity::EmptyUnnamed { variant } => quote! { Self::#variant() },
            InstructionEntity::Named {
                variant: None,
                field,
                union: true,
                ..
            } => quote! {
                Self { #field: ::core::convert::Into::into(v) }
            },
            InstructionEntity::Named {
                variant: None,
                field,
                ..
            } => {
                quote! {
                    Self { #field: ::core::convert::Into::into(v), ..::core::default::Default::default() }
                }
            }
            InstructionEntity::Named {
                variant: Some(var),
                field,
                other,
                ..
            } => {
                quote! {
                    Self :: #var { #field: ::core::convert::Into::into(v), #( #other: ::core::default::Default::default(), )* }
                }
            }
            InstructionEntity::Unnamed {
                variant,
                index,
                total,
            } => {
                let var = variant.map_or(quote! {}, |v| quote! {:: #v});
                let prefix = (0..index).fold(TokenStream2::new(), |mut stream, _| {
                    stream.extend(quote! {::core::default::Default::default(),});
                    stream
                });
                let suffix = ((index + 1)..total).fold(TokenStream2::new(), |mut stream, _| {
                    stream.extend(quote! {::core::default::Default::default(),});
                    stream
                });
                quote! {
                    Self #var ( #prefix ::core::convert::Into::into(v), #suffix )
                }
            }
            InstructionEntity::DefaultEnumFields { variant, fields } => {
                quote! {
                    Self::#variant { #( #fields: ::core::default::Default::default(), )* }
                }
            }
        }
    }
}

#[derive(Clone)]
struct InstructionEntry(pub Type, pub InstructionEntity, pub Span);

impl PartialEq for InstructionEntry {
    // Ugly way, but with current `syn` version no other way is possible
    fn eq(&self, other: &Self) -> bool {
        let l = &self.0;
        let r = &other.0;
        let a = quote! { #l };
        let b = quote! { #r };
        format!("{}", a) == format!("{}", b)
    }
}

impl InstructionEntry {
    pub fn with_type(ty: &Type, entity: &InstructionEntity, span: Span) -> Self {
        Self(ty.clone(), entity.clone(), span)
    }

    pub fn parse(
        fields: &Fields,
        attrs: &[Attribute],
        entity: InstructionEntity,
    ) -> Result<Vec<InstructionEntry>> {
        let mut list = Vec::<InstructionEntry>::new();
        for attr in rules().each(attrs)? {
            let ty = match (attr.unnamed().first().and_then(Value::as_type), fields.iter().next()) {
                (Some(ty), _) => ty,
                (None, Some(field)) if fields.len() == 1 => &field.ty,
                _ => {
                    return Err(attr.error(
                        "empty attribute is allowed only for entities with a single field; for \
                         multi-field entities specify the attribute right ahead of the target \
                         field",
                    ));
                }
            };
            list.push(InstructionEntry::with_type(ty, &entity, attr.span()));
        }
        Ok(list)
    }
}

#[derive(Default)]
struct InstructionTable(Vec<InstructionEntry>);

impl InstructionTable {
    pub fn new() -> Self { ::core::default::Default::default() }

    pub fn parse(
        &mut self,
        fields: &Fields,
        attrs: &[Attribute],
        variant: Option<Ident>,
        union: bool,
    ) -> Result<&Self> {
        let entity = InstructionEntity::with_fields(fields, variant.clone(), union)?;
        self.extend(InstructionEntry::parse(fields, attrs, entity.clone())?)?;
        for (index, field) in fields.iter().enumerate() {
            let mut punctuated = Punctuated::new();
            punctuated.push_value(field.clone());
            self.extend(InstructionEntry::parse(
                &field.ident.as_ref().map_or(
                    Fields::Unnamed(FieldsUnnamed {
                        paren_token: ::core::default::Default::default(),
                        unnamed: punctuated.clone(),
                    }),
                    |_| {
                        Fields::Named(FieldsNamed {
                            brace_token: ::core::default::Default::default(),
                            named: punctuated,
                        })
                    },
                ),
                &field.attrs,
                InstructionEntity::with_field(
                    index,
                    fields.len(),
                    field,
                    fields,
                    variant.clone(),
                    union,
                ),
            )?)?;
        }
        if variant.is_none() && fields.len() == 1 && self.0.is_empty() {
            let field = fields
                .into_iter()
                .next()
                .expect("we know we have at least one item");
            self.push(InstructionEntry::with_type(&field.ty, &entity, field.span()));
        }
        Ok(self)
    }

    fn push(&mut self, item: InstructionEntry) { self.0.push(item) }

    fn extend<T>(&mut self, list: T) -> Result<()>
    where T: IntoIterator<Item = InstructionEntry> {
        for entry in list {
            if self.0.contains(&entry) {
                let ty = &entry.0;
                return Err(Error::new(
                    entry.2,
                    format!("Attribute `#[{}]`: repeated use of type `{}`", NAME, quote! {#ty}),
                ));
            }
            self.0.push(entry);
        }
        Ok(())
    }

    pub fn into_token_stream2(self, input: &DeriveInput) -> TokenStream2 {
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident_name = &input.ident;

        self.0.into_iter().fold(TokenStream2::new(), |mut stream, InstructionEntry(from, entity, _)| {
            let convert = entity.into_token_stream2();
            stream.extend(quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::From<#from> for #ident_name #ty_generics #where_clause {
                    fn from(v: #from) -> Self {
                        #convert
                    }
                }
            });
            stream
        })
    }
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        Data::Struct(ref data) => inner_struct(&input, data),
        Data::Enum(ref data) => inner_enum(&input, data),
        Data::Union(ref data) => inner_union(&input, data),
    }
}

fn inner_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let mut instructions = InstructionTable::new();
    instructions.parse(&data.fields, &input.attrs, None, false)?;
    Ok(instructions.into_token_stream2(input))
}

fn inner_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    // Do not let top-level `from` on enums
    if let Some(attr) = input.attrs.iter().find(|attr| attr.path().is_ident(NAME)) {
        return Err(Error::new_spanned(
            attr.path(),
            "top-level attribute is not allowed, use it for specific fields or variants",
        ));
    }

    let mut instructions = InstructionTable::new();
    for v in &data.variants {
        instructions.parse(&v.fields, &v.attrs, Some(v.ident.clone()), false)?;
    }
    Ok(instructions.into_token_stream2(input))
}

fn inner_union(input: &DeriveInput, data: &DataUnion) -> Result<TokenStream2> {
    let mut instructions = InstructionTable::new();
    instructions.parse(&Fields::Named(data.fields.clone()), &input.attrs, None, true)?;
    Ok(instructions.into_token_stream2(input))
}
