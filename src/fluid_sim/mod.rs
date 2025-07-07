use crate::{fluid_sim::vec2::Vec2, render::vertex::Vertex};
use rand::Rng;
use std::f32::consts::PI;

mod vec2;

const MIN: f32 = -PI / 16.;
const MAX: f32 = PI / 16.;
const NUMBER_OF_SECTORS_HEIGHT_WIDTH: (u32, u32) = (1, 1);
const GRAVITY_NUMBER: f32 = 150.;
const PARTICLE_NUMBER: usize = 1000;
const MAX_START_SPEED: f32 = 140.0;
const DECAY_FACTOR: f32 = 0.9;
const NUM_OF_SECTORS: u32 = NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1;
const INTERACTION_RADIUS: f32 = 100.;
const INTERACTION_RADIUS_SQUARED: f32 = INTERACTION_RADIUS * INTERACTION_RADIUS;
/* this is going to have to go */
const REPULSION_STRENGTH: f32 = 200.0;
const FALLOFF_CONSTANT: f32 = 20.0;

#[derive(Clone, Debug)]
pub struct FluidSim {
    particles_positions: Box<[Vec2]>,
    particles_velocities: Box<[Vec2]>,
    sectors: Box<[usize]>,
    sector_grid: Vec<Vec<usize>>,
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

        let sectors = vec![0usize; PARTICLE_NUMBER];
        let sector_grid = vec![Vec::new(); NUM_OF_SECTORS as usize];

        let mut sim = Self {
            particles_positions: particles_positions.into_boxed_slice(),
            particles_velocities: particles_velocities.into_boxed_slice(),
            sectors: sectors.into_boxed_slice(),
            sector_grid,
        };

        sim.update_sectors(&size);
        sim
    }

    fn update_sectors(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        let height_width = (
            (size.height / NUMBER_OF_SECTORS_HEIGHT_WIDTH.0) as f32,
            (size.width / NUMBER_OF_SECTORS_HEIGHT_WIDTH.1) as f32,
        );

        for group in self.sector_grid.iter_mut() {
            group.clear();
        }

        for (i, particle_pos) in self.particles_positions.iter().enumerate() {
            let col = (particle_pos.x / height_width.1)
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 - 1) as f32)
                as u32;
            let row = (particle_pos.y / height_width.0)
                .clamp(0.0, (NUMBER_OF_SECTORS_HEIGHT_WIDTH.0 - 1) as f32)
                as u32;

            let sector_idx = (row * NUMBER_OF_SECTORS_HEIGHT_WIDTH.1 + col) as usize;
            self.sectors[i] = sector_idx;
            self.sector_grid[sector_idx].push(i);
        }
    }

    pub(crate) fn update(&mut self, delta: f32, size: winit::dpi::PhysicalSize<u32>) {
        let delta_vec = Vec2 { x: delta, y: delta };

        // give them new velocities
        for i in 0..PARTICLE_NUMBER {
            self.particles_velocities[i].y += GRAVITY_NUMBER * delta;
        }

        // do the sectors and calculate the other particles velocities away from each particle in
        // the sector
        //
        // TODO make a circle around each particle and do all of the sectors that fall within that
        // circle so that we don't miss out along boarders
        self.update_sectors(&size);
        self.apply_sector_velocities(&delta);

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

        // apply the changes that we just made
        for i in 0..PARTICLE_NUMBER {
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

    fn apply_sector_velocities(&mut self, delta: &f32) {
        for particle_group in self.sector_grid.iter() {
            for i in 0..particle_group.len() {
                for j in (i + 1)..particle_group.len() {
                    let p1 = particle_group[i];
                    let p2 = particle_group[j];

                    let dir_vec = self.particle_distance(p1, p2);

                    let pythagoras_unrooted = dir_vec.x.powi(2) + dir_vec.y.powi(2);

                    if pythagoras_unrooted > 0.0 && pythagoras_unrooted < INTERACTION_RADIUS_SQUARED
                    {
                        let pythagoras_rooted = pythagoras_unrooted.sqrt();

                        let force =
                            REPULSION_STRENGTH * (1. - pythagoras_rooted / INTERACTION_RADIUS);

                        let acceleration = (dir_vec / pythagoras_rooted) * force;

                        self.particles_velocities[p1] += acceleration * *delta;
                        self.particles_velocities[p2] -= acceleration * *delta;
                    }
                }
            }
        }
    }

    fn apply_vels(&mut self, delta: &f32) {}

    /// gives back the vector from point 1 to point 2. Both poits are indicies into the owned
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

    /// Prints the number of particles in each sector to the console in a grid format.
    #[allow(dead_code)]
    fn dbg_print_sector_populations(&self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_size() -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(400, 400)
    }

    fn dummy_sim(positions: Vec<Vec2>, velocities: Vec<Vec2>) -> FluidSim {
        let particle_count = positions.len();
        FluidSim {
            particles_positions: positions.into_boxed_slice(),
            particles_velocities: velocities.into_boxed_slice(),
            sectors: vec![0; particle_count].into_boxed_slice(),
            sector_grid: vec![Vec::new(); NUM_OF_SECTORS as usize],
        }
    }

    #[test]
    fn rand_init_works() {
        let sim = FluidSim::new_rand(test_size());

        assert_eq!(sim.particles_velocities.len(), PARTICLE_NUMBER);
        assert_eq!(sim.sectors.len(), PARTICLE_NUMBER);
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

    #[test]
    fn sector_update() {
        let size = test_size();

        // calculate the size of how large the sectors are going to be.
        let one_sector_length_x = size.width / NUM_OF_SECTORS;
        let one_sector_length_y = size.height / NUM_OF_SECTORS;
        let one_third_height = size.height / 3;
        let one_third_width = size.width / 3;
        let half_height = size.height / 2;
        let half_width = size.width / 2;

        // the sim has to have known particles so that we can check that they're in the right spots
        // later with an assert.
        let positions = vec![
            Vec2 { x: 20., y: 20. },
            Vec2 {
                x: size.width as f32,
                y: size.height as f32,
            },
            Vec2 {
                x: half_width as f32,
                y: half_height as f32,
            },
            Vec2 {
                x: one_third_width as f32,
                y: one_third_height as f32,
            },
        ];
        let mut sim = dummy_sim(positions, vec![]);
        sim.update_sectors(&size);
    }
}
