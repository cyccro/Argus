use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Sampler, TextureView};

#[derive(Debug)]
pub enum BindGroupInfoKind {
    Texture(u32),
    Uniform(u32),
    VfUniform(u32),
}

pub struct BindGroupHelper;

impl BindGroupHelper {
    pub fn create_texture(
        device: &Device,
        txt_view: &TextureView,
        sampler: &Sampler,
        binding: u32,
    ) -> (BindGroup, BindGroupLayout) {
        let layout = Self::create_layout_texture(device, binding);
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding,
                        resource: wgpu::BindingResource::TextureView(txt_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: binding + 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            }),
            layout,
        )
    }
    pub fn create_layout_texture(device: &Device, binding: u32) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: binding + 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
    pub fn create_uniform_vf(
        device: &Device,
        buffer: &Buffer,
        binding: u32,
    ) -> (BindGroup, BindGroupLayout) {
        let layout = Self::create_uniform_vf_layout(device, binding);
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                label: None,
                entries: &[wgpu::BindGroupEntry {
                    binding,
                    resource: buffer.as_entire_binding(),
                }],
            }),
            layout,
        )
    }
    pub fn create_uniform(
        device: &Device,
        buffer: &Buffer,
        binding: u32,
    ) -> (BindGroup, BindGroupLayout) {
        let layout = Self::create_uniform_layout(device, binding);
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                label: None,
                entries: &[wgpu::BindGroupEntry {
                    binding,
                    resource: buffer.as_entire_binding(),
                }],
            }),
            layout,
        )
    }
    //vf stands for vertex/fragment
    pub fn create_uniform_vf_layout(device: &Device, binding: u32) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                count: None,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }
    pub fn create_uniform_layout(device: &Device, binding: u32) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }
}
