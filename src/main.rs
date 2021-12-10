mod fluid;
//use crate::fluid::Fluid;
//use crate::fluid;
//use fluid;

fn main() {
    let simulator: fluid::FluidSimulator = fluid::FluidSimulator{..Default::default()};
    simulator.fluid_simulation();
    println!("Hello world!");
}
