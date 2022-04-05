use crate::ff_error::*;
use crate::*;
use libc::{c_char, EAGAIN};
use log::error;
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Debug)]
pub struct Decoder {
    codec_ctx: *mut AVCodecContext,
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            if !self.codec_ctx.is_null() {
                avcodec_free_context(&mut self.codec_ctx);
            }
        }
    }
}

impl Deref for Decoder {
    type Target = AVCodecContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.codec_ctx }
    }
}

impl DerefMut for Decoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.codec_ctx }
    }
}

impl Decoder {
    pub fn with_stream(stream: Stream) -> Option<Self> {
        unsafe {
            let codec = avcodec_find_decoder((*stream.codecpar).codec_id as u32);
            if codec.is_null() {
                error!(
                    "failed to find decoder with codec_id:{}",
                    (*stream.codecpar).codec_id
                );
                return None;
            }

            let mut codec_ctx = avcodec_alloc_context3(codec);
            if codec_ctx.is_null() {
                error!("avcodec_alloc_context3 failed");
                return None;
            }

            let ret = avcodec_parameters_to_context(codec_ctx, stream.codecpar);
            if ret < 0 {
                ff_error!(ret, "avcodec_parameters_to_context failed");
                avcodec_free_context(&mut codec_ctx);
                return None;
            }

            let ret = avcodec_open2(codec_ctx, codec, ptr::null_mut());
            if ret < 0 {
                ff_error!(ret, "avcodec_open2 failed");
                avcodec_free_context(&mut codec_ctx);
                return None;
            }

            log::info!(
                "created decoder({:?}) for stream {}",
                CStr::from_ptr((*codec).name),
                stream.id
            );

            Some(Decoder { codec_ctx })
        }
    }

    /// return 0 on success, AVERROR(EAGAIN) if dequeue_frame() is expected
    /// to be called and packet to be resent, AVERROR_EOF if the decoder
    /// has been flushed and not more packets can be sent, other negative values
    /// are legitimate decoding errors.
    pub fn enqueue_packet(&self, in_packet: &Packet) -> i32 {
        unsafe {
            let ret = avcodec_send_packet(self.codec_ctx, &**in_packet);
            if ret < 0 && ret != AVERROR(EAGAIN) && ret != AVERROR_EOF {
                ff_error!(ret, "failed to call avcodec_send_packet");
            }
            ret
        }
    }

    /// return 0 on success, AVERROR(EAGAIN) if more AVPackets are expected,
    /// AVERROR_EOF if the decoder has been flushed and there will be not more
    /// output frames, other negative values are legitimate decoding errors.
    pub fn dequeue_frame(&self, out_frame: &mut Frame) -> i32 {
        unsafe {
            let ret = avcodec_receive_frame(self.codec_ctx, &mut **out_frame);
            if ret < 0 && ret != AVERROR(EAGAIN) && ret != AVERROR_EOF {
                ff_error!(ret, "failed to call avcodec_send_packet");
            }
            ret
        }
    }
}
