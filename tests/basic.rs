use json_patch::{AddOperation, Patch, PatchOperation, RemoveOperation};
use serde_json::{from_str, from_value, json, Value};

#[test]
fn parse_from_value() {
    let json = json!([{"op": "add", "path": "/a/b", "value": 1}, {"op": "remove", "path": "/c"}]);
    let patch: Patch = from_value(json).unwrap();

    assert_eq!(
        patch,
        Patch(vec![
            PatchOperation::Add(AddOperation {
                path: String::from("/a/b"),
                value: Value::from(1),
            }),
            PatchOperation::Remove(RemoveOperation {
                path: String::from("/c"),
            }),
        ])
    );

    let _patch: Patch =
        from_str(r#"[{"op": "add", "path": "/a/b", "value": 1}, {"op": "remove", "path": "/c"}]"#)
            .unwrap();
}

#[test]
fn parse_from_string() {
    let patch: Patch =
        from_str(r#"[{"op": "add", "path": "/a/b", "value": 1}, {"op": "remove", "path": "/c"}]"#)
            .unwrap();

    assert_eq!(
        patch,
        Patch(vec![
            PatchOperation::Add(AddOperation {
                path: String::from("/a/b"),
                value: Value::from(1),
            }),
            PatchOperation::Remove(RemoveOperation {
                path: String::from("/c"),
            }),
        ])
    );
}

#[test]
fn serialize_patch() {
    let s = r#"[{"op":"add","path":"/a/b","value":1},{"op":"remove","path":"/c"}]"#;
    let patch: Patch = from_str(s).unwrap();

    let serialized = serde_json::to_string(&patch).unwrap();
    assert_eq!(serialized, s);
}
