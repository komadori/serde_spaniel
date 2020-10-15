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
