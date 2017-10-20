[![crates.io](https://img.shields.io/crates/v/json-patch.svg)](https://crates.io/crates/json-patch)
[![crates.io](https://img.shields.io/crates/d/json-patch.svg)](https://crates.io/crates/json-patch)
[![CircleCI](https://img.shields.io/circleci/project/github/idubrov/json-patch.svg)](https://circleci.com/gh/idubrov/json-patch)
[![Codecov](https://img.shields.io/codecov/c/github/idubrov/json-patch.svg)](https://codecov.io/gh/idubrov/json-patch)

# json-patch

Library that implements [RFC 6902](https://tools.ietf.org/html/rfc6902), JavaScript Object Notation (JSON) Patch
## Examples
Create and patch document:

```rust
#[macro_use]
extern crate serde_json;
extern crate json_patch;
use json_patch::{patch, from_value};

let mut doc = json!([
    { "name": "Andrew" },
    { "name": "Maxim" }
]);

let ops = from_value(json!([
  { "op": "test", "path": "/0/name", "value": "Andrew" },
  { "op": "add", "path": "/0/happy", "value": true }
])).unwrap();

patch(&mut doc, &ops).unwrap();
assert_eq!(doc, json!([
  { "name": "Andrew", "happy": true },
  { "name": "Maxim" }
]));

```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
