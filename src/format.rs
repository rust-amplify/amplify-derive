use std::fmt;

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
}

/// Scans a format string and rewrites positional references to named fields.
pub(crate) fn scan(source: &str) -> Result<Format, String> {
    let mut text = String::with_capacity(source.len());
    let mut args = Vec::new();
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

    Ok(Format { text, args })
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

fn push_unique(args: &mut Vec<ArgRef>, arg: ArgRef) {
    if !args.contains(&arg) {
        args.push(arg);
    }
}

impl fmt::Display for ArgRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(index) => write!(f, "{index}"),
            Self::Name(name) => f.write_str(name),
        }
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
