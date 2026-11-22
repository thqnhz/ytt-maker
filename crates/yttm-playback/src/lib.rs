use std::ffi::{CString, c_void};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result, anyhow};
use libmpv2::Mpv;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewQuality {
    Full,
    Half,
    #[default]
    Quarter,
    Eighth,
}

impl PreviewQuality {
    pub const ALL: [Self; 4] = [Self::Full, Self::Half, Self::Quarter, Self::Eighth];

    pub fn label(self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Half => "1/2",
            Self::Quarter => "1/4",
            Self::Eighth => "1/8",
        }
    }

    pub fn divisor(self) -> u32 {
        match self {
            Self::Full => 1,
            Self::Half => 2,
            Self::Quarter => 4,
            Self::Eighth => 8,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VideoState {
    pub path: Option<PathBuf>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub playing: bool,
    pub preview_quality: PreviewQuality,
}

impl VideoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, path: PathBuf) {
        let quality = self.preview_quality;
        *self = Self::default();
        self.preview_quality = quality;
        self.path = Some(path);
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

struct Inner {
    mpv: Mpv,
    render: *mut libmpv2_sys::mpv_render_context,
    source_dims: Mutex<Option<(u32, u32)>>,
    quality: Mutex<PreviewQuality>,
    update_flag: Arc<AtomicBool>,
    cb_ctx: *const AtomicBool,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            if !self.render.is_null() {
                libmpv2_sys::mpv_render_context_free(self.render);
                self.render = std::ptr::null_mut();
            }
            if !self.cb_ctx.is_null() {
                drop(Arc::from_raw(self.cb_ctx));
                self.cb_ctx = std::ptr::null();
            }
        }
        let _ = self.mpv.command("stop", &[]);
    }
}

pub struct Player {
    inner: Arc<Inner>,
}

impl Player {
    pub fn new() -> Result<Self> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv")?;
            init.set_property("hwdec", "auto-safe")?;
            init.set_property("keep-open", "yes")?;
            init.set_property("idle", "yes")?;
            Ok(())
        })
        .context("failed to initialize mpv")?;

        let update_flag = Arc::new(AtomicBool::new(true));
        let flag_cb = Arc::clone(&update_flag);
        let (render, cb_ctx) = unsafe { create_sw_render_context(mpv.ctx.as_ptr(), flag_cb)? };

        let inner = Arc::new(Inner {
            mpv,
            render,
            source_dims: Mutex::new(None),
            quality: Mutex::new(PreviewQuality::default()),
            update_flag,
            cb_ctx,
        });

        Ok(Self { inner })
    }

    pub fn open(&self, path: &Path) -> Result<()> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow!("video path is not valid utf-8"))?;
        self.inner
            .mpv
            .command("loadfile", &[path, "replace"])
            .context("failed to load video")?;
        self.inner
            .mpv
            .set_property("pause", true)
            .context("failed to pause after load")?;
        *self.inner.source_dims.lock().expect("source_dims poisoned") = None;
        self.inner.update_flag.store(true, Ordering::Release);
        Ok(())
    }

    pub fn set_preview_quality(&self, quality: PreviewQuality) -> Result<()> {
        *self.inner.quality.lock().expect("quality poisoned") = quality;
        self.inner.update_flag.store(true, Ordering::Release);
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.inner
            .mpv
            .set_property("pause", false)
            .context("failed to start playback")?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.inner
            .mpv
            .set_property("pause", true)
            .context("failed to pause playback")?;
        Ok(())
    }

    pub fn position_ms(&self) -> Option<u64> {
        self.inner
            .mpv
            .get_property::<f64>("time-pos")
            .ok()
            .map(|s| (s.max(0.0) * 1000.0) as u64)
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.inner
            .mpv
            .get_property::<f64>("duration")
            .ok()
            .map(|s| (s.max(0.0) * 1000.0) as u64)
    }

    pub fn seek_ms(&self, position_ms: u64) -> Result<()> {
        let seconds = position_ms as f64 / 1000.0;
        self.inner
            .mpv
            .command("seek", &[&seconds.to_string(), "absolute+exact"])
            .context("failed to seek video")?;
        self.inner.update_flag.store(true, Ordering::Release);
        Ok(())
    }

    pub fn latest_frame(&self) -> Option<VideoFrame> {
        if !self.inner.update_flag.swap(false, Ordering::AcqRel) {
            return None;
        }
        let flags = unsafe { libmpv2_sys::mpv_render_context_update(self.inner.render) };
        if (flags & libmpv2_sys::mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME as u64) == 0 {
            return None;
        }
        let (sw, sh) = self.source_dims()?;
        let quality = *self.inner.quality.lock().expect("quality poisoned");
        let (w, h) = scaled_dims(sw, sh, quality);
        let stride = (w as usize) * 4;
        let mut bgra = vec![0u8; stride * (h as usize)];

        let mut size: [i32; 2] = [w as i32, h as i32];
        let format_c = CString::new("bgra").ok()?;
        let mut stride_val: usize = stride;

        let params = [
            libmpv2_sys::mpv_render_param {
                type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr() as *mut c_void,
            },
            libmpv2_sys::mpv_render_param {
                type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
                data: format_c.as_ptr() as *mut c_void,
            },
            libmpv2_sys::mpv_render_param {
                type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE,
                data: &mut stride_val as *mut usize as *mut c_void,
            },
            libmpv2_sys::mpv_render_param {
                type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
                data: bgra.as_mut_ptr() as *mut c_void,
            },
            libmpv2_sys::mpv_render_param {
                type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: std::ptr::null_mut(),
            },
        ];

        let ret = unsafe {
            libmpv2_sys::mpv_render_context_render(self.inner.render, params.as_ptr() as *mut _)
        };
        if ret < 0 {
            self.inner.update_flag.store(true, Ordering::Release);
            return None;
        }

        Some(VideoFrame {
            width: w,
            height: h,
            bgra,
        })
    }

    pub fn request_render(&self) {
        self.inner.update_flag.store(true, Ordering::Release);
    }

    pub fn has_frame_update(&self) -> bool {
        self.inner.update_flag.load(Ordering::Acquire)
    }

    fn source_dims(&self) -> Option<(u32, u32)> {
        if let Some(dims) = *self.inner.source_dims.lock().expect("source_dims poisoned") {
            return Some(dims);
        }
        let w = self.inner.mpv.get_property::<i64>("width").ok()?;
        let h = self.inner.mpv.get_property::<i64>("height").ok()?;
        if w <= 0 || h <= 0 {
            return None;
        }
        let dims = (w as u32, h as u32);
        *self.inner.source_dims.lock().expect("source_dims poisoned") = Some(dims);
        Some(dims)
    }
}

fn scaled_dims(width: u32, height: u32, quality: PreviewQuality) -> (u32, u32) {
    let divisor = quality.divisor().max(1);
    let w = (width / divisor).max(1);
    let h = (height / divisor).max(1);
    (w, h)
}

unsafe extern "C" fn update_cb(ctx: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let flag = unsafe { &*(ctx as *const AtomicBool) };
    flag.store(true, Ordering::Release);
}

unsafe fn create_sw_render_context(
    mpv: *mut libmpv2_sys::mpv_handle,
    flag: Arc<AtomicBool>,
) -> Result<(*mut libmpv2_sys::mpv_render_context, *const AtomicBool)> {
    let api_type = CString::new("sw").unwrap();
    let params = [
        libmpv2_sys::mpv_render_param {
            type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
            data: api_type.as_ptr() as *mut c_void,
        },
        libmpv2_sys::mpv_render_param {
            type_: libmpv2_sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
            data: std::ptr::null_mut(),
        },
    ];

    let mut ctx: *mut libmpv2_sys::mpv_render_context = std::ptr::null_mut();
    let err =
        unsafe { libmpv2_sys::mpv_render_context_create(&mut ctx, mpv, params.as_ptr() as *mut _) };
    if err < 0 {
        return Err(anyhow!("mpv_render_context_create failed: {err}"));
    }

    let cb_ctx = Arc::into_raw(flag);
    unsafe {
        libmpv2_sys::mpv_render_context_set_update_callback(
            ctx,
            Some(update_cb),
            cb_ctx as *mut c_void,
        );
    }
    Ok((ctx, cb_ctx))
}
