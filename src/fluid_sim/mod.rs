use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;

pub mod vec2;

const GRAVITY_NUMBER: f32 = 50.;
const PARTICLE_NUMBER: usize = 256;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;

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

        let sectors: [Vec<usize>; 9] = Default::default();

        let mut endgame = Self {
            particles_positions,
            particles_velocities,
            sectors,
        };
        endgame.update_sectors(size);
        endgame
    }

    fn update_sectors(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let w_one_third: f32 = (size.width / 3) as f32;
        let w_two_third: f32 = w_one_third + w_one_third;
        let w_three_third: f32 = size.width as f32;
        let h_one_third: f32 = (size.height / 3) as f32;
        let h_two_third: f32 = h_one_third + h_one_third;
        let h_three_third: f32 = size.height as f32;

        let mut sectors: [Vec<usize>; 9] = Default::default();

        for (i, steve) in self.particles_positions.iter().enumerate() {
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

        self.sectors = sectors;
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        let delta_vec = Vec2 { x: delta, y: delta };
        for (iter, particle) in self.particles_positions.iter_mut().enumerate() {
            self.particles_velocities[iter].y += GRAVITY_NUMBER * delta;
            *particle += self.particles_velocities[iter] * delta_vec;
            if particle.x < 0. {
                particle.x = 0.;
                self.particles_velocities[iter].x *= -1.;
                self.particles_velocities[iter].x *= DECAY_FACTOR;
            } else if particle.x > size.width as f32 {
                particle.x = size.width as f32;
                self.particles_velocities[iter].x *= -1.;
                self.particles_velocities[iter].x *= DECAY_FACTOR;
            } else if particle.y < 0. {
                particle.y = 0.;
                self.particles_velocities[iter].y *= -1.;
                self.particles_velocities[iter].y *= DECAY_FACTOR;
            } else if particle.y > size.height as f32 {
                particle.y = size.height as f32;
                self.particles_velocities[iter].y *= -1.;
                self.particles_velocities[iter].y *= DECAY_FACTOR;
            }
        }
    }

    pub(crate) fn get_particles_vertexes(&self) -> Vec<Vertex> {
        self.particles_positions
            .iter()
            .map(|particle| Vertex {
                position: [particle.x, particle.y],
            })
            .collect()
    }
}
