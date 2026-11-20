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

/// A field referenced by a format string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArgRef {
    /// A tuple field or positional format argument.
    Index(usize),
    /// A named field or named format argument.
    Name(String),
}

/// A scanned format string and its referenced fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Format {
    /// Format text with positional references rewritten to `_N` names.
    pub(crate) text: String,
    /// Referenced fields in first-use order, without duplicates.
    pub(crate) args: Vec<ArgRef>,
    /// The trait of `core::fmt` each placeholder formats its field with,
    /// without duplicates.
    pub(crate) traits: Vec<(ArgRef, &'static str)>,
}

/// Scans a format string and rewrites positional references to named fields.
pub(crate) fn scan(source: &str) -> Result<Format, String> {
    let mut text = String::with_capacity(source.len());
    let mut args = Vec::new();
    let mut traits = Vec::new();
    let mut implicit = 0;
    let mut cursor = 0;

    while cursor < source.len() {
        let rest = &source[cursor..];
        if rest.starts_with("{{") {
            text.push_str("{{");
            cursor += 2;
            continue;
        }
        if rest.starts_with("}}") {
            text.push_str("}}");
            cursor += 2;
            continue;
        }

        let ch = rest.chars().next().expect("cursor is before end");
        match ch {
            '{' => {
                let start = cursor + 1;
                let close = source[start..]
                    .find('}')
                    .map(|offset| start + offset)
                    .ok_or_else(|| "unclosed `{` in format string".to_owned())?;
                let body = &source[start..close];
                if body.contains('{') {
                    return Err("unmatched `{` in format string".to_owned());
                }

                let (head, spec) = body
                    .split_once(':')
                    .map_or((body, None), |(head, spec)| (head, Some(spec)));
                let leading = head.len() - head.trim_start().len();
                let trimmed = head.trim();
                let trailing = head.len() - leading - trimmed.len();
                let (rewritten_head, reference) = if trimmed.is_empty() {
                    let index = implicit;
                    implicit += 1;
                    (format!("_{index}"), ArgRef::Index(index))
                } else if trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
                    let index = trimmed
                        .parse::<usize>()
                        .map_err(|_| format!("invalid positional index `{trimmed}`"))?;
                    (format!("_{index}"), ArgRef::Index(index))
                } else {
                    (trimmed.to_owned(), ArgRef::Name(trimmed.to_owned()))
                };
                push_unique(&mut traits, (reference.clone(), spec_trait(spec)));
                push_unique(&mut args, reference);

                text.push('{');
                text.push_str(&head[..leading]);
                text.push_str(&rewritten_head);
                text.push_str(&head[head.len() - trailing..]);
                if let Some(spec) = spec {
                    text.push(':');
                    text.push_str(&rewrite_spec(spec, &mut args)?);
                }
                text.push('}');
                cursor = close + 1;
            }
            '}' => return Err("unmatched `}` in format string".to_owned()),
            _ => {
                text.push(ch);
                cursor += ch.len_utf8();
            }
        }
    }

    Ok(Format { text, args, traits })
}

/// Names the formatting trait a placeholder with the format `spec` uses.
///
/// The type is the last part of a format spec, and nothing else in a spec
/// ends with one of its letters: a fill character is followed by an
/// alignment, and a named width or precision by `$`.
fn spec_trait(spec: Option<&str>) -> &'static str {
    match spec.and_then(|spec| spec.trim_end().chars().last()) {
        Some('?') => "Debug",
        Some('x') => "LowerHex",
        Some('X') => "UpperHex",
        Some('o') => "Octal",
        Some('b') => "Binary",
        Some('e') => "LowerExp",
        Some('E') => "UpperExp",
        Some('p') => "Pointer",
        _ => "Display",
    }
}

fn rewrite_spec(spec: &str, args: &mut Vec<ArgRef>) -> Result<String, String> {
    let mut output = String::with_capacity(spec.len());
    let bytes = spec.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let start = cursor;
        if bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_' {
            cursor += 1;
            while cursor < bytes.len() &&
                (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
            if cursor < bytes.len() && bytes[cursor] == b'$' {
                let token = &spec[start..cursor];
                if token.bytes().all(|byte| byte.is_ascii_digit()) {
                    let index = token
                        .parse::<usize>()
                        .map_err(|_| format!("invalid positional index `{token}`"))?;
                    push_unique(args, ArgRef::Index(index));
                    output.push('_');
                    output.push_str(token);
                } else {
                    push_unique(args, ArgRef::Name(token.to_owned()));
                    output.push_str(token);
                }
                output.push('$');
                cursor += 1;
            } else {
                output.push_str(&spec[start..cursor]);
            }
        } else {
            let ch = spec[cursor..].chars().next().expect("cursor is before end");
            output.push(ch);
            cursor += ch.len_utf8();
        }
    }
    Ok(output)
}

fn push_unique<T: PartialEq>(list: &mut Vec<T>, item: T) {
    if !list.contains(&item) {
        list.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::{ArgRef, scan};

    #[test]
    fn rewrites_indices_and_tracks_values_and_format_parameters() {
        let cases = [
            ("{0}", "{_0}", vec![ArgRef::Index(0)]),
            ("{10}", "{_10}", vec![ArgRef::Index(10)]),
            ("{} {}", "{_0} {_1}", vec![ArgRef::Index(0), ArgRef::Index(1)]),
            ("{x}", "{x}", vec![ArgRef::Name("x".to_owned())]),
            ("{x:?}", "{x:?}", vec![ArgRef::Name("x".to_owned())]),
            ("{x:>8}", "{x:>8}", vec![ArgRef::Name("x".to_owned())]),
            ("{x:>w$}", "{x:>w$}", vec![
                ArgRef::Name("x".to_owned()),
                ArgRef::Name("w".to_owned()),
            ]),
            ("{0:.1$}", "{_0:._1$}", vec![ArgRef::Index(0), ArgRef::Index(1)]),
            ("{x }", "{x }", vec![ArgRef::Name("x".to_owned())]),
        ];

        for (source, expected_text, expected_args) in cases {
            let scanned = scan(source).expect("valid format string");
            assert_eq!(scanned.text, expected_text, "text for `{source}`");
            assert_eq!(scanned.args, expected_args, "arguments for `{source}`");
        }
    }

    #[test]
    fn names_the_formatting_trait_of_each_placeholder() {
        let cases = [
            ("{} {0} {x} {x }", "Display"),
            ("{:>8} {:<} {:x<} {:?<5} {:e>w$} {:+.3} {:.p$} {:#}", "Display"),
            ("{:?} {:#?} {:x?} {:X?} {:<?} {:? }", "Debug"),
            ("{:x} {:#x} {:08x} {:>w$x}", "LowerHex"),
            ("{:X} {:#010X}", "UpperHex"),
            ("{:o}", "Octal"),
            ("{:b} {:#b}", "Binary"),
            ("{:e} {:10.3e} {:.p$e}", "LowerExp"),
            ("{:E}", "UpperExp"),
            ("{:p}", "Pointer"),
        ];
        for (source, expected) in cases {
            let scanned = scan(source).expect("valid format string");
            for (arg, name) in &scanned.traits {
                assert_eq!(*name, expected, "trait of {arg:?} in `{source}`");
            }
        }

        let scanned = scan("{0} {0:?} {0} {x:>0$}").expect("valid format string");
        assert_eq!(scanned.traits, vec![
            (ArgRef::Index(0), "Display"),
            (ArgRef::Index(0), "Debug"),
            (ArgRef::Name("x".to_owned()), "Display"),
        ]);
    }

    #[test]
    fn escaped_braces_are_not_scanned_as_placeholders() {
        let scanned = scan("{{0}} {{{1}}}").expect("escaped braces parse");
        assert_eq!(scanned.text, "{{0}} {{{_1}}}");
        assert_eq!(scanned.args, vec![ArgRef::Index(1)]);
    }

    #[test]
    fn reports_unmatched_braces() {
        assert!(scan("{x").is_err());
        assert!(scan("x}").is_err());
    }
}
