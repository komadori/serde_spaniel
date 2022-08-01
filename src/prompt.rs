use crate::error::{Error, Result, UserAction};
use std::str::FromStr;

/// Represents the kind of a prompt request or response.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RequestKind {
  /// A value which makes up the type being deserialised.
  Datum,
  /// A question related to the type being deserialised.
  Question,
  /// The deserialiser already knows the value.
  Synthetic,
}

/// Represents the kind of a message reported to a prompt.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReportKind {
  /// The previous response was not accepted.
  BadResponse,
  /// The associated text is informative only.
  Help,
}

/// Traits for prompts which can display output.
pub trait PromptResponder {
  /// Begins a new named scope.
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()>;
  /// Ends the current scope.
  fn end_scope(&mut self) -> Result<()>;
  /// Sends a response to the prompt.
  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()>;
}

/// Trait for prompts which can obtain input.
pub trait PromptRequester: PromptResponder {
  /// Returns true if the prompt is currently interactive.
  fn is_interactive(&self) -> bool;
  /// Requests a response given a prompt message and optional variants.
  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String>;
  /// Reports an informative or error message to the prompt.
  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()>;
}

impl<P: PromptResponder> PromptResponder for &mut P {
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    (*self).begin_scope(name, size)
  }

  fn end_scope(&mut self) -> Result<()> {
    (*self).end_scope()
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    (*self).respond(kind, prompt, response)
  }
}

impl<P: PromptRequester> PromptRequester for &mut P {
  fn is_interactive(&self) -> bool {
    (**self).is_interactive()
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    (*self).request(kind, prompt, variants)
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    (*self).report(kind, msg)
  }
}

/// Prompt decorator which allows `UserAction`s to be triggered in-band.
///
/// This prompt intercepts responses which begin with an exclamation mark.
/// Entering the meta-commands `!cancel`, `!undo`, and `!restart` as a response
/// will cause the request to fail with the corresponding `UserAction`. The
/// commands may be abbreviated to single letters. The undo and restart
/// commands may be followed by a number otherwise `Undo(1)` and `Restart(0)`
/// is implied. Responses that actually begin with an exclamation mark can be
/// escaped by doubling the exclamation mark.
pub struct MetaCommandPrompt<P> {
  inner: P,
}

impl<P> MetaCommandPrompt<P> {
  pub fn new(inner: P) -> Self {
    MetaCommandPrompt { inner }
  }
}

impl<P: PromptResponder> PromptResponder for MetaCommandPrompt<P> {
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    self.inner.begin_scope(name, size)
  }

  fn end_scope(&mut self) -> Result<()> {
    self.inner.end_scope()
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    if response.starts_with('!') {
      self.inner.respond(kind, prompt, &["!", response].concat())
    } else {
      self.inner.respond(kind, prompt, response)
    }
  }
}

const HELP_TEXT: &[&str] = &[
  "Spaniel Meta-Command Reference: [optional] <argument>",
  "  !!              - Escape responses beginning with an exclamation mark",
  "  !c[ancel]       - Cancel deserialisation",
  "  !u[ndo][<n>]    - Undo the previous or the last <n> responses",
  "  !r[estart][<n>] - Restart from the beginning or from the <n>th response",
  "  !h[elp]         - This message",
];

impl<P: PromptRequester> PromptRequester for MetaCommandPrompt<P> {
  fn is_interactive(&self) -> bool {
    self.inner.is_interactive()
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    loop {
      let mut s = self.inner.request(kind, prompt, variants)?;
      if !s.starts_with('!') {
        return Ok(s);
      }
      if s.starts_with("!!") {
        s.replace_range(0..1, "");
        return Ok(s);
      }
      let prefix = s.trim_end_matches(|c: char| c.is_ascii_digit());
      let suffix_str = &s[prefix.len()..];
      let suffix = if suffix_str.is_empty() {
        None
      } else {
        Some(usize::from_str(suffix_str).ok())
      };
      match (prefix, suffix) {
        ("!c", None) | ("!cancel", None) => {
          return Err(Error::UserAction(UserAction::Cancel))
        }
        ("!u", None) | ("!undo", None) => {
          return Err(Error::UserAction(UserAction::Undo(1)))
        }
        ("!u", Some(Some(n))) | ("!undo", Some(Some(n))) => {
          return Err(Error::UserAction(UserAction::Undo(n)))
        }
        ("!r", None) | ("!restart", None) => {
          return Err(Error::UserAction(UserAction::Restart(0)))
        }
        ("!r", Some(Some(n))) | ("!restart", Some(Some(n))) => {
          return Err(Error::UserAction(UserAction::Restart(n)))
        }
        ("!h", None) | ("!help", None) => {
          self.report(
            ReportKind::Help,
            &format!("Variants are: {:?}", variants),
          )?;
          for line in HELP_TEXT {
            self.report(ReportKind::Help, line)?;
          }
        }
        _ => {
          self.report(
            ReportKind::BadResponse,
            "Bad user action (try !help for help)",
          )?;
        }
      }
      if !self.is_interactive() {
        return Err(Error::BadResponse);
      }
    }
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    self.inner.report(kind, msg)
  }
}

#[derive(Clone, Debug)]
enum ReplayState {
  Disabled,
  Recording,
  Replaying(std::vec::IntoIter<String>),
}

/// Prompt decorator which logs responses and can replay them.
///
/// This prompt is used to facilitate undo and restart operations while
/// interactively deserialising. The deserialiser itself cannot be stepped
/// backwards once a response has been submitted. It's necessary to start the
/// deserialiser again, but the `ReplayPrompt` can quickly bring it up to the
/// correct point by replaying its log.
pub struct ReplayPrompt<P> {
  inner: P,
  log: Vec<String>,
  state: ReplayState,
}

impl<P> ReplayPrompt<P> {
  pub fn new(inner: P) -> Self {
    ReplayPrompt {
      inner,
      log: Vec::new(),
      state: ReplayState::Disabled,
    }
  }

  /// Clear log and disable recording.
  pub fn reset(&mut self) {
    self.log.clear();
    self.state = ReplayState::Disabled;
  }

  /// Start recording responses.
  pub fn record(&mut self) {
    self.log.clear();
    self.state = ReplayState::Recording;
  }

  /// Replay log and continue recording new responses.
  pub fn replay(&mut self) -> Result<()> {
    if let ReplayState::Recording = self.state {
      let old_log = std::mem::take(&mut self.log);
      self.state = ReplayState::Replaying(old_log.into_iter());
      Ok(())
    } else {
      Err(Error::CannotReplay)
    }
  }

  /// Remove the last n responses from the log.
  pub fn undo(&mut self, n: usize) {
    let len = self.log.len();
    self.log.truncate(len - std::cmp::min(len, n));
  }

  /// Truncate the log to the first nth responses in the log.
  pub fn restart_from(&mut self, n: usize) {
    self.log.truncate(n);
  }
}

impl<P: PromptResponder> PromptResponder for ReplayPrompt<P> {
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    self.inner.begin_scope(name, size)
  }

  fn end_scope(&mut self) -> Result<()> {
    self.inner.end_scope()
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    if kind != RequestKind::Synthetic {
      if let ReplayState::Recording = self.state {
        self.log.push(response.to_string());
      };
    };
    self.inner.respond(kind, prompt, response)
  }
}

impl<P: PromptRequester> PromptRequester for ReplayPrompt<P> {
  fn is_interactive(&self) -> bool {
    if let ReplayState::Replaying(_) = self.state {
      false
    } else {
      self.inner.is_interactive()
    }
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    if let ReplayState::Replaying(iter) = &mut self.state {
      if let Some(res) = iter.next() {
        self.log.push(res.clone());
        self.inner.respond(kind, prompt, &res)?;
        return Ok(res);
      };
      self.state = ReplayState::Recording;
    }

    let res = self.inner.request(kind, prompt, variants)?;
    if let ReplayState::Recording = self.state {
      self.log.push(res.clone());
    }
    Ok(res)
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    if kind == ReportKind::BadResponse {
      self.log.pop();
    }
    self.inner.report(kind, msg)
  }
}

struct CompactScope {
  name: String,
  compact: bool,
}

/// Prompt decorator which reduces the verbosity of output.
pub struct CompactPrompt<P> {
  inner: P,
  scopes: Vec<CompactScope>,
}

impl<P> CompactPrompt<P> {
  pub fn new(inner: P) -> Self {
    CompactPrompt {
      inner,
      scopes: Vec::new(),
    }
  }

  fn get_compacted(&self) -> &[CompactScope] {
    let mut i = self.scopes.len();
    for x in self.scopes.iter().rev() {
      if !x.compact {
        break;
      }
      i -= 1;
    }
    self.scopes.split_at(i).1
  }

  fn compound_name(&self, name: &str) -> String {
    let compacted = self.get_compacted();
    let mut names = Vec::<&str>::with_capacity(compacted.len() + 1);
    for scope in compacted {
      names.push(&scope.name);
    }
    names.push(name);
    names.join(" -> ")
  }
}

impl<P: PromptResponder> PromptResponder for CompactPrompt<P> {
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    let compact = size == Some(1);
    if !compact {
      self.inner.begin_scope(&self.compound_name(name), size)?;
    }
    self.scopes.push(CompactScope {
      name: name.to_string(),
      compact,
    });
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    if !self.scopes.pop().unwrap().compact {
      self.inner.end_scope()?;
    }
    Ok(())
  }

  fn respond(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    if kind != RequestKind::Synthetic {
      self
        .inner
        .respond(kind, &self.compound_name(prompt), response)?;
    }
    Ok(())
  }
}

impl<P: PromptRequester> PromptRequester for CompactPrompt<P> {
  fn is_interactive(&self) -> bool {
    self.inner.is_interactive()
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    self
      .inner
      .request(kind, &self.compound_name(prompt), variants)
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    self.inner.report(kind, msg)
  }
}
