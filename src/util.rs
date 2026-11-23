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
use quote::ToTokens;
use syn::{
    DeriveInput, GenericArgument, Generics, Path, PathArguments, Result, ReturnType, Type,
    TypeParamBound, WhereClause, parse_quote,
};

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
    pub fn require(&mut self, ty: &Type, bound: impl ToTokens) {
        if self.is_generic(ty) {
            let clause = self.clause.get_or_insert_with(|| parse_quote!(where));
            let needs_parens = matches!(
                ty,
                Type::FnPtr(fn_ptr) if fn_ptr.lifetimes.is_some()
            ) || matches!(ty, Type::TraitObject(_) | Type::ImplTrait(_));
            let predicate = if needs_parens {
                parse_quote!((#ty): #bound)
            } else {
                parse_quote!(#ty: #bound)
            };
            if !clause.predicates.iter().any(|known| *known == predicate) {
                clause.predicates.push(predicate);
            }
        }
    }

    /// Returns the where clause of the impl.
    pub fn into_where_clause(self) -> Option<WhereClause> { self.clause }

    fn is_generic(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(type_path) => {
                if let Some(qself) = &type_path.qself {
                    if self.is_generic(&qself.ty) {
                        return true;
                    }
                } else if type_path.path.leading_colon.is_none() &&
                    let Some(first) = type_path.path.segments.first()
                {
                    if first.ident == "Self" && self.generics.type_params().next().is_some() {
                        return true;
                    }
                    if self
                        .generics
                        .type_params()
                        .any(|param| param.ident == first.ident)
                    {
                        return true;
                    }
                }
                self.path_args_are_generic(&type_path.path)
            }
            Type::Reference(reference) => self.is_generic(&reference.elem),
            Type::Slice(slice) => self.is_generic(&slice.elem),
            Type::Array(array) => self.is_generic(&array.elem),
            Type::Ptr(ptr) => self.is_generic(&ptr.elem),
            Type::Tuple(tuple) => tuple.elems.iter().any(|elem| self.is_generic(elem)),
            Type::Paren(paren) => self.is_generic(&paren.elem),
            Type::Group(group) => self.is_generic(&group.elem),
            Type::FnPtr(fn_ptr) => {
                fn_ptr.inputs.iter().any(|arg| self.is_generic(&arg.ty)) ||
                    matches!(&fn_ptr.output, ReturnType::Type(_, ret) if self.is_generic(ret.as_ref()))
            }
            Type::TraitObject(obj) => obj.bounds.iter().any(|bound| self.bound_is_generic(bound)),
            Type::ImplTrait(imp) => imp.bounds.iter().any(|bound| self.bound_is_generic(bound)),
            _ => false,
        }
    }

    fn path_args_are_generic(&self, path: &Path) -> bool {
        path.segments
            .iter()
            .any(|segment| match &segment.arguments {
                PathArguments::None => false,
                PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| match arg {
                    GenericArgument::Type(ty) => self.is_generic(ty),
                    GenericArgument::AssocType(assoc) => self.is_generic(&assoc.ty),
                    GenericArgument::Constraint(constraint) => constraint
                        .bounds
                        .iter()
                        .any(|bound| self.bound_is_generic(bound)),
                    _ => false,
                }),
                PathArguments::Parenthesized(args) => {
                    args.inputs.iter().any(|arg| self.is_generic(&arg.ty)) ||
                        matches!(&args.output, ReturnType::Type(_, ret) if self.is_generic(ret.as_ref()))
                }
            })
    }

    fn bound_is_generic(&self, bound: &TypeParamBound) -> bool {
        match bound {
            TypeParamBound::Trait(trait_bound) => self.path_args_are_generic(&trait_bound.path),
            _ => false,
        }
    }
}

/// Resolves `#[amplify_crate(path)]`, defaulting to `::amplify`.
pub(crate) fn amplify_crate(input: &DeriveInput) -> Result<Path> {
    let rules = AttrReq::new("amplify_crate").unnamed(ListReq::one(ValueClass::Path));
    let attr = rules.single(&input.attrs)?;
    Ok(attr
        .and_then(|attr| attr.unnamed().first().and_then(Value::as_path).cloned())
        .unwrap_or_else(|| parse_quote!(::amplify)))
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{Generics, Type, parse_quote};

    use super::Bounds;

    #[test]
    fn detects_generic_types_without_false_positives_and_parenthesizes_hrtb_fn() {
        let generics: Generics = parse_quote!(<T>);
        let mut bounds = Bounds::new(&generics);

        let concrete_mod: Type = parse_quote!(my_mod::T);
        let concrete_assoc: Type = parse_quote!(<Concrete as Trait>::T);
        bounds.require(&concrete_mod, quote!(::core::fmt::Debug));
        bounds.require(&concrete_assoc, quote!(::core::fmt::Debug));
        assert!(bounds.clause.is_none());

        let param_assoc: Type = parse_quote!(<T as Trait>::Item);
        let hrtb_fn: Type = parse_quote!(for<'a> fn(&'a T));
        bounds.require(&param_assoc, quote!(::core::fmt::Display));
        bounds.require(&hrtb_fn, quote!(::core::fmt::Debug));

        let clause = bounds.into_where_clause().expect("predicates added");
        assert_eq!(clause.predicates.len(), 2);
        let rendered = quote!(#clause).to_string();
        assert!(rendered.contains("< T as Trait > :: Item : :: core :: fmt :: Display"));
        assert!(rendered.contains("(for < 'a > fn (& 'a T)) : :: core :: fmt :: Debug"));
    }
}
