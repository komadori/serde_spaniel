use serde::Deserialize;
use spaniel_id::*;
use std::collections::HashMap;

mod util;
use crate::util::MockPrompt;

#[test]
fn struct_of_prims() {
  #[derive(Debug, Deserialize, PartialEq)]
  struct StructOfPrims {
    boolean: bool,
    character: char,
    int8: i8,
    int16: i16,
    int32: i32,
    int64: i64,
    int128: i128,
    intptr: isize,
    uint8: u8,
    uint16: u16,
    uint32: u32,
    uint64: u64,
    uint128: u128,
    uintptr: usize,
    single: f32,
    double: f64,
  }

  let vec = vec![
    "true",
    "A",
    "-123",
    "-12345",
    "-1234567891",
    "-1234567891234567891",
    "-123456789123456789123456789123456789123",
    "-1",
    "255",
    "65535",
    "4294967295",
    "18446744073709551615",
    "340282366920938463463374607431768211455",
    "0",
    "2.7182818284590452353602",
    "3.141592653589793115997963468544185161590576171875",
  ]
  .into_iter();
  let value: StructOfPrims = from_bare_prompt(MockPrompt::new(vec)).unwrap();
  assert_eq!(
    value,
    StructOfPrims {
      boolean: true,
      character: 'A',
      int8: -123,
      int16: -12345,
      int32: -1234567891,
      int64: -1234567891234567891,
      int128: -123456789123456789123456789123456789123,
      intptr: -1,
      uint8: 255,
      uint16: 65535,
      uint32: 4294967295,
      uint64: 18446744073709551615,
      uint128: 340282366920938463463374607431768211455,
      uintptr: 0,
      single: 2.7182818284590452353602,
      double: 3.141592653589793115997963468544185161590576171875,
    }
  );
}

#[test]
fn tuple_of_options() {
  type TupleOfOptions<'a> = (
    Option<bool>,
    Option<String>,
    Option<Option<bool>>,
    Option<Option<String>>,
    Option<Option<bool>>,
  );
  let vec =
    vec!["n", "y", "Tinker", "y", "n", "y", "y", "Tailor", "n"].into_iter();
  let value: TupleOfOptions = from_bare_prompt(MockPrompt::new(vec)).unwrap();
  assert_eq!(
    value,
    (
      None,
      Some("Tinker".to_string()),
      Some(None),
      Some(Some("Tailor".to_string())),
      None
    )
  );
}

#[test]
fn tuple_of_units() {
  #[derive(Debug, Deserialize, PartialEq)]
  struct NewtypeUnit();
  #[derive(Debug, Deserialize, PartialEq)]
  enum EnumUnit {
    Unit,
  }
  type TupleOfUnits = ((), NewtypeUnit, EnumUnit);
  let vec = vec![].into_iter();
  let value: TupleOfUnits = from_bare_prompt(MockPrompt::new(vec)).unwrap();
  assert_eq!(value, ((), NewtypeUnit(), EnumUnit::Unit));
}

#[test]
fn seq_of_seqs() {
  let vec = &mut vec![
    "y", "y", "H", "y", "e", "y", "l", "y", "l", "y", "o", "n", "y", "n", "y",
    "yes", "Y", "yes", "O", "yes", "U", "no", "n",
  ]
  .into_iter();
  let value: Vec<Vec<char>> = from_bare_prompt(MockPrompt::new(vec)).unwrap();
  assert_eq!(
    value,
    vec!(vec!('H', 'e', 'l', 'l', 'o'), vec!(), vec!('Y', 'O', 'U'))
  );
}

#[test]
fn map_of_enums_and_newtypes() {
  #[derive(Debug, Deserialize, PartialEq)]
  struct BoxSize(u32);
  #[derive(Debug, Deserialize, PartialEq)]
  struct Kilos(u32);
  #[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
  enum Fruit {
    Apple,
    Orange,
    Other(String),
  }
  #[derive(Debug, Deserialize, PartialEq)]
  enum Quantity {
    Single,
    Boxes(u32, BoxSize),
    Weight(Kilos),
  }
  let vec = vec![
    "y", "Apple", "Single", "y", "Orange", "Boxes", "1000", "12", "y", "Other",
    "Banana", "Weight", "50", "n",
  ]
  .into_iter();
  let value: HashMap<Fruit, Quantity> =
    from_bare_prompt(MockPrompt::new(vec)).unwrap();
  let mut golden = HashMap::new();
  golden.insert(Fruit::Apple, Quantity::Single);
  golden.insert(Fruit::Orange, Quantity::Boxes(1000, BoxSize(12)));
  golden.insert(
    Fruit::Other("Banana".to_string()),
    Quantity::Weight(Kilos(50)),
  );
  assert_eq!(value, golden);
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
