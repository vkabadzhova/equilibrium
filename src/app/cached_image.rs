use eframe::egui::TextureId;

/// A record of a cached image
#[derive(Clone)]
pub struct CashedImage {
    /// The path of the last rendered image is saved. The field is used for comparison if the last
    /// texture should be used.
    pub path: String,

    /// The previous zoom factor for the image is saved. The field is used for comparison if the
    /// last texture should be used.
    pub zoom_factor: u8,

    /// The image.
    pub rendered_texture: TextureId,

    /// The dimensions of the texture
    pub dimensions: eframe::egui::Vec2,

    /// Flag here if the images has been changed by any factor, independent of path and zoom factor
    pub has_changed: bool,
}

impl CashedImage {
    /// States if the given structure is already cashed in the current object.
    pub fn consists_of(&self, path: &str, zoom: u8) -> bool {
        self.path == path && self.zoom_factor == zoom
    }
}
