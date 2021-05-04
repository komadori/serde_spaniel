use serde_spaniel::prompt::*;
use serde_spaniel::{Error, Result};
use std::iter::ExactSizeIterator;
use std::mem;

#[derive(Debug, PartialEq)]
pub enum LogEntry {
  BeginScope(String, Option<usize>),
  EndScope,
  Response(RequestKind, String, &'static [&'static str], String),
  Report(ReportKind, String),
}

pub struct MockPrompt<I: ExactSizeIterator<Item = &'static str>> {
  responses: I,
  log: Vec<LogEntry>,
  interactive: bool,
  level: usize,
}

impl<I: ExactSizeIterator<Item = &'static str>> MockPrompt<I> {
  pub fn new(responses: I) -> MockPrompt<I> {
    MockPrompt {
      responses,
      log: Vec::new(),
      interactive: false,
      level: 0,
    }
  }

  pub fn with_interactive(mut self) -> Self {
    self.interactive = true;
    self
  }

  pub fn into_log(mut self) -> Vec<LogEntry> {
    mem::replace(&mut self.log, Vec::new())
  }

  pub fn responses(&self) -> Vec<String> {
    let mut rs = Vec::new();
    for entry in self.log.iter() {
      match entry {
        LogEntry::Response(RequestKind::Synthetic, _, _, _) => {}
        LogEntry::Response(_, _, _, str) => rs.push(str.clone()),
        _ => {}
      }
    }
    rs
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
    self.log.push(LogEntry::BeginScope(name.to_string(), size));
    println!(
      "begin_scope({:?}, {:?}) enters level {:?}",
      name, size, self.level
    );
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    println!("end_scope() exits level {:?}", self.level);
    self.level -= 1;
    self.log.push(LogEntry::EndScope);
    Ok(())
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    println!("response({:?}, {:?}, {:?})", kind, prompt, response);
    self.log.push(LogEntry::Response(
      kind,
      prompt.to_string(),
      &[],
      response.to_string(),
    ));
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
    variants: &'static [&'static str],
  ) -> Result<String> {
    match self.responses.next() {
      Some(s) => {
        println!(
          "request({:?}, {:?}, {:?}) returns {:?}",
          kind, prompt, variants, s
        );
        self.log.push(LogEntry::Response(
          kind,
          prompt.to_string(),
          variants,
          s.to_string(),
        ));
        Ok(s.to_string())
      }
      None => {
        println!("request({:?}, {:?}, {:?}) failed", kind, prompt, variants);
        Err(Error::IoError("Out of responses".to_string()))
      }
    }
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    println!("report({:?}, {:?})", kind, msg);
    self.log.push(LogEntry::Report(kind, msg.to_string()));
    Ok(())
  }
}
