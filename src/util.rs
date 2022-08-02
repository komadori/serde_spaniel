use crate::de::Deserializer;
use crate::error::{Error, Result, UserAction};
use crate::prompt::{
  CompactPrompt, MetaCommandPrompt, PromptRequester, PromptResponder,
  ReplayPrompt, ReportKind, RequestKind,
};
use crate::ser::Serializer;
use serde::{Deserialize, Serialize};

/// Deserialise a value of type `T` from a prompt using the bare deserialiser.
pub fn from_bare_prompt<'de, T: Deserialize<'de>, P: PromptRequester>(
  prompt: P,
) -> Result<T> {
  Deserialize::deserialize(&mut Deserializer::from_prompt(prompt))
}

pub(crate) fn ask_yes_no<P: PromptRequester>(
  p: &mut P,
  prompt: &str,
) -> Result<bool> {
  loop {
    let s = p.request(RequestKind::Question, prompt, &["yes", "no"])?;
    match s.to_lowercase().as_ref() {
      "y" | "yes" => return Ok(true),
      "n" | "no" => return Ok(false),
      _ => {
        p.report(ReportKind::BadResponse, "Must answer yes or no")?;
      }
    }
    if !p.is_interactive() {
      return Err(Error::BadResponse);
    }
  }
}

/// Deserialise a value of type `T` from a prompt using the bare deserialiser
/// followed by confirmation.
pub fn from_bare_prompt_confirm<
  'de,
  T: Deserialize<'de>,
  P: PromptRequester,
>(
  mut prompt: P,
) -> Result<T> {
  let res = from_bare_prompt(&mut prompt)?;
  if ask_yes_no(&mut prompt, "Accept value?")? {
    Ok(res)
  } else {
    Err(Error::UserAction(UserAction::Restart(0)))
  }
}

/// Deserialise a value of type `T` from a prompt while handling undos and
/// restarts.
pub fn from_replay_prompt<'de, T: Deserialize<'de>, P: PromptRequester>(
  prompt: P,
) -> Result<T> {
  let mut replay = ReplayPrompt::new(prompt);
  replay.record();
  loop {
    match from_bare_prompt_confirm(&mut replay) {
      Ok(s) => return Ok(s),
      Err(Error::SerdeError(msg)) if replay.is_interactive() => {
        // Assume Serde error is caused by malformed input and undo 1 step
        replay
          .report(ReportKind::BadResponse, &format!("Serde Error: {}", msg))?;
        replay.undo(1);
        replay.replay()?;
      }
      Err(Error::UserAction(UserAction::Undo(n)))
        if replay.is_interactive() =>
      {
        replay.undo(n);
        replay.replay()?;
      }
      Err(Error::UserAction(UserAction::Restart(n)))
        if replay.is_interactive() =>
      {
        replay.restart_from(n);
        replay.replay()?;
      }
      Err(e) => return Err(e),
    }
  }
}

/// Deserialise a value of type `T` from a prompt while handling undos,
/// restarts, meta-commands, and scope compacting.
pub fn from_prompt<'de, T: Deserialize<'de>, P: PromptRequester>(
  prompt: P,
) -> Result<T> {
  from_replay_prompt(MetaCommandPrompt::new(CompactPrompt::new(prompt)))
}

/// Deserialise an instance of type `T` from the console.
pub fn from_console<'de, T: Deserialize<'de>>() -> Result<T> {
  #[cfg(feature = "rustyline")]
  {
    from_prompt(crate::rustyline::RustyLinePrompt::new()?)
  }

  #[cfg(all(not(feature = "rustyline"), feature = "stdio"))]
  {
    from_prompt(crate::stdio::ReadWritePrompt::new_stdio())
  }

  #[cfg(all(not(feature = "rustyline"), not(feature = "stdio")))]
  {
    Err(Error::IoError("No console support!".to_string()))
  }
}

/// Serialise an instance of type `T` to a prompt using the bare serialiser.
pub fn to_bare_prompt<T: Serialize, P: PromptResponder>(
  value: &T,
  prompt: P,
) -> Result<()> {
  Serialize::serialize(value, &mut Serializer::from_prompt(prompt))
}

/// Serialise an instance of type `T` to a prompt while handling meta-commands
/// and scope compacting.
pub fn to_prompt<T: Serialize, P: PromptResponder>(
  value: &T,
  prompt: P,
) -> Result<()> {
  to_bare_prompt(value, MetaCommandPrompt::new(CompactPrompt::new(prompt)))
}
