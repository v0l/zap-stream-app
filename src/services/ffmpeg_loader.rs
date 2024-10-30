use anyhow::Error;
use egui::ColorImage;
use egui_video::ffmpeg_rs_raw::{Decoder, Demuxer, Scaler};
use egui_video::ffmpeg_sys_the_third::{av_frame_free, av_packet_free, AVPixelFormat};
use egui_video::media_player::video_frame_to_image;
use std::path::PathBuf;

pub struct FfmpegLoader {}

impl FfmpegLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_image(&self, path: PathBuf) -> Result<ColorImage, Error> {
        let demux = Demuxer::new(path.to_str().unwrap());
        Self::load_image_from_demuxer(demux)
    }

    pub fn load_image_bytes<'a>(
        &self,
        key: &str,
        data: &'static [u8],
    ) -> Result<ColorImage, Error> {
        let demux = Demuxer::new_custom_io(data, Some(key.to_string()));
        Self::load_image_from_demuxer(demux)
    }

    fn load_image_from_demuxer(mut demuxer: Demuxer) -> Result<ColorImage, Error> {
        unsafe {
            let info = demuxer.probe_input()?;

            let bv = if let Some(v) = info.best_video() {
                v
            } else {
                anyhow::bail!("Not a video/image");
            };
            let mut decode = Decoder::new();
            let rgb = AVPixelFormat::AV_PIX_FMT_RGB24;
            let mut scaler = Scaler::new(rgb);

            decode.setup_decoder(bv, None)?;

            let mut n_pkt = 0;
            loop {
                let (mut pkt, stream) = demuxer.get_packet()?;
                if pkt.is_null() {
                    break;
                }
                if (*stream).index as usize == bv.index {
                    let frames = decode.decode_pkt(pkt, stream)?;
                    if let Some((frame, _)) = frames.first() {
                        let mut frame = *frame;
                        let frame_rgb = scaler.process_frame(
                            frame,
                            (*frame).width as u16,
                            (*frame).height as u16,
                        )?;
                        av_frame_free(&mut frame);

                        let image = video_frame_to_image(frame_rgb);
                        return Ok(image);
                    }
                }
                av_packet_free(&mut pkt);

                n_pkt += 1;
                if n_pkt > 10 {
                    break;
                }
            }
            anyhow::bail!("No image found");
        }
    }
}
