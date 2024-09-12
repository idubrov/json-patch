# json-patch for ktmrtb

is a modefied version of [json-patch](https://github.com/idubrov/json-patch) with extra features for ktmrtb.

[![crates.io](https://img.shields.io/crates/v/json-patch.svg)](https://crates.io/crates/json-patch)
[![crates.io](https://img.shields.io/crates/d/json-patch.svg)](https://crates.io/crates/json-patch)
[![Build](https://github.com/idubrov/json-patch/actions/workflows/main.yml/badge.svg)](https://github.com/idubrov/json-patch/actions)
[![Codecov](https://codecov.io/gh/idubrov/json-patch/branch/main/graph/badge.svg?token=hdcr6yfBfa)](https://codecov.io/gh/idubrov/json-patch)

## json-patch

A [JSON Patch (RFC 6902)](https://tools.ietf.org/html/rfc6902) and
[JSON Merge Patch (RFC 7396)](https://tools.ietf.org/html/rfc7396) implementation for Rust.

## Usage

Add this to your *Cargo.toml*:

```toml
[dependencies]
json-patch = "*"
```

## Examples

Create and patch document using JSON Patch:

```rust
#[macro_use]
use json_patch::{Patch, patch};
use serde_json::{from_value, json};

let mut doc = json!([
    { "name": "Andrew" },
    { "name": "Maxim" }
]);

let p: Patch = from_value(json!([
  { "op": "test", "path": "/0/name", "value": "Andrew" },
  { "op": "add", "path": "/0/happy", "value": true }
])).unwrap();

patch(&mut doc, &p).unwrap();
assert_eq!(doc, json!([
  { "name": "Andrew", "happy": true },
  { "name": "Maxim" }
]));

```

Create and patch document using JSON Merge Patch:

```rust
#[macro_use]
use json_patch::merge;
use serde_json::json;

let mut doc = json!({
      "title": "Goodbye!",

      "imps": [
        {
          "id": "1",
          "c": {
            "d": "e",
            "f": "g"
          }
        },
        {
          "id": "2",
          "c": {
            "d": "e",
            "f": "g"
          }
        }
      ]
    });

let patch = json!({
      "imps": {
        "add": "added new key",
        "c": {
          "d": null,
          "e": "added",
          "f": "modified"
        }
      }
    });

merge(&mut doc, &patch);
assert_eq!(doc, json!({
      "title": "Goodbye!",

      "imps": [
        {
          "id": "1",
          "add": "added new key",
          "c": {
            "e": "added",
            "f": "modified"
          }
        },
        {
          "id": "2",
          "add": "added new key",
          "c": {
            "e": "added",
            "f": "modified"
          }
        }
      ]
    }));
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
