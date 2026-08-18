use image::{DynamicImage, RgbaImage};

// loads a single 64x64 png texture, returns the texture u32 vector
pub fn load_texture(filepath: &str) -> Vec<u32> {
    // tries to open the image file (png only), panics if can't
    let img: DynamicImage = image::open(filepath)
        .unwrap_or_else(|e| panic!("Failed to open image {}: {}", filepath, e));

    // ensures 64x64 pixel size
    let img: DynamicImage = img.resize_exact(64, 64, image::imageops::FilterType::Nearest);

    // transforms the images to rgba format
    let rgba_image: RgbaImage = img.to_rgba8();

    // initializes the buffer where the textures will be held
    let mut texture_buffer: Vec<u32> = vec![0; 64 * 64];

    // loops through each pixel, unpacks them to the rgba value, and appends each to the texture buffer
    for (x, y, pixel) in rgba_image.enumerate_pixels() {
        let r: u32 = pixel.0[0] as u32;
        let g: u32 = pixel.0[1] as u32;
        let b: u32 = pixel.0[2] as u32;
        let _a: u32 = pixel.0[3] as u32;

        let color: u32 = (r << 16) | (g << 8) | b;
        texture_buffer[(y as usize) * 64 + (x as usize)] = color;
    }
    // returns the image as an u32 vector of RGB pixels
    texture_buffer
}
