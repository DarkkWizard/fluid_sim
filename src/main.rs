mod fluid_sim;
mod render;

fn main() {
    pollster::block_on(render::run());
}
