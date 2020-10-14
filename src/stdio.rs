use crate::error::{Error, Result};
use crate::prompt::{
  PromptRequester, PromptResponder, ReportKind, RequestKind,
};
use std::fmt::Arguments;
use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout, Stdin, Stdout};

/// Prompt which reads and writes via `ReadLine` and `WriteLine` traits.
pub struct ReadWritePrompt<R, W> {
  read: R,
  write: W,
  is_interactive: bool,
  level: usize,
}

/// Wrapper which implements `ReadLine` and `WriteLine` over `BufRead` and
/// `Write`.
pub struct StdLine<T>(T);

/// Trait for reading lines.
pub trait ReadLine {
  fn read_line(&mut self, buf: &mut String) -> io::Result<usize>;
}

impl<T: BufRead> ReadLine for StdLine<T> {
  fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
    self.0.read_line(buf)
  }
}

impl ReadLine for Stdin {
  fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
    self.lock().read_line(buf)
  }
}

/// Trait for writing lines.
pub trait WriteLine {
  fn write_fmt(&mut self, fmt: Arguments<'_>) -> io::Result<()>;
  fn flush(&mut self) -> io::Result<()>;
}

impl<T: Write> WriteLine for StdLine<T> {
  fn write_fmt(&mut self, fmt: Arguments<'_>) -> io::Result<()> {
    self.0.write_fmt(fmt)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.0.flush()
  }
}

impl WriteLine for Stdout {
  fn write_fmt(&mut self, fmt: Arguments<'_>) -> io::Result<()> {
    self.lock().write_fmt(fmt)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.lock().flush()
  }
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

impl<R: BufRead, W: Write> ReadWritePrompt<StdLine<R>, StdLine<W>> {
  pub fn new_requester(read: R, write: W, is_interactive: bool) -> Self {
    ReadWritePrompt {
      read: StdLine(read),
      write: StdLine(write),
      is_interactive,
      level: 0,
    }
  }
}

impl<W: Write> ReadWritePrompt<(), StdLine<W>> {
  pub fn new_responder(write: W) -> Self {
    ReadWritePrompt {
      read: (),
      write: StdLine(write),
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
    Err(e) => Err(Error::IOError(e.to_string())),
  }
}

impl<'a, R, W: WriteLine> PromptResponder for ReadWritePrompt<R, W> {
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

impl<'a, R: ReadLine, W: WriteLine> PromptRequester for ReadWritePrompt<R, W> {
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
