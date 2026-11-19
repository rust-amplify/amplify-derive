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

//! Rules for the content of derive macro attributes, and the grammar the rules
//! are checked against.
//!
//! A derive macro states what its attribute may contain as an [`AttrReq`]:
//! the named arguments with the values they hold ([`ArgValueReq`]), the
//! unnamed arguments ([`ListReq`]) and the arguments which exclude each
//! other. Reading an attribute through its rules gives a
//! [`ParametrizedAttr`], or errors pointing at each argument which breaks
//! them.
//!
//! The grammar ([`Attr`], [`Form`], [`Arg`], [`NamedArg`], [`Value`]) covers
//! bare attributes, single-value attributes, and comma-separated lists
//! containing literals, types, and named values.

mod arg;
mod attr;
mod req;

pub use arg::{Arg, NamedArg, Value};
pub use attr::{Attr, Form, ParametrizedAttr};
pub use req::{ArgValueReq, AttrReq, ListReq, ValueClass};
