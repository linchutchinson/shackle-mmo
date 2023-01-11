mod vec2;

pub use vec2::Vec2;

#[derive(Copy, Clone)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        let position = Vec2::new(x, y);
        let size = Vec2::new(w, h);

        Self { position, size }
    }

    pub fn center(&self) -> Vec2 {
        self.position + (self.size * 0.5)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    pub fn left(&self) -> f32 {
        self.position.x
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.size.x
    }

    pub fn top(&self) -> f32 {
        self.position.y
    }

    pub fn bottom(&self) -> f32 {
        self.position.y + self.size.y
    }
}
