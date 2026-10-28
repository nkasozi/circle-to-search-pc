use super::*;
use crate::presentation::design::icons::ALL_ICONS;

#[test]
fn test_icon_bodies_are_indexed_by_icon_discriminant() {
    for (position, (icon, _)) in ICON_BODIES.iter().enumerate() {
        assert_eq!(*icon as usize, position);
    }
}

#[test]
fn test_icon_bodies_cover_every_icon() {
    assert_eq!(ICON_BODIES.len(), ALL_ICONS.len());
    for candidate in ALL_ICONS {
        assert_eq!(ICON_BODIES[candidate as usize].0, candidate);
    }
}
