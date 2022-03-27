use rs_ffmpeg::*;
use std::ptr::*;
use std::ffi::CString;
fn main() {
    unsafe {
        let path = CString::new("/path/to/media_file").unwrap();
        let mut fmt = avformat_alloc_context();
        let input = avformat_open_input(&mut fmt, path.as_ptr(), null_mut(), null_mut());
        
        let codec_name = CString::new("libx264").unwrap();
        let codec = avcodec_find_encoder_by_name(codec_name.as_ptr());
        
        println!("input:{}, codec:{:?}", input, codec);
    }
}
