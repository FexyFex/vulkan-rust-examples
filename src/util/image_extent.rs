#![allow(dead_code)]

pub struct ImageExtent {
    pub width: u32,
    pub height: u32
}

impl ImageExtent {
    pub fn new(width: u32, height: u32) -> ImageExtent {
        return ImageExtent { width, height };
    }
}