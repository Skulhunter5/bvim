#[derive(Debug, Clone, Copy)]
pub struct Position<T: Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Position<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
