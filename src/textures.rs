// textures.rs

use raylib::prelude::*;
use std::slice;

pub struct TextureManager {
    wall_image: Image,       // Store wall image for pixel access
    wall_texture: Texture2D, // Store GPU texture for rendering
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        // Load wall texture
        let wall_image = Image::load_image("assets/wall.png")
            .expect("Failed to load wall.png");
        let wall_texture = rl.load_texture(thread, "assets/wall.png")
            .expect("Failed to load wall texture");

        TextureManager { 
            wall_image, 
            wall_texture 
        }
    }

    pub fn get_wall_pixel_color(&self, tx: u32, ty: u32) -> Color {
        let x = tx.min(self.wall_image.width as u32 - 1) as i32;
        let y = ty.min(self.wall_image.height as u32 - 1) as i32;
        get_pixel_color(&self.wall_image, x, y)
    }

    pub fn get_wall_texture(&self) -> &Texture2D {
        &self.wall_texture
    }
}

fn get_pixel_color(image: &Image, x: i32, y: i32) -> Color {
    let width = image.width as usize;
    let height = image.height as usize;

    if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
        return Color::GRAY; // Fallback color
    }

    let x = x as usize;
    let y = y as usize;

    let data_len = width * height * 4;

    unsafe {
        let data = slice::from_raw_parts(image.data as *const u8, data_len);

        let idx = (y * width + x) * 4;

        if idx + 3 >= data_len {
            return Color::GRAY; // Fallback color
        }

        Color::new(data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
    }
}
