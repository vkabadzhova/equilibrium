use crate::fluid::Fluid;

/// Utility for visualization and interaction with the fluid simulation
pub struct Renderer {
    //fluid: Fluid,
}

impl Renderer {
    /// Creates new Renderer
    pub fn new() -> Renderer {
        Renderer { 
            //fluid: fluid 
        }
    }

    fn render_density(&self, fluid: &Fluid, frame_number: u32) {
        let mut imgbuf = image::ImageBuffer::new(*fluid.simulation_configs().size(), *fluid.simulation_configs().size());

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let density = fluid.density()[idx!(x, y, fluid.simulation_configs().size())];
            *pixel = image::Rgb([(density * 255.0) as u8, 200, density as u8]);
        }

        let img_name = format!("rendered_images/density{}.jpg", frame_number);
        imgbuf.save(img_name).unwrap();
    }

    /// Runs the fluid simulation
    pub fn simulate(&self, fluid: &mut Fluid) {
        fluid.init();
        for i in 0..*fluid.simulation_configs().iterations() {
            fluid.add_noise();
            fluid.step();

            self.render_density(fluid, i);
        }
    }
}
