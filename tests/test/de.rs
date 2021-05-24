use serde::Deserialize;
use serde_spaniel::*;

use super::golden::{self, Golden};
use super::mock::MockPrompt;

fn test_de<'a, G: Golden>()
where
  G::V: Deserialize<'a>,
{
  let mut prompt = MockPrompt::new(G::responses(false).into_iter());
  let value: G::V = from_bare_prompt(&mut prompt).unwrap();
  assert_eq!(value, G::value());
  assert_eq!(prompt.scope_names(), G::scope_names())
}

#[test]
fn struct_of_prims() {
  test_de::<golden::StructOfPrimsCase>()
}

#[test]
fn struct_of_seqs() {
  test_de::<golden::StructOfSeqsCase>()
}

#[test]
fn tuple_of_options() {
  test_de::<golden::TupleOfOptionsCase>()
}

#[test]
fn tuple_of_units() {
  test_de::<golden::TupleOfUnitsCase>()
}

#[test]
fn seq_of_seqs() {
  test_de::<golden::SeqOfSeqsCase>()
}

#[test]
fn map_of_enums_and_newtypes() {
  test_de::<golden::MapOfEnumsAndNewtypesCase>()
}

#[test]
fn bad_u32() {
  let vec = vec!["not a number"].into_iter();
  let value: Result<u32> = from_bare_prompt(MockPrompt::new(vec));
  assert_eq!(value, Err(Error::BadResponse))
}

#[test]
fn bad_u32_interactive() {
  let vec = vec!["not a number", "another string", "123"].into_iter();
  let value: u32 =
    from_bare_prompt(MockPrompt::new(vec).with_interactive()).unwrap();
  assert_eq!(value, 123)
}

#[test]
fn bytes() {
  test_de::<golden::BytesCase>()
}
