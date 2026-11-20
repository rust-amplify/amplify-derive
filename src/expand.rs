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

//! Snapshots of the code which the derive macros generate.
//!
//! Each `tests/expand/<name>.rs` holds types with `#[derive(...)]` on them,
//! and `tests/expand/<name>.expanded.rs` holds what the macros of this crate
//! generate for them. Run the tests with `EXPAND=overwrite` to accept a
//! change in the generated code, and review it as a diff of the snapshots.

use std::path::Path;
use std::{env, fs};

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, File, Item, Result, Token};

/// Runs the derive macros of this crate which are named on `item`.
fn expand(item: &Item) -> TokenStream2 {
    let input: DeriveInput = syn::parse2(item.to_token_stream()).expect("a type definition");
    let mut output = TokenStream2::new();
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
    {
        let names = Punctuated::<syn::Path, Token![,]>::parse_terminated;
        for name in attr
            .parse_args_with(names)
            .expect("a list of derive macros")
        {
            let derive: fn(DeriveInput) -> Result<TokenStream2> = match name.get_ident() {
                Some(name) if name == "Display" => crate::display::inner,
                Some(name) if name == "Error" => crate::error::inner,
                Some(name) if name == "From" => crate::from::inner,
                Some(name) if name == "AsAny" => crate::as_any::inner,
                Some(name) if name == "Getters" => crate::getters::derive,
                Some(name) if name == "Wrapper" => crate::wrapper::inner,
                Some(name) if name == "WrapperMut" => crate::wrapper::inner_mut,
                _ => continue,
            };
            let generated = derive(input.clone());
            output.extend(generated.unwrap_or_else(|error| error.to_compile_error()));
        }
    }
    output
}

#[test]
fn generated_code_matches_the_snapshots() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/expand");
    let overwrite = env::var_os("EXPAND").is_some_and(|value| value == "overwrite");
    let files = fs::read_dir(&dir).expect("the directory of the snapshots");
    let mut inputs = files
        .map(|file| file.expect("a file").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .filter(|path| !path.to_string_lossy().ends_with(".expanded.rs"))
        .collect::<Vec<_>>();
    inputs.sort();
    assert!(!inputs.is_empty(), "no inputs in {}", dir.display());

    for input in inputs {
        let source = fs::read_to_string(&input).expect("a readable input");
        let source = syn::parse_file(&source).expect("an input which parses");
        let generated = source.items.iter().map(expand).collect();
        let generated: File = syn::parse2(generated).expect("generated code which parses");
        let generated = prettyplease::unparse(&generated);

        let snapshot = input.with_extension("expanded.rs");
        if overwrite {
            fs::write(&snapshot, generated).expect("a writable snapshot");
            continue;
        }
        // The snapshot goes through the same printer, so its layout is free.
        let expected = fs::read_to_string(&snapshot).unwrap_or_default();
        let expected = syn::parse_file(&expected).expect("a snapshot which parses");
        let expected = prettyplease::unparse(&expected);
        if generated != expected {
            let line = |text: &str, number| text.lines().nth(number).unwrap_or_default().to_owned();
            let differs = |number: &usize| line(&generated, *number) != line(&expected, *number);
            let number = (0..generated.lines().count())
                .find(differs)
                .unwrap_or_default();
            let (generated, expected) = (line(&generated, number), line(&expected, number));
            panic!(
                "the code generated for {} differs from {} at line {}:\n  snapshot:  \
                 {expected}\n  generated: {generated}\nrun the tests with `EXPAND=overwrite` to \
                 accept the generated code",
                input.display(),
                snapshot.display(),
                number + 1,
            );
        }
    }
}
