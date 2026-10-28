use super::*;

#[test]
fn test_every_icon_produces_a_well_formed_svg_document() {
    for candidate in ALL_ICONS {
        let document = String::from_utf8(build_svg_document(candidate)).unwrap_or_default();
        assert!(document.starts_with("<svg"));
        assert!(document.ends_with("</svg>"));
        assert!(document.len() > SVG_DOCUMENT_PREFIX.len() + SVG_DOCUMENT_SUFFIX.len());
    }
}

#[test]
fn test_every_icon_parses_with_usvg() {
    for candidate in ALL_ICONS {
        let document = build_svg_document(candidate);
        let parse_result = usvg::Tree::from_data(&document, &usvg::Options::default());
        assert!(parse_result.is_ok(), "icon {:?} failed to parse", candidate);
    }
}

#[test]
fn test_icon_list_has_no_duplicates() {
    let unique_count = ALL_ICONS
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(unique_count, ALL_ICONS.len());
}
