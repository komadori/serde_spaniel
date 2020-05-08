Spaniel Interactive Deserialiser
================================

This crate is a Rust library which uses the Serde serialisation framework to
capture data interactively from users.

[Serde]: https://github.com/serde-rs/serde


## Dependency

```toml
[dependencies]
spaniel_id = "0.1"
```

## Using Spaniel

Spaniel can produce a value of any type which implements Serde's `Deserialize`
trait by interactively querying the user for information. For example:

```rust
let strs: Vec<String> = spaniel_id::from_console();
```
