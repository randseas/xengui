// SPDX-License-Identifier: Apache-2.0
use super::{Background, Color, Edges, Length, Size, Style};

pub trait StyleBuilder: Sized {
    fn style_mut(&mut self) -> &mut Style;
    fn mark_dirty(&mut self) {}

    fn width<L: Into<Length>>(mut self, width: L) -> Self {
        self.style_mut()
            .size
            .get_or_insert_with(Default::default)
            .width = width.into();
        self.mark_dirty();
        self
    }

    fn height<L: Into<Length>>(mut self, height: L) -> Self {
        self.style_mut()
            .size
            .get_or_insert_with(Default::default)
            .height = height.into();
        self.mark_dirty();
        self
    }

    fn size<W: Into<Length>, H: Into<Length>>(mut self, width: W, height: H) -> Self {
        self.style_mut().size = Some(Size::new(width.into(), height.into()));
        self.mark_dirty();
        self
    }

    fn padding<E: Into<Edges>>(mut self, padding: E) -> Self {
        self.style_mut().padding = Some(padding.into());
        self.mark_dirty();
        self
    }

    fn color(mut self, color: Color) -> Self {
        self.style_mut().color = Some(color);
        self.mark_dirty();
        self
    }

    fn background<B: Into<Background>>(mut self, background: B) -> Self {
        self.style_mut().background = Some(background.into());
        self.mark_dirty();
        self
    }

    fn font_size<L: Into<Length>>(mut self, size: L) -> Self {
        self.style_mut().font_size = Some(size.into());
        self.mark_dirty();
        self
    }
}
