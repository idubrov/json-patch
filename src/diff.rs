extern crate treediff;

use serde_json::Value;

struct PatchDiffer {
    path: String,
    patch: super::Patch,
}

impl PatchDiffer {
    fn new() -> Self {
        Self {
            path: "/".to_string(),
            patch: super::Patch(Vec::new()),
        }
    }
}

impl<'a> treediff::Delegate<'a, treediff::value::Key, Value> for PatchDiffer {
    fn push<'b>(&mut self, key: &'b treediff::value::Key) {
        use std::fmt::Write;
        if self.path.len() != 1 {
            self.path.push('/');
        }
        match *key {
            treediff::value::Key::Index(idx) => write!(self.path, "{}", idx).unwrap(),
            treediff::value::Key::String(ref key) => self.path += key,
        }
    }

    fn pop(&mut self) {
        let mut pos = self.path.rfind('/').unwrap_or(0);
        if pos == 0 {
            pos = 1;
        }
        self.path.truncate(pos);
    }

    fn removed<'b>(&mut self, k: &'b treediff::value::Key, _v: &'a Value) {
        self.push(k);
        self.patch
            .0
            .push(super::PatchOperation::Remove(super::RemoveOperation {
                path: self.path.clone(),
            }));
        self.pop();
    }

    fn added(&mut self, k: &treediff::value::Key, v: &Value) {
        self.push(k);
        self.patch
            .0
            .push(super::PatchOperation::Add(super::AddOperation {
                path: self.path.clone(),
                value: v.clone(),
            }));
        self.pop();
    }

    fn modified(&mut self, _old: &'a Value, new: &'a Value) {
        self.patch
            .0
            .push(super::PatchOperation::Replace(super::ReplaceOperation {
                path: self.path.clone(),
                value: new.clone(),
            }));
    }
}

/// Diff two JSON documents and generate a JSON Patch (RFC 6902).
///
/// # Example
/// Diff two JSONs:
///
/// ```rust
/// #[macro_use]
/// extern crate serde_json;
/// extern crate json_patch;
///
/// use json_patch::{patch, diff, from_value};
///
/// # pub fn main() {
/// let left = json!({
///   "title": "Goodbye!",
///   "author" : {
///     "givenName" : "John",
///     "familyName" : "Doe"
///   },
///   "tags":[ "example", "sample" ],
///   "content": "This will be unchanged"
/// });
///
/// let right = json!({
///   "title": "Hello!",
///   "author" : {
///     "givenName" : "John"
///   },
///   "tags": [ "example" ],
///   "content": "This will be unchanged",
///   "phoneNumber": "+01-123-456-7890"
/// });
///
/// let p = diff(&left, &right);
/// assert_eq!(p, from_value(json!([
///   { "op": "remove", "path": "/author/familyName" },
///   { "op": "remove", "path": "/tags/1" },
///   { "op": "replace", "path": "/title", "value": "Hello!" },
///   { "op": "add", "path": "/phoneNumber", "value": "+01-123-456-7890" },
/// ])).unwrap());
///
/// let mut doc = left.clone();
/// patch(&mut doc, &p).unwrap();
/// assert_eq!(doc, right);
///
/// # }
/// ```
pub fn diff(left: &Value, right: &Value) -> super::Patch {
    let mut differ = PatchDiffer::new();
    treediff::diff(left, right, &mut differ);
    differ.patch
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    #[test]
    pub fn replace_all() {
        let left = json!({"title": "Hello!"});
        let p = super::diff(&left, &Value::Null);
        assert_eq!(
            p,
            ::from_value(json!([
                { "op": "replace", "path": "/", "value": null },
            ]))
            .unwrap()
        );
    }

    #[test]
    pub fn add_all() {
        let right = json!({"title": "Hello!"});
        let p = super::diff(&Value::Null, &right);
        assert_eq!(
            p,
            ::from_value(json!([
                { "op": "replace", "path": "/", "value": { "title": "Hello!" } },
            ]))
            .unwrap()
        );
    }
}
