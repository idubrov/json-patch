use super::super::{patch, Patch};
use std::{io, fs};
use serde_json;
use serde_json::Value;
use std::fmt::Write;

#[derive(Debug, Deserialize)]
struct TestCase {
    comment: Option<String>,
    doc: Value,
    patch: Value,
    expected: Option<Value>,
    error: Option<String>,
    #[serde(default)]
    disabled: bool
}

fn run_case(doc: Value, patches: Value) -> Result<Value, String> {
    let patches: Patch = Patch(serde_json::from_value(patches).map_err(|e| e.to_string())?);
    let mut actual = doc.clone();

    // Patch and verify that in case of error document wasn't changed
    patch(&mut actual, &patches)
        .map_err(|e| {
            assert_eq!(doc, actual, "no changes should be made to the original document");
            e
        })
        .map_err(|e| e.to_string())?;
    Ok(actual)
}

pub fn run_specs(path: &str) {
    let file = fs::File::open(path).unwrap();
    let buf = io::BufReader::new(file);
    let cases: Vec<TestCase> = serde_json::from_reader(buf).unwrap();

    for (idx, tc) in cases.into_iter().enumerate() {
        print!("Running test case {}", idx);
        if let Some(comment) = tc.comment {
            print!(" ({})... ", comment);
        } else {
            print!("... ");
        }

        if tc.disabled {
            println!("disabled...");
            continue;
        }

        match run_case(tc.doc, tc.patch) {
            Ok(actual) => {
                if let Some(ref error) = tc.error {
                    println!("expected to fail with '{}'", error);
                    panic!("expected to fail, got document {:?}", actual);
                }
                println!();
                if let Some(expected) = tc.expected {
                    assert_eq!(expected, actual);
                }
            },
            Err(err) => {
                println!("failed with '{}'", err);
                tc.error.expect("patch expected to succeed");
            }
        }
    }
}

pub fn all_leafs(value: &Value) -> Vec<String> {
    let mut result = Vec::new();
    collect_leafs(value, &mut String::new(), &mut result);
    result
}

fn collect_leafs(value: &Value, prefix: &mut String, result: &mut Vec<String>) {
    match *value {
        Value::Array(ref arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let l = prefix.len();
                write!(prefix, "/{}", idx).unwrap();
                collect_leafs(val, prefix, result);
                prefix.truncate(l);
            }
        },
        Value::Object(ref map) => {
            for (key, val) in map.iter() {
                let l = prefix.len();
                write!(prefix, "/{}", key).unwrap();
                collect_leafs(val, prefix, result);
                prefix.truncate(l);
            }
        },
        _ => {
           result.push(prefix.clone());
        }
    }
}