//! How a storage's entries and a stack's components cross the NBT boundary.
//!
//! The port keeps two things as plain data, and both are the runtime's own text
//! form: a storage is a list of `"<amount>|<id>|<component key>"` lines
//! ([`crate::runtime::Storage::encode`]), and a stack's *resource identity* is
//! that component key. The key has to be equal exactly when two stacks would
//! stack together, so it is the item's component patch written as the game's
//! network NBT and hex-encoded: `Compound` is a `BTreeMap`, so the bytes are
//! canonical for a given patch, and no `|` (the save's separator) can appear in
//! them.
//!
//! This module is pure: no host, no game types beyond `Compound`/`Tag`, so
//! `cargo test` exercises it on its own.

use mod_api::{Compound, Tag};

/// The resource identity of a stack's component patch: empty for no patch,
/// otherwise the patch's network NBT bytes in lowercase hex.
pub fn component_key(patch: Option<&Compound>) -> String {
    let Some(patch) = patch.filter(|patch| !patch.is_empty()) else {
        return String::new();
    };
    let tag = Tag::from(patch);
    // A patch past the network quota cannot be given an identity; treating it as
    // plain is what a stack with no components would be, which is at least
    // consistent between the save, the lookup and the extract.
    match <Vec<u8>>::try_from(&tag) {
        Ok(bytes) => hex(&bytes),
        Err(_) => String::new(),
    }
}

/// The component patch a [`component_key`] stands for, or `None` when the key
/// is empty or is not a compound's network NBT.
pub fn patch(key: &str) -> Option<Compound> {
    let bytes = unhex(key)?;
    match Tag::try_from(bytes).ok()? {
        Tag::Compound(patch) => Some(patch),
        _ => None,
    }
}

/// The saved lines of a storage: a list of strings, in order, skipping any
/// entry that is not a string (a damaged save).
pub fn saved_lines(tag: Option<&Tag>) -> Vec<String> {
    let Some(Tag::List(items)) = tag else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            Tag::String(text) => Some(text.to_string_lossy()),
            _ => None,
        })
        .collect()
}

/// The list tag a storage's lines are saved as.
pub fn lines_tag(lines: &[String]) -> Tag {
    Tag::List(
        lines
            .iter()
            .map(|line| Tag::String(line.as_str().into()))
            .collect(),
    )
}

/// `bytes` in lowercase hex.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing into a String cannot fail.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The bytes `text` encodes, or `None` when it is not an even run of hex digits.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() || !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            u8::try_from(high * 16 + low).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patch_named(name: &str) -> Compound {
        let tag = Tag::compound([
            ("minecraft:custom_name", Tag::String(name.into())),
            ("minecraft:damage", Tag::Int(3)),
        ]);
        match tag {
            Tag::Compound(compound) => compound,
            other => panic!("expected a compound, got {other:?}"),
        }
    }

    #[test]
    fn a_missing_patch_has_an_empty_key() {
        assert_eq!(component_key(None), "");
        assert_eq!(component_key(Some(&Compound::new())), "");
    }

    #[test]
    fn a_patch_round_trips_through_its_key() {
        let saved = patch_named("kept");
        let key = component_key(Some(&saved));
        assert!(!key.is_empty(), "a patch has a key");
        assert_eq!(
            patch(&key),
            Some(saved),
            "the key is the patch it came from"
        );
    }

    #[test]
    fn the_key_is_canonical_for_the_same_patch() {
        let one = patch_named("kept");
        let mut two = Compound::new();
        two.insert("minecraft:damage".into(), Tag::Int(3));
        two.insert("minecraft:custom_name".into(), Tag::String("kept".into()));
        assert_eq!(one, two, "a compound has no insertion order");
        assert_eq!(component_key(Some(&one)), component_key(Some(&two)));
        assert_ne!(
            component_key(Some(&one)),
            component_key(Some(&patch_named("other"))),
            "two named resources are two keys"
        );
    }

    #[test]
    fn a_key_that_is_not_a_compound_is_no_patch() {
        assert!(
            component_key(Some(&patch_named("kept")))
                .len()
                .is_multiple_of(2)
        );
        assert_eq!(patch(""), None);
        assert_eq!(patch("not hex at all"), None);
        assert_eq!(patch("0f0"), None, "an odd run is not bytes");
        assert_eq!(patch("0f0f"), None, "truncated bytes are not a tag");
    }

    #[test]
    fn saved_lines_round_trip_and_skip_what_is_not_a_string() {
        let lines = vec!["3|minecraft:diamond|{custom_name:\"kept\"}".to_owned()];
        let tag = lines_tag(&lines);
        assert!(matches!(&tag, Tag::List(items) if items.len() == 1));
        assert_eq!(saved_lines(Some(&tag)), lines);
        let damaged = Tag::List(vec![
            Tag::String("5|minecraft:stone|".into()),
            Tag::Int(7),
            Tag::String("1|minecraft:dirt|".into()),
        ]);
        assert_eq!(
            saved_lines(Some(&damaged)),
            vec![
                "5|minecraft:stone|".to_owned(),
                "1|minecraft:dirt|".to_owned()
            ]
        );
    }

    #[test]
    fn nothing_saved_is_no_lines() {
        assert!(saved_lines(None).is_empty());
        assert!(saved_lines(Some(&Tag::Int(1))).is_empty());
        assert!(matches!(lines_tag(&[]), Tag::List(items) if items.is_empty()));
    }
}
