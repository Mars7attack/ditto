use bytemuck::{Pod, Zeroable};
use ditto::{
    core::{Asset, Cell, Color, Document},
    font, shader_gpu, shaders, typeface,
};
use std::{ops::Range, sync::Arc};
use wgpu::util::DeviceExt;
use winit::window::Window;
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.1 >= self.y && p.0 < self.x + self.w && p.1 < self.y + self.h
    }
    pub fn intersect(self, b: Self) -> Self {
        let x = self.x.max(b.x);
        let y = self.y.max(b.y);
        Self {
            x,
            y,
            w: ((self.x + self.w).min(b.x + b.w) - x).max(0.),
            h: ((self.y + self.h).min(b.y + b.h) - y).max(0.),
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
    alpha_filter: f32,
}
struct Batch {
    texture: u8,
    clip: Rect,
    range: Range<u32>,
}
pub struct Draw {
    vertices: Vec<Vertex>,
    batches: Vec<Batch>,
    pub width: f32,
    pub height: f32,
    pub clip: Rect,
}
impl Draw {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            vertices: Vec::new(),
            batches: Vec::new(),
            width,
            height,
            clip: Rect::new(0., 0., width, height),
        }
    }
    fn quad(&mut self, r: Rect, uv: [f32; 4], color: [f32; 4], texture: u8) {
        if r.w <= 0. || r.h <= 0. || self.clip.w <= 0. || self.clip.h <= 0. {
            return;
        }
        let start = self.vertices.len() as u32;
        let pos = |x: f32, y: f32| [x / self.width * 2. - 1., 1. - y / self.height * 2.];
        let v = |x, y, u, v| Vertex {
            position: pos(x, y),
            uv: [u, v],
            color,
            alpha_filter: if texture >= 2 { 1. } else { 0. },
        };
        let [u0, v0, u1, v1] = uv;
        self.vertices.extend_from_slice(&[
            v(r.x, r.y, u0, v0),
            v(r.x + r.w, r.y, u1, v0),
            v(r.x, r.y + r.h, u0, v1),
            v(r.x, r.y + r.h, u0, v1),
            v(r.x + r.w, r.y, u1, v0),
            v(r.x + r.w, r.y + r.h, u1, v1),
        ]);
        if let Some(b) = self.batches.last_mut()
            && b.texture == texture
            && b.clip.x == self.clip.x
            && b.clip.y == self.clip.y
            && b.clip.w == self.clip.w
            && b.clip.h == self.clip.h
        {
            b.range.end += 6;
            return;
        }
        self.batches.push(Batch {
            texture,
            clip: self.clip,
            range: start..start + 6,
        });
    }
    pub fn rect(&mut self, r: Rect, c: Color, a: f32) {
        let tx = 0.5 / typeface::ATLAS_WIDTH as f32;
        let ty = 0.5 / typeface::atlas_height() as f32;
        self.quad(
            r,
            [tx, ty, tx, ty],
            [
                c[0] as f32 / 255.,
                c[1] as f32 / 255.,
                c[2] as f32 / 255.,
                a,
            ],
            0,
        );
    }
    fn solid_triangle(&mut self, points: [[f32; 2]; 3], c: Color, alpha: [f32; 3]) {
        let start = self.vertices.len() as u32;
        for (p, a) in points.into_iter().zip(alpha) {
            self.vertices.push(Vertex {
                position: [p[0] / self.width * 2. - 1., 1. - p[1] / self.height * 2.],
                uv: [
                    0.5 / typeface::ATLAS_WIDTH as f32,
                    0.5 / typeface::atlas_height() as f32,
                ],
                color: [
                    c[0] as f32 / 255.,
                    c[1] as f32 / 255.,
                    c[2] as f32 / 255.,
                    a,
                ],
                alpha_filter: 0.,
            });
        }
        if let Some(b) = self.batches.last_mut()
            && b.texture == 0
            && b.clip.x == self.clip.x
            && b.clip.y == self.clip.y
            && b.clip.w == self.clip.w
            && b.clip.h == self.clip.h
        {
            b.range.end += 3;
        } else {
            self.batches.push(Batch {
                texture: 0,
                clip: self.clip,
                range: start..start + 3,
            });
        }
    }
    /// A continuous, feathered ribbon: guides need no glyph or art texture.
    pub fn polyline(&mut self, points: &[[f32; 2]], width: f32, c: Color, a: f32) {
        if points.is_empty() {
            return;
        }
        let radius = (width / 2.).max(0.35);
        if points.len() == 1 {
            let center = points[0];
            for i in 0..24 {
                let p = |j: usize, r: f32| {
                    let theta = j as f32 * std::f32::consts::TAU / 24.;
                    [center[0] + theta.cos() * r, center[1] + theta.sin() * r]
                };
                let x = p(i, radius);
                let y = p(i + 1, radius);
                let ox = p(i, radius + 0.65);
                let oy = p(i + 1, radius + 0.65);
                self.solid_triangle([center, x, y], c, [a; 3]);
                self.solid_triangle([x, ox, y], c, [a, 0., a]);
                self.solid_triangle([y, ox, oy], c, [a, 0., 0.]);
            }
            return;
        }
        let normal = |a: [f32; 2], b: [f32; 2]| {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = dx.hypot(dy).max(0.0001);
            [-dy / length, dx / length]
        };
        let sections: Vec<_> = points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let prev = normal(points[i.saturating_sub(1)], points[i]);
                let next = normal(points[i], points[(i + 1).min(points.len() - 1)]);
                let n = if i == 0 {
                    next
                } else if i + 1 == points.len() {
                    prev
                } else {
                    let length = (prev[0] + next[0]).hypot(prev[1] + next[1]);
                    if length < 0.01 {
                        next
                    } else {
                        [(prev[0] + next[0]) / length, (prev[1] + next[1]) / length]
                    }
                };
                let scale = if i == 0 || i + 1 == points.len() {
                    1.
                } else {
                    (n[0] * next[0] + n[1] * next[1]).abs().max(0.5).recip()
                };
                let offset = |r: f32| [p[0] + n[0] * r * scale, p[1] + n[1] * r * scale];
                [
                    offset(-radius - 0.65),
                    offset(-radius),
                    offset(radius),
                    offset(radius + 0.65),
                ]
            })
            .collect();
        for pair in sections.windows(2) {
            for j in 0..3 {
                let opacity = [0., a, a, 0.];
                self.solid_triangle(
                    [pair[0][j], pair[1][j], pair[0][j + 1]],
                    c,
                    [opacity[j], opacity[j], opacity[j + 1]],
                );
                self.solid_triangle(
                    [pair[0][j + 1], pair[1][j], pair[1][j + 1]],
                    c,
                    [opacity[j + 1], opacity[j], opacity[j + 1]],
                );
            }
        }
    }
    pub fn border(&mut self, r: Rect, c: Color) {
        self.rect(Rect::new(r.x, r.y, r.w, 1.), c, 1.);
        self.rect(Rect::new(r.x, r.y + r.h - 1., r.w, 1.), c, 1.);
        self.rect(Rect::new(r.x, r.y, 1., r.h), c, 1.);
        self.rect(Rect::new(r.x + r.w - 1., r.y, 1., r.h), c, 1.);
    }
    pub fn glyph(&mut self, g: font::Glyph, r: Rect, c: Color, a: f32) {
        if g == 0 || g == 32 || g == 255 {
            return;
        }
        self.quad(
            r,
            typeface::uv(g),
            [
                c[0] as f32 / 255.,
                c[1] as f32 / 255.,
                c[2] as f32 / 255.,
                a,
            ],
            0,
        );
    }
    pub fn text(&mut self, x: f32, y: f32, text: &str, c: Color, scale: f32) {
        for (i, ch) in text.chars().enumerate() {
            let g = typeface::ui_glyph(ch);
            self.glyph(
                g,
                Rect::new(x + i as f32 * 8. * scale, y, 8. * scale, 16. * scale),
                c,
                1.,
            );
        }
    }
    pub fn image(&mut self, r: Rect, a: f32) {
        self.quad(r, [0., 0., 1., 1.], [1., 1., 1., a], 1);
    }
    pub fn artwork(&mut self, r: Rect, a: f32, original: bool) {
        self.quad(
            r,
            [0., 0., 1., 1.],
            [1., 1., 1., a],
            if original { 3 } else { 2 },
        );
    }
}
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    atlas: wgpu::BindGroup,
    reference: wgpu::BindGroup,
    ref_asset: Option<Arc<Asset>>,
    effects: shader_gpu::Pipeline,
    artwork: Option<wgpu::BindGroup>,
    original: Option<wgpu::BindGroup>,
    source_cells: Option<Arc<Vec<Cell>>>,
    source_image: Option<image::RgbaImage>,
    stack: Option<shaders::Stack>,
    pub capture_next: Option<std::path::PathBuf>,
    pub capture_error: Option<String>,
    pub adapter_name: String,
}
impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL | wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Aucun adaptateur graphique compatible"))?;
        let adapter_name = format!(
            "{} / {:?}",
            adapter.get_info().name,
            adapter.get_info().backend
        );
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ditto"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await?;
        let caps = surface.get_capabilities(&adapter);
        let format = *caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .unwrap_or(&caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | (caps.usages & wgpu::TextureUsages::COPY_SRC),
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Textures"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pixels = typeface::atlas_pixels();
        let atlas = Self::texture(
            &device,
            &queue,
            &layout,
            typeface::ATLAS_WIDTH,
            typeface::atlas_height(),
            &pixels,
        );
        let reference = Self::texture(&device, &queue, &layout, 1, 1, &[0, 0, 0, 0]);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terminal glyph renderer"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let constants = std::collections::HashMap::from([(
            "SURFACE_SRGB".to_string(),
            if format.is_srgb() { 1.0 } else { 0.0 },
        )]);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ditto GPU"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Float32x4,3=>Float32],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let effects = shader_gpu::Pipeline::new(&device);
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            layout,
            atlas,
            reference,
            ref_asset: None,
            effects,
            artwork: None,
            original: None,
            source_cells: None,
            source_image: None,
            stack: None,
            capture_next: None,
            capture_error: None,
            adapter_name,
        })
    }
    fn texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> wgpu::BindGroup {
        let t = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph/image texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &t,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = t.create_view(&Default::default());
        Self::bind_texture(device, layout, &view)
    }
    fn bind_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
        }
    }
    pub fn set_artwork(&mut self, doc: &Document, needed: bool) -> anyhow::Result<()> {
        if !needed {
            self.artwork = None;
            self.original = None;
            self.source_cells = None;
            self.source_image = None;
            self.stack = None;
            return Ok(());
        }
        let scale = shader_gpu::preview_scale(
            doc.width,
            doc.height,
            self.device.limits().max_texture_dimension_2d,
        )?;
        let source_changed = self
            .source_cells
            .as_ref()
            .is_none_or(|c| !Arc::ptr_eq(c, &doc.cells))
            || self.source_image.as_ref().is_none_or(|i| {
                i.width() != doc.width * typeface::CELL_WIDTH * scale
                    || i.height() != doc.height * typeface::CELL_HEIGHT * scale
            });
        if source_changed {
            self.source_image = Some(ditto::project::render_cells(doc, scale, None)?);
            let im = self.source_image.as_ref().unwrap();
            self.original = Some(Self::texture(
                &self.device,
                &self.queue,
                &self.layout,
                im.width(),
                im.height(),
                im.as_raw(),
            ));
            self.source_cells = Some(doc.cells.clone());
        }
        if source_changed || self.stack.as_ref() != Some(&doc.shaders) {
            self.artwork = None;
            if doc.shaders.active() {
                let mut frame = self.effects.upload(
                    &self.device,
                    &self.queue,
                    self.source_image.as_ref().unwrap(),
                )?;
                self.effects.process(
                    &self.device,
                    &self.queue,
                    &mut frame,
                    &doc.shaders,
                    scale as f32,
                )?;
                self.artwork = Some(Self::bind_texture(&self.device, &self.layout, frame.view()));
            }
            self.stack = Some(doc.shaders.clone());
        }
        Ok(())
    }
    pub fn set_reference(&mut self, asset: Option<&Arc<Asset>>) {
        match (asset, &self.ref_asset) {
            (Some(a), Some(b)) if Arc::ptr_eq(a, b) => return,
            (None, None) => return,
            _ => {}
        }
        if let Some(a) = asset {
            self.reference = Self::texture(
                &self.device,
                &self.queue,
                &self.layout,
                a.width,
                a.height,
                &a.rgba,
            );
        }
        self.ref_asset = asset.cloned();
    }
    pub fn draw(&mut self, draw: Draw, scale: f32) -> Result<(), wgpu::SurfaceError> {
        let surface = match self.surface.get_current_texture() {
            Ok(s) => s,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture()?
            }
            Err(e) => return Err(e),
        };
        let view = surface.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let vb = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Frame vertices"),
                contents: bytemuck::cast_slice(&draw.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Terminal"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vb.slice(..));
            for b in draw.batches {
                let x = (b.clip.x.max(0.) * scale).round() as u32;
                let y = (b.clip.y.max(0.) * scale).round() as u32;
                let endx = ((b.clip.x + b.clip.w) * scale).round().max(0.) as u32;
                let endy = ((b.clip.y + b.clip.h) * scale).round().max(0.) as u32;
                let w = endx.min(self.config.width).saturating_sub(x);
                let h = endy.min(self.config.height).saturating_sub(y);
                if w == 0 || h == 0 {
                    continue;
                }
                pass.set_scissor_rect(x, y, w, h);
                pass.set_bind_group(
                    0,
                    match b.texture {
                        1 => &self.reference,
                        2 => self
                            .artwork
                            .as_ref()
                            .or(self.original.as_ref())
                            .unwrap_or(&self.reference),
                        3 => self.original.as_ref().unwrap_or(&self.reference),
                        _ => &self.atlas,
                    },
                    &[],
                );
                pass.draw(b.range, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        if let Some(path) = self.capture_next.take() {
            self.capture_error = self
                .capture(&surface.texture, &path)
                .err()
                .map(|e| format!("{e:#}"));
        }
        surface.present();
        Ok(())
    }
    fn capture(&self, texture: &wgpu::Texture, path: &std::path::Path) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.config.usage.contains(wgpu::TextureUsages::COPY_SRC),
            "Capture GPU non prise en charge"
        );
        let (width, height) = (self.config.width, self.config.height);
        let stride = (width * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Smoke frame readback"),
            size: stride as u64 * height as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()??;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut image = image::RgbaImage::new(width, height);
        for (src, dst) in mapped
            .chunks(stride as usize)
            .zip(image.as_mut().chunks_mut(width as usize * 4))
        {
            dst.copy_from_slice(&src[..dst.len()]);
        }
        drop(mapped);
        buffer.unmap();
        if matches!(
            self.config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        ) {
            for pixel in image.pixels_mut() {
                pixel.0.swap(0, 2);
            }
        }
        image.save(path)?;
        Ok(())
    }
}
