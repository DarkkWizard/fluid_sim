mod fluid_sim;
mod render;

use render::run;

fn main() {
    pollster::block_on(run());
}
