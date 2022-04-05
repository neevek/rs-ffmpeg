use std::ops::{Deref, DerefMut};

use crate::*;

pub struct Packet {
    pkt: AVPacket,
}

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe {
            av_packet_unref(&mut self.pkt);
        }
    }
}

impl Deref for Packet {
    type Target = AVPacket;

    fn deref(&self) -> &Self::Target {
        &self.pkt
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pkt
    }
}

impl Packet {
    pub fn new() -> Self {
        unsafe {
            let mut pkt = std::mem::zeroed();
            av_init_packet(&mut pkt);
            Packet { pkt }
        }
    }
}
