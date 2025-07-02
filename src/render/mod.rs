pub mod vertex;

use crate::fluid_sim::FluidSim;
use std::time::Instant;
use wgpu::{util::DeviceExt, Backends, DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

const PARTICLE_SIZE: f32 = 5.;

struct BigRenderBoy<'a> {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    window: &'a Window,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    fluid_sim: crate::fluid_sim::FluidSim,
    particle_pos_buffer: wgpu::Buffer,
    last_frame_time: Instant,
    screen_size: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
}

impl<'a> BigRenderBoy<'a> {
    pub async fn new(window: &'a Window) -> BigRenderBoy<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Couldn't adapt");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Main device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to create device from adapter");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let uniform_data = [size.width as f32, size.height as f32, PARTICLE_SIZE];
        let screen_size_and_particle_size =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen size buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&uniform_data),
            });

        let initial_screen_size = [size.width as f32, size.height as f32];
        queue.write_buffer(
            &screen_size_and_particle_size,
            0,
            bytemuck::cast_slice(&initial_screen_size),
        );

        let fluid_sim = FluidSim::new_rand(size);
        let particles = fluid_sim.get_particles_vertexes();
        let particle_data = bytemuck::cast_slice(&particles);

        let color = wgpu::Color::BLACK;

        let particle_pos_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer Pos"),
            contents: particle_data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("the one and only shader one shall ever need"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader2.wgsl").into()),
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let screen_particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &screen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_size_and_particle_size.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_pos_buffer.as_entire_binding(),
                },
            ],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&screen_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
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
        });

        let last_frame_time = Instant::now();

        BigRenderBoy {
            surface,
            window,
            size,
            config,
            device,
            queue,
            color,
            render_pipeline,
            fluid_sim,
            particle_pos_buffer,
            last_frame_time,
            screen_size: screen_size_and_particle_size,
            screen_bind_group: screen_particle_bind_group,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let particles = self.fluid_sim.get_particles_vertexes();
        if particles.is_empty() {
            return Ok(());
        }

        self.queue.write_buffer(
            &self.particle_pos_buffer,
            0,
            bytemuck::cast_slice(&particles),
        );

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("the one and only"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &vec![Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]);

            let num_particles = particles.len() as u32;
            render_pass.draw(0..(num_particles * 6), 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn update(&mut self, delta: std::time::Duration) {
        let dt = delta.as_secs_f32();
        self.fluid_sim.update(dt, self.size);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        let new_screen_size = [new_size.width as f32, new_size.height as f32, PARTICLE_SIZE];
        self.queue
            .write_buffer(&self.screen_size, 0, bytemuck::cast_slice(&new_screen_size));
    }

    fn input(&self, event: &WindowEvent) -> bool {
        match event {
            _ => false,
        }
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = BigRenderBoy::new(&window).await;

    _ = event_loop.run(move |event, control_flow| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();

                        let now = Instant::now();
                        let delta = now - state.last_frame_time;
                        state.last_frame_time = now;

                        state.update(delta);
                        match state.render() {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                                eprintln!("oh fuck we out of space");
                                control_flow.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                eprintln!("oh fuck you slow");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
