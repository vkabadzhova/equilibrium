use crate::simulation::configs::{FluidConfigs, SimulationConfigs};
use crate::simulation::fluid::Fluid;
use crate::simulation::obstacle::ObstaclesType;
use std::sync::mpsc::Sender;

/// Saves the state of the current step of the fluid. The purpose of this structure is to be sent
/// over from [`CurrentSimulation`](crate::simulation::current_simulation::CurrentSimulation) to
/// the [`Renderer`](crate::simulation::renderer::Renderer).
pub struct FluidStep {
    fluid: Fluid,
    frame_number: i64,
}

/// Performs the simulation of the fluid. It sends a
/// [`FluidStep`](crate::simulation::current_simulation::FluidStep) over a [`std::sync::mpsc`]
/// channel directly to the [`Renderer`].
pub struct CurrentSimulation {
    /// The Renderer owns the fluid that it simulates
    pub fluid: Fluid,

    /// Buffered fluid configurations for the next run. The configurations of the fluid are not
    /// changed while the fluid is being simulated.
    pub fluid_configs: FluidConfigs,

    /// Buffered simulation configurations for the next run. The configurations of the fluid are
    /// not changed while the fluid is being simulated.
    pub simulation_configs: SimulationConfigs,

    /// The directory in which the rendered images that result from the simulation are stored.
    pub save_into_dir: String,

    /// The color of all obstacles. Obstacles cannot be set individual colors.
    pub obstacles_color: eframe::egui::Color32,

    /// Collection of all the obstacles. To update the fluid's behaviour to correspond to the
    /// obstacles, use [`update_obstacles`].
    pub obstacles: Vec<ObstaclesType>,
}

impl CurrentSimulation {
    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<FluidStep>) {
        self.fluid = Fluid::new(self.fluid_configs, self.simulation_configs);
        self.mark_fluid_obstacles();
        for i in 0..self.fluid.simulation_configs.frames {
            if self.fluid.fluid_configs.has_perlin_noise {
                self.fluid.add_noise();
            }
            self.fluid.step();

            //TODO: self.render_density(i); in renderer
            tx.send(FluidStep {
                fluid: self.fluid,
                frame_number: i,
            })
            .unwrap();
        }
    }

    /// After altering the obstacles list. Refresh the fluid's configuration regarding its
    /// obstacles.
    pub fn mark_fluid_obstacles(&mut self) {
        for obstacle in self.obstacles.iter_mut() {
            self.fluid.fill_obstacle(obstacle);
        }
    }
}
