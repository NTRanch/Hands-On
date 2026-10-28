//! Own the Puppet (Study S1's twin): a puppet-file parser
//! whose lesson is ownership, taught by a text format small
//! enough to hold in one hand.
//!
//! A puppet file names fields (`key = value` lines, `#`
//! comments). The parser's contract is the lesson: it works on
//! a buffer it *borrows* (`&str` in, values borrowed out —
//! zero copies), and repairs are expressed as owned edits
//! applied to an owned document. A borrow error in the
//! student's head is staged exactly as the file staging it:
//! try to hold a borrowed value across a mutation and the
//! compiler stops the sitting, safely, which is what the
//! lesson's exercise reproduces in the TUI.

/// A parse failure, typed — the parser refuses, it never
/// guesses.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    /// A non-comment line without `=`.
    MissingEquals {
        /// 1-based line number.
        line: usize,
    },
    /// A line whose key is empty.
    EmptyKey {
        /// 1-based line number.
        line: usize,
    },
    /// The same key defined twice — a puppet has one value
    /// per field; ambiguity is refused, not resolved.
    DuplicateKey {
        /// The repeated key.
        key: String,
        /// 1-based line number of the second definition.
        line: usize,
    },
}

/// A parsed puppet file: ordered `(key, value)` pairs whose
/// values *borrow* the source text. The document owns nothing
/// but the index — that is the point of the lesson.
#[derive(Clone, Debug, PartialEq)]
pub struct PuppetFile<'a> {
    fields: Vec<(&'a str, &'a str)>,
}

impl<'a> PuppetFile<'a> {
    /// Parse `text`. Borrows throughout: no field is copied.
    ///
    /// # Errors
    /// [`ParseError`] on the first malformed line — missing
    /// `=`, empty key, or a duplicate key.
    pub fn parse(text: &'a str) -> Result<Self, ParseError> {
        let mut fields: Vec<(&'a str, &'a str)> = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line_no = i + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some(eq) = line.find('=') else {
                return Err(ParseError::MissingEquals { line: line_no });
            };
            let key = line[..eq].trim();
            let value = line[eq + 1..].trim();
            if key.is_empty() {
                return Err(ParseError::EmptyKey { line: line_no });
            }
            if fields.iter().any(|(k, _)| *k == key) {
                return Err(ParseError::DuplicateKey {
                    key: key.to_string(),
                    line: line_no,
                });
            }
            fields.push((key, value));
        }
        Ok(Self { fields })
    }

    /// Look up a field by key — a borrow of a borrow.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// All keys, in file order.
    #[must_use]
    pub fn keys(&self) -> Vec<&'a str> {
        self.fields.iter().map(|(k, _)| *k).collect()
    }

    /// Field count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Whether the file has no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Apply owned edits to owned text: replace the value of
/// `key` (or append the field if absent) and return the new
/// document. This is the move half of the lesson — after
/// mutation, old borrows are gone, and the document must be
/// re-parsed. The function takes and returns ownership so
/// that rule is in its signature.
#[must_use]
pub fn apply_edit(text: String, key: &str, value: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    for raw in text.lines() {
        let line = raw.trim();
        if !line.starts_with('#')
            && !line.is_empty()
            && let Some(eq) = line.find('=')
            && line[..eq].trim() == key
        {
            out.push(format!("{key} = {value}"));
            done = true;
        } else {
            out.push(raw.to_string());
        }
    }
    if !done {
        out.push(format!("{key} = {value}"));
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Opal rig\nname = Opal\ngain = 1.0\nrole = host\n";

    #[test]
    fn parse_borrows_the_fields() {
        let doc = PuppetFile::parse(DOC).unwrap();
        assert_eq!(doc.get("name"), Some("Opal"));
        assert_eq!(doc.get("gain"), Some("1.0"));
        assert_eq!(doc.get("missing"), None);
        assert_eq!(doc.keys(), vec!["name", "gain", "role"]);
        assert_eq!(doc.len(), 3);
        assert!(!doc.is_empty());
    }

    #[test]
    fn edits_replace_in_place_and_reparse() {
        let edited = apply_edit(DOC.to_string(), "gain", "2.6");
        let doc = PuppetFile::parse(&edited).unwrap();
        assert_eq!(doc.get("gain"), Some("2.6"));
        assert_eq!(doc.get("name"), Some("Opal"));
        let appended = apply_edit(DOC.to_string(), "lane", "House Cel");
        assert_eq!(
            PuppetFile::parse(&appended).unwrap().get("lane"),
            Some("House Cel")
        );
    }

    #[test]
    fn comments_and_blanks_are_not_fields() {
        let doc = PuppetFile::parse("# only a comment\n\n").unwrap();
        assert!(doc.is_empty());
    }

    // ---- adversarial ----

    #[test]
    fn malformed_lines_refuse_typed() {
        assert_eq!(
            PuppetFile::parse("name Opal\n"),
            Err(ParseError::MissingEquals { line: 1 })
        );
        assert_eq!(
            PuppetFile::parse("ok = 1\n = nope\n"),
            Err(ParseError::EmptyKey { line: 2 })
        );
    }

    #[test]
    fn duplicate_keys_refuse_rather_than_resolve() {
        assert_eq!(
            PuppetFile::parse("a = 1\na = 2\n"),
            Err(ParseError::DuplicateKey {
                key: "a".to_string(),
                line: 2
            })
        );
    }

    #[test]
    fn values_may_contain_equals() {
        let doc = PuppetFile::parse("expr = a == b\n").unwrap();
        assert_eq!(doc.get("expr"), Some("a == b"));
    }
}
