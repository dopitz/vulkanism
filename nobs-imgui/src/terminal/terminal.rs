use crate::components::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;

pub struct Terminal<S: Style> {
  wnd: Window<ColumnLayout, S>,

  output_wnd: Window<ColumnLayout, S>,
  output: TextBox<S>,

  input: TextEdit<S>,
}

impl<S: Style> Size for Terminal<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.wnd.rect(rect);
    let mut r = self.wnd.get_client_rect();
    r.size.y = r.size.y.saturating_sub(self.input.get_size_hint().y + 10); 
    self.output_wnd.rect(r);
    self
  }

  fn get_rect(&self) -> Rect {
    self.wnd.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.wnd.get_size_hint()
  }
}

impl<S: Style> Component<S> for Terminal<S> {
  type Event = ();
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event> {
    layout.apply(self);
    if let Some(_) = self.wnd.draw(screen, layout, focus) {
      self.output_wnd.focus(true);
    }

    self.output_wnd.draw(screen, &mut self.wnd, focus);
    if let Some(_) = self.output.draw(screen, &mut self.output_wnd, focus) {
      self.output_wnd.focus(true);
    }

    Spacer::new(vec2!(10)).draw(screen, &mut self.wnd, focus);

    if let Some(e) = self.input.draw(screen, &mut self.wnd, focus) {
      self.output_wnd.focus(true);
      println!("{:?}", e);
    }

    None
  }
}

impl<S: Style> Terminal<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let mut wnd = Window::new(gui, ColumnLayout::default());
    wnd
      .caption("terminal")
      .position(20, 20)
      .size(500, 500)
      .focus(true)
      .draw_caption(false);

    let mut output_wnd = Window::new(gui, ColumnLayout::default());
    output_wnd.draw_caption(false);
    output_wnd.style("NoStyle", false, false);
    let mut output = TextBox::new(gui);
    output.style("NoStyle").text("Welcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nAOEUAOEUAOEUAOEU!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nAOEUAOEUAOEUAOEU!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!\nWelcome!");

    let mut input = TextBox::new(gui);
    input.text("~$:");

    Self { wnd, output_wnd, output, input }
  }

  pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
    self.wnd.position(x, y);
    self
  }

  pub fn size(&mut self, x: u32, y: u32) -> &mut Self {
    self.wnd.size(x, y);
    self
  }
}
