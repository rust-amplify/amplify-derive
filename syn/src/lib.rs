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

//! Rules for the content of derive macro attributes, and the grammar the rules
//! are checked against.
//!
//! A derive macro states what its attribute may contain as an [`AttrReq`]:
//! the named arguments with the values they hold ([`ArgValueReq`]), the
//! unnamed arguments ([`ListReq`]) and the arguments which exclude each
//! other. Reading an attribute through its rules gives a
//! [`ParametrizedAttr`], or errors pointing at each argument which breaks
//! them. [`collect_all`] puts the errors of several attributes together.
//!
//! The grammar ([`Attr`], [`Form`], [`Arg`], [`NamedArg`], [`Value`]) covers
//! bare attributes, single-value attributes, and comma-separated lists
//! containing literals, types, and named values.

mod arg;
mod attr;
mod req;

pub use arg::{Arg, NamedArg, Value};
pub use attr::{Attr, Form, ParametrizedAttr};
pub use req::{ArgValueReq, AttrReq, ListReq, ValueClass, collect_all};
