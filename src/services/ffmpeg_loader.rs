use anyhow::Error;
use egui::ColorImage;
use std::path::PathBuf;

pub struct FfmpegLoader {}

impl FfmpegLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_image(&self, path: PathBuf) -> Result<ColorImage, Error> {
        unsafe {
            let mut demux = egui_video::ffmpeg::demux::Demuxer::new(path.to_str().unwrap());
            let info = demux.probe_input()?;

            let bv = info.best_video();
            if bv.is_none() {
                anyhow::bail!("Not a video/image");
            }
            let bv = bv.unwrap();
            let mut decode = egui_video::ffmpeg::decode::Decoder::new();
            let rgb = egui_video::ffmpeg_sys_the_third::AVPixelFormat::AV_PIX_FMT_RGB24;
            let mut scaler = egui_video::ffmpeg::scale::Scaler::new(rgb);

            let mut n_pkt = 0;
            loop {
                let (mut pkt, stream) = demux.get_packet()?;
                if (*stream).index as usize == bv.index {
                    let frames = decode.decode_pkt(pkt, stream)?;
                    if let Some((frame, _)) = frames.first() {
                        let mut frame = *frame;
                        let mut frame_rgb = scaler.process_frame(
                            frame,
                            (*frame).width as u16,
                            (*frame).height as u16,
                        )?;
                        egui_video::ffmpeg_sys_the_third::av_frame_free(&mut frame);

                        let image = egui_video::ffmpeg::video_frame_to_image(frame_rgb);
                        egui_video::ffmpeg_sys_the_third::av_frame_free(&mut frame_rgb);
                        return Ok(image);
                    }
                }
                egui_video::ffmpeg_sys_the_third::av_packet_free(&mut pkt);

                n_pkt += 1;
                if n_pkt > 10 {
                    anyhow::bail!("No image found");
                }
            }
        }
    }
}
