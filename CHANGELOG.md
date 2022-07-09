# Changelog

---
## [0.3.0] - 2020-07-09
### Additions
- `escape::charset`
- `find::longest_unique_substr`
- `parse::{FromStrFront, FromStrBack, FromStrPartialRadixExt}`
- `split::n_times`
- `util::Sorted` and `util::SortedSlice` for sorted slices and arrays

### Changes
- Made `split::non_escaped*` use `util::Sorted` for delimiter slice
- Added `parse_front`/`parse_back` to `StrTools`

### Fixes
- documentation typos


---
## [0.2.0] - 2022-06-03
### Additions
- `split::char_boundary_*`
- `split::non_escaped`

### Changes
- Moved splitting functions in `split` module wihtout `split_` prefix
- Renamed `split_non_escaped` to `split::non_escaped_sanitize`

### Fixes
- Fixed README example parameter ordering being incorrect


---
## [0.1.1] - 2022-04-18
### Fixes
- typo in README.md
- removed redundant word from package description


---
## [0.1.0] - 2022-04-18
Initial Release


---
[0.2.1]: https://github.com/epbuennig/strtools/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/epbuennig/strtools/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/epbuennig/strtools/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/epbuennig/strtools/compare/master...v0.1.0