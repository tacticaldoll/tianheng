//! Module anchors: the one spelling a boundary names a module by, and whether that module exists.
//!
//! A module-anchored boundary's anchor is its violation `target`, so it is an identity role. Two
//! spellings of one module would be two identities, and a baseline entry recorded under one would
//! not suppress the same finding declared under the other. So an anchor has exactly one accepted
//! spelling — `crate`, or `crate::` followed by `::`-separated identifiers — and every other
//! spelling is refused rather than rewritten. The one equivalence folded is the one rustc defines:
//! `r#x` and `x` are the same identifier, so the returned form carries no raw prefix.
//!
//! An allowed-location list names modules the same way, and is held to the same spelling.

use syn::ext::IdentExt;
use syn::parse::Parser;

use crate::errors::{
    non_canonical_module_anchor_error, unknown_location_error, unknown_module_error,
};
use crate::file_scope::CompilationUnit;
use crate::module_resolve::resolve_module_branches;
use crate::resolve::canonical_path_str;

/// Whether `segment` is exactly one identifier token, written with nothing around it.
///
/// The identifier set is the lexer's, not a list kept here: `parse_any` accepts every identifier
/// and keyword spelling, raw forms included, and comparing the token with the written segment
/// refuses a segment the lexer read past — surrounding whitespace, a comment, a second token. The
/// comparison keeps the raw prefix, so `r#x` equals only `"r#x"`. Keywords are
/// not refused at this layer: a path segment naming `self` or `type` names no module, and the
/// existence check that follows every accepted anchor is what refuses it.
fn is_identifier(segment: &str) -> bool {
    syn::Ident::parse_any
        .parse_str(segment)
        .is_ok_and(|ident| ident == segment)
}

/// The canonical spelling of a written module anchor, or the constitution error naming it.
///
/// `crate` and `crate::a::b` are accepted; `crate::r#a` is accepted and returned as `crate::a`.
/// Every other spelling — a trailing or leading `::`, the empty string, a path not starting at
/// `crate`, a `self::` or `super::` path, a segment carrying whitespace — is refused.
pub(crate) fn canonical_module_anchor(
    written: &str,
    crate_package: &str,
) -> Result<String, String> {
    if is_canonical_spelling(written) {
        Ok(canonical_path_str(written))
    } else {
        Err(non_canonical_module_anchor_error(
            written,
            crate_package,
            suggested_spelling(written).as_deref(),
        ))
    }
}

/// `crate`, or `crate::` followed by `::`-separated identifiers.
fn is_canonical_spelling(written: &str) -> bool {
    let mut segments = written.split("::");
    segments.next() == Some("crate") && segments.all(is_identifier)
}

/// The canonical spelling a refused anchor most plausibly meant, when there is one: its non-empty,
/// trimmed segments rooted at `crate`. A `self::` or `super::` path is relative to a module the
/// declaration does not have, and a candidate the rule itself would refuse is no suggestion, so
/// either answers `None`.
fn suggested_spelling(written: &str) -> Option<String> {
    let segments: Vec<&str> = written
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    let head = segments
        .first()
        .map(|head| head.strip_prefix("r#").unwrap_or(head));
    let candidate = match head {
        None => return Some("crate".to_string()),
        Some("self" | "super") => return None,
        Some("crate") => std::iter::once("crate")
            .chain(segments[1..].iter().copied())
            .collect::<Vec<_>>()
            .join("::"),
        Some(_) => format!("crate::{}", segments.join("::")),
    };
    is_canonical_spelling(&candidate).then_some(candidate)
}

/// Every entry of an allowed-location list in its canonical spelling, refusing the first that has
/// none. Order is preserved.
pub(crate) fn canonical_module_locations(
    written: &[String],
    crate_package: &str,
) -> Result<Vec<String>, String> {
    written
        .iter()
        .map(|location| canonical_module_anchor(location, crate_package))
        .collect()
}

/// Whether the canonical `module` exists in one compilation unit's module graph.
///
/// The descent is the one every module-anchored boundary resolves its anchor with, so a module
/// this reports present is one those boundaries would govern. Only its absence answer is read as
/// absence; any other failure — an unreadable or unparseable file on the way — is returned, because
/// a descent that could not finish has not shown the module is missing.
pub(crate) fn module_exists_in_unit(
    src_dir: &std::path::Path,
    root_file: &std::path::Path,
    module: &str,
    crate_package: &str,
) -> Result<bool, String> {
    match resolve_module_branches(src_dir, root_file, module, crate_package) {
        Ok(_) => Ok(true),
        Err(reason) if reason == unknown_module_error(module, crate_package) => Ok(false),
        Err(reason) => Err(reason),
    }
}

/// Refuse the first canonical location that no compilation unit of the package declares.
///
/// A package's roots are separate module graphs, so a location present in the library and absent
/// from a binary beside it is a real location; it is refused only when it is present nowhere —
/// the policy `over_each_unit` applies to a module anchor.
pub(crate) fn require_locations_exist(
    units: &[CompilationUnit],
    locations: &[String],
    crate_package: &str,
) -> Result<(), String> {
    for location in locations {
        let mut found = false;
        for (root_file, src_dir, _unit) in units {
            if module_exists_in_unit(src_dir, root_file, location, crate_package)? {
                found = true;
                break;
            }
        }
        if !found {
            return Err(unknown_location_error(location, crate_package));
        }
    }
    Ok(())
}
