use crate::types::Id;

#[test]
fn test_id_unique() {
    let a = Id::new();
    let b = Id::new();
    assert_ne!(a, b);
}

#[test]
fn test_id_nil() {
    let id = Id::nil();
    assert!(id.is_nil());
    assert!(!Id::new().is_nil());
}

#[test]
fn test_id_roundtrip() {
    let id = Id::new();
    let s = id.to_string();
    let parsed: Id = s.parse().unwrap();
    assert_eq!(id, parsed);
}

#[test]
fn test_id_serde() {
    let id = Id::new();
    let json = serde_json::to_string(&id).unwrap();
    let parsed: Id = serde_json::from_str(&json).unwrap();
    assert_eq!(id, parsed);
}
