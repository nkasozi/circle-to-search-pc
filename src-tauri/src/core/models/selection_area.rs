use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectionArea {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
}

impl SelectionArea {
    pub fn width(&self) -> u32 {
        ((self.end_x - self.start_x).abs()) as u32
    }

    pub fn height(&self) -> u32 {
        ((self.end_y - self.start_y).abs()) as u32
    }

    pub fn min_x(&self) -> i32 {
        self.start_x.min(self.end_x)
    }

    pub fn min_y(&self) -> i32 {
        self.start_y.min(self.end_y)
    }
}
