use crate::error::{Error, Result, UserAction};
use crate::prompt::{
  PromptRequester, PromptResponder, ReportKind, RequestKind,
};
use rustyline::completion::{Candidate, Completer};
use rustyline::config::Configurer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::borrow::BorrowMut;
use std::marker::PhantomData;

/// Trait for RustyLine `Helper`s which support tab completion of variant
/// names.
pub trait SpanielHelper: Helper {
  fn set_variants(&mut self, variants: &'static [&'static str]);
}

impl SpanielHelper for () {
  fn set_variants(&mut self, _variants: &'static [&'static str]) {}
}

/// RustyLine `Helper` which supports tab completion of variant names.
pub struct SimpleHelper {
  variants: &'static [&'static str],
}

impl SimpleHelper {
  pub fn new() -> Self {
    SimpleHelper { variants: &[] }
  }
}

impl Default for SimpleHelper {
  fn default() -> Self {
    Self::new()
  }
}

impl SpanielHelper for SimpleHelper {
  fn set_variants(&mut self, variants: &'static [&'static str]) {
    self.variants = variants;
  }
}

/// Tab completion candidate with a static lifetime.
pub struct StaticCandidate(&'static str);

impl Candidate for StaticCandidate {
  fn display(&self) -> &str {
    self.0
  }

  fn replacement(&self) -> &str {
    self.0
  }
}

impl Completer for SimpleHelper {
  type Candidate = StaticCandidate;

  fn complete(
    &self,
    line: &str,
    pos: usize,
    _ctx: &Context<'_>,
  ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
    let mut cands = Vec::new();
    let partial = &line[..pos];
    for variant in self.variants {
      if variant.starts_with(partial) {
        cands.push(StaticCandidate(variant));
      }
    }
    Ok((0, cands))
  }
}

impl Validator for SimpleHelper {}
impl Highlighter for SimpleHelper {}
impl Hinter for SimpleHelper {}
impl Helper for SimpleHelper {}

/// Prompt based on RustyLine
pub struct RustyLinePrompt<T: BorrowMut<Editor<H>>, H: SpanielHelper> {
  editor: T,
  helper: PhantomData<H>,
  level: usize,
  was_autohistory: bool,
}

impl<T: BorrowMut<Editor<H>>, H: SpanielHelper> RustyLinePrompt<T, H> {
  pub fn new(mut editor: T) -> Self {
    let ed_ref = editor.borrow_mut();
    let was_autohistory = ed_ref.config_mut().auto_add_history();
    ed_ref.set_auto_add_history(true);
    RustyLinePrompt {
      editor,
      helper: PhantomData,
      level: 0,
      was_autohistory,
    }
  }

  fn spaces(&self) -> usize {
    2 * self.level
  }
}

impl RustyLinePrompt<Editor<SimpleHelper>, SimpleHelper> {
  pub fn new_editor() -> Self {
    let mut editor = Editor::new();
    editor.set_helper(Some(SimpleHelper::new()));
    Self::new(editor)
  }
}

impl<T: BorrowMut<Editor<H>>, H: SpanielHelper> Drop for RustyLinePrompt<T, H> {
  fn drop(&mut self) {
    self
      .editor
      .borrow_mut()
      .set_auto_add_history(self.was_autohistory);
  }
}

impl<T: BorrowMut<Editor<H>>, H: SpanielHelper> PromptResponder
  for RustyLinePrompt<T, H>
{
  fn begin_scope(&mut self, name: &str, _size: Option<usize>) -> Result<()> {
    println!("{:indent$}{} {{", "", name, indent = self.spaces());
    self.level += 1;
    Ok(())
  }

  fn end_scope(&mut self) -> Result<()> {
    self.level -= 1;
    println!("{:indent$}}}", "", indent = self.spaces());
    Ok(())
  }

  fn respond(
    &mut self,
    _kind: RequestKind,
    prompt: &str,
    response: &str,
  ) -> Result<()> {
    println!(
      "{:indent$}{}: {}",
      "",
      prompt,
      response,
      indent = self.spaces()
    );
    Ok(())
  }
}

impl<T: BorrowMut<Editor<H>>, H: SpanielHelper> PromptRequester
  for RustyLinePrompt<T, H>
{
  fn is_interactive(&self) -> bool {
    true
  }

  fn request(
    &mut self,
    _kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    let fmt_prompt =
      format!("{:indent$}{}: ", "", prompt, indent = self.spaces());
    let editor = self.editor.borrow_mut();
    if let Some(h) = editor.helper_mut() {
      h.set_variants(variants)
    }
    let res = editor.readline(&fmt_prompt);
    if let Some(h) = editor.helper_mut() {
      h.set_variants(&[])
    }
    match res {
      Ok(line) => Ok(line),
      Err(rustyline::error::ReadlineError::Interrupted) => {
        Err(Error::UserAction(UserAction::Cancel))
      }
      Err(e) => Err(Error::IOError(e.to_string())),
    }
  }

  fn report(&mut self, _kind: ReportKind, msg: &str) -> Result<()> {
    println!("{:indent$}{}", "", msg, indent = self.spaces());
    Ok(())
  }
}
