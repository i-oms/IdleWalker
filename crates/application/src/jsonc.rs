//! Small edits to JSON-with-comments settings files (VS Code's and Windows Terminal's)
//! that leave everything else as it was: comments, trailing commas and the person's own
//! layout. Only what is needed: reading one string at a key path, and setting one.

struct Member {
    key: String,
    key_start: usize,
    value_start: usize,
    value_end: usize,
}

struct Scanner<'a> {
    b: &'a [u8],
}

impl Scanner<'_> {
    /// Past whitespace and comments.
    fn skip_blank(&self, mut i: usize) -> usize {
        let b = self.b;
        loop {
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            } else if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
            } else {
                return i;
            }
        }
    }

    /// `i` at an opening quote; the index after the closing one.
    fn string_end(&self, i: usize) -> Option<usize> {
        let b = self.b;
        let mut j = i + 1;
        while j < b.len() {
            match b[j] {
                b'\\' => j += 2,
                b'"' => return Some(j + 1),
                _ => j += 1,
            }
        }
        None
    }

    /// The index after any value starting at `i`.
    fn value_end(&self, i: usize) -> Option<usize> {
        let b = self.b;
        match b.get(i)? {
            b'"' => self.string_end(i),
            b'{' | b'[' => {
                let mut depth = 0;
                let mut j = i;
                while j < b.len() {
                    match b[j] {
                        b'"' => j = self.string_end(j)?,
                        b'/' => {
                            let k = self.skip_blank(j);
                            j = if k == j { j + 1 } else { k };
                        }
                        b'{' | b'[' => {
                            depth += 1;
                            j += 1;
                        }
                        b'}' | b']' => {
                            depth -= 1;
                            j += 1;
                            if depth == 0 {
                                return Some(j);
                            }
                        }
                        _ => j += 1,
                    }
                }
                None
            }
            _ => {
                let mut j = i;
                while j < b.len()
                    && !matches!(b[j], b',' | b'}' | b']' | b'/')
                    && !b[j].is_ascii_whitespace()
                {
                    j += 1;
                }
                (j > i).then_some(j)
            }
        }
    }

    fn unquote(&self, i: usize, end: usize) -> String {
        let mut out = String::new();
        let mut chars = std::str::from_utf8(&self.b[i + 1..end - 1])
            .unwrap_or("")
            .chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(o) => out.push(o),
                    None => {}
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    /// The members of the object whose `{` is at `open`; `None` if it is not one.
    fn members(&self, open: usize) -> Option<Vec<Member>> {
        let b = self.b;
        if b.get(open) != Some(&b'{') {
            return None;
        }
        let mut out = Vec::new();
        let mut i = self.skip_blank(open + 1);
        loop {
            match b.get(i)? {
                b'}' => return Some(out),
                b'"' => {}
                _ => return None,
            }
            let key_end = self.string_end(i)?;
            let colon = self.skip_blank(key_end);
            if b.get(colon) != Some(&b':') {
                return None;
            }
            let value_start = self.skip_blank(colon + 1);
            let value_end = self.value_end(value_start)?;
            out.push(Member {
                key: self.unquote(i, key_end),
                key_start: i,
                value_start,
                value_end,
            });
            let next = self.skip_blank(value_end);
            match b.get(next)? {
                b',' => i = self.skip_blank(next + 1),
                b'}' => return Some(out),
                _ => return None,
            }
        }
    }

    /// The whitespace that starts the line `at` is on, if the line holds nothing else before it.
    fn indent_before(&self, at: usize) -> Option<String> {
        let lead = &self.b[self.line_start(at)..at];
        lead.iter()
            .all(|c| *c == b' ' || *c == b'\t')
            .then(|| String::from_utf8_lossy(lead).into_owned())
    }

    /// The whitespace that starts the line `at` is on, whatever follows it.
    fn line_indent(&self, at: usize) -> String {
        let start = self.line_start(at);
        let n = self.b[start..at]
            .iter()
            .take_while(|c| **c == b' ' || **c == b'\t')
            .count();
        String::from_utf8_lossy(&self.b[start..start + n]).into_owned()
    }

    fn line_start(&self, at: usize) -> usize {
        self.b[..at]
            .iter()
            .rposition(|&c| c == b'\n')
            .map_or(0, |p| p + 1)
    }
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `{ "a": { "b": "value" } }` for a path, or just the quoted value for an empty one.
fn nested(path: &[&str], value: &str) -> String {
    match path.split_first() {
        None => quote(value),
        Some((k, rest)) => format!("{{ {}: {} }}", quote(k), nested(rest, value)),
    }
}

/// The string at a key path, when it is there and is a string.
pub fn get_string(text: &str, path: &[&str]) -> Option<String> {
    let sc = Scanner { b: text.as_bytes() };
    let mut open = sc.skip_blank(0);
    let mut path = path;
    loop {
        let (key, rest) = path.split_first()?;
        let members = sc.members(open)?;
        let m = members.into_iter().find(|m| m.key == *key)?;
        if rest.is_empty() {
            return (text.as_bytes()[m.value_start] == b'"')
                .then(|| sc.unquote(m.value_start, m.value_end));
        }
        open = m.value_start;
        path = rest;
    }
}

/// The text with the string at a key path set, the objects on the way made as needed.
/// An empty file becomes an object; a file that is not an object is left alone.
pub fn set_string(text: &str, path: &[&str], value: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err("no key".into());
    }
    let sc = Scanner { b: text.as_bytes() };
    let open = sc.skip_blank(0);
    if open >= text.len() {
        let member = format!("{}: {}", quote(path[0]), nested(&path[1..], value));
        return Ok(format!("{text}{{\n    {member}\n}}\n"));
    }
    set_in(text, open, path, value)
}

fn set_in(text: &str, open: usize, path: &[&str], value: &str) -> Result<String, String> {
    let sc = Scanner { b: text.as_bytes() };
    let members = sc
        .members(open)
        .ok_or_else(|| "the file is not a JSON object".to_string())?;
    let (key, rest) = (path[0], &path[1..]);
    let splice = |start: usize, end: usize, with: &str| {
        format!("{}{}{}", &text[..start], with, &text[end..])
    };
    if let Some(m) = members.iter().find(|m| m.key == key) {
        if rest.is_empty() {
            return Ok(splice(m.value_start, m.value_end, &quote(value)));
        }
        if text.as_bytes()[m.value_start] == b'{' {
            return set_in(text, m.value_start, rest, value);
        }
        return Ok(splice(m.value_start, m.value_end, &nested(rest, value)));
    }
    let member = format!("{}: {}", quote(key), nested(rest, value));
    match members.first() {
        Some(first) => {
            // In front of the others, on a line of its own with their indentation.
            let indent = sc
                .indent_before(first.key_start)
                .unwrap_or_else(|| "    ".into());
            Ok(splice(open + 1, open + 1, &format!("\n{indent}{member},")))
        }
        None => {
            let close = sc.skip_blank(open + 1);
            let outer = sc.line_indent(open);
            let inner = format!("{outer}    ");
            Ok(splice(
                open + 1,
                close,
                &format!("\n{inner}{member}\n{outer}"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VSCODE: &str = r#"{
    // The editor
    "editor.fontSize": 14, /* px */
    "terminal.integrated.fontFamily": "Consolas",
    "files.autoSave": "afterDelay",
}
"#;

    #[test]
    fn a_string_is_read_past_comments_and_trailing_commas() {
        assert_eq!(
            get_string(VSCODE, &["terminal.integrated.fontFamily"]).as_deref(),
            Some("Consolas")
        );
        assert_eq!(get_string(VSCODE, &["editor.fontSize"]), None);
        assert_eq!(get_string(VSCODE, &["missing"]), None);
        assert_eq!(get_string("not json", &["a"]), None);
    }

    #[test]
    fn an_existing_string_is_replaced_in_place() {
        let out = set_string(
            VSCODE,
            &["terminal.integrated.fontFamily"],
            "MesloLGM Nerd Font",
        )
        .unwrap();
        assert_eq!(
            out,
            VSCODE.replace("\"Consolas\"", "\"MesloLGM Nerd Font\"")
        );
    }

    #[test]
    fn a_missing_key_goes_in_front_with_the_file_s_indentation() {
        let text = "{\n  \"a\": 1\n}\n";
        let out = set_string(text, &["k"], "v").unwrap();
        assert_eq!(out, "{\n  \"k\": \"v\",\n  \"a\": 1\n}\n");
        assert_eq!(get_string(&out, &["k"]).as_deref(), Some("v"));
    }

    #[test]
    fn an_empty_file_or_object_becomes_one_member() {
        assert_eq!(
            set_string("", &["k"], "v").unwrap(),
            "{\n    \"k\": \"v\"\n}\n"
        );
        assert_eq!(
            set_string("  \n", &["k"], "v").unwrap(),
            "  \n{\n    \"k\": \"v\"\n}\n"
        );
        assert_eq!(
            set_string("{}", &["k"], "v").unwrap(),
            "{\n    \"k\": \"v\"\n}"
        );
        assert_eq!(
            set_string("{ /* nothing */ }", &["k"], "v").unwrap(),
            "{\n    \"k\": \"v\"\n}"
        );
    }

    #[test]
    fn nested_objects_are_entered_or_made() {
        let wt = r#"{
    "$schema": "https://aka.ms/terminal-profiles-schema",
    "profiles":
    {
        "defaults": {},
        "list": [ { "name": "x", "font": { "face": "y" } } ]
    }
}"#;
        let path = ["profiles", "defaults", "font", "face"];
        let out = set_string(wt, &path, "MesloLGM Nerd Font").unwrap();
        assert_eq!(
            get_string(&out, &path).as_deref(),
            Some("MesloLGM Nerd Font")
        );
        assert!(out.contains("\"defaults\": {\n            \"font\": { \"face\": \"MesloLGM Nerd Font\" }\n        }"));
        // The list's own font is not touched.
        assert!(out.contains("\"font\": { \"face\": \"y\" }"));

        let again = set_string(&out, &path, "Other").unwrap();
        assert_eq!(get_string(&again, &path).as_deref(), Some("Other"));

        let none = set_string("{ \"a\": 1 }", &path, "F").unwrap();
        assert_eq!(get_string(&none, &path).as_deref(), Some("F"));
        assert!(none.ends_with("\"a\": 1 }"));

        // A member on the way that is not an object is replaced by one.
        let wrong = set_string("{ \"profiles\": \"x\" }", &path, "F").unwrap();
        assert_eq!(get_string(&wrong, &path).as_deref(), Some("F"));
    }

    #[test]
    fn a_file_that_is_not_an_object_is_refused() {
        assert!(set_string("[1, 2]", &["k"], "v").is_err());
        assert!(set_string("{ \"a\": }", &["k"], "v").is_err());
    }

    #[test]
    fn quoting_is_escaped() {
        let out = set_string("{}", &["k"], "a \"b\" \\ c").unwrap();
        assert_eq!(get_string(&out, &["k"]).as_deref(), Some("a \"b\" \\ c"));
    }
}
