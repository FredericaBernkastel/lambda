mod controller;
mod model;
mod view;

pub use controller::main as controller;
pub use model::main as model;

use crate::error::Result;
use model::Context;
use maud::Markup;
use async_trait::async_trait;
use std::rc::Rc;

#[async_trait(?Send)] trait Model {
  type View: View;
  async fn render (ctx: Rc<Context>) -> Result<Markup>;
}

trait View {
  fn render (self, ctx: &Context) -> Result<Markup>;
}