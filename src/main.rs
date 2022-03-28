use rs_ffmpeg::*;
use std::ptr::*;
use std::ffi::CString;

#[link(name = "x264")]
fn main() {
    unsafe {
        let path = CString::new("https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_10MB.mp4").unwrap();
        let mut fmt = avformat_alloc_context();
        let input = avformat_open_input(&mut fmt, path.as_ptr(), null_mut(), null_mut());
        let ret = avformat_find_stream_info(fmt, null_mut());
        let dura = (*fmt).duration;
        let st = (*fmt).nb_streams;
        
        let codec_name = CString::new("libx264").unwrap();
        let codec = avcodec_find_encoder_by_name(codec_name.as_ptr());
        let codec2 = avcodec_find_encoder(AVCodecID_AV_CODEC_ID_H264);
        
        println!("input:{}, codec:{:?}, codec2:{:?}, ret:{}, dur:{}, streams:{}",
                 input, codec, codec2, ret, dura, st);
    }
}
