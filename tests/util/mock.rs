use spaniel_id::prompt::*;
use spaniel_id::{Error, Result};
use std::iter::ExactSizeIterator;

pub struct MockPrompt<I: ExactSizeIterator<Item = &'static str>> {
  responses: I,
  interactive: bool,
  level: usize,
}

impl<I: ExactSizeIterator<Item = &'static str>> MockPrompt<I> {
  pub fn new(responses: I) -> MockPrompt<I> {
    MockPrompt {
      responses,
      interactive: false,
      level: 0,
    }
  }

  pub fn with_interactive(mut self) -> Self {
    self.interactive = true;
    self
  }
}

impl<I: ExactSizeIterator<Item = &'static str>> Drop for MockPrompt<I> {
  fn drop(&mut self) {
    assert_eq!(self.responses.len(), 0);
    assert_eq!(self.level, 0);
  }
}

impl<I: ExactSizeIterator<Item = &'static str>> PromptResponder
  for MockPrompt<I>
{
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    self.level += 1;
    println!(
      "begin_scope({:?}, {:?}) enters level {:?}",
      name, size, self.level
    );
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    println!("end_scope() exits level {:?}", self.level);
    self.level -= 1;
    Ok(())
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    println!("response({:?}, {:?}, {:?})", kind, prompt, response);
    Ok(())
  }
}

impl<I: ExactSizeIterator<Item = &'static str>> PromptRequester
  for MockPrompt<I>
{
  fn is_interactive(&self) -> bool {
    self.interactive
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &[&str],
  ) -> Result<String> {
    match self.responses.next() {
      Some(s) => {
        println!(
          "request({:?}, {:?}, {:?}) returns {:?}",
          kind, prompt, variants, s
        );
        Ok(s.to_string())
      }
      None => {
        println!("request({:?}, {:?}, {:?}) failed", kind, prompt, variants);
        Err(Error::IOError("Out of responses".to_string()))
      }
    }
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    println!("report({:?}, {:?})", kind, msg);
    Ok(())
  }
}
