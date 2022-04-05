use crate::*;
use std::ops::{Deref, DerefMut};

pub struct Frame {
    frame: AVFrame,
}

impl Deref for Frame {
    type Target = AVFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}

impl Frame {
    pub fn new() -> Self {
        unsafe {
            let frame = std::mem::zeroed();
            Frame { frame }
        }
    }
}
