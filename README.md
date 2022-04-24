# strtools
**strtools** is a rust library for various helpful string extensions. It is under active
development and adding more features is planned, criticism and contributions are welcome.

## Examples
```rust
use strtools::StrTools;

// split a string by some separator but ignore escaped ones
let parts: Vec<_> = r"this string\ is split by\ spaces unless they are\ escaped"
    .split_non_escaped('\\', &[' '])
    .collect();

assert_eq!(
    parts,
    [
        "this",
        "string is",
        "split",
        "by spaces",
        "unless",
        "they",
        "are escaped"
    ]
);
```

## License
**strtools** is currently licensed under the <a href="LICENSE-MIT">MIT license</a>.
