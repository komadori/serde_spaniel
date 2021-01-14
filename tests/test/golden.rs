use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;

pub trait Golden {
  type V: Debug + PartialEq;
  fn value() -> Self::V;
  fn responses(include_unit_variants: bool) -> Vec<&'static str>;
}

pub enum StructOfPrimsCase {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct StructOfPrims {
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

impl Golden for StructOfPrimsCase {
  type V = StructOfPrims;

  fn value() -> Self::V {
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
      single: 2.7182817,
      double: 3.141592653589793,
    }
  }

  fn responses(_: bool) -> Vec<&'static str> {
    vec![
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
      "2.7182817",
      "3.141592653589793",
    ]
  }
}

pub enum StructOfSeqsCase {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct StructOfSeqs {
  ints: Vec<u32>,
  option_units: Vec<Option<()>>,
}

impl Golden for StructOfSeqsCase {
  type V = StructOfSeqs;

  fn value() -> Self::V {
    StructOfSeqs {
      ints: vec![60, 3600],
      option_units: vec![None, Some(()), None],
    }
  }

  fn responses(_: bool) -> Vec<&'static str> {
    vec![
      "yes", "60", "yes", "3600", "no", "yes", "no", "yes", "yes", "yes", "no",
      "no",
    ]
  }
}

pub enum TupleOfOptionsCase {}

impl Golden for TupleOfOptionsCase {
  type V = (
    Option<bool>,
    Option<String>,
    Option<Option<bool>>,
    Option<Option<String>>,
    Option<Option<bool>>,
  );

  fn value() -> Self::V {
    (
      None,
      Some("Tinker".to_string()),
      Some(None),
      Some(Some("Tailor".to_string())),
      None,
    )
  }

  fn responses(_: bool) -> Vec<&'static str> {
    vec![
      "no", "yes", "Tinker", "yes", "no", "yes", "yes", "Tailor", "no",
    ]
  }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct NewtypeUnit();

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum EnumUnit {
  Unit,
}

pub enum TupleOfUnitsCase {}

impl Golden for TupleOfUnitsCase {
  type V = ((), NewtypeUnit, EnumUnit);

  fn value() -> Self::V {
    ((), NewtypeUnit(), EnumUnit::Unit)
  }

  fn responses(include_unit_variants: bool) -> Vec<&'static str> {
    if include_unit_variants {
      vec!["Unit"]
    } else {
      vec![]
    }
  }
}

pub enum SeqOfSeqsCase {}

impl Golden for SeqOfSeqsCase {
  type V = Vec<Vec<char>>;

  fn value() -> Self::V {
    vec![vec!['H', 'e', 'l', 'l', 'o'], vec![], vec!['Y', 'O', 'U']]
  }

  fn responses(_: bool) -> Vec<&'static str> {
    vec![
      "yes", "yes", "H", "yes", "e", "yes", "l", "yes", "l", "yes", "o", "no",
      "yes", "no", "yes", "yes", "Y", "yes", "O", "yes", "U", "no", "no",
    ]
  }
}

pub enum MapOfEnumsAndNewtypesCase {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct BoxSize(u32);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Kilos(u32);

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Serialize, Eq, Ord)]
pub enum Fruit {
  Apple,
  Orange,
  Other(String),
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum Quantity {
  Single,
  Boxes(u32, BoxSize),
  Weight(Kilos),
}

impl Golden for MapOfEnumsAndNewtypesCase {
  type V = BTreeMap<Fruit, Quantity>;

  fn value() -> Self::V {
    let mut golden = BTreeMap::new();
    golden.insert(Fruit::Apple, Quantity::Single);
    golden.insert(Fruit::Orange, Quantity::Boxes(1000, BoxSize(12)));
    golden.insert(
      Fruit::Other("Banana".to_string()),
      Quantity::Weight(Kilos(50)),
    );
    golden
  }

  fn responses(_: bool) -> Vec<&'static str> {
    vec![
      "yes", "Apple", "Single", "yes", "Orange", "Boxes", "1000", "12", "yes",
      "Other", "Banana", "Weight", "50", "no",
    ]
  }
}
