Spaniel Interactive Deserialiser for Serde
==========================================


[![crates.io](https://img.shields.io/crates/v/serde_spaniel.svg)](https://crates.io/crates/serde_spaniel)
[![docs](https://docs.rs/serde_spaniel/badge.svg)](https://docs.rs/serde_spaniel)

This crate is a Rust library which uses the [Serde][serde] serialisation
framework to capture data interactively from users.

[serde]: https://github.com/serde-rs/serde

## Dependency

```toml
[dependencies]
serde_spaniel = "0.4"
```

## Using Spaniel

Spaniel can produce a value of any type which implements Serde's `Deserialize`
trait by interactively querying the user for information. For example, to
interactively obtain a vector of strings:

```rust
let strs: Vec<String> = serde_spaniel::from_console()?;
```

Hence, a user could input the value `vec!["Hello", "World"]` using a dialogue
such as below:

```
seq {
  [0] {
    Add element?: y
    string: Hello
  }
  [1] {
    Add element?: y
    string: World
  }
  [2] {
    Add element?: n
  }
}
Accept value?: y
```

## Licence

Dual-licensed under either of

- Apache Licence, Version 2.0, ([LICENCE-APACHE](/LICENCE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT licence ([LICENCE-MIT](/LICENCE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 licence, shall
be dual-licensed as above, without any additional terms or conditions.
