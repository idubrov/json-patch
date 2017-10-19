//! Library that implements [RFC 6902](https://tools.ietf.org/html/rfc6902), JavaScript Object Notation (JSON) Patch
#![feature(test)]
#![deny(warnings)]
#![warn(missing_docs)]
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_json::Value;
use std::mem;

mod util;
use util::{parse_index, split_pointer};

pub use util::PatchError;


/// Representation of JSON Patch (list of patch operations)
pub type Patch = Vec<PatchOperation>;



/// JSON Patch 'add' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct AddOperation {
    path: String,
    value: Value
}

/// JSON Patch 'remove' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct RemoveOperation {
    path: String
}

/// JSON Patch 'replace' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct ReplaceOperation {
    path: String,
    value: Value
}

/// JSON Patch 'move' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct MoveOperation {
    from: String,
    path: String
}

/// JSON Patch 'copy' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct CopyOperation {
    from: String,
    path: String
}

/// JSON Patch 'test' operation representation
#[derive(Debug, Deserialize, Clone)]
pub struct TestOperation {
    path: String,
    value: Value
}

/// JSON Patch single patch operation
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "op")]
#[serde(rename_all = "lowercase")]
pub enum PatchOperation {
    /// 'add' operation
    Add(AddOperation),
    /// 'remove' operation
    Remove(RemoveOperation),
    /// 'replace' operation
    Replace(ReplaceOperation),
    /// 'move' operation
    Move(MoveOperation),
    /// 'copy' operation
    Copy(CopyOperation),
    /// 'test' operation
    Test(TestOperation)
}

// FIXME: don't take value, take ref? Otherwise, we clone too early...
fn add(doc: &mut Value, path: &str, value: Value) -> Result<Option<Value>, PatchError> {
    if path == "" {
        return Ok(Some(mem::replace(doc, value)));
    }

    let (parent, last) = split_pointer(path)?;
    let parent = doc.pointer_mut(parent)
        .ok_or(PatchError::InvalidPointer)?;

    match *parent {
        Value::Object(ref mut obj) => {
            Ok(obj.insert(String::from(last), value))
        }
        Value::Array(ref mut arr) if last == "-" => {
            arr.push(value);
            Ok(None)
        }
        Value::Array(ref mut arr) => {
            let idx = parse_index(last.as_str(), arr.len() + 1)?;
            arr.insert(idx, value);
            Ok(None)
        }
        _ => Err(PatchError::InvalidPointer)
    }
}

fn remove(doc: &mut Value, path: &str, allow_last: bool) -> Result<Value, PatchError> {
    let (parent, last) = split_pointer(path)?;
    let parent = doc.pointer_mut(parent)
        .ok_or(PatchError::InvalidPointer)?;

    match *parent {
        Value::Object(ref mut obj) => {
            match obj.remove(last.as_str()) {
                None => Err(PatchError::InvalidPointer),
                Some(val) => Ok(val)
            }
        },
        Value::Array(ref mut arr) if allow_last && last == "-" => {
            Ok(arr.pop().unwrap())
        },
        Value::Array(ref mut arr) => {
            let idx = parse_index(last.as_str(), arr.len())?;
            Ok(arr.remove(idx))
        }
        _ => Err(PatchError::InvalidPointer)
    }
}

fn replace(doc: &mut Value, path: &str, value: &Value) -> Result<Value, PatchError> {
    let target = doc
        .pointer_mut(path)
        .ok_or(PatchError::InvalidPointer)?;
    Ok(mem::replace(target, value.clone()))
}

fn mov(doc: &mut Value, from: &str, path: &str) -> Result<Option<Value>, PatchError> {
    // Check we are not moving inside own child
    if path.starts_with(from) && path[from.len()..].starts_with("/") {
        return Err(PatchError::InvalidPointer);
    }
    let val = remove(doc, from, false)?; // FIXME: check...
    add(doc, path, val)
}

fn copy(doc: &mut Value, from: &str, path: &str) -> Result<Option<Value>, PatchError> {
    let source = doc
        .pointer(from)
        .ok_or(PatchError::InvalidPointer)?
        .clone();
    add(doc, path, source)
}

fn test(doc: &Value, path: &str, expected: &Value) -> Result<(), PatchError> {
    let target = doc
        .pointer(path)
        .ok_or(PatchError::InvalidPointer)?;
    if *target == *expected {
        Ok(())
    } else {
        Err(PatchError::TestFailed)
    }
}

/// Patch provided JSON document (given as `serde_json::Value`) in place.
/// Operation is atomic, i.e, if any of the patch is failed, no modifications to the value are made.
pub fn patch_mut(doc: &mut Value, patches: &[PatchOperation]) -> Result<(), PatchError> {
    match patches.split_first() {
        None => Ok(()),
        Some((patch, tail)) => {
            use PatchOperation::*;
            match *patch {
                Add(ref op) => {
                    let prev = add(doc, op.path.as_str(), op.value.clone())?;
                    patch_mut(doc, tail).map_err(move |e| {
                        match prev {
                            None => remove(doc, op.path.as_str(), true).unwrap(),
                            Some(v) => add(doc, op.path.as_str(), v).unwrap().unwrap()
                        };
                        e
                    })
                }
                Remove(ref op) => {
                    let prev = remove(doc, op.path.as_str(), false)?;
                    patch_mut(doc, tail).map_err(move |e| {
                        add(doc, op.path.as_str(), prev).unwrap().unwrap();
                        e
                    })
                }
                Replace(ref op) => {
                    let prev = replace(doc, op.path.as_str(), &op.value)?;
                    patch_mut(doc, tail).map_err(move |e| {
                        add(doc, op.path.as_str(), prev).unwrap().unwrap();
                        e
                    })
                }
                Move(ref op) => {
                    let prev = mov(doc, op.from.as_str(), op.path.as_str())?;
                    patch_mut(doc, tail).map_err(move |e| {
                        // FIXME: check "-" revert works!
                        mov(doc, op.path.as_str(), op.from.as_str()).unwrap();
                        if let Some(prev) = prev {
                            add(doc, op.path.as_str(), prev).unwrap().unwrap();
                        }
                        e
                    })
                }
                Copy(ref op) => {
                    let prev = copy(doc, op.from.as_str(), op.path.as_str())?;
                    patch_mut(doc, tail).map_err(move |e| {
                        match prev {
                            None => remove(doc, op.path.as_str(), true).unwrap(),
                            Some(v) => add(doc, op.path.as_str(), v).unwrap().unwrap()
                        };
                        e
                    })
                }
                Test(ref op) => {
                    test(doc, op.path.as_str(), &op.value)?;
                    patch_mut(doc, tail)
                }
            }
        }
    }
}

/// Patch provided JSON document (given as `serde_json::Value`) and return a new `serde_json::Value`
/// representing patched document.
pub fn patch(doc: &Value, patches: &[PatchOperation]) -> std::result::Result<Value, PatchError> {
    let mut copy = doc.clone();
    patch_mut(&mut copy, patches)?;
    Ok(copy)
}

#[cfg(test)]
mod tests;