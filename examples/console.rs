use serde::{Deserialize, Serialize};
use serde_spaniel::stdio::ReadWritePrompt;
use serde_spaniel::{from_console, to_prompt};

#[derive(Serialize, Deserialize)]
struct ChildInfo {
  name: String,
  age: u32,
}

#[derive(Serialize, Deserialize)]
struct ParentInfo {
  name: String,
  age: u32,
  children: Vec<ChildInfo>,
}

fn main() {
  let mut parent: ParentInfo = from_console().expect("ParentInfo required!");

  parent.age += 1;
  for child in parent.children.iter_mut() {
    child.age += 1;
  }

  println!("One year from now you will have to type:");
  to_prompt(&parent, ReadWritePrompt::new_stdio())
    .expect("Error while serialising.");
}
