Spaniel Interactive Deserialiser for Serde
==========================================

This crate is a Rust library which uses the Serde serialisation framework to
capture data interactively from users.

[Serde]: https://github.com/serde-rs/serde


## Dependency

```toml
[dependencies]
serde_spaniel = "0.1"
```

## Using Spaniel

Spaniel can produce a value of any type which implements Serde's `Deserialize`
trait by interactively querying the user for information. For example:

```rust
let strs: Vec<String> = serde_spaniel::from_console();
```
