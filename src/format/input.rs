use crate::ff_error::*;
use crate::*;
use libc::c_char;
use log::error;
use std::ffi::CStr;
use std::ops::Deref;
use std::{ffi::CString, ptr::null_mut};

pub struct Input {
    fmt: *mut AVFormatContext,
}

impl Drop for Input {
    fn drop(&mut self) {
        if !self.fmt.is_null() {
            unsafe {
                avformat_free_context(self.fmt);
            }
        }
    }
}

impl Deref for Input {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fmt }
    }
}

impl Input {
    pub fn with_url(url: &str) -> Option<Self> {
        unsafe {
            let mut fmt = avformat_alloc_context();
            let c_url = CString::new(url).unwrap();
            let ret = avformat_open_input(&mut fmt, c_url.as_ptr(), null_mut(), null_mut());
            if ret < 0 {
                ff_error!(ret, "avformat_open_input failed");
                return None;
            }

            let ret = avformat_find_stream_info(fmt, null_mut());
            if ret < 0 {
                ff_error!(ret, "avformat_find_stream_info failed");
                return None;
            }

            Some(Input { fmt })
        }
    }

    // return 0 on success, AVERROR_EOF on end of file or other negative values are errors
    pub fn read_packet(&self, out_packet: *mut AVPacket) -> i32 {
        unsafe {
            let ret = av_read_frame(self.fmt, out_packet);
            if ret < 0 && ret == AVERROR_EOF {
                ff_error!(ret, "av_read_frame failed");
            }
            ret
        }
    }

    pub fn get_stream(&self, index: u32) -> Option<Stream> {
        unsafe {
            let fmt = *self.fmt;
            if index < fmt.nb_streams {
                let st = *(fmt.streams.offset(index as isize) as *mut *mut AVStream);
                Some(Stream::wrap(st))
            } else {
                None
            }
        }
    }

    pub fn get_stream_count(&self) -> u32 {
        unsafe { (*self.fmt).nb_streams }
    }
}
