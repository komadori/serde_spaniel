use serde::Serialize;
use serde_spaniel::prompt::RequestKind;
use serde_spaniel::*;
use std::iter::empty;

use super::mock::MockPrompt;

#[test]
fn struct_of_seqs() {
  #[derive(Debug, Serialize, PartialEq)]
  struct StructOfSeqs {
    ints: Vec<u32>,
    option_units: Vec<Option<()>>,
  }
  let value = StructOfSeqs {
    ints: vec![60, 3600],
    option_units: vec![None, Some(()), None],
  };
  let mut mock = MockPrompt::new(empty());
  to_prompt(&value, &mut mock).unwrap();
  assert_eq!(
    mock.responses(),
    vec![
      "yes", "60", "yes", "3600", "no", "yes", "no", "yes", "yes", "()", "yes",
      "no", "no"
    ]
  );
}
