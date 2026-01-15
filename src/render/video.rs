use std::path::Path;

use anyhow::{Result, anyhow};
use ffmpeg_next as ffmpeg;

/// Video decoder for BGA video files
pub struct VideoDecoder {
    input_ctx: ffmpeg::format::context::Input,
    decoder: ffmpeg::decoder::Video,
    scaler: ffmpeg::software::scaling::Context,
    video_stream_index: usize,
    width: u32,
    height: u32,
    time_base_ms: f64,
    frame_buffer: Vec<u8>,
    last_pts: Option<i64>,
}

impl VideoDecoder {
    /// Open a video file for decoding
    #[allow(dead_code)] // For future BGA feature implementation
    pub fn open(path: &Path) -> Result<Self> {
        ffmpeg::init()?;

        let input_ctx = ffmpeg::format::input(&path)?;

        let stream = input_ctx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow!("No video stream found"))?;

        let video_stream_index = stream.index();
        let time_base = stream.time_base();
        let time_base_ms = f64::from(time_base) * 1000.0;

        let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = context.decoder().video()?;

        let width = decoder.width();
        let height = decoder.height();

        let scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            width,
            height,
            ffmpeg::format::Pixel::RGBA,
            width,
            height,
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        Ok(Self {
            input_ctx,
            decoder,
            scaler,
            video_stream_index,
            width,
            height,
            time_base_ms,
            frame_buffer: vec![0u8; (width * height * 4) as usize],
            last_pts: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Decode and return the next frame as RGBA data
    pub fn decode_next_frame(&mut self) -> Option<&[u8]> {
        loop {
            // Try to receive a frame from decoder first
            let mut frame = ffmpeg::frame::Video::empty();
            if self.decoder.receive_frame(&mut frame).is_ok() {
                self.last_pts = frame.pts();
                return self.scale_frame(&frame);
            }

            // Need more packets - find next video packet
            loop {
                match self.input_ctx.packets().next() {
                    Some((stream, packet)) => {
                        if stream.index() == self.video_stream_index
                            && self.decoder.send_packet(&packet).is_ok()
                        {
                            break; // Try to receive frame again
                        }
                    }
                    None => {
                        // End of stream - flush decoder
                        let _ = self.decoder.send_eof();
                        let mut frame = ffmpeg::frame::Video::empty();
                        if self.decoder.receive_frame(&mut frame).is_ok() {
                            self.last_pts = frame.pts();
                            return self.scale_frame(&frame);
                        }
                        return None;
                    }
                }
            }
        }
    }

    /// Get frame at approximately the given time (sequential decode)
    /// Note: For BGA playback, we decode sequentially and skip frames if needed
    pub fn decode_frame_at(&mut self, target_time_ms: f64) -> Option<&[u8]> {
        let target_pts = (target_time_ms / self.time_base_ms) as i64;

        // If we're behind, decode frames until we catch up
        loop {
            if let Some(last_pts) = self.last_pts {
                if last_pts >= target_pts {
                    // We have a frame at or past the target time
                    return Some(&self.frame_buffer);
                }
            }

            // Decode next frame
            if self.decode_next_frame().is_none() {
                // End of video - return last frame if available
                if self.last_pts.is_some() {
                    return Some(&self.frame_buffer);
                }
                return None;
            }
        }
    }

    fn scale_frame(&mut self, frame: &ffmpeg::frame::Video) -> Option<&[u8]> {
        let mut rgb_frame = ffmpeg::frame::Video::empty();
        if self.scaler.run(frame, &mut rgb_frame).is_ok() {
            let data = rgb_frame.data(0);
            let copy_len = data.len().min(self.frame_buffer.len());
            self.frame_buffer[..copy_len].copy_from_slice(&data[..copy_len]);
            Some(&self.frame_buffer)
        } else {
            None
        }
    }

    /// Reset decoder to beginning (for looping)
    #[allow(dead_code)]
    pub fn reset(&mut self) -> Result<()> {
        // Seek to beginning
        self.input_ctx
            .seek(0, 0..i64::MAX)
            .map_err(|e| anyhow!("Seek failed: {:?}", e))?;

        // Flush decoder
        self.decoder.flush();
        self.last_pts = None;

        Ok(())
    }
}
