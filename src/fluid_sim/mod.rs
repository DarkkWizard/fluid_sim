use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;
use std::f32::consts::PI;

mod vec2;

const MIN: f32 = -PI / 16.;
const MAX: f32 = PI / 16.;
const GRAVITY_NUMBER: f32 = 150.;
const PARTICLE_NUMBER: usize = 1000;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;
const NUMBER_OF_SECTORS_HEIGHT_WIDTH: (u32, u32) = (20, 20);
const NUM_OF_SECTORS: u32 = NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1;

#[derive(Clone, Debug)]
pub struct FluidSim {
    particles_positions: Box<[Vec2]>,
    particles_velocities: Box<[Vec2]>,
    sectors: Box<[usize]>,
}

impl FluidSim {
    pub fn new_rand(size: winit::dpi::PhysicalSize<u32>) -> Self {
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

        let sectors = vec![0usize; PARTICLE_NUMBER];

        let mut sim = Self {
            particles_positions: particles_positions.into_boxed_slice(),
            particles_velocities: particles_velocities.into_boxed_slice(),
            sectors: sectors.into_boxed_slice(),
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
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 - 1) as f32)
                as u32;
            let row = (particle_pos.y / height_width.0)
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 - 1) as f32)
                as u32;

            self.sectors[i] = (row * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 + col) as usize;
        }
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        let delta_vec = Vec2 { x: delta, y: delta };

        // give them new velocities
        for i in 0..PARTICLE_NUMBER {
            self.particles_velocities[i].y += GRAVITY_NUMBER * delta;
        }

        self.update_sectors(&size);
        // TODO: do the sector velocity updating based of distance from other particles

        // for sector in 0..NUM_OF_SECTORS {
        //     let mut skip_list = vec![];
        //     for (j, designator) in self.sectors.iter().enumerate() {
        //         if *designator != sector as usize {
        //             skip_list.push(sector);
        //             continue;
        //         }

        //
        //     }
        // }

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
        for i in 0..PARTICLE_NUMBER {
            self.particles_positions[i] += self.particles_velocities[i] * delta_vec;
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

    /// Prints the number of particles in each sector to the console in a grid format.
    #[allow(dead_code)]
    fn print_sector_populations(&self) {
        let mut sector_counts = [0; NUM_OF_SECTORS as usize];

        // Count particles in each sector
        for &sector_index in self.sectors.iter() {
            if let Some(count) = sector_counts.get_mut(sector_index) {
                *count += 1;
            }
        }

        let (rows, cols) = NUMBER_OF_SECTORS_HEIGHT_WIDTH;
        for row in 0..rows {
            for col in 0..cols {
                let index = (row * cols + col) as usize;
                print!("[{:3}]", sector_counts[index]); // 3-wide formatting for alignment
            }
            println!();
        }
        println!();
    }
}
