mod controller;
mod model;
mod view;

pub use controller::main as controller;
pub use model::main as model;

use crate::error::Result;
use async_trait::async_trait;
use error_chain::bail;
use maud::{html, Markup};
use model::Context;
use std::rc::Rc;

#[async_trait(?Send)]
trait Model {
  type View: View = ();
  async fn exec(&self, ctx: Rc<Context>) -> Result<Markup> {
    self.exec_sync(ctx)
  }
  fn exec_sync(&self, _ctx: Rc<Context>) -> Result<Markup> {
    bail!("unimplemented!")
  }
}

trait View {
  fn render(self, ctx: &Context) -> Result<Markup>;
}

impl View for () {
  fn render(self, _: &Context) -> Result<Markup> {
    bail!("unimplemented!")
  }
}
