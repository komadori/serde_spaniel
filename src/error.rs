use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

/// This type represents actions a user may take while interacting with the
/// deserialiser.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UserAction {
  /// Cancel deserialising.
  Cancel,
  /// Restart deserialising from response `n`.
  Restart(usize),
  /// Undo the last `n` responses to the deserialiser.
  Undo(usize),
}

/// This type represents errors that may occur.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
  SerdeError(String),
  IOError(String),
  UserAction(UserAction),
  BadResponse,
  CannotReplay,
}

impl ser::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::SerdeError(msg.to_string())
  }
}

impl de::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::SerdeError(msg.to_string())
  }
}

impl Display for Error {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::SerdeError(msg) => write!(fmt, "Serde: {}", msg),
      Error::IOError(msg) => write!(fmt, "I/O: {}", msg),
      Error::UserAction(action) => write!(fmt, "UserAction: {:?}", action),
      Error::BadResponse => write!(fmt, "Bad Response"),
      Error::CannotReplay => write!(fmt, "Cannot Replay"),
    }
  }
}

impl std::error::Error for Error {}
