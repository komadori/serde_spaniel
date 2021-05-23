use serde::Serialize;
use serde_spaniel::*;
use std::iter::empty;

use super::golden::{self, Golden};
use super::mock::MockPrompt;

fn test_ser<G: Golden>()
where
  G::V: Serialize,
{
  let mut prompt = MockPrompt::new(empty());
  to_bare_prompt(&G::value(), &mut prompt).unwrap();
  assert_eq!(prompt.responses(), G::responses(true));
  assert_eq!(prompt.scope_names(), G::scope_names())
}

#[test]
fn struct_of_prims() {
  test_ser::<golden::StructOfPrimsCase>()
}

#[test]
fn struct_of_seqs() {
  test_ser::<golden::StructOfSeqsCase>()
}

#[test]
fn tuple_of_options() {
  test_ser::<golden::TupleOfOptionsCase>()
}

#[test]
fn tuple_of_units() {
  test_ser::<golden::TupleOfUnitsCase>()
}

#[test]
fn seq_of_seqs() {
  test_ser::<golden::SeqOfSeqsCase>()
}

#[test]
fn map_of_enums_and_newtypes() {
  test_ser::<golden::MapOfEnumsAndNewtypesCase>()
}
