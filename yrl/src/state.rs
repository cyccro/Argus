use std::collections::HashMap;

use image::{GenericImageView, ImageError};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::window;

use crate::{
    bindgroups::{BindGroupHelper, BindGroupInfoKind},
    pipeline_helper,
};
pub type VertexInfo<'a> = wgpu::VertexBufferLayout<'a>;
pub trait Vertex {
    fn layout() -> VertexInfo<'static>;
}

pub struct Texture2D {
    view: wgpu::TextureView,
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
}

pub struct ShaderData<T> {
    pub vf: bool,
    pub group: u32,
    pub binding: u32,
    pub data: T,
}
pub struct ShaderBuffer {
    pub buffer_data: ShaderData<wgpu::Buffer>,
    pub kind: BindGroupInfoKind,
}
pub struct Shader {
    pipeline: wgpu::RenderPipeline,
    bindgroups: Vec<(wgpu::BindGroup, wgpu::BindGroupLayout)>,
    texture: Option<ShaderData<Texture2D>>,
}
impl Texture2D {
    pub fn from_file(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        path: &std::path::Path,
    ) -> std::io::Result<Result<Self, ImageError>> {
        Ok(Self::new(queue, device, std::fs::read(path)?))
    }
    pub fn new(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        bytes: Vec<u8>,
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes.as_slice())?;
        let dimensions = img.dimensions();
        let txt_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: txt_size,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let rgba = img.to_rgba8();
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            txt_size,
        );
        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Ok(Self {
            view,
            sampler,
            texture: wgpu_texture,
        })
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }
    pub fn height(&self) -> u32 {
        self.texture.height()
    }
    pub fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
impl Shader {
    pub fn new(
        source: wgpu::ShaderModule,
        device: &wgpu::Device,
        data: Vec<ShaderBuffer>,
        vertices: Vec<VertexInfo>,
        texture: Option<ShaderData<Texture2D>>,
    ) -> Self {
        let (mut groups, infos) = {
            let mut vec = Vec::with_capacity(data.len());
            let mut infos = Vec::with_capacity(data.len());
            for buff in data {
                let group = if buff.buffer_data.vf {
                    BindGroupHelper::create_uniform_vf(
                        device,
                        &buff.buffer_data.data,
                        buff.buffer_data.binding,
                    )
                } else {
                    BindGroupHelper::create_uniform(
                        device,
                        &buff.buffer_data.data,
                        buff.buffer_data.binding,
                    )
                };
                vec.push(group);
                infos.push(buff.kind);
            }
            (vec, infos)
        };
        if let Some(ref texture) = texture {
            let txt = BindGroupHelper::create_texture(
                device,
                texture.data.view(),
                texture.data.sampler(),
                texture.binding,
            );
            groups.push(txt);
        }
        Self {
            texture,
            bindgroups: groups,
            pipeline: pipeline_helper::create_pipeline(device, &source, Some(&vertices), &infos),
        }
    }
}

pub struct WgpuState<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    shaders: HashMap<String, Shader>,
    current_shader: Option<String>,
}
impl<'a> WgpuState<'a> {
    pub fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = {
            let rdh = window.display_handle().unwrap();
            let rwh = window.window_handle().unwrap();
            unsafe {
                instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: rdh.into(),
                        raw_window_handle: rwh.into(),
                    })
                    .unwrap()
            }
        };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed when trying to request adapter");
        let (device, queue) = pollster::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device descriptor"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                std::env::var("WGPU_TRACE")
                    .ok()
                    .as_deref()
                    .map(std::path::Path::new),
            ),
        )
        .unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        Self {
            instance,
            surface,
            adapter,
            device,
            config,
            queue,
            shaders: HashMap::new(),
            current_shader: None,
        }
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn current_shader(&self) -> Option<&Shader> {
        self.shaders.get(self.current_shader.as_ref()?)
    }
    pub fn create_shader(
        &mut self,
        shader_id: String,
        shader_source: &str,
        buffers: Vec<ShaderBuffer>,
        vertices: Vec<VertexInfo<'a>>,
        txt: Option<ShaderData<Texture2D>>,
    ) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
        let shader = Shader::new(shader, &self.device, buffers, vertices, txt);
        self.shaders.insert(shader_id, shader);
    }
    pub fn set_current_shader(&mut self, shader: &str) {
        if let Some(ref mut curr) = self.current_shader {
            curr.clear();
            curr.push_str(shader);
        } else {
            self.current_shader = Some(shader.to_string())
        }
    }
    pub fn render(
        &self,
        vertices: wgpu::Buffer,
        indices: wgpu::Buffer,
    ) -> Result<(), wgpu::SurfaceError> {
        let txt = self.surface.get_current_texture()?;
        let view = txt
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        //make rpass go out of scope to encoder finish be able
        if let Some(shader) = self.current_shader() {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            for (idx, (group, _)) in shader.bindgroups.iter().enumerate() {
                rpass.set_bind_group(idx as u32, group, &[]);
            }
            rpass.set_vertex_buffer(0, vertices.slice(..));
            rpass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..(indices.size() >> 3) as u32, 0, 0..1);
        }
        let finish = encoder.finish();
        self.queue.submit(Some(finish));
        txt.present();
        Ok(())
    }
}
