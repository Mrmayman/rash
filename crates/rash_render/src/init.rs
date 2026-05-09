use std::collections::HashMap;
use std::time::Instant;

use rash_vm::{GraphicsState, RunState, Runtime, SpriteData, SpriteLoadData};
use svg_render::SvgRenderer;
use wgpu::util::DeviceExt;

use crate::WindowSize;

use super::texture::Costume;
use super::to_bytes;

use super::{Renderer, buffers::GlobalBuffer};

impl Renderer {
    pub async fn new(
        window_size: WindowSize,
        vm: &Runtime,
        surface: &wgpu::Surface<'_>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let WindowSize { width, height } = window_size;

        let surface_caps = surface.get_capabilities(adapter);
        // Shader code here assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: if surface_caps
                .present_modes
                .contains(&wgpu::PresentMode::Fifo)
            {
                wgpu::PresentMode::Fifo
            } else {
                surface_caps.present_modes[0]
            },
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let common = include_str!("shaders/common.wgsl");
        let vert = common.to_owned() + include_str!("shaders/vert.wgsl");
        let frag = common.to_owned() + include_str!("shaders/frag.wgsl");

        let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(vert.into()),
        });
        let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(frag.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<GlobalBuffer>() as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                },
            ],
        });

        let costume_layout = Costume::get_bind_group_layout(device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &costume_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sprites_state = vec![GraphicsState::default(); vm.sprite_load_info.len()];

        let sprites_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite State Buffer"),
            contents: to_bytes(sprites_state.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let global_state = GlobalBuffer {
            resolution: [width as f32, height as f32],
        };
        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: to_bytes(&[global_state]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sprites_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: global_buffer.as_entire_binding(),
                },
            ],
        });

        let sampler = Costume::create_sampler(device);

        let svg_renderer = SvgRenderer::new();

        let costumes: Result<HashMap<_, _>, Box<dyn std::error::Error>> = vm
            .costume_data
            .iter()
            .map(|(id, costume)| {
                if costume.is_svg
                    && let Ok(svg_text) = String::from_utf8(costume.bytes.clone())
                {
                    let img = svg_renderer.render(&svg_text)?;

                    return Ok((
                        *id,
                        Costume::from_image(
                            costume,
                            &device,
                            queue,
                            &img,
                            &sampler,
                            &costume_layout,
                        ),
                    ));
                }

                Ok(
                    Costume::from_bytes(costume, device, queue, &sampler, &costume_layout)
                        .map(|n| (*id, n))?,
                )
            })
            .collect();
        let costumes = match costumes {
            Ok(n) => n,
            Err(err) => {
                eprintln!("While loading costumes: {err}");
                HashMap::new()
            }
        };

        let sprites = vm
            .sprite_load_info
            .iter()
            .map(|(id, sprite_info)| {
                let costume = costumes.get(&sprite_info.costume).unwrap();
                let graphics = graphics(sprite_info, costume);
                (*id, SpriteData { graphics })
            })
            .collect();

        Self {
            render_pipeline,
            config,
            window_size,
            bind_group,
            sprites_buffer,
            global_state,
            global_buffer,
            last_time: Instant::now(),
            costumes,
            state: RunState { sprites },
        }
    }
}

fn graphics(sprite_info: &SpriteLoadData, costume_info: &Costume) -> GraphicsState {
    GraphicsState {
        x: sprite_info.x as f32,
        y: sprite_info.y as f32,
        texture_width: costume_info.texture_width as f32,
        texture_height: costume_info.texture_height as f32,
        size: sprite_info.size as f32,
        current_costume: sprite_info.costume,
        center_x: costume_info.rotation_center_x as f32,
        center_y: costume_info.rotation_center_y as f32,
        shown: i32::from(sprite_info.shown),
        padding: [0; _],
    }
}
