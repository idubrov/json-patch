//! Library that implements [RFC 6902](https://tools.ietf.org/html/rfc6902), JavaScript Object Notation (JSON) Patch
//! # Examples
//! Create and patch document:
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_json;
//! extern crate json_patch;
//! use json_patch::{patch, from_value};
//!
//! # pub fn main() {
//! let mut doc = json!([
//!     { "name": "Andrew" },
//!     { "name": "Maxim" }
//! ]);
//!
//! let ops = from_value(json!([
//!   { "op": "test", "path": "/0/name", "value": "Andrew" },
//!   { "op": "add", "path": "/0/happy", "value": true }
//! ])).unwrap();
//!
//! patch(&mut doc, &ops).unwrap();
//! assert_eq!(doc, json!([
//!   { "name": "Andrew", "happy": true },
//!   { "name": "Maxim" }
//! ]));
//!
//! # }
//! ```
#![feature(test)]
#![deny(warnings)]
#![warn(missing_docs)]
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_json::Value;
use std::mem;
use util::{parse_index, split_pointer};
pub use util::PatchError;

mod util;

/// Representation of JSON Patch (list of patch operations)
pub struct Patch(Vec<PatchOperation>);


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
        }
        Value::Array(ref mut arr) if allow_last && last == "-" => {
            Ok(arr.pop().unwrap())
        }
        Value::Array(ref mut arr) => {
            let idx = parse_index(last.as_str(), arr.len())?;
            Ok(arr.remove(idx))
        }
        _ => Err(PatchError::InvalidPointer)
    }
}

fn replace(doc: &mut Value, path: &str, value: Value) -> Result<Value, PatchError> {
    let target = doc
        .pointer_mut(path)
        .ok_or(PatchError::InvalidPointer)?;
    Ok(mem::replace(target, value))
}

fn mov(doc: &mut Value, from: &str, path: &str, allow_last: bool) -> Result<Option<Value>, PatchError> {
    // Check we are not moving inside own child
    if path.starts_with(from) && path[from.len()..].starts_with('/') {
        return Err(PatchError::InvalidPointer);
    }
    let val = remove(doc, from, allow_last)?;
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

/// Create JSON Patch from JSON Value
pub fn from_value(value: Value) -> Result<Patch, serde_json::Error> {
    let patch = serde_json::from_value::<Vec<PatchOperation>>(value)?;
    Ok(Patch(patch))
}

/// Patch provided JSON document (given as `serde_json::Value`) in-place. If any of the patch is
/// failed, all previous operations are reverted. In case of internal error resulting in panic,
/// document might be left in inconsistent state.
pub fn patch(doc: &mut Value, patch: &Patch) -> Result<(), PatchError> {
    apply_patches(doc, &patch.0)
}

fn apply_patches(doc: &mut Value, patches: &[PatchOperation]) -> Result<(), PatchError> {
    let (patch, tail) = match patches.split_first() {
        None => return Ok(()),
        Some((patch, tail)) => (patch, tail)
    };

    use PatchOperation::*;
    match *patch {
        Add(ref op) => {
            let prev = add(doc, op.path.as_str(), op.value.clone())?;
            apply_patches(doc, tail).map_err(move |e| {
                match prev {
                    None => remove(doc, op.path.as_str(), true).unwrap(),
                    Some(v) => add(doc, op.path.as_str(), v).unwrap().unwrap()
                };
                e
            })
        }
        Remove(ref op) => {
            let prev = remove(doc, op.path.as_str(), false)?;
            apply_patches(doc, tail).map_err(move |e| {
                assert!(add(doc, op.path.as_str(), prev).unwrap().is_none());
                e
            })
        }
        Replace(ref op) => {
            let prev = replace(doc, op.path.as_str(), op.value.clone())?;
            apply_patches(doc, tail).map_err(move |e| {
                replace(doc, op.path.as_str(), prev).unwrap();
                e
            })
        }
        Move(ref op) => {
            let prev = mov(doc, op.from.as_str(), op.path.as_str(), false)?;
            apply_patches(doc, tail).map_err(move |e| {
                mov(doc, op.path.as_str(), op.from.as_str(), true).unwrap();
                if let Some(prev) = prev {
                    assert!(add(doc, op.path.as_str(), prev).unwrap().is_none());
                }
                e
            })
        }
        Copy(ref op) => {
            let prev = copy(doc, op.from.as_str(), op.path.as_str())?;
            apply_patches(doc, tail).map_err(move |e| {
                match prev {
                    None => remove(doc, op.path.as_str(), true).unwrap(),
                    Some(v) => add(doc, op.path.as_str(), v).unwrap().unwrap()
                };
                e
            })
        }
        Test(ref op) => {
            test(doc, op.path.as_str(), &op.value)?;
            apply_patches(doc, tail)
        }
    }
}


/// Patch provided JSON document (given as `serde_json::Value`) in place.
/// Operations are applied in unsafe manner. If any of the operations fails, all previous
/// operations are not reverted.
pub unsafe fn patch_unsafe(doc: &mut Value, patch: &Patch) -> Result<(), PatchError> {
    use PatchOperation::*;
    for op in &patch.0 {
        match *op {
            Add(ref op) => { add(doc, op.path.as_str(), op.value.clone())?; }
            Remove(ref op) => { remove(doc, op.path.as_str(), false)?; }
            Replace(ref op) => { replace(doc, op.path.as_str(), op.value.clone())?; }
            Move(ref op) => { mov(doc, op.from.as_str(), op.path.as_str(), false)?; }
            Copy(ref op) => { copy(doc, op.from.as_str(), op.path.as_str())?; }
            Test(ref op) => { test(doc, op.path.as_str(), &op.value)?; }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests;