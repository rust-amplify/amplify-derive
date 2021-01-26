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

use std::fmt::{Display, Formatter, self};
use syn::{Path, MetaList};
use proc_macro2::Ident;

/// Errors representing inconsistency in proc macro attribute structure
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    /// Attribute must be in a singular form (`#[attr]` or `#[attr = ...]`)
    SingularAttrRequired(Ident),

    /// Attribute must be in a parametrized form (`#[attr(...)]`)
    ParametrizedAttrRequired(Ident),

    /// Attribute argument must be a path identifier like `#[attr(std::io)]`
    /// or `#[attr = std::io]`
    ArgMustBePath,

    /// Attribute or attribute argument must has a name
    ArgNameRequired,

    /// Attribute or attribute argument name (in form of `#[attr(arg = ...)]`)
    /// must be an identifier (like `arg`) and not a path (like `std::io`)
    ArgNameMustBeIdent(Path),

    /// Attribute or attribute argument must has a value:
    /// `#[attr(arg = value)]`
    ArgValueRequired(Ident),

    /// Parametrized attribute argument must have a literal value (string,
    /// integer etc): `#[attr(arg = "value")]` or `#[arg = 4]`
    ArgValueMustBeLiteral,

    /// Parametrized attribute argument must be a valid type name:
    /// `#[attr(arg = u8)]` or `#[arg = String]`
    ArgValueMustBeType,

    /// Parametrized attribute (in form of `#[attr(...)]`) does not
    /// have a single value
    ParametrizedAttrHasNoValue(Ident),

    /// Lists nested within attribute arguments, like `#[attr(arg(...))]`
    /// are not supported
    NestedListsNotSupported(Ident, MetaList),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::SingularAttrRequired(name) => write!(
                f,
                "Attribute `{}` must be in a singular form (`#[attr]` or `#[attr = ...]`)",
                name
            ),
            Error::ParametrizedAttrRequired(name) => write!(
                f,
                "Attribute `{}` must be in a parametrized form (`#[attr(...)]`)",
                name
            ),
            Error::ArgMustBePath => f.write_str("attribute argument must be a path identifier"),
            Error::ArgNameRequired => f.write_str("attribute argument name is required"),
            Error::ArgNameMustBeIdent(path) => write!(
                f,
                "attribute arguments must be identifiers, not paths `{:?}`",
                path
            ),
            Error::ArgValueRequired(name) => write!(
                f,
                "attribute or attribute argument value is required for `{}`",
                name
            ),
            Error::ArgValueMustBeLiteral => f.write_str(
                "attribute value must be a literal (string, int etc)",
            ),
            Error::ArgValueMustBeType => {
                f.write_str("attribute value for must be a valid type name")
            }
            Error::ParametrizedAttrHasNoValue(name) => {
                write!(
                    f,
                    "attribute `{name}` must be in a `#[{name} = ...]` form",
                    name = name
                )
            }
            Error::NestedListsNotSupported(name, list) => write!(f, "attribute `{name}` must be in `{name} = ...` form and not in a form of nested list `{list:?}`", name = name, list = list),
        }
    }
}

impl std::error::Error for Error {}
