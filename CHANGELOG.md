# Changelog

## 0.28.0 (2022-12-09)

### Fixed

- Fixed incorrect diffing for the whole document. Previously, differ would incorrectly yield path of `"/"` when the
  whole document is replaced. The correct path should be `""`. This is a breaking change.
  [#18](https://github.com/idubrov/json-patch/pull/18)
