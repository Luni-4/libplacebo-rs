use crate::vulkan::*;
use crate::*;

use libplacebo_sys::*;

use std::default::Default;
use std::ptr::{null, null_mut};
use std::ffi::CString;
use std::mem::MaybeUninit;

create_enum!(
    FmtType,
    pl_fmt_type,
    (
        FMT_UNKNOWN,
        FMT_UNORM,
        FMT_SNORM,
        FMT_UINT,
        FMT_SINT,
        FMT_FLOAT,
        FMT_TYPE_COUNT,
    )
);

create_enum!(
    FmtCaps,
    pl_fmt_caps,
    (
        FMT_CAP_SAMPLEABLE,
        FMT_CAP_STORABLE,
        FMT_CAP_LINEAR,
        FMT_CAP_RENDERABLE,
        FMT_CAP_BLENDABLE,
        FMT_CAP_BLITTABLE,
        FMT_CAP_VERTEX,
        FMT_CAP_TEXEL_UNIFORM,
        FMT_CAP_TEXEL_STORAGE,
    )
);

simple_enum!(
    BufType,
    (
        BUF_INVALID,
        BUF_TEX_TRANSFER,
        BUF_UNIFORM,
        BUF_STORAGE,
        BUF_TEXEL_UNIFORM,
        BUF_TEXEL_STORAGE,
        BUF_PRIVATE,
        BUF_TYPE_COUNT,
    )
);

simple_enum!(
    HandleType,
    (HANDLE_FD, HANDLE_WIN32, HANDLE_WIN32_KMT, HANDLE_DMA_BUF)
);

simple_enum!(BufMemType, (BUF_MEM_AUTO, BUF_MEM_HOST, BUF_MEM_DEVICE));

create_enum!(SampleMode, pl_tex_sample_mode,
             (TEX_SAMPLE_NEAREST, TEX_SAMPLE_LINEAR)
);

create_enum!(AddressMode, pl_tex_address_mode,
             (TEX_ADDRESS_CLAMP, TEX_ADDRESS_REPEAT, TEX_ADDRESS_MIRROR)
);

#[derive(Clone)]
pub struct Gpu {
    gpu: *const pl_gpu,
}

impl Gpu {
    pub fn new(vk: &Vulkan) -> Self {
        let gpu = unsafe { (*vk.as_ptr()).gpu };
        Gpu { gpu }
    }

    pub fn gpu_flush(&self) {
        unsafe {
            pl_gpu_flush(self.gpu);
        }
    }

    pub fn gpu_finish(&self) {
        unsafe {
            pl_gpu_finish(self.gpu);
        }
    }

    pub fn pass_run(&self, params: &pl_pass_run_params) {
        unsafe {
            pl_pass_run(self.gpu, params);
        }
    }

    pub(crate) fn as_ptr(&self) -> *const pl_gpu {
        self.gpu
    }
}

pub struct Fmt {
    fmt: *const pl_fmt,
}

impl Fmt {
    pub fn find_fmt(gpu: &Gpu, type_: FmtType, num_components: usize, min_depth: usize, host_bits: usize, caps: FmtCaps) -> Self {
        let fmt = unsafe {
            pl_find_fmt(gpu.gpu, FmtType::to_pl_fmt_type(&type_), num_components as i32, min_depth as i32, host_bits as i32, FmtCaps::to_pl_fmt_caps(&caps))
        };
        Fmt { fmt }
    }

    pub fn find_vertex_fmt(gpu: &Gpu, type_: FmtType, num_components: usize) -> Self {
        let fmt = unsafe {
            pl_find_vertex_fmt(gpu.gpu, FmtType::to_pl_fmt_type(&type_), num_components as i32)
        };
        Fmt { fmt }
    }

    pub fn find_named_fmt(gpu: &Gpu, name: &str) -> Self {
        let source = CString::new(name).unwrap();
        let fmt = unsafe {
            pl_find_named_fmt(gpu.gpu, source.as_ptr())
        };
        Fmt { fmt }
    }

    pub fn is_ordered(&self) -> bool {
        unsafe { pl_fmt_is_ordered(self.fmt) }
    }

    pub(crate) fn create_struct(fmt: *const pl_fmt) -> Self {
        Fmt { fmt }
    }
}

set_struct!(BufParams, buf_params, pl_buf_params);

impl Default for BufParams {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let handle_type = pl_handle_type::PL_HANDLE_WIN32;
        #[cfg(target_os = "linux")]
        let handle_type = pl_handle_type::PL_HANDLE_FD;

        let buf_params = pl_buf_params {
            type_: pl_buf_type::PL_BUF_INVALID,
            size: 0,
            host_mapped: false,
            host_writable: false,
            host_readable: false,
            memory_type: pl_buf_mem_type::PL_BUF_MEM_AUTO,
            format: null(),
            handle_type,
            initial_data: null(),
            user_data: null_mut(),
        };
        BufParams { buf_params }
    }
}

pub struct Buf {
    buf: *const pl_buf,
    gpu: *const pl_gpu,
}

impl Buf {
    pub fn new(gpu: &Gpu, params: &BufParams) -> Self {
        let buf = unsafe { pl_buf_create(gpu.gpu, &params.buf_params) };
        assert!(!buf.is_null());
        Buf { buf, gpu: gpu.gpu }
    }

    pub(crate) fn as_ptr(&self) -> *const pl_buf {
        self.buf
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe {
            pl_buf_destroy(self.gpu, &mut self.buf);
        }
    }
}

set_struct!(TexParams, tex_params, pl_tex_params);

impl Default for TexParams {
    fn default() -> Self {
        let tex_params_u: MaybeUninit<pl_tex_params> = MaybeUninit::uninit();
        // FIXME This is an horrible hack that should be fixed in some way eventually
        let tex_params = unsafe { tex_params_u.assume_init() };
        TexParams { tex_params }
    }
}

set_params!(TexParams, tex_params,
            (
                w, h, d,
                format,
                sampleable,
                renderable,
                storable,
                blit_src,
                blit_dst,
                host_writable,
                host_readable,
                sample_mode,
                address_mode,
            ),
            (
                usize, usize, usize,
                &Fmt,
                bool,
                bool,
                bool,
                bool,
                bool,
                bool,
                bool,
                &SampleMode,
                &AddressMode,
            ),
            (
                w as i32, h as i32, d as i32,
                format.fmt,
                sampleable as bool,
                renderable as bool,
                storable as bool,
                blit_src as bool,
                blit_dst as bool,
                host_writable as bool,
                host_readable as bool,
                SampleMode::to_pl_tex_sample_mode(sample_mode),
                AddressMode::to_pl_tex_address_mode(address_mode),
            )
);

pub struct Tex {
    tex: *const pl_tex,
    gpu: *const pl_gpu,
}

impl Tex {
    pub fn default(gpu: &Gpu) -> Self {
        Tex {
            tex: null(),
            gpu: gpu.gpu,
        }
    }

    pub fn new(gpu: &Gpu, params: &TexParams) -> Self {
        let tex = unsafe { pl_tex_create(gpu.gpu, &params.tex_params) };
        assert!(!tex.is_null());

        Tex { tex, gpu: gpu.gpu }
    }

    pub fn tex_recreate(&mut self, params: &TexParams) -> bool {
        unsafe { pl_tex_recreate(self.gpu, &mut self.tex, &params.tex_params) }
    }

    pub fn tex_invalidate(&mut self) {
        unsafe { pl_tex_invalidate(self.gpu, self.tex) };
    }

    // TODO We need to test these ones because surely they hide some memory errors
    /*pub fn tex_clear(&mut self, color: &f32) {
        unsafe { pl_tex_clear(self.gpu.gpu, self.tex, color) };
    }*/

    /*pub fn tex_blit(&self, dst: &Tex, src: &Tex, dst_src: &Rect3D, src_rc: &Rect3D) {
        unsafe {
            pl_tex_blit(
                self.gpu.gpu,
                dst.tex,
                src.tex,
                dst_src.internal_object(),
                src_rc.internal_object(),
            )
        };
    }*/

    /*
    pub fn tex_upload(&self, params: &pl_tex_transfer_params) -> bool {
        unsafe { pl_tex_upload(self.gpu.gpu, params) as bool }
    }

    pub fn tex_download(&self, params: &pl_tex_transfer_params) -> bool {
        unsafe { pl_tex_download(self.gpu.gpu, params) as bool }
    }

    pub fn tex_export(&self, sync: &Sync) -> bool {
        unsafe { pl_tex_export(self.gpu.gpu, self.tex, sync.sync) }
    } */

    pub(crate) fn as_ptr(&self) -> *const pl_tex {
        self.tex
    }

    pub(crate) fn set_ptr(&mut self, ptr: *const pl_tex) {
        self.tex = ptr;
    }
}

impl Drop for Tex {
    fn drop(&mut self) {
        unsafe {
            pl_tex_destroy(self.gpu, &mut self.tex);
        }
    }
}
