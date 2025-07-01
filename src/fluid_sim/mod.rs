use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;

pub mod vec2;

const GRAVITY_NUMBER: f32 = 50.;
const PARTICLE_NUMBER: usize = 256;
const MAX_START_SPEED: f32 = 140.0;

#[derive(Clone, Debug, Default)]
pub struct FluidSim {
    particles_positions: Vec<Vec2>,
    particles_velocities: Vec<Vec2>,
    sectors: [Vec<usize>; 9],
}

impl FluidSim {
    pub fn new_rand(size: winit::dpi::PhysicalSize<u32>) -> Self {
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        let width = size.width;
        let height = size.height;

        let particles_positions: Vec<Vec2> = (0..PARTICLE_NUMBER)
            .map(|_| Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(0.0..width as f32),
                #[allow(deprecated)]
                y: rng.gen_range(0.0..height as f32),
            })
            .collect();

        let particles_velocities: Vec<Vec2> = (0..PARTICLE_NUMBER)
            .map(|_| Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
                #[allow(deprecated)]
                y: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
            })
            .collect();

        let mut sectors: [Vec<usize>; 9] = Default::default();

        let w_one_third: f32 = (width / 3) as f32;
        let w_two_third: f32 = w_one_third + w_one_third;
        let w_three_third: f32 = size.width as f32;
        let h_one_third: f32 = (height / 3) as f32;
        let h_two_third: f32 = h_one_third + h_one_third;
        let h_three_third: f32 = size.height as f32;

        // ignore how ugly this is I don't like it either
        for (i, steve) in particles_positions.iter().enumerate() {
            if steve.x < w_one_third {
                if steve.y < h_one_third {
                    sectors[0].push(i);
                } else if steve.y < h_two_third {
                    sectors[1].push(i);
                } else if steve.y < h_three_third {
                    sectors[2].push(i);
                }
            } else if steve.x < w_two_third {
                if steve.y < h_one_third {
                    sectors[3].push(i);
                } else if steve.y < h_two_third {
                    sectors[4].push(i);
                } else if steve.y < h_three_third {
                    sectors[5].push(i);
                }
            } else if steve.x < w_three_third {
                if steve.y < h_one_third {
                    sectors[6].push(i);
                } else if steve.y < h_two_third {
                    sectors[7].push(i);
                } else if steve.y < h_three_third {
                    sectors[8].push(i);
                }
            }
        }

        dbg!(&sectors);

        Self {
            particles_positions,
            particles_velocities,
            sectors,
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
