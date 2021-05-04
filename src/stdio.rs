use crate::error::{Error, Result};
use crate::prompt::{
  PromptRequester, PromptResponder, ReportKind, RequestKind,
};
use std::io::prelude::*;
use std::io::{stdin, stdout, Stdin, Stdout};

/// Prompt which reads and writes via `std::io` traits.
pub struct ReadWritePrompt<R, W> {
  read: R,
  write: W,
  is_interactive: bool,
  level: usize,
}

impl<'a> ReadWritePrompt<Stdin, Stdout> {
  pub fn new_stdio() -> Self {
    ReadWritePrompt {
      read: stdin(),
      write: stdout(),
      is_interactive: true,
      level: 0,
    }
  }
}

impl<R: BufRead, W: Write> ReadWritePrompt<R, W> {
  pub fn new_requester(read: R, write: W, is_interactive: bool) -> Self {
    ReadWritePrompt {
      read,
      write,
      is_interactive,
      level: 0,
    }
  }
}

impl<W: Write> ReadWritePrompt<(), W> {
  pub fn new_responder(write: W) -> Self {
    ReadWritePrompt {
      read: (),
      write,
      is_interactive: false,
      level: 0,
    }
  }
}

impl<R, W> ReadWritePrompt<R, W> {
  fn spaces(&self) -> usize {
    2 * self.level
  }
}

fn lift_result<T, E: std::error::Error>(
  value: std::result::Result<T, E>,
) -> Result<T> {
  match value {
    Ok(v) => Ok(v),
    Err(e) => Err(Error::IoError(e.to_string())),
  }
}

impl<'a, R, W: Write> PromptResponder for ReadWritePrompt<R, W> {
  fn begin_scope(&mut self, name: &str, _size: Option<usize>) -> Result<()> {
    lift_result(writeln!(
      self.write,
      "{:indent$}{} {{",
      "",
      name,
      indent = self.spaces()
    ))?;
    self.level += 1;
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    self.level -= 1;
    lift_result(writeln!(
      self.write,
      "{:indent$}}}",
      "",
      indent = self.spaces()
    ))?;
    Ok(())
  }

  fn respond(
    &mut self,
    _kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    lift_result(writeln!(
      self.write,
      "{:indent$}{}: {}",
      "",
      prompt,
      response,
      indent = self.spaces()
    ))?;
    Ok(())
  }
}

impl<'a, R: BufRead, W: Write> PromptRequester for ReadWritePrompt<R, W> {
  fn is_interactive(&self) -> bool {
    self.is_interactive
  }

  fn request(
    &mut self,
    _kind: RequestKind,
    prompt: &str,
    _variants: &[&str],
  ) -> Result<String> {
    lift_result(write!(
      self.write,
      "{:indent$}{}: ",
      "",
      prompt,
      indent = self.spaces()
    ))?;
    lift_result(self.write.flush())?;
    let mut line = String::new();
    lift_result(self.read.read_line(&mut line))?;
    if line.ends_with('\n') {
      line.pop();
      if line.ends_with('\r') {
        line.pop();
      }
    }
    Ok(line)
  }

  fn report(&mut self, _kind: ReportKind, msg: &str) -> Result<()> {
    lift_result(writeln!(
      self.write,
      "{:indent$}{}",
      "",
      msg,
      indent = self.spaces()
    ))?;
    Ok(())
  }
}
