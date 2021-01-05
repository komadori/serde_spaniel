use crate::error::Result;
use crate::prompt::{
  PromptRequester, PromptResponder, ReportKind, RequestKind,
};

#[derive(PartialEq, Eq)]
pub(crate) enum ScopeLimit {
  Explicit,
  Implicit,
}

pub(crate) struct ScopeEntry(ScopeLimit, u32);

pub(crate) struct InternalPrompt<P: PromptResponder> {
  inner: P,
  scopes: Vec<ScopeEntry>,
}

impl<P: PromptResponder> InternalPrompt<P> {
  pub fn from_prompt(inner: P) -> Self {
    InternalPrompt {
      inner,
      scopes: Vec::new(),
    }
  }

  pub fn cleanup(&mut self) -> Result<()> {
    while let Some(ScopeEntry(_, n)) = self.scopes.last_mut() {
      *n -= 1;
      if *n == 0 {
        self.scopes.pop();
      }
      self.inner.end_scope()?;
    }
    Ok(())
  }

  pub fn begin_scope(
    &mut self,
    name: &str,
    size: Option<usize>,
    limit: ScopeLimit,
  ) -> Result<()> {
    self.inner.begin_scope(name, size)?;
    match self.scopes.last_mut() {
      Some(ScopeEntry(lim, n)) if limit == *lim => *n += 1,
      _ => self.scopes.push(ScopeEntry(limit, 1)),
    }
    Ok(())
  }

  pub fn end_implicit_scopes(&mut self) -> Result<()> {
    while let Some(ScopeEntry(ScopeLimit::Implicit, n)) = self.scopes.last_mut()
    {
      *n -= 1;
      if *n == 0 {
        self.scopes.pop();
      }
      self.inner.end_scope()?;
    }
    Ok(())
  }
}

impl<P: PromptResponder> Drop for InternalPrompt<P> {
  fn drop(&mut self) {
    let _ = self.cleanup();
  }
}

impl<P: PromptResponder> PromptResponder for InternalPrompt<P> {
  fn begin_scope(&mut self, name: &str, size: Option<usize>) -> Result<()> {
    self.begin_scope(name, size, ScopeLimit::Explicit)
  }

  fn end_scope(&mut self) -> Result<()> {
    self.inner.end_scope()?;
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

  fn respond(
    &mut self,
    kind: RequestKind,
    inner: &str,
    response: &str,
  ) -> Result<()> {
    self.inner.respond(kind, inner, response)
  }
}

impl<P: PromptRequester> PromptRequester for InternalPrompt<P> {
  fn is_interactive(&self) -> bool {
    self.inner.is_interactive()
  }

  fn request(
    &mut self,
    kind: RequestKind,
    prompt: &str,
    variants: &'static [&'static str],
  ) -> Result<String> {
    self.inner.request(kind, prompt, variants)
  }

  fn report(&mut self, kind: ReportKind, msg: &str) -> Result<()> {
    self.inner.report(kind, msg)
  }
}

#[doc(hidden)]
#[macro_export]
macro_rules! internal_prompt_responder_mixin {
  ($vname:ident) => {
    fn begin_scope(
      &mut self,
      name: &str,
      size: Option<usize>,
      limit: ScopeLimit,
    ) -> Result<()> {
      self.$vname.begin_scope(name, size, limit)
    }
    fn end_scope(&mut self) -> Result<()> {
      self.$vname.end_scope()
    }
    fn end_implicit_scopes(&mut self) -> Result<()> {
      self.$vname.end_implicit_scopes()
    }
    fn respond(
      &mut self,
      kind: RequestKind,
      inner: &str,
      response: &str,
    ) -> Result<()> {
      self.$vname.respond(kind, inner, response)
    }
  };
}
