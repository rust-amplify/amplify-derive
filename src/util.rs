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

use amplify_syn::{AttrReq, ListReq, Value, ValueClass};
use syn::{DeriveInput, Ident, Path, Result};

/// Resolves `#[amplify_crate(path)]`, defaulting to `amplify`.
pub(crate) fn amplify_crate(input: &DeriveInput) -> Result<Path> {
    Ok(explicit_amplify_crate(input)?
        .unwrap_or_else(|| Path::from(Ident::new("amplify", input.ident.span()))))
}

/// Reads `#[amplify_crate(path)]`, if the attribute is present.
pub(crate) fn explicit_amplify_crate(input: &DeriveInput) -> Result<Option<Path>> {
    let rules = AttrReq::new("amplify_crate").unnamed(ListReq::one(ValueClass::Path));
    let attr = rules.single(&input.attrs)?;
    Ok(attr.and_then(|attr| attr.unnamed().first().and_then(Value::as_path).cloned()))
}
