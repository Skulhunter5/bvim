pub struct Rectangle<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Copy> Rectangle<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    pub fn unpack(&self) -> (T, T, T, T) {
        (
            self.x.clone(),
            self.y.clone(),
            self.w.clone(),
            self.h.clone(),
        )
    }
}
