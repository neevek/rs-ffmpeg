use crate::*;
use std::ops::Deref;

pub struct Stream {
    stream: *mut AVStream,
}

impl Deref for Stream {
    type Target = AVStream;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.stream }
    }
}

impl Stream {
    pub fn wrap(stream: *mut AVStream) -> Self {
        Stream { stream }
    }

    pub fn is_video_stream(&self) -> bool {
        unsafe { (*self.codecpar).codec_type == AVMediaType_AVMEDIA_TYPE_VIDEO }
    }

    pub fn is_audio_stream(&self) -> bool {
        unsafe { (*self.codecpar).codec_type == AVMediaType_AVMEDIA_TYPE_AUDIO }
    }

    pub fn is_subtitle_stream(&self) -> bool {
        unsafe { (*self.codecpar).codec_type == AVMediaType_AVMEDIA_TYPE_SUBTITLE }
    }

    pub fn codecpar(&self) -> *const AVCodecParameters {
        self.deref().codecpar
    }

    pub fn id(&self) -> i32 {
        self.deref().id
    }
}
