use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;
use wgpu::*;
use std::num::NonZeroU32;
use std::borrow::Cow;
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use futures::executor::block_on;
use wgpu::util::DeviceExt;
use crevice::std140::AsStd140;
use crevice::std140::Std140;

#[repr(C)]
#[derive(Debug, Copy, Clone, AsStd140)]
struct VertexUniform {
    pub z: f32,
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance.request_adapter(
        &RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
        }
    ).await.unwrap();
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
                label: None,
            },
            None,
        )
        .await
        .unwrap();
    let preferred_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_configs = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: preferred_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
    };
    surface.configure(&device, &surface_configs);
    let mut uniforms = VertexUniform {
        z: 0.0
    };
    let mut z_u8 = std::num::Wrapping(0_u8);
    let uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: uniforms.as_std140().as_bytes(),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Uniform Bindgroup layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
    });
    let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }
        ],
    });

    let (_tex3d, tex3d_view, tex3d_sampler, _tex3d_format) = {
        let format = TextureFormat::Rgba8UnormSrgb;
        let extent = Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 2,
        };
        let desc = TextureDescriptor {
            label: Some("3D_Tex"),
            size: extent.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };
        let texture = device.create_texture(&desc);
        let tex_data: &[u8] = &[
            0, 0, 0, 255,
            0, 0, 255, 255,
            0, 255, 0, 255,
            0, 255, 255, 255,
            255, 0, 0, 255,
            255, 0, 255, 255,
            255, 255, 0, 255,
            255, 255, 255, 255,
        ];
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: Default::default(),
            },
            tex_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(extent.width * 4).unwrap()),
                rows_per_image: Some(NonZeroU32::new(extent.height).unwrap()),
            },
            extent.clone(),
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        (texture, view, sampler, format)
    };
    let volume_bind_group_layout = device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("volume bind group"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler { filtering: true, comparison: false },
                    count: None,
                }
            ],
        }
    );

    let volume_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Volume bind group"),
        layout: &volume_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&tex3d_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&tex3d_sampler),
            }
        ],
    });
    // Load the shaders from disk
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &uniform_bind_group_layout,
            &volume_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[ColorTargetState {
                format: preferred_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            }],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(IndexFormat::Uint32),
            front_face: FrontFace::Ccw,
            cull_mode: None,
            clamp_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });
    let mut need_redraw = true;
    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: ref window_event,
                ..
            } => {
                match (window_event) {
                    WindowEvent::Resized(size) => {// Reconfigure the surface with the new size
                        surface_configs.width = size.width;
                        surface_configs.height = size.height;
                        surface.configure(&device, &surface_configs);
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        } => {
                            z_u8 += std::num::Wrapping(8_u8);
                            uniforms.z = (z_u8.0 as f32) / 255.0;
                            queue.write_buffer(&uniform_buffer, 0, uniforms.as_std140().as_bytes());
                            need_redraw = true;
                        }
                        _ => {}
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &uniform_bind_group, &[]);
                    rpass.set_bind_group(1, &volume_bind_group, &[]);
                    rpass.draw(0..4, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }
            Event::MainEventsCleared => {
                if need_redraw {
                    window.request_redraw();
                    need_redraw = false;
                }
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();
            block_on(run(event_loop, window))
        }
    #[cfg(target_arch="wasm32")]
        {
            use std::panic;
            panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("Could not initialize logger");
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("Could not append canvas to document body");
            wasm_bindgen_futures::spawn_local(run(event_loop, window));
        }
}