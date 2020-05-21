use serde::Deserialize;
use spaniel_id::prompt::{CompactPrompt, RequestKind};
use spaniel_id::*;

mod util;
use crate::util::{LogEntry, MockPrompt};

#[test]
fn compact_request() {
  #[derive(Debug, Deserialize, PartialEq)]
  struct SimpleStruct {
    my_field: String,
  }

  let vec = vec!["Test"].into_iter();
  let mut mock = MockPrompt::new(vec);
  let value: SimpleStruct =
    from_bare_prompt(CompactPrompt::new(&mut mock)).unwrap();
  assert_eq!(
    value,
    SimpleStruct {
      my_field: "Test".into()
    }
  );
  assert_eq!(
    mock.into_log(),
    vec![LogEntry::Response(
      RequestKind::Datum,
      "SimpleStruct -> my_field -> string".into(),
      &[],
      "Test".into()
    ),]
  );
}

#[test]
fn compact_scope() {
  #[derive(Debug, Deserialize, PartialEq)]
  enum SimpleEnum {
    Test,
    Fish,
  }
  #[derive(Debug, Deserialize, PartialEq)]
  struct SimpleStruct {
    my_field: SimpleEnum,
  }

  let vec = vec!["Test"].into_iter();
  let mut mock = MockPrompt::new(vec);
  let value: SimpleStruct =
    from_bare_prompt(CompactPrompt::new(&mut mock)).unwrap();
  assert_eq!(
    value,
    SimpleStruct {
      my_field: SimpleEnum::Test
    }
  );
  assert_eq!(
    mock.into_log(),
    vec![
      LogEntry::BeginScope(
        "SimpleStruct -> my_field -> SimpleEnum".into(),
        None
      ),
      LogEntry::Response(
        RequestKind::Datum,
        "variant".into(),
        &["Test", "Fish"],
        "Test".into()
      ),
      LogEntry::EndScope
    ]
  );
}
