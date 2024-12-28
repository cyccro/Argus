pub mod pipeline_helper {
    use crate::bindgroups::{BindGroupHelper, BindGroupInfoKind};
    use wgpu::{
        ColorWrites, Device, RenderPipeline, ShaderModule, TextureFormat, VertexBufferLayout,
    };

    pub fn create_pipeline_layout(
        device: &Device,
        info: &Vec<BindGroupInfoKind>,
    ) -> wgpu::PipelineLayout {
        let vec = {
            let mut vec = Vec::with_capacity(info.len());
            for kind in info {
                vec.push(match kind {
                    BindGroupInfoKind::Uniform(binding) => {
                        BindGroupHelper::create_uniform_layout(device, *binding)
                    }
                    BindGroupInfoKind::VfUniform(binding) => {
                        BindGroupHelper::create_uniform_vf_layout(device, *binding)
                    }
                    BindGroupInfoKind::Texture(binding) => {
                        BindGroupHelper::create_layout_texture(device, *binding)
                    }
                })
            }
            vec
        };
        let layouts = {
            let mut layouts = Vec::with_capacity(vec.len());
            for i in vec.iter() {
                layouts.push(i)
            }
            layouts
        };
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        })
    }
    pub fn create_pipeline_descriptor<'b>(
        layout: &'b wgpu::PipelineLayout,
        shader: &'b wgpu::ShaderModule,
        buffers: &'b [wgpu::VertexBufferLayout],
    ) -> wgpu::RenderPipelineDescriptor<'b> {
        wgpu::RenderPipelineDescriptor {
            layout: Some(layout),
            label: None,
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    write_mask: ColorWrites::ALL,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        }
    }
    pub fn create_render_pipeline(
        device: &Device,
        layout: &wgpu::PipelineLayout,
        shader: &ShaderModule,
        buffers: Option<&Vec<VertexBufferLayout>>,
    ) -> RenderPipeline {
        if let Some(buffers) = buffers {
            device.create_render_pipeline(&create_pipeline_descriptor(
                layout,
                shader,
                buffers.as_slice(),
            ))
        } else {
            device.create_render_pipeline(&create_pipeline_descriptor(layout, shader, &[]))
        }
    }
    pub fn create_pipeline(
        device: &Device,
        shader: &ShaderModule,
        buffers: Option<&Vec<VertexBufferLayout>>,
        infos: &Vec<BindGroupInfoKind>,
    ) -> wgpu::RenderPipeline {
        let layout = create_pipeline_layout(device, infos);
        create_render_pipeline(device, &layout, shader, buffers)
    }
}
