mod vertex;

use crate::fluid_sim::FluidSim;
use wgpu::{Backends, DeviceDescriptor, RequestAdapterOptions, TextureUsages, VertexBufferLayout};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct State<'a> {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    window: &'a Window,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    fluid_sim: crate::fluid_sim::FluidSim,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> State<'a> {
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
                power_preference: wgpu::PowerPreference::default(),
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

        let color = wgpu::Color {
            r: 2.9,
            g: 3.4,
            b: 3.4,
            a: 3.4,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("the one and only shader one shall ever need"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[vertex::Vertex::desc()],
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
                topology: wgpu::PrimitiveTopology::PointList,
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

        let fluid_sim = FluidSim::default();

        State {
            surface,
            window,
            size,
            config,
            device,
            queue,
            color,
            render_pipeline,
            fluid_sim,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

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
            render_pass.draw(2..68, 0..3);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn update(&self) {}

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Moved(position) => {
                println!(
                    "we can move the thing? Here is where we are now: {:?}",
                    position
                );
                true
            }
            _ => false,
        }
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut surface_configured = false;

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
                        surface_configured = true;
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();

                        if !surface_configured {
                            return;
                        }

                        state.update();
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
