use crate::fluid::Fluid;
use crate::fluid::SimulationConfig;
use std::fs;
use std::sync::mpsc::Sender;

/// Utility for visualization and interaction with the fluid simulation
pub struct Renderer {
    /// The Renderer owns the fluid that it simulates
    pub fluid: Fluid,
    /// Copy of the fluid's configurations for the simulation
    pub simulation_configs: SimulationConfig,
    /// The directory where the results from the simulation is stored
    pub rendered_images_dir: String,
}

impl Renderer {
    fn make_rendered_images_dir() -> String {
        let project_root = project_root::get_project_root()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        project_root + "/rendered_images"
    }

    /// Creates new Renderer
    pub fn new(fluid: Fluid) -> Renderer {
        Renderer {
            simulation_configs: fluid.simulation_configs.clone(),
            fluid: fluid,
            rendered_images_dir: Renderer::make_rendered_images_dir(),
        }
    }

    fn render_density(&self, frame_number: i64) {
        let mut imgbuf = image::ImageBuffer::new(
            self.fluid.simulation_configs.size,
            self.fluid.simulation_configs.size,
        );

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = self.fluid.density[idx!(x, y, self.fluid.simulation_configs.size)];
            *pixel = image::Rgba([(density * 255.0) as u8, 200, density as u8, 1]);
        }

        fs::create_dir(&self.rendered_images_dir);

        let img_name = format!("{}/density{}.jpg", &self.rendered_images_dir, frame_number);
        imgbuf.save(img_name).unwrap();
    }

    /// Runs the fluid simulation
    pub fn simulate(&mut self, tx: Sender<i64>) {
        self.fluid.init();
        for i in 0..self.fluid.simulation_configs.frames {
            self.fluid.add_noise();
            self.fluid.step();

            self.render_density(i);
            tx.send(i).unwrap();
        }
    }
}