use image::{ImageBuffer, ImageFormat, Rgb};
use rand::Rng;
use std::io::Cursor;

pub fn random_image() -> String {
    // Set the dimensions.
    let width = 64;
    let height = 64;

    // Create a new image buffer with a base color (black).
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgb([0, 0, 0]));
    let mut rng = rand::thread_rng();

    // Determine how many blobs (circles) to draw.
    let num_blobs = rng.gen_range(2..20);

    for _ in 0..num_blobs {
        // Choose a random center within image boundaries.
        let cx = rng.gen_range(0..width) as i32;
        let cy = rng.gen_range(0..height) as i32;
        // Choose a random radius (e.g., between 5 and 15 pixels).
        let radius = rng.gen_range(5..15) as i32;
        // Choose a random color.
        let color = Rgb([rng.r#gen::<u8>(), rng.r#gen::<u8>(), rng.r#gen::<u8>()]);

        // Draw the blob by iterating over the bounding box of the circle.
        let x_start = (cx - radius).max(0);
        let x_end = (cx + radius).min(width as i32);
        let y_start = (cy - radius).max(0);
        let y_end = (cy + radius).min(height as i32);

        for x in x_start..x_end {
            for y in y_start..y_end {
                // Check if the pixel (x,y) is within the circle.
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    // Encode the image as PNG.
    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png)
            .expect("Failed to encode image as PNG");
    }

    // Convert the PNG bytes to base64.
    let encoded = base64::encode(&buffer);

    // Format as a data URI.
    format!("data:image/png;base64,{}", encoded)
}

pub fn rotating_circle() -> String {
    // Set the dimensions.
    let width = 64;
    let height = 64;

    // Create a new image buffer with a base color (black).
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgb([0, 0, 0]));
    let rng = rand::thread_rng();

    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let time = (time as f64) * 0.0025;

    let cx = (32.0 + 15.0 * time.sin()) as i32;
    let cy = (32.0 + 15.0 * time.cos()) as i32;
    let radius = 5;
    let color = Rgb([255, 0, 0]);

    // Draw the blob by iterating over the bounding box of the circle.
    let x_start = (cx - radius).max(0);
    let x_end = (cx + radius).min(width as i32);
    let y_start = (cy - radius).max(0);
    let y_end = (cy + radius).min(height as i32);

    for x in x_start..x_end {
        for y in y_start..y_end {
            // Check if the pixel (x,y) is within the circle.
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }

    // Encode the image as PNG.
    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png)
            .expect("Failed to encode image as PNG");
    }

    // Convert the PNG bytes to base64.
    let encoded = base64::encode(&buffer);

    // Format as a data URI.
    format!("data:image/png;base64,{}", encoded)
}
