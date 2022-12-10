use json_patch::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct TestCase {
    doc: Value,
    patch: Patch,
    error: String,
}

#[test]
fn errors() {
    let tests = std::fs::read_to_string("tests/errors.yaml").unwrap();
    let cases: Vec<TestCase> = serde_yaml::from_str(&tests).unwrap();
    for (idx, mut case) in cases.into_iter().enumerate() {
        match json_patch::patch(&mut case.doc, &case.patch).map_err(|err| err.to_string()) {
            Ok(_) if !case.error.is_empty() => {
                panic!("Expected test case {} to fail with an error!", idx);
            }
            Err(err) if err != case.error => {
                panic!("Expected test case {} to fail with an error:\n{}\n\nbut instead failed with an error:\n{}", idx, case.error, err);
            }
            _ => {}
        }
    }
}
