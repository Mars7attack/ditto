//! The exact same WGSL pipeline is used for the live preview and PNG export.
use crate::shaders::{Kind, Layer, Stack};
use anyhow::{Context, Result, ensure};
use bytemuck::{Pod, Zeroable};
use image::RgbaImage;
use wgpu::util::DeviceExt;

/// Preserve the full font atlas detail before applying effects. Bound intermediate
/// textures on large drawings, retaining the previous 1x capacity as a fallback.
pub fn preview_scale(width: u32, height: u32, max_dimension: u32) -> Result<u32> {
    let w = u64::from(width) * u64::from(crate::typeface::CELL_WIDTH);
    let h = u64::from(height) * u64::from(crate::typeface::CELL_HEIGHT);
    ensure!(
        w > 0
            && h > 0
            && w <= u64::from(max_dimension)
            && h <= u64::from(max_dimension)
            && w * h <= 64 * 1024 * 1024,
        "Dimensions trop grandes pour les shaders sur ce GPU."
    );
    Ok([4, 2, 1]
        .into_iter()
        .find(|&scale| {
            w * u64::from(scale) <= u64::from(max_dimension)
                && h * u64::from(scale) <= u64::from(max_dimension)
                && w * h * u64::from(scale * scale) <= 16 * 1024 * 1024
        })
        .unwrap_or(1))
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Settings {
    control: [f32; 4],
    params: [f32; 4],
    dark: [f32; 4],
    light: [f32; 4],
}
pub struct Pipeline {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}
pub struct Frame {
    textures: [wgpu::Texture; 3],
    views: [wgpu::TextureView; 3],
    pub output: usize,
    pub width: u32,
    pub height: u32,
}
impl Frame {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.views[self.output]
    }
}
impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shader stack inputs"),
            entries: &[
                texture(0),
                texture(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ditto effects WGSL"),
            source: wgpu::ShaderSource::Wgsl(include_str!("effects.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader stack"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shader stack"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Self { layout, pipeline }
    }
    pub fn upload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &RgbaImage,
    ) -> Result<Frame> {
        let (width, height) = image.dimensions();
        ensure!(
            width > 0
                && height > 0
                && width <= device.limits().max_texture_dimension_2d
                && height <= device.limits().max_texture_dimension_2d,
            "Dimensions trop grandes pour les shaders sur ce GPU."
        );
        let textures = std::array::from_fn(|_| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Shader artwork"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        });
        let views = std::array::from_fn(|i| textures[i].create_view(&Default::default()));
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &textures[0],
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_raw(),
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
        Ok(Frame {
            textures,
            views,
            output: 0,
            width,
            height,
        })
    }
    /// Each frame starts from a fresh unprocessed upload, never from last frame's result.
    pub fn process(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &mut Frame,
        stack: &Stack,
        scale: f32,
    ) -> Result<()> {
        stack.validate()?;
        ensure!(scale.is_finite() && scale > 0., "Échelle shader invalide.");
        let mut encoder = device.create_command_encoder(&Default::default());
        let mut source = 0;
        if stack.enabled {
            for layer in stack.layers.iter().filter(|l| l.enabled && l.mix > 0.) {
                if matches!(layer.kind, Kind::Blur | Kind::ContourBlur)
                    && (layer.params[0] == 0.
                        || (layer.kind == Kind::Blur
                            && layer.params[1] == 0.
                            && layer.params[2] == 0.))
                {
                    continue;
                }
                let target = (source + 1) % 3;
                if layer.kind == Kind::Glow {
                    let vertical = (source + 2) % 3;
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [source, source, target],
                        layer,
                        [9., 1., scale, 0.],
                    );
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [target, target, vertical],
                        layer,
                        [10., 1., scale, 0.],
                    );
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [vertical, source, target],
                        layer,
                        [0., layer.mix, scale, 0.],
                    );
                } else if matches!(layer.kind, Kind::Blur | Kind::ContourBlur) {
                    let vertical = (source + 2) % 3;
                    let (x, y) = if layer.kind == Kind::Blur {
                        (
                            layer.params[0] * layer.params[1],
                            layer.params[0] * layer.params[2],
                        )
                    } else {
                        (layer.params[0], layer.params[0])
                    };
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [source, source, target],
                        layer,
                        [13., 1., scale, x],
                    );
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [target, target, vertical],
                        layer,
                        [14., 1., scale, y],
                    );
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [vertical, source, target],
                        layer,
                        [layer.kind as u32 as f32, layer.mix, scale, 0.],
                    );
                } else {
                    self.pass(
                        device,
                        &mut encoder,
                        frame,
                        [source, source, target],
                        layer,
                        [layer.kind as u32 as f32, layer.mix, scale, 0.],
                    );
                }
                source = target;
            }
        }
        frame.output = source;
        queue.submit(Some(encoder.finish()));
        Ok(())
    }
    fn pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
        slots: [usize; 3],
        layer: &Layer,
        control: [f32; 4],
    ) {
        let color = |c: [u8; 3]| {
            [
                c[0] as f32 / 255.,
                c[1] as f32 / 255.,
                c[2] as f32 / 255.,
                1.,
            ]
        };
        let settings = Settings {
            control,
            params: [layer.params[0], layer.params[1], layer.params[2], 0.],
            dark: color(layer.colors[0]),
            light: color(layer.colors[1]),
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader parameters"),
            contents: bytemuck::bytes_of(&settings),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&frame.views[slots[0]]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&frame.views[slots[1]]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Artwork post-processing"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame.views[slots[2]],
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Headless export device, shared between background exports. No window capture or UI state.
struct ExportGpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
}
impl ExportGpu {
    async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .context("Aucun GPU disponible pour les shaders")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ditto shader export"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await?;
        let pipeline = Pipeline::new(&device);
        Ok(Self {
            device,
            queue,
            pipeline,
        })
    }
    fn render(&self, image: &RgbaImage, stack: &Stack, scale: f32) -> Result<RgbaImage> {
        let mut frame = self.pipeline.upload(&self.device, &self.queue, image)?;
        self.pipeline
            .process(&self.device, &self.queue, &mut frame, stack, scale)?;
        let stride = (frame.width * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PNG shader readback"),
            size: stride as u64 * frame.height as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &frame.textures[frame.output],
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(frame.height),
                },
            },
            wgpu::Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().context("Lecture GPU interrompue")??;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut output = RgbaImage::new(frame.width, frame.height);
        for (src, dst) in mapped
            .chunks(stride as usize)
            .zip(output.as_mut().chunks_mut(frame.width as usize * 4))
        {
            dst.copy_from_slice(&src[..dst.len()]);
        }
        drop(mapped);
        buffer.unmap();
        Ok(output)
    }
}
pub fn render(image: &RgbaImage, stack: &Stack, scale: f32) -> Result<RgbaImage> {
    use std::sync::{Mutex, OnceLock};
    stack.validate()?;
    if !stack.active() {
        return Ok(image.clone());
    }
    static GPU: OnceLock<Mutex<Option<ExportGpu>>> = OnceLock::new();
    let mut gpu = GPU
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| anyhow::anyhow!("GPU d’export indisponible"))?;
    if gpu.is_none() {
        *gpu = Some(pollster::block_on(ExportGpu::new())?);
    }
    gpu.as_ref().unwrap().render(image, stack, scale)
}
