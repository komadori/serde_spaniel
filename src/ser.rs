use serde::ser::{self, Serialize};

use crate::error::{Error, Result};
use crate::internal::{InternalPrompt, ScopeLimit};
use crate::internal_prompt_responder_mixin;
use crate::prompt::{PromptResponder, RequestKind};

pub struct Serializer<P: PromptResponder> {
  prompt: InternalPrompt<P>,
}

impl<P: PromptResponder> Serializer<P> {
  pub fn from_prompt(prompt: P) -> Self {
    Serializer {
      prompt: InternalPrompt::from_prompt(prompt),
    }
  }

  pub fn cleanup(&mut self) -> Result<()> {
    self.prompt.cleanup()
  }

  internal_prompt_responder_mixin!(prompt);
}

macro_rules! serialize_to_str {
  ($tname:ident, $dmethod:ident) => {
    fn $dmethod(self, v: $tname) -> Result<()> {
      self.respond(RequestKind::Datum, stringify!($tname), &v.to_string())?;
      self.end_implicit_scopes()
    }
  };
}

impl<'a, P: PromptResponder> ser::Serializer for &'a mut Serializer<P> {
  type Ok = ();
  type Error = Error;

  type SerializeSeq = Seq<'a, P>;
  type SerializeTuple = Tuple<'a, P>;
  type SerializeTupleStruct = Self;
  type SerializeTupleVariant = Self;
  type SerializeMap = Map<'a, P>;
  type SerializeStruct = Self;
  type SerializeStructVariant = Self;

  serialize_to_str!(bool, serialize_bool);
  serialize_to_str!(u8, serialize_u8);
  serialize_to_str!(u16, serialize_u16);
  serialize_to_str!(u32, serialize_u32);
  serialize_to_str!(u64, serialize_u64);
  serialize_to_str!(u128, serialize_u128);
  serialize_to_str!(i8, serialize_i8);
  serialize_to_str!(i16, serialize_i16);
  serialize_to_str!(i32, serialize_i32);
  serialize_to_str!(i64, serialize_i64);
  serialize_to_str!(i128, serialize_i128);
  serialize_to_str!(f32, serialize_f32);
  serialize_to_str!(f64, serialize_f64);
  serialize_to_str!(char, serialize_char);

  fn serialize_str(self, v: &str) -> Result<()> {
    self.respond(RequestKind::Datum, "string", v)?;
    self.end_implicit_scopes()
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<()> {
    for byte in v {
      self.respond(RequestKind::Question, "Add byte?", "yes")?;
      self.respond(RequestKind::Datum, "u8", &byte.to_string())?;
    }
    self.respond(RequestKind::Question, "Add byte?", "no")
  }

  fn serialize_none(self) -> Result<()> {
    self.begin_scope("option", None, ScopeLimit::Explicit)?;
    self.respond(RequestKind::Question, "Some value?", "no")?;
    self.end_scope()
  }

  fn serialize_some<T>(self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.begin_scope("option", None, ScopeLimit::Implicit)?;
    self.respond(RequestKind::Question, "Some value?", "yes")?;
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<()> {
    self.respond(RequestKind::Synthetic, "unit", "()")?;
    self.end_implicit_scopes()
  }

  fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
    self.begin_scope(name, Some(1), ScopeLimit::Explicit)?;
    self.respond(RequestKind::Synthetic, "unit", "()")?;
    self.end_scope()
  }

  fn serialize_unit_variant(
    self,
    name: &'static str,
    _variant_index: u32,
    variant: &'static str,
  ) -> Result<()> {
    self.begin_scope(name, None, ScopeLimit::Explicit)?;
    self.respond(RequestKind::Datum, "variant", variant)?;
    self.end_scope()
  }

  fn serialize_newtype_struct<T>(
    self,
    name: &'static str,
    value: &T,
  ) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.begin_scope(name, Some(1), ScopeLimit::Implicit)?;
    value.serialize(self)
  }

  fn serialize_newtype_variant<T>(
    self,
    name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.begin_scope(name, Some(1), ScopeLimit::Implicit)?;
    self.respond(RequestKind::Datum, "variant", variant)?;
    value.serialize(self)
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
    self.begin_scope("seq", len, ScopeLimit::Explicit)?;
    Ok(Seq::new(self))
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
    self.begin_scope("tuple", Some(len), ScopeLimit::Explicit)?;
    Ok(Tuple::new(self, len))
  }

  fn serialize_tuple_struct(
    self,
    name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleStruct> {
    self.begin_scope(name, Some(len), ScopeLimit::Explicit)?;
    Ok(self)
  }

  fn serialize_tuple_variant(
    self,
    name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeTupleVariant> {
    self.begin_scope(name, None, ScopeLimit::Explicit)?;
    self.respond(RequestKind::Datum, "variant", variant)?;
    Ok(self)
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
    self.begin_scope("map", None, ScopeLimit::Explicit)?;
    Ok(Map::new(self))
  }

  fn serialize_struct(
    self,
    name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStruct> {
    self.begin_scope(name, Some(len), ScopeLimit::Explicit)?;
    Ok(self)
  }

  fn serialize_struct_variant(
    self,
    name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeStructVariant> {
    self.begin_scope(name, None, ScopeLimit::Explicit)?;
    self.respond(RequestKind::Datum, "variant", variant)?;
    Ok(self)
  }
}

#[doc(hidden)]
pub struct Seq<'a, P: PromptResponder> {
  ser: &'a mut Serializer<P>,
  index: usize,
}

impl<'a, P: PromptResponder> Seq<'a, P> {
  fn new(ser: &'a mut Serializer<P>) -> Self {
    Seq { ser, index: 0 }
  }
}

impl<'a, P: PromptResponder> ser::SerializeSeq for Seq<'a, P> {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.ser.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Explicit,
    )?;
    self.index += 1;
    self
      .ser
      .respond(RequestKind::Question, "Add element?", "yes")?;
    value.serialize(&mut *self.ser)?;
    self.ser.end_scope()
  }

  fn end(self) -> Result<()> {
    self.ser.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Explicit,
    )?;
    self
      .ser
      .respond(RequestKind::Question, "Add element?", "no")?;
    self.ser.end_scope()?;
    self.ser.end_scope()
  }
}

#[doc(hidden)]
pub struct Tuple<'a, P: PromptResponder> {
  ser: &'a mut Serializer<P>,
  index: usize,
  len: usize,
}

impl<'a, P: PromptResponder> Tuple<'a, P> {
  fn new(ser: &'a mut Serializer<P>, len: usize) -> Self {
    Tuple { ser, index: 0, len }
  }
}

impl<'a, P: PromptResponder> ser::SerializeTuple for Tuple<'a, P> {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.ser.begin_scope(
      &format!("[{}/{}]", self.index + 1, self.len),
      None,
      ScopeLimit::Explicit,
    )?;
    self.index += 1;
    value.serialize(&mut *self.ser)?;
    self.ser.end_scope()
  }

  fn end(self) -> Result<()> {
    self.ser.end_scope()
  }
}

impl<'a, P: PromptResponder> ser::SerializeTupleStruct
  for &'a mut Serializer<P>
{
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    self.end_scope()
  }
}

impl<'a, P: PromptResponder> ser::SerializeTupleVariant
  for &'a mut Serializer<P>
{
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    self.end_scope()
  }
}

#[doc(hidden)]
pub struct Map<'a, P: PromptResponder> {
  ser: &'a mut Serializer<P>,
  index: usize,
}

impl<'a, P: PromptResponder> Map<'a, P> {
  fn new(ser: &'a mut Serializer<P>) -> Self {
    Map { ser, index: 0 }
  }
}

impl<'a, P: PromptResponder> ser::SerializeMap for Map<'a, P> {
  type Ok = ();
  type Error = Error;

  fn serialize_key<T>(&mut self, key: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.ser.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Explicit,
    )?;
    self.index += 1;
    self
      .ser
      .prompt
      .respond(RequestKind::Question, "Add entry?", "yes")?;
    key.serialize(&mut *self.ser)
  }

  fn serialize_value<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut *self.ser)?;
    self.ser.end_scope()
  }

  fn end(self) -> Result<()> {
    self.ser.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Explicit,
    )?;
    self
      .ser
      .respond(RequestKind::Question, "Add entry?", "no")?;
    self.ser.end_scope()?;
    self.ser.end_scope()
  }
}

impl<'a, P: PromptResponder> ser::SerializeStruct for &'a mut Serializer<P> {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.begin_scope(key, Some(1), ScopeLimit::Explicit)?;
    value.serialize(&mut **self)?;
    self.end_scope()
  }

  fn end(self) -> Result<()> {
    self.end_scope()
  }
}

impl<'a, P: PromptResponder> ser::SerializeStructVariant
  for &'a mut Serializer<P>
{
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    self.begin_scope(key, Some(1), ScopeLimit::Explicit)?;
    value.serialize(&mut **self)?;
    self.end_scope()
  }

  fn end(self) -> Result<()> {
    self.end_scope()
  }
}
