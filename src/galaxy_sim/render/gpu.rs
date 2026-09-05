//! Backend GPU (wgpu compute) — diadaptasi dari `andromeda-simulation-tui`'s
//! `src/render/gpu.rs`, TAPI error handling pakai `String` biasa (bukan `anyhow`, dep lbh
//! ringan — konsisten `SimBackend::render`'s signature `Result<&[u8],String>`, sama pola
//! `cpu.rs`). Dua pipeline compute (accumulate+tonemap, `shader.wgsl`) lalu readback buffer
//! RGBA kecil tiap frame. `probe()` HARUS graceful fallback CPU headless (sandbox/CI TAK
//! PY GPU nyata) — pola SAMA `GalaxyPixel::detect_falls_back_safely_in_headless_test_env`.

use super::SimBackend;
use crate::galaxy_sim::model::{StarGpu, Uniforms};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: wgpu::AdapterInfo,
}

/// Cari adapter GPU sungguhan (headless -- tanpa surface). Rasterizer software
/// (llvmpipe/lavapipe/SwiftShader) DITOLAK: rayon (Phase 3) lebih cepat drpd GPU emulasi.
pub fn probe() -> Result<GpuContext, String> {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    if wgpu::Backends::from_env().is_none() {
        desc.backends = wgpu::Backends::PRIMARY | wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        ..Default::default()
    }))
    .map_err(|e| format!("tidak ada adapter GPU: {e}"))?;

    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu {
        return Err(format!("hanya adapter software ({})", info.name));
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("galaxy-idle-galaxy_sim-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        ..Default::default()
    }))
    .map_err(|e| format!("request_device gagal: {e}"))?;

    Ok(GpuContext {
        device,
        queue,
        info,
    })
}

/// Backend GPU: dua pipeline compute (accumulate + tonemap) lalu readback buffer RGBA kecil
/// (~beberapa puluh KB, viewport TUI jauh lbh kecil drpd reference's 400k bintang/layar desktop).
pub struct GpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    label: String,
    pipeline_accumulate: wgpu::ComputePipeline,
    pipeline_tonemap: wgpu::ComputePipeline,
    bind_layout: wgpu::BindGroupLayout,
    uniform_buf: wgpu::Buffer,
    star_buf: wgpu::Buffer,
    star_count: u32,
    // Bergantung ukuran viewport (dibuat ulang di resize):
    bind_group: Option<wgpu::BindGroup>,
    accum_buf: Option<wgpu::Buffer>,
    out_buf: Option<wgpu::Buffer>,
    staging_buf: Option<wgpu::Buffer>,
    w: u32,
    h: u32,
    out_vec: Vec<u8>,
}

fn buffer_layout_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl GpuBackend {
    pub fn new(ctx: GpuContext, stars: &[StarGpu]) -> Result<Self, String> {
        let GpuContext {
            device,
            queue,
            info,
        } = ctx;
        // Tangkap error validasi (default handler wgpu = panic).
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("galaxy-idle-galaxy_sim-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("galaxy-idle-galaxy_sim-bgl"),
            entries: &[
                buffer_layout_entry(0, wgpu::BufferBindingType::Uniform),
                buffer_layout_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(2, wgpu::BufferBindingType::Storage { read_only: false }),
                buffer_layout_entry(3, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("galaxy-idle-galaxy_sim-pl"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let make_pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let pipeline_accumulate = make_pipeline("accumulate");
        let pipeline_tonemap = make_pipeline("tonemap");

        let star_bytes: &[u8] = bytemuck::cast_slice(stars);
        let star_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stars"),
            size: star_bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&star_buf, 0, star_bytes);

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        if let Some(err) = pollster::block_on(error_scope.pop()) {
            return Err(format!("validasi wgpu gagal: {err}"));
        }

        let label = format!("GPU: {} ({:?})", info.name, info.backend);
        Ok(Self {
            device,
            queue,
            label,
            pipeline_accumulate,
            pipeline_tonemap,
            bind_layout,
            uniform_buf,
            star_buf,
            star_count: stars.len() as u32,
            bind_group: None,
            accum_buf: None,
            out_buf: None,
            staging_buf: None,
            w: 0,
            h: 0,
            out_vec: Vec::new(),
        })
    }

    /// Bangun ulang bind group dr buffer saat ini (dipanggil stlh resize atau stlh buffer
    /// bintang dibuat ulang). No-op sblm resize pertama.
    fn rebuild_bind_group(&mut self) {
        let (Some(accum_buf), Some(out_buf)) = (&self.accum_buf, &self.out_buf) else {
            return;
        };
        self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("galaxy-idle-galaxy_sim-bg"),
            layout: &self.bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.star_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: accum_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: out_buf.as_entire_binding(),
                },
            ],
        }));
    }
}

impl SimBackend for GpuBackend {
    fn name(&self) -> String {
        self.label.clone()
    }

    fn resize(&mut self, w: u32, h: u32) {
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        let px = (w * h) as u64;
        // wgpu meng-nol-kan buffer baru -- akumulasi mulai bersih.
        let accum_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("accum"),
            size: px * 4 * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let out_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out-rgba"),
            size: px * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: px * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.accum_buf = Some(accum_buf);
        self.out_buf = Some(out_buf);
        self.staging_buf = Some(staging_buf);
        self.out_vec = vec![0; (px * 4) as usize];
        self.rebuild_bind_group();
    }

    fn set_stars(&mut self, stars: &[StarGpu]) {
        let bytes: &[u8] = bytemuck::cast_slice(stars);
        self.star_count = stars.len() as u32;
        if bytes.len() as u64 == self.star_buf.size() {
            self.queue.write_buffer(&self.star_buf, 0, bytes);
        } else {
            self.star_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("stars"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&self.star_buf, 0, bytes);
            self.rebuild_bind_group();
        }
    }

    fn render(&mut self, u: &Uniforms) -> Result<&[u8], String> {
        let bind_group = self
            .bind_group
            .as_ref()
            .ok_or_else(|| "render dipanggil sblm resize".to_string())?;
        let out_buf = self.out_buf.as_ref().unwrap();
        let staging = self.staging_buf.as_ref().unwrap();
        let size = (self.w * self.h * 4) as u64;

        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(u));

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        // Dua pass terpisah: batas pass menjamin sinkronisasi accumulate -> tonemap.
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline_accumulate);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(self.star_count.div_ceil(256), 1, 1);
        }
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline_tonemap);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups((self.w * self.h).div_ceil(256), 1, 1);
        }
        enc.copy_buffer_to_buffer(out_buf, 0, staging, 0, size);
        self.queue.submit([enc.finish()]);

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("device poll gagal: {e:?}"))?;
        rx.recv()
            .map_err(|_| "callback map_async hilang".to_string())?
            .map_err(|e| format!("map buffer gagal: {e:?}"))?;
        {
            let view = slice
                .get_mapped_range()
                .map_err(|e| format!("get_mapped_range: {e:?}"))?;
            self.out_vec.copy_from_slice(&view);
        }
        staging.unmap();

        Ok(&self.out_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `probe()` di sandbox/CI headless ini HARUS genuinely `Err` (tak ada GPU nyata) --
    /// dites LANGSUNG, bukan diasumsikan. Ini KUNCI: `create_backend` HARUS jatuh ke CPU
    /// tanpa panic bila ini terjadi (dites di `mod.rs`'s `create_backend_falls_back_to_cpu_
    /// when_gpu_unavailable`).
    #[test]
    fn probe_fails_gracefully_in_headless_sandbox_without_panicking() {
        // Sengaja TAK unwrap -- tujuan test ini PRESISI: pastikan `probe()` mengembalikan Err
        // dgn PESAN (bukan panic) di lingkungan tanpa GPU asli, dites LANGSUNG bukan diasumsikan.
        let result = probe();
        if let Err(reason) = &result {
            assert!(
                !reason.is_empty(),
                "alasan fallback HARUS genuinely tak kosong"
            );
        }
        // Catatan: bila sandbox ini SUATU SAAT genuinely py GPU asli (mis. CI beda), `probe()`
        // boleh Ok -- test ini HANYA membuktikan tak panic, bukan memaksa hasil tertentu.
    }
}
