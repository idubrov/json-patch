//! Library that implements [RFC 6902](https://tools.ietf.org/html/rfc6902), JavaScript Object Notation (JSON) Patch
#![deny(warnings)]
#![warn(missing_docs)]
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_json::Value;

use std::fmt;
use std::error::Error;

/// This type represents all possible errors that can occur when applying JSON patch
#[derive(Debug)]
pub enum PatchError {
    /// One of the paths in the patch is invalid
    InvalidPointer,

    /// 'test' operation failed
    TestFailed
}

impl Error for PatchError {
    fn description(&self) -> &str {
        use PatchError::*;
        match *self {
            InvalidPointer => "invalid pointer",
            TestFailed => "test failed"
        }
    }
}

impl fmt::Display for PatchError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(fmt)
    }
}

type Result = std::result::Result<(), PatchError>;

trait Operation {
    fn apply_mut(&self, doc: &mut Value) -> Result;
}

/// JSON Patch 'add' operation representation
#[derive(Debug, Deserialize)]
pub struct AddOperation {
    path: String,
    value: Value
}

impl Operation for AddOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        if self.path == "" {
            *doc = self.value.clone();
            return Ok(());
        }

        let (parent, last) = split_pointer(self.path.as_str())?;
        let last: String = last.replace("~1", "/").replace("~0", "~");

        let parent = doc.pointer_mut(parent)
            .ok_or(PatchError::InvalidPointer)?;

        let value = self.value.clone();
        match *parent {
            Value::Object(ref mut obj) => {
                obj.insert(String::from(last), value);
            }
            Value::Array(ref mut arr) if last == "-" => {
                arr.push(value);
            },
            Value::Array(ref mut arr) => {
                let idx = last.parse::<usize>().map_err(|_| PatchError::InvalidPointer)?;
                if idx > arr.len() {
                    return Err(PatchError::InvalidPointer)
                }
                arr.insert(idx, value);
            }
            _ => return Err(PatchError::InvalidPointer)
        }
        Ok(())
    }
}

/// JSON Patch 'remove' operation representation
#[derive(Debug, Deserialize)]
pub struct RemoveOperation {
    path: String
}

impl Operation for RemoveOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        let (parent, last) = split_pointer(self.path.as_str())?;
        let last: String = last.replace("~1", "/").replace("~0", "~");

        let parent = doc.pointer_mut(parent)
            .ok_or(PatchError::InvalidPointer)?;

        match *parent {
            Value::Object(ref mut obj) => {
                if obj.remove(&last).is_none() {
                    Err(PatchError::InvalidPointer)
                } else {
                    Ok(())
                }
            }
            Value::Array(ref mut arr) => {
                let idx = last.parse::<usize>().map_err(|_| PatchError::InvalidPointer)?;
                if idx as usize >= arr.len() {
                    return Err(PatchError::InvalidPointer);
                }
                arr.remove(idx as usize);
                Ok(())
            }
            _ => Err(PatchError::InvalidPointer)
        }
    }
}

/// JSON Patch 'replace' operation representation
#[derive(Debug, Deserialize)]
pub struct ReplaceOperation {
    path: String,
    value: Value
}

impl Operation for ReplaceOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        let val = doc
            .pointer_mut(self.path.as_str())
            .ok_or(PatchError::InvalidPointer)?;
        *val = self.value.clone();
        Ok(())
    }
}

/// JSON Patch 'move' operation representation
#[derive(Debug, Deserialize)]
pub struct MoveOperation {
    from: String,
    path: String
}

impl Operation for MoveOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        // FIXME: more optimal implementation...
        let value = doc
            .pointer(self.from.as_str())
            .ok_or(PatchError::InvalidPointer)?
            .clone();

        let remove = RemoveOperation { path: self.from.clone() };
        remove.apply_mut(doc)?;
        let add = AddOperation { path: self.path.clone(), value };
        add.apply_mut(doc)?;
        Ok(())
    }
}

/// JSON Patch 'copy' operation representation
#[derive(Debug, Deserialize)]
pub struct CopyOperation {
    from: String,
    path: String
}

impl Operation for CopyOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        let value = doc
            .pointer(self.from.as_str())
            .ok_or(PatchError::InvalidPointer)?
            .clone();


        let add = AddOperation { path: self.path.clone(), value };
        add.apply_mut(doc)?;
        Ok(())
    }
}

/// JSON Patch 'test' operation representation
#[derive(Debug, Deserialize)]
pub struct TestOperation {
    path: String,
    value: Value
}

impl Operation for TestOperation {
    fn apply_mut(&self, doc: &mut Value) -> Result {
        let val = doc
            .pointer(self.path.as_str())
            .ok_or(PatchError::InvalidPointer)?;
        if *val == self.value {
            Ok(())
        } else {
            Err(PatchError::TestFailed)
        }
    }
}

/// JSON Patch single patch operation
#[derive(Debug, Deserialize)]
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

/// Representation of JSON Patch (list of patch operations)
pub type Patch = Vec<PatchOperation>;

fn split_pointer(pointer: &str) -> std::result::Result<(&str, &str), PatchError> {
    pointer.rfind('/')
        .ok_or(PatchError::InvalidPointer)
        .map(|idx| (&pointer[0..idx], &pointer[idx + 1..]))
}

fn patch_internal(doc: &mut Value, patches: &[PatchOperation]) -> Result {
    use PatchOperation::*;
    for patch in patches {
        match *patch {
            Add(ref add) => add.apply_mut(doc)?,
            Remove(ref remove) => remove.apply_mut(doc)?,
            Replace(ref replace) => replace.apply_mut(doc)?,
            Move(ref mov) => mov.apply_mut(doc)?,
            Copy(ref copy) => copy.apply_mut(doc)?,
            Test(ref test) => test.apply_mut(doc)?,
        }
    }
    Ok(())
}

/// Patch provided JSON document (given as `serde_json::Value`) in place.
/// Operation is atomic, i.e, if any of the patch is failed, no modifications to the value are made.
pub fn patch_mut(doc: &mut Value, patches: &[PatchOperation]) -> Result {
    let mut copy: Value = doc.clone();
    patch_internal(&mut copy, patches)?;
    *doc = copy;
    Ok(())
}

/// Patch provided JSON document (given as `serde_json::Value`) and return a new `serde_json::Value`
/// representing patched document.
pub fn patch(doc: &Value, patches: &[PatchOperation]) -> std::result::Result<Value, PatchError> {
    let mut copy = doc.clone();
    patch_internal(&mut copy, patches)?;
    Ok(copy)
}