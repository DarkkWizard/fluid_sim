use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;

mod dom;
pub mod vec2;

const GRAVITY_NUMBER: f32 = 50.;
const PARTICLE_NUMBER: usize = 256;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;
const NUMBER_OF_SECTORS_HEIGHT_WIDTH: (u32, u32) = (5, 5);
const NUM_OF_SECTORS: u32 = NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1;

#[derive(Clone, Debug, Default)]
pub struct FluidSim {
    particles_positions: Vec<Vec2>,
    particles_velocities: Vec<Vec2>,
    sectors: [Vec<usize>; NUM_OF_SECTORS as usize],
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

        let sectors: [Vec<usize>; NUM_OF_SECTORS as usize] = Default::default();

        let mut endgame = Self {
            particles_positions,
            particles_velocities,
            sectors,
        };
        endgame.update_sectors_using_logic(&size);
        endgame
    }

    fn update_sectors_using_logic(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        let height_width = (
            (size.height / NUMBER_OF_SECTORS_HEIGHT_WIDTH.0) as f32,
            (size.width / NUMBER_OF_SECTORS_HEIGHT_WIDTH.1) as f32,
        );

        let mut sectors: [Vec<usize>; NUM_OF_SECTORS as usize] = Default::default();

        for (i, particle) in self.particles_positions.iter().enumerate() {
            let col = (particle.x / height_width.1).floor() as u32;
            let row = (particle.y / height_width.0).floor() as u32;

            let clamped_col = col.min(NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 - 1);
            let clamped_row = row.min(NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 - 1);

            let index = (clamped_row * NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 + clamped_col) as usize;

            sectors[index].push(i);
        }

        self.sectors = sectors;
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        // make them bounce off the walls
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

        // deal with sectors and dispersion
        self.update_sectors_using_logic(&size);

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

    // debug
    #[allow(dead_code)]
    fn print_sector_grid(&self) {
        let (cols, rows) = (
            NUMBER_OF_SECTORS_HEIGHT_WIDTH.0,
            NUMBER_OF_SECTORS_HEIGHT_WIDTH.1,
        );

        for row in 0..rows {
            for col in 0..cols {
                let index = (row * cols + col) as usize;
                let count = self.sectors[index].len();
                print!("[{:3}]", count); // 3-wide formatting for alignment
            }
            println!();
        }
        println!();
    }

    pub(crate) fn get_particles_vertexes(&self) -> Vec<Vertex> {
        self.particles_positions
            .iter()
            .map(|particle| Vertex {
                position: [particle.x, particle.y],
            })
            .collect()
    }

    /// returns (the position)
    fn sort_particles_by_sector(&mut self) -> Vec<(Vec2,)> {
        let v = vec![];
        for g in self.sectors.iter() {}
    }
}
