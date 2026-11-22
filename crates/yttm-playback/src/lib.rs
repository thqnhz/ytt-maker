use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

#[derive(Clone, Debug, Default)]
pub struct VideoState {
    pub path: Option<PathBuf>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub playing: bool,
}

impl VideoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, path: PathBuf) {
        self.path = Some(path);
        self.position_ms = 0;
        self.duration_ms = None;
        self.playing = false;
    }

    pub fn close(&mut self) {
        *self = Self::default();
    }

    pub fn file_name(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
    }
}

#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

pub struct Player {
    playbin: gst::Element,
    appsink: gst_app::AppSink,
}

impl Player {
    pub fn new() -> Result<Self> {
        gst::init().context("failed to initialize GStreamer")?;

        let playbin = gst::ElementFactory::make("playbin")
            .build()
            .context("failed to create playbin element")?;

        let appsink = gst_app::AppSink::builder()
            .caps(
                &gst_video::VideoCapsBuilder::new()
                    .format(gst_video::VideoFormat::Bgra)
                    .build(),
            )
            .build();

        playbin.set_property("video-sink", &appsink);

        Ok(Self { playbin, appsink })
    }

    pub fn open(&self, path: &Path) -> Result<()> {
        let uri = gst::glib::filename_to_uri(path, None).context("failed to build file URI")?;
        self.playbin
            .set_state(gst::State::Null)
            .context("failed to reset playbin state")?;
        self.playbin.set_property("uri", uri.as_str());
        self.playbin
            .set_state(gst::State::Paused)
            .context("failed to pause playbin for preroll")?;
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.playbin
            .set_state(gst::State::Playing)
            .context("failed to start playback")?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.playbin
            .set_state(gst::State::Paused)
            .context("failed to pause playback")?;
        Ok(())
    }

    pub fn position_ms(&self) -> Option<u64> {
        self.playbin
            .query_position::<gst::ClockTime>()
            .map(|p| p.mseconds())
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.playbin
            .query_duration::<gst::ClockTime>()
            .map(|d| d.mseconds())
    }

    pub fn pull_frame(&self) -> Option<VideoFrame> {
        let sample = self.appsink.try_pull_sample(gst::ClockTime::ZERO)?;
        let caps = sample.caps()?;
        let info = gst_video::VideoInfo::from_caps(caps).ok()?;
        let buffer = sample.buffer()?;
        let map = buffer.map_readable().ok()?;

        let width = info.width() as usize;
        let height = info.height() as usize;
        let stride = info.stride()[0] as usize;
        let row_bytes = width * 4;
        let src = map.as_slice();

        let mut bgra = vec![0u8; row_bytes * height];
        for y in 0..height {
            let src_off = y * stride;
            let dst_off = y * row_bytes;
            let src_end = src_off + row_bytes;
            if src_end > src.len() {
                return None;
            }
            bgra[dst_off..dst_off + row_bytes].copy_from_slice(&src[src_off..src_end]);
        }

        Some(VideoFrame {
            width: info.width(),
            height: info.height(),
            bgra,
        })
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.playbin.set_state(gst::State::Null);
    }
}
