use std::str::FromStr;

use serde::de::{
  self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess,
  SeqAccess, VariantAccess, Visitor,
};

use crate::error::{Error, Result};
use crate::prompt::{PromptRequester, ReportKind, RequestKind};
use crate::u8i8;
use crate::util;

#[derive(PartialEq, Eq)]
enum ScopeLimit {
  Explicit,
  Implicit,
}

struct ScopeEntry(ScopeLimit, u32);

pub struct Deserializer<P: PromptRequester> {
  prompt: P,
  scopes: Vec<ScopeEntry>,
}

impl<P: PromptRequester> Deserializer<P> {
  pub fn from_prompt(prompt: P) -> Self {
    Deserializer {
      prompt,
      scopes: Vec::new(),
    }
  }

  pub fn cleanup(&mut self) -> Result<()> {
    while let Some(ScopeEntry(_, n)) = self.scopes.last_mut() {
      *n -= 1;
      if *n == 0 {
        self.scopes.pop();
      }
      self.prompt.end_scope()?;
    }
    Ok(())
  }

  fn begin_scope(
    &mut self,
    name: &str,
    size: Option<usize>,
    limit: ScopeLimit,
  ) -> Result<()> {
    self.prompt.begin_scope(name, size)?;
    match self.scopes.last_mut() {
      Some(ScopeEntry(lim, n)) if limit == *lim => *n += 1,
      _ => self.scopes.push(ScopeEntry(limit, 1)),
    }
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    self.prompt.end_scope()?;
    match self.scopes.last_mut() {
      Some(ScopeEntry(ScopeLimit::Explicit, n)) => {
        *n -= 1;
        if *n == 0 {
          self.scopes.pop();
        }
      }
      _ => unreachable!(),
    };
    self.end_implicit_scopes()?;
    Ok(())
  }

  fn end_implicit_scopes(&mut self) -> Result<()> {
    while let Some(ScopeEntry(ScopeLimit::Implicit, n)) = self.scopes.last_mut()
    {
      *n -= 1;
      if *n == 0 {
        self.scopes.pop();
      }
      self.prompt.end_scope()?;
    }
    Ok(())
  }

  fn is_interactive(&self) -> bool {
    self.prompt.is_interactive()
  }

  fn request(
    &mut self,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    self.prompt.request(RequestKind::Datum, prompt, variants)
  }

  fn respond(&mut self, prompt: &str, response: &str) -> Result<()> {
    self
      .prompt
      .respond(RequestKind::Synthetic, prompt, response)
  }

  fn report_bad_response(&mut self, msg: &str) -> Result<()> {
    self.prompt.report(ReportKind::BadResponse, msg)
  }

  fn ask_yes_no(&mut self, prompt: &str) -> Result<bool> {
    util::ask_yes_no(&mut self.prompt, prompt)
  }
}

macro_rules! deserialize_from_str {
  ($tname:ident, $dmethod:ident, $vmethod:ident) => {
    deserialize_from_str!($tname, $dmethod, $vmethod, []);
  };
  ($tname:ident, $dmethod:ident, $vmethod:ident, $variants:expr) => {
    fn $dmethod<V>(self, visitor: V) -> Result<V::Value>
    where
      V: de::Visitor<'de>,
    {
      loop {
        let s = self.request(stringify!($tname), &$variants)?;
        match $tname::from_str(&s) {
          Ok(v) => {
            self.end_implicit_scopes()?;
            return visitor.$vmethod(v);
          }
          Err(e) => {
            self.report_bad_response(&format!("Failed to parse: {}", e))?;
            if !self.is_interactive() {
              return Err(Error::BadResponse);
            }
          }
        }
      }
    }
  };
}

impl<'de, 'a, P: PromptRequester> de::Deserializer<'de>
  for &'a mut Deserializer<P>
{
  type Error = Error;

  fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    unimplemented!()
  }

  deserialize_from_str!(bool, deserialize_bool, visit_bool, ["true", "false"]);
  deserialize_from_str!(u8, deserialize_u8, visit_u8, u8i8::U8_VARIANTS);
  deserialize_from_str!(u16, deserialize_u16, visit_u16);
  deserialize_from_str!(u32, deserialize_u32, visit_u32);
  deserialize_from_str!(u64, deserialize_u64, visit_u64);
  deserialize_from_str!(u128, deserialize_u128, visit_u128);
  deserialize_from_str!(i8, deserialize_i8, visit_i8, u8i8::I8_VARIANTS);
  deserialize_from_str!(i16, deserialize_i16, visit_i16);
  deserialize_from_str!(i32, deserialize_i32, visit_i32);
  deserialize_from_str!(i64, deserialize_i64, visit_i64);
  deserialize_from_str!(i128, deserialize_i128, visit_i128);
  deserialize_from_str!(f32, deserialize_f32, visit_f32);
  deserialize_from_str!(f64, deserialize_f64, visit_f64);
  deserialize_from_str!(char, deserialize_char, visit_char);

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_string(visitor)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let s = self.request("string", &[])?;
    self.end_implicit_scopes()?;
    visitor.visit_string(s)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_byte_buf(visitor)
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let mut buf = Vec::<u8>::new();
    while self.ask_yes_no("Add byte?")? {
      loop {
        let s = self.request("u8", u8i8::U8_VARIANTS)?;
        match u8::from_str(&s) {
          Ok(v) => {
            buf.push(v);
            break;
          }
          Err(_e) => {
            self.report_bad_response("Failed to parse")?;
            if !self.is_interactive() {
              return Err(Error::BadResponse);
            }
          }
        }
      }
    }
    visitor.visit_byte_buf(buf)
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope("option", None, ScopeLimit::Implicit)?;
    if self.ask_yes_no("Some value?")? {
      visitor.visit_some(self)
    } else {
      self.end_implicit_scopes()?;
      visitor.visit_none()
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.respond("unit", "()")?;
    self.end_implicit_scopes()?;
    visitor.visit_unit()
  }

  fn deserialize_unit_struct<V>(
    self,
    name: &'static str,
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope(name, Some(1), ScopeLimit::Explicit)?;
    self.respond("unit", "()")?;
    self.end_scope()?;
    visitor.visit_unit()
  }

  fn deserialize_newtype_struct<V>(
    self,
    name: &'static str,
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope(name, Some(1), ScopeLimit::Implicit)?;
    visitor.visit_newtype_struct(self)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope("seq", None, ScopeLimit::Explicit)?;
    let res = visitor.visit_seq(Seq::new(self))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope("tuple", Some(len), ScopeLimit::Explicit)?;
    let res = visitor.visit_seq(Tuple::new(self, len))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_tuple_struct<V>(
    self,
    name: &'static str,
    len: usize,
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope(name, Some(len), ScopeLimit::Explicit)?;
    let res = visitor.visit_seq(Tuple::new(self, len))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope("map", None, ScopeLimit::Explicit)?;
    let res = visitor.visit_map(Map::new(self))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_struct<V>(
    self,
    name: &'static str,
    fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope(name, Some(fields.len()), ScopeLimit::Explicit)?;
    let res = visitor.visit_map(Struct::new(self, fields))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_enum<V>(
    self,
    name: &'static str,
    variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.begin_scope(name, None, ScopeLimit::Explicit)?;
    let res = visitor.visit_enum(Enum::new(self, variants))?;
    self.end_scope()?;
    Ok(res)
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let s = self.request("identifier", &[])?;
    self.end_implicit_scopes()?;
    visitor.visit_string(s)
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.respond("any", "IGNORED")?;
    self.end_implicit_scopes()?;
    visitor.visit_unit()
  }
}

impl<P: PromptRequester> Drop for Deserializer<P> {
  fn drop(&mut self) {
    let _ = self.cleanup();
  }
}

struct Enum<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  variants: &'static [&'static str],
}

impl<'a, P: PromptRequester> Enum<'a, P> {
  fn new(
    de: &'a mut Deserializer<P>,
    variants: &'static [&'static str],
  ) -> Self {
    Enum { de, variants }
  }
}

impl<'de: 'a, 'a, P: PromptRequester> EnumAccess<'de> for Enum<'a, P> {
  type Error = Error;
  type Variant = EnumVariant<'a, P>;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
  where
    V: DeserializeSeed<'de>,
  {
    if self.variants.len() == 1 {
      let v = self.variants[0];
      self.de.respond("variant", v)?;
      let val = seed.deserialize(v.into_deserializer())?;
      return Ok((val, EnumVariant::new(self.de, v)));
    }
    loop {
      let s = self.de.request("variant", self.variants)?;
      match self.variants.iter().find(|v| **v == s) {
        Some(v) => {
          let val = seed.deserialize(v.into_deserializer())?;
          return Ok((val, EnumVariant::new(self.de, v)));
        }
        None => {
          self
            .de
            .report_bad_response(&format!("Invalid variant: '{}'", s))?;
          if !self.de.is_interactive() {
            return Err(Error::BadResponse);
          }
        }
      }
    }
  }
}

struct EnumVariant<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  variant: &'static str,
}

impl<'a, P: PromptRequester> EnumVariant<'a, P> {
  fn new(de: &'a mut Deserializer<P>, variant: &'static str) -> Self {
    EnumVariant { de, variant }
  }
}

impl<'de: 'a, 'a, P: PromptRequester> VariantAccess<'de>
  for EnumVariant<'a, P>
{
  type Error = Error;

  fn unit_variant(self) -> Result<()> {
    Deserialize::deserialize(self.de)
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
  where
    T: DeserializeSeed<'de>,
  {
    seed.deserialize(self.de)
  }

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    de::Deserializer::deserialize_tuple(self.de, len, visitor)
  }

  fn struct_variant<V>(
    self,
    fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    de::Deserializer::deserialize_struct(self.de, self.variant, fields, visitor)
  }
}

struct Struct<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  fields: std::slice::Iter<'static, &'static str>,
}

impl<'a, P: PromptRequester> Struct<'a, P> {
  fn new(de: &'a mut Deserializer<P>, fields: &'static [&'static str]) -> Self {
    Struct {
      de,
      fields: fields.iter(),
    }
  }
}

impl<'de: 'a, 'a, P: PromptRequester> MapAccess<'de> for Struct<'a, P> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    match self.fields.next() {
      Some(fld) => {
        self.de.begin_scope(fld, Some(1), ScopeLimit::Implicit)?;
        seed.deserialize(fld.into_deserializer()).map(Some)
      }
      None => Ok(None),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    seed.deserialize(&mut *self.de)
  }
}

struct Seq<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  index: usize,
}

impl<'a, P: PromptRequester> Seq<'a, P> {
  fn new(de: &'a mut Deserializer<P>) -> Self {
    Seq { de, index: 0 }
  }
}

impl<'de: 'a, 'a, P: PromptRequester> SeqAccess<'de> for Seq<'a, P> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    self.de.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Implicit,
    )?;
    self.index += 1;
    if self.de.ask_yes_no("Add element?")? {
      seed.deserialize(&mut *self.de).map(Some)
    } else {
      self.de.end_implicit_scopes()?;
      Ok(None)
    }
  }
}

struct Tuple<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  index: usize,
  len: usize,
}

impl<'a, P: PromptRequester> Tuple<'a, P> {
  fn new(de: &'a mut Deserializer<P>, len: usize) -> Self {
    Tuple { de, index: 0, len }
  }
}

impl<'de, 'a, P: PromptRequester> SeqAccess<'de> for Tuple<'a, P> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    self.de.begin_scope(
      &format!("[{}..{}]", self.index, self.len),
      None,
      ScopeLimit::Implicit,
    )?;
    if self.index < self.len {
      self.index += 1;
      seed.deserialize(&mut *self.de).map(Some)
    } else {
      self.de.end_implicit_scopes()?;
      Ok(None)
    }
  }
}

struct Map<'a, P: PromptRequester> {
  de: &'a mut Deserializer<P>,
  index: usize,
}

impl<'a, P: PromptRequester> Map<'a, P> {
  fn new(de: &'a mut Deserializer<P>) -> Self {
    Map { de, index: 0 }
  }
}

impl<'de: 'a, 'a, P: PromptRequester> MapAccess<'de> for Map<'a, P> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    self.de.begin_scope(
      &format!("[{}]", self.index),
      None,
      ScopeLimit::Explicit,
    )?;
    self.index += 1;
    if self.de.ask_yes_no("Add entry?")? {
      seed.deserialize(&mut *self.de).map(Some)
    } else {
      self.de.end_scope()?;
      Ok(None)
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    let res = seed.deserialize(&mut *self.de)?;
    self.de.end_scope()?;
    Ok(res)
  }
}
