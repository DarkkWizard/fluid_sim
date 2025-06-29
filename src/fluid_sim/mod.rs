use crate::render::vertex::Vertex;

pub mod vec2;

#[derive(Clone, Debug, Default)]
pub struct FluidSim {
    particles: Vec<vec2::Vec2>,
}

impl FluidSim {
    pub(crate) fn update(&self, delta: f32) {}

    pub(crate) fn get_particles(&self) -> &[Vertex] {
        &[Vertex {
            position: [100., 100., 100.],
        }]
    }
}
