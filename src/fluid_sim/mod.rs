use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;

pub mod vec2;

const GRAVITY_NUMBER: f32 = 30.;
const PARTICLE_NUMBER: usize = 288;

#[derive(Clone, Debug, Default)]
pub struct FluidSim {
    particles_positions: Vec<Vec2>,
    particles_velocities: Vec<Vec2>,
}

impl FluidSim {
    pub fn new_rand(size: winit::dpi::PhysicalSize<u32>) -> Self {
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();

        let particles_positions = (0..PARTICLE_NUMBER)
            .map(|_| Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(0.0..size.width as f32),
                #[allow(deprecated)]
                y: rng.gen_range(0.0..size.height as f32),
            })
            .collect();

        Self {
            particles_positions,
            particles_velocities: vec![Vec2::default(); PARTICLE_NUMBER],
        }
    }

    pub(crate) fn update(&mut self, delta: f32) {
        for (iter, particle) in self.particles_positions.iter_mut().enumerate() {
            self.particles_velocities[iter].y += GRAVITY_NUMBER * delta;
            *particle += self.particles_velocities[iter] * Vec2 { x: delta, y: delta };
        }
    }

    pub(crate) fn get_particles_vertexes(&self) -> Vec<Vertex> {
        self.particles_positions
            .iter()
            .map(|particle| Vertex {
                position: [particle.x, particle.y, 0.0],
            })
            .collect()
    }
}
