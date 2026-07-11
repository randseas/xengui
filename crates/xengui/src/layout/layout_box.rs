#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutBox {
    /// Rounded rectangle hit-test.
    /// Radius 0 ise klasik rectangle testi yapar.
    pub fn contains_rounded(&self, point: (f32, f32), radius: f32) -> bool {
        let (px, py) = point;

        // Önce bounding box kontrolü
        if px < self.x || px > self.x + self.width || py < self.y || py > self.y + self.height {
            return false;
        }

        if radius <= 0.0 {
            return true;
        }

        let r = radius.min(self.width * 0.5).min(self.height * 0.5);

        // Merkez bölgeleri direkt kabul et
        if px >= self.x + r && px <= self.x + self.width - r {
            return true;
        }

        if py >= self.y + r && py <= self.y + self.height - r {
            return true;
        }

        // Köşe circle testleri
        let (cx, cy) = if px < self.x + r {
            if py < self.y + r {
                (self.x + r, self.y + r) // top-left
            } else {
                (self.x + r, self.y + self.height - r) // bottom-left
            }
        } else {
            if py < self.y + r {
                (self.x + self.width - r, self.y + r) // top-right
            } else {
                (self.x + self.width - r, self.y + self.height - r) // bottom-right
            }
        };

        let dx = px - cx;
        let dy = py - cy;

        dx * dx + dy * dy <= r * r
    }
}
