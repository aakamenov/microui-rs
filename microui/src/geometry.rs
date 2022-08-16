#[derive(Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy)]
pub enum Clip {
    None,
    Part,
    All
}

#[inline(always)]
pub const fn vec2(x: i32, y: i32) -> Vec2 {
    Vec2 { x, y }
}

#[inline(always)]
pub const fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

impl Vec2 {
    pub const ZERO: Self = vec2(0, 0);
}

impl Rect {
    pub const UNCLIPPED: Self = rect(0, 0, 0x1000000, 0x1000000);

    #[inline]
    #[must_use]
    pub fn expand(self, n: i32) -> Rect {
        Self {
            x: self.x - n,
            y: self.y - n,
            w: self.w + n * 2,
            h: self.h + n * 2
        }
    }

    pub fn intersect(&self, rhs: Self) -> Self {
        let x1 = self.x.max(rhs.x);
        let y1 = self.y.max(rhs.y);
        let mut x2 = Ord::min(self.x + self.w, rhs.x + rhs.w);
        let mut y2 = Ord::min(self.y + self.h, rhs.y + rhs.h);

        if x2 < x1 {
            x2 = x1;
        }

        if y2 < y1 {
            y2 = y1;
        }

        Self {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1
        }
    }

    pub fn clip(&self, rhs: Self) -> Clip {
        if rhs.x > self.x + self.w || rhs.x + rhs.w < self.x ||
            rhs.y > self.y + self.h || rhs.y + rhs.h < self.y { 
            return Clip::All;
        }

        if rhs.x >= self.x && rhs.x + rhs.w <= self.x + self.w &&
            rhs.y >= self.y && rhs.y + rhs.h <= self.y + self.h { 
            return Clip::None; 
        }

        Clip::Part
    }

    #[inline]
    pub fn overlaps(&self, point: Vec2) -> bool {
        point.x >= self.x &&
        point.x < self.x + self.w &&
        point.y >= self.y &&
        point.y < self.y + self.h
    }
}
