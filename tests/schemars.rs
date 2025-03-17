#[cfg(feature = "schemars")]
#[test]
fn schema() {
    use json_patch::*;

    let schema = schemars::schema_for!(PatchOperation);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    expectorate::assert_contents("tests/schemars.json", &json);
}
