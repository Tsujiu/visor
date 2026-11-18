//! Operações de transformação de imagens (rotações e ajustes em buffers de pixel).

pub fn rotate_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    clockwise: bool,
) -> (Vec<u8>, u32, u32) {
    let Some(buffer) =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels.to_vec())
    else {
        return (pixels.to_vec(), width, height);
    };
    let rotated = if clockwise {
        image::imageops::rotate90(&buffer)
    } else {
        image::imageops::rotate270(&buffer)
    };
    let (width, height) = rotated.dimensions();
    (rotated.into_raw(), width, height)
}
