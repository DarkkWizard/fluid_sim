use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;
use std::f32::consts::PI;

mod vec2;

const MIN: f32 = -PI / 16.;
const MAX: f32 = PI / 16.;
const GRAVITY_NUMBER: f32 = 150.;
const PARTICLE_NUMBER: usize = 10000;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;
const INTERACTION_RADIUS: f32 = 100.;
const FALLOFF_CONSTANT: f32 = 100.;
const INTERACTION_RADIUS_SQUARED: f32 = INTERACTION_RADIUS * INTERACTION_RADIUS;

#[derive(Clone, Debug)]
pub struct FluidSim {
    particles_positions: Box<[Vec2]>,
    particles_velocities: Box<[Vec2]>,
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
            particles_positions: particles_positions.into_boxed_slice(),
            particles_velocities: particles_velocities.into_boxed_slice(),
        }
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        let delta_vec = Vec2 { x: delta, y: delta };
        // TODO make a circle around each particle and do all of the sectors that fall within that
        // circle so that we don't miss out along boarders

        // bounce with some randomness
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        for i in 0..PARTICLE_NUMBER {
            if self.particles_positions[i].x < 0.0 {
                self.particles_positions[i].x = 0.0;
                #[allow(deprecated)]
                self.particles_velocities[i].rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                self.particles_velocities[i].x *= -DECAY_FACTOR;
            } else if self.particles_positions[i].x > size.width as f32 {
                self.particles_positions[i].x = size.width as f32;
                #[allow(deprecated)]
                self.particles_velocities[i].rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                self.particles_velocities[i].x *= -DECAY_FACTOR;
            }
            if self.particles_positions[i].y < 0.0 {
                self.particles_positions[i].y = 0.0;
                #[allow(deprecated)]
                self.particles_velocities[i].rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                self.particles_velocities[i].y *= -DECAY_FACTOR;
            } else if self.particles_positions[i].y > size.height as f32 {
                self.particles_positions[i].y = size.height as f32;
                #[allow(deprecated)]
                self.particles_velocities[i].rotate_degrees(cgmath::Rad(rng.gen_range(MIN..MAX)));
                self.particles_velocities[i].y *= -DECAY_FACTOR;
            }
        }

        // we want it loop through as few times as possible, so we got this going.
        for i in 0..PARTICLE_NUMBER {
            // give them new velocities
            self.particles_velocities[i].y += GRAVITY_NUMBER * delta;
            self.apply_vel_for_i(i, &delta);

            // apply the new velocities
            self.particles_positions[i] += self.particles_velocities[i] * delta_vec;
        }
    }

    // for the render if we want the particles to be outputed as vertexs for the pipeline
    pub(crate) fn get_particles_vertexes(&self) -> Vec<Vertex> {
        self.particles_positions
            .iter()
            .map(|particle| Vertex {
                position: [particle.x, particle.y],
            })
            .collect()
    }

    /// call this in the same loop as the gravity loop, that means that we're already looping over
    /// every particle so we don't need to loop again inside.
    fn apply_vel_for_i(&mut self, particle: usize, delta: &f32) {}

    /// gives back the vector from point 1 to point 2. Both points are indicies into the owned
    /// position field of the struct
    ///
    fn particle_distance(&self, first: usize, second: usize) -> Vec2 {
        let (primary, secondary) = (
            self.particles_positions[first],
            self.particles_positions[second],
        );

        let x_dist = primary.x - secondary.x;
        let y_dist = primary.y - secondary.y;

        Vec2 {
            x: -x_dist,
            y: -y_dist,
        }
    }

    fn falloff_function(mut input: Vec2) -> Vec2 {
        let input_squared = input * input;
        let xinput = FALLOFF_CONSTANT / input_squared.x;
        let yinput = FALLOFF_CONSTANT / input_squared.y;
        input.x = xinput;
        input.y = yinput;
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_size() -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(400, 400)
    }

    fn dummy_sim(positions: Vec<Vec2>, velocities: Vec<Vec2>) -> FluidSim {
        FluidSim {
            particles_positions: positions.into_boxed_slice(),
            particles_velocities: velocities.into_boxed_slice(),
        }
    }

    #[test]
    fn rand_init_works() {
        let sim = FluidSim::new_rand(test_size());

        assert_eq!(sim.particles_velocities.len(), PARTICLE_NUMBER);
        assert_eq!(sim.particles_positions.len(), PARTICLE_NUMBER);
        // TODO there's probably more to test here that I'm not thinking about.
    }

    #[test]
    fn distance_function() {
        let sim = dummy_sim(
            vec![Vec2 { x: 10., y: 10. }, Vec2 { x: 13., y: 13. }],
            vec![],
        );

        let dist_vec = sim.particle_distance(0, 1);
        println!("{dist_vec:?}");
        assert_eq!(dist_vec.x, 3.);
        assert_eq!(dist_vec.y, 3.);
    }
}
