use json_patch::{
    AddOperation, CopyOperation, MoveOperation, Patch, PatchOperation, RemoveOperation,
    ReplaceOperation, TestOperation,
};
use jsonptr::Pointer;
use serde_json::{from_str, from_value, json, Value};

#[test]
fn parse_from_value() {
    let json = json!([{"op": "add", "path": "/a/b", "value": 1}, {"op": "remove", "path": "/c"}]);
    let patch: Patch = from_value(json).unwrap();

    assert_eq!(
        patch,
        Patch(vec![
            PatchOperation::Add(AddOperation {
                path: Pointer::new(["a", "b"]),
                value: Value::from(1),
            }),
            PatchOperation::Remove(RemoveOperation {
                path: Pointer::new(["c"]),
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
                path: Pointer::new(["a", "b"]),
                value: Value::from(1),
            }),
            PatchOperation::Remove(RemoveOperation {
                path: Pointer::new(["c"]),
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

#[test]
fn display_add_operation() {
    let op = PatchOperation::Add(AddOperation {
        path: Pointer::new(["a", "b", "c"]),
        value: json!(["hello", "bye"]),
    });
    assert_eq!(
        op.to_string(),
        r#"{"op":"add","path":"/a/b/c","value":["hello","bye"]}"#
    );
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "add",
  "path": "/a/b/c",
  "value": [
    "hello",
    "bye"
  ]
}"#
    );
}

#[test]
fn display_remove_operation() {
    let op = PatchOperation::Remove(RemoveOperation {
        path: Pointer::new(["a", "b", "c"]),
    });
    assert_eq!(op.to_string(), r#"{"op":"remove","path":"/a/b/c"}"#);
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "remove",
  "path": "/a/b/c"
}"#
    );
}

#[test]
fn display_replace_operation() {
    let op = PatchOperation::Replace(ReplaceOperation {
        path: Pointer::new(["a", "b", "c"]),
        value: json!(42),
    });
    assert_eq!(
        op.to_string(),
        r#"{"op":"replace","path":"/a/b/c","value":42}"#
    );
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "replace",
  "path": "/a/b/c",
  "value": 42
}"#
    );
}

#[test]
fn display_move_operation() {
    let op = PatchOperation::Move(MoveOperation {
        from: Pointer::new(["a", "b", "c"]),
        path: Pointer::new(["a", "b", "d"]),
    });
    assert_eq!(
        op.to_string(),
        r#"{"op":"move","from":"/a/b/c","path":"/a/b/d"}"#
    );
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "move",
  "from": "/a/b/c",
  "path": "/a/b/d"
}"#
    );
}

#[test]
fn display_copy_operation() {
    let op = PatchOperation::Copy(CopyOperation {
        from: Pointer::new(["a", "b", "d"]),
        path: Pointer::new(["a", "b", "e"]),
    });
    assert_eq!(
        op.to_string(),
        r#"{"op":"copy","from":"/a/b/d","path":"/a/b/e"}"#
    );
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "copy",
  "from": "/a/b/d",
  "path": "/a/b/e"
}"#
    );
}

#[test]
fn display_test_operation() {
    let op = PatchOperation::Test(TestOperation {
        path: Pointer::new(["a", "b", "c"]),
        value: json!("hello"),
    });
    assert_eq!(
        op.to_string(),
        r#"{"op":"test","path":"/a/b/c","value":"hello"}"#
    );
    assert_eq!(
        format!("{:#}", op),
        r#"{
  "op": "test",
  "path": "/a/b/c",
  "value": "hello"
}"#
    );
}

#[test]
fn display_patch() {
    let patch = Patch(vec![
        PatchOperation::Add(AddOperation {
            path: Pointer::new(["a", "b", "c"]),
            value: json!(["hello", "bye"]),
        }),
        PatchOperation::Remove(RemoveOperation {
            path: Pointer::new(["a", "b", "c"]),
        }),
    ]);

    assert_eq!(
        patch.to_string(),
        r#"[{"op":"add","path":"/a/b/c","value":["hello","bye"]},{"op":"remove","path":"/a/b/c"}]"#
    );
    assert_eq!(
        format!("{:#}", patch),
        r#"[
  {
    "op": "add",
    "path": "/a/b/c",
    "value": [
      "hello",
      "bye"
    ]
  },
  {
    "op": "remove",
    "path": "/a/b/c"
  }
]"#
    );
}
