use crate::error::{Error, Result};
use crate::prompt::{
  PromptRequester, PromptResponder, ReportKind, RequestKind,
};
use std::io::prelude::*;
use std::io::{stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};

/// Prompt which reads and writes via `std::io` traits.
pub struct ReadWritePrompt<R, W, Owned = ()> {
  read: R,
  write: W,
  _owned: Owned,
  is_interactive: bool,
  level: usize,
}

impl<'a> ReadWritePrompt<StdinLock<'a>, StdoutLock<'a>, (Stdin, Stdout)> {
  pub fn new_stdio() -> Self {
    let stdin = stdin();
    let stdout = stdout();
    ReadWritePrompt {
      // It's safe to move stdin and stdout while locked because they are Arcs.
      read: unsafe { std::mem::transmute(stdin.lock()) },
      write: unsafe { std::mem::transmute(stdout.lock()) },
      _owned: (stdin, stdout),
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
      _owned: (),
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
      _owned: (),
      is_interactive: false,
      level: 0,
    }
  }
}

impl<R, W, Owned> ReadWritePrompt<R, W, Owned> {
  fn spaces(&self) -> usize {
    2 * self.level
  }
}

fn lift_result<T, E: std::error::Error>(
  value: std::result::Result<T, E>,
) -> Result<T> {
  match value {
    Ok(v) => Ok(v),
    Err(e) => Err(Error::IOError(e.to_string())),
  }
}

impl<R, W: Write, Owned> PromptResponder for ReadWritePrompt<R, W, Owned> {
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

impl<R: BufRead, W: Write, Owned> PromptRequester
  for ReadWritePrompt<R, W, Owned>
{
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
