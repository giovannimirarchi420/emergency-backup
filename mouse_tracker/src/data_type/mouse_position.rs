#[derive(PartialEq, Debug, Clone)]
pub struct MousePosition {
    x: i32,
    y: i32
}

impl MousePosition {
    pub fn new(tuple: (i32, i32)) -> Self{
        MousePosition{x: tuple.0, y: tuple.1 }
    }
}