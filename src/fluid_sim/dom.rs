use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;

const GRAVITY_NUMBER: f32 = 50.;
const PARTICLE_NUMBER: usize = 256;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;
const NUMBER_OF_SECTORS_HEIGHT_WIDTH: (u32, u32) = (5, 5);

#[derive(Clone, Debug, Default)]
pub struct FluidSim {
    particles_positions: [Vec2; PARTICLE_NUMBER],
    particles_velocities: [Vec2; PARTICLE_NUMBER],
    sectors: [usize; PARTICLE_NUMBER],
}

impl FluidSim {
    pub fn new_rand(size: winit::dpi::PhysicalSize<u32>) -> Self {
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        let width = size.width;
        let height = size.height;

        let mut particles_positions = [Vec2::default(); PARTICLE_NUMBER];
        for pos in particles_positions.iter_mut() {
            *pos = Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(0.0..width as f32),
                #[allow(deprecated)]
                y: rng.gen_range(0.0..height as f32),
            }
        }

        let mut particles_velocities = [Vec2::default(); PARTICLE_NUMBER];
        for vel in particles_velocities.iter_mut() {
            *vel = Vec2 {
                #[allow(deprecated)]
                x: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
                #[allow(deprecated)]
                y: rng.gen_range(-MAX_START_SPEED..MAX_START_SPEED),
            }
        }

        let mut sim = Self {
            particles_positions,
            particles_velocities,
            sectors: [0; PARTICLE_NUMBER],
        };

        sim.update_sectors(&size);
        sim
    }

    fn update_sectors(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        let height_width = (
            (size.height / NUMBER_OF_SECTORS_HEIGHT_WIDTH.0) as f32,
            (size.width / NUMBER_OF_SECTORS_HEIGHT_WIDTH.1) as f32,
        );

        for (i, particle_pos) in self.particles_positions.iter().enumerate() {
            let col = (particle_pos.x / height_width.1)
                .floor()
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 - 1) as f32)
                as u32;
            let row = (particle_pos.y / height_width.0)
                .floor()
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 - 1) as f32)
                as u32;

            self.sectors[i] = (row * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 + col) as usize;
        }
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        // make them bounce off the walls
        let delta_vec = Vec2 { x: delta, y: delta };

        for i in 0..PARTICLE_NUMBER {
            self.particles_velocities[i].y += GRAVITY_NUMBER * delta;
            self.particles_positions[i] += self.particles_velocities[i] * delta_vec;

            if self.particles_positions[i].x < 0.0 {
                self.particles_positions[i].x = 0.0;
                self.particles_velocities[i].x *= DECAY_FACTOR;
            } else if self.particles_positions[i].x > size.width as f32 {
                self.particles_positions[i].x = size.width as f32;
                self.particles_velocities[i].x *= -DECAY_FACTOR;
            }
            if self.particles_positions[i].y < 0.0 {
                self.particles_positions[i].y = 0.0;
                self.particles_velocities[i].y *= DECAY_FACTOR;
            } else if self.particles_positions[i].x > size.height as f32 {
                self.particles_positions[i].y = size.height as f32;
                self.particles_velocities[i].y *= -DECAY_FACTOR;
            }
        }
        self.update_sectors(&size);

        // want to create a list of tuples that has (particle num, sector) so that we can go
        // through every particle and use the positions to apply the effects.
        let mut to_do: Vec<(usize, usize)> = Vec::default();
        for (i, sector) in self.sectors.iter().enumerate() {
            for j in sector.iter() {
                to_do.push((*j, i));
            }
        }

        self.particles_positions = self.sort_particles_by_sector();

        // debuging options
        // self.print_sector_grid();
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
