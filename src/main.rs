use log::info;
use rs_ffmpeg::Decoder;
use rs_ffmpeg::*;

fn main() {
    init_logger("T");

    let url =
        "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_10MB.mp4";

    let input = Input::with_url(url).unwrap();
    let st = input.get_stream(0).unwrap();
    let d = Decoder::with_stream(st).unwrap();

    let mut packet = Packet::new();
    input.read_packet(&mut *packet);
    d.enqueue_packet(&packet);
    input.read_packet(&mut *packet);
    d.enqueue_packet(&packet);
    input.read_packet(&mut *packet);
    d.enqueue_packet(&packet);
    input.read_packet(&mut *packet);
    d.enqueue_packet(&packet);

    let mut frame = Frame::new();
    d.dequeue_frame(&mut frame);
    info!(
        ">>>>>>>>>> read: {}, width:{}",
        packet.size, frame.pkt_duration
    );
}
