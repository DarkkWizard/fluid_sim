use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;
use rayon::prelude::*;
use std::{cmp::min, f32::consts::PI};

mod vec2;

const MIN: f32 = -PI / 16.;
const MAX: f32 = PI / 16.;
const GRAVITY_NUMBER: f32 = 300.;
const PARTICLE_NUMBER: usize = 1000;
const MAX_START_SPEED: f32 = 140.;
const MAX_AWAY_SPEED: f32 = 400.;
const DECAY_FACTOR: f32 = 0.9;
const FALLOFF_CONSTANT: f32 = 1000.;
const INTERACTION_RADIUS: f32 = 200.;
const INTERACTION_RADIUS_SQUARED: f32 = INTERACTION_RADIUS * INTERACTION_RADIUS;

#[derive(Clone, Debug)]
pub struct FluidSim {
    current_positions: Box<[Vec2]>,
    current_velocities: Box<[Vec2]>,

    next_positions: Box<[Vec2]>,
    next_velocities: Box<[Vec2]>,
}

impl FluidSim {
    pub fn new_rand(size: winit::dpi::PhysicalSize<u32>) -> Self {
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        let width = size.width;
        let height = size.height;

        let mut particles_positions = Vec::with_capacity(PARTICLE_NUMBER);
        let mut particles_velocities = Vec::with_capacity(PARTICLE_NUMBER);

        for _ in 0..PARTICLE_NUMBER {
            particles_positions.push(Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(0.0..width as f32),
                #[allow(deprecated)]
                y: rng.gen_range(0.0..height as f32),
            });

            particles_velocities.push(Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
                #[allow(deprecated)]
                y: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
            });
        }

        Self {
            current_positions: particles_positions.clone().into_boxed_slice(),
            current_velocities: particles_velocities.clone().into_boxed_slice(),
            next_positions: particles_positions.into_boxed_slice(),
            next_velocities: particles_velocities.into_boxed_slice(),
        }
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        let delta_vec = Vec2 { x: delta, y: delta };

        self.next_velocities
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, new_velocity)| {
                *new_velocity = self.current_velocities[i];

                // gravity
                new_velocity.y += GRAVITY_NUMBER * delta;

                let pos = self.current_positions[i];
                // pressure from the other particles around it
                for j in 0..PARTICLE_NUMBER {
                    if i == j {
                        continue;
                    }

                    let dist_vec = particle_distance(self.current_positions[j], pos);
                    let dist_squared = dist_vec.x.powi(2) + dist_vec.y.powi(2);

                    if dist_squared < INTERACTION_RADIUS_SQUARED && dist_squared > 1e-6 {
                        let magnatude = (FALLOFF_CONSTANT / dist_squared).min(MAX_AWAY_SPEED);
                        let force_direction = dist_vec / dist_squared.sqrt();
                        *new_velocity += force_direction * magnatude * delta;
                    }
                }
            });

        // update the positions with some fancy zipping
        self.next_positions
            .par_iter_mut()
            .zip(&*self.current_positions)
            .zip(&*self.next_velocities)
            .for_each(|((next_pos, current_pos), next_vel)| {
                *next_pos = *current_pos + *next_vel * delta_vec
            });

        // bounce with some randomness
        self.next_positions
            .par_iter_mut()
            .zip(self.next_velocities.par_iter_mut())
            .for_each(|(pos, vel)| {
                #[allow(deprecated)]
                let mut rng = rand::thread_rng();
                if pos.x < 0.0 {
                    pos.x = 0.0;
                    #[allow(deprecated)]
                    vel.rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                    vel.x *= -DECAY_FACTOR;
                } else if pos.x > size.width as f32 {
                    pos.x = size.width as f32;
                    #[allow(deprecated)]
                    vel.rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                    vel.x *= -DECAY_FACTOR;
                }
                if pos.y < 0.0 {
                    pos.y = 0.0;
                    #[allow(deprecated)]
                    vel.rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                    vel.y *= -DECAY_FACTOR;
                } else if pos.y > size.height as f32 {
                    pos.y = size.height as f32;
                    #[allow(deprecated)]
                    vel.rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                    vel.y *= -DECAY_FACTOR;
                }
            });

        // SWAP THEM!!!
        std::mem::swap(&mut self.current_positions, &mut self.next_positions);
        std::mem::swap(&mut self.current_velocities, &mut self.next_velocities);
    }

    // for the render if we want the particles to be outputed as vertexs for the pipeline
    pub(crate) fn get_particles_vertexes(&self) -> Vec<Vertex> {
        self.current_positions
            .iter()
            .map(|particle| Vertex {
                position: [particle.x, particle.y],
            })
            .collect()
    }
}

/// gives back the vector from point 1 to point 2. Both points are indicies into the owned
/// position field of the struct
///
fn particle_distance(first: Vec2, second: Vec2) -> Vec2 {
    let x_dist = first.x - second.x;
    let y_dist = first.y - second.y;

    Vec2 {
        x: -x_dist,
        y: -y_dist,
    }
}

/// treats the Vec2 as a distance rather than a point. Might be a little confusing
#[allow(dead_code)]
fn falloff_function(mut input: Vec2) -> Vec2 {
    let input_squared = input * input;
    let xinput = FALLOFF_CONSTANT / input_squared.x;
    let yinput = FALLOFF_CONSTANT / input_squared.y;
    input.x = xinput;
    input.y = yinput;
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_size() -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(400, 400)
    }

    fn dummy_sim(positions: Vec<Vec2>, velocities: Vec<Vec2>) -> FluidSim {
        FluidSim {
            current_positions: positions.clone().into_boxed_slice(),
            current_velocities: velocities.clone().into_boxed_slice(),
            next_positions: positions.into_boxed_slice(),
            next_velocities: velocities.into_boxed_slice(),
        }
    }

    #[test]
    fn rand_init_works() {
        let sim = FluidSim::new_rand(test_size());

        assert_eq!(sim.current_velocities.len(), PARTICLE_NUMBER);
        assert_eq!(sim.current_positions.len(), PARTICLE_NUMBER);
        // TODO there's probably more to test here that I'm not thinking about.
    }

    #[test]
    fn falloff_actually_works() {
        assert!(
            falloff_function(Vec2 { x: 10., y: 10. }) < falloff_function(Vec2 { x: 1., y: 1. })
        );
        assert!(falloff_function(Vec2 { x: 1., y: 1. }) == falloff_function(Vec2 { x: 1., y: 1. }));
        assert!(
            falloff_function(Vec2 { x: -1., y: -1. }) == falloff_function(Vec2 { x: 1., y: 1. })
        );
        assert!(
            falloff_function(Vec2 { x: -10., y: -10. }) < falloff_function(Vec2 { x: 1., y: 1. })
        );
    }
}
