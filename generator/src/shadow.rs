use image::{ImageBuffer, Rgba, RgbaImage};

/// Single-pass horizontal box blur on an alpha-only buffer stored row-major.
fn box_blur_h(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    let diam = radius * 2 + 1;
    for y in 0..h {
        let row = y * w;
        // running sum seeded with the left edge repeated into the margin
        let mut sum: u32 = (radius as u32 + 1) * src[row] as u32;
        for i in 0..radius {
            sum += src[row + i.min(w - 1)] as u32;
        }
        for x in 0..w {
            let right = (x + radius).min(w - 1);
            let left = (x as isize - radius as isize - 1).max(0) as usize;
            sum += src[row + right] as u32;
            dst[row + x] = (sum / diam as u32).min(255) as u8;
            sum -= src[row + left] as u32;
        }
    }
}

/// Single-pass vertical box blur on an alpha-only buffer stored row-major.
fn box_blur_v(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    let diam = radius * 2 + 1;
    for x in 0..w {
        let mut sum: u32 = (radius as u32 + 1) * src[x] as u32;
        for i in 0..radius {
            sum += src[i.min(h - 1) * w + x] as u32;
        }
        for y in 0..h {
            let bottom = (y + radius).min(h - 1);
            let top = (y as isize - radius as isize - 1).max(0) as usize;
            sum += src[bottom * w + x] as u32;
            dst[y * w + x] = (sum / diam as u32).min(255) as u8;
            sum -= src[top * w + x] as u32;
        }
    }
}

/// Approximate Gaussian blur via three successive box-blur passes on an alpha buffer.
fn gaussian_blur_alpha(buf: &mut Vec<u8>, w: usize, h: usize, sigma: f32) {
    // Compute box radius for 3-pass approximation of Gaussian
    // See: "Fast Almost-Gaussian Filtering" - W. Wells (1986)
    let boxes = boxes_for_gauss(sigma, 3);
    let mut tmp = vec![0u8; w * h];

    for &box_size in &boxes {
        let radius = ((box_size - 1) / 2) as usize;
        if radius == 0 {
            continue;
        }
        box_blur_h(buf, &mut tmp, w, h, radius);
        box_blur_v(&tmp, buf, w, h, radius);
    }
}

/// Compute ideal box sizes for n-pass box blur approximating a Gaussian with given sigma.
fn boxes_for_gauss(sigma: f32, n: usize) -> Vec<i32> {
    let w_ideal = ((12.0 * sigma * sigma / n as f32) + 1.0).sqrt();
    let mut wl = w_ideal.floor() as i32;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;

    let m_ideal = (12.0 * sigma * sigma
        - (n as f32 * wl as f32 * wl as f32)
        - 4.0 * n as f32 * wl as f32
        - 3.0 * n as f32)
        / (-4.0 * wl as f32 - 4.0);
    let m = m_ideal.round() as usize;

    (0..n).map(|i| if i < m { wl } else { wu }).collect()
}

/// Compose a drop-shadow behind `card` and return the final image.
///
/// The shadow is created by:
/// 1. Extracting the card's alpha channel as a silhouette
/// 2. Blurring the silhouette with a Gaussian approximation
/// 3. Tinting the blurred silhouette black at `shadow_opacity`
/// 4. Placing the blurred shadow on a transparent canvas, offset slightly downward
/// 5. Compositing the original card on top
pub fn apply_outer_shadow(
    card: &RgbaImage,
    shadow_blur: f32,
    shadow_opacity: f32,
    offset_x: i32,
    offset_y: i32,
    outer_bg: Rgba<u8>,
    outer_padding: u32,
) -> RgbaImage {
    let (cw, ch) = card.dimensions();
    let blur_margin = (shadow_blur * 3.0).ceil() as u32; // extra space for the blur
    let margin = blur_margin + outer_padding; // blur space + user-specified outer padding

    let out_w = cw + margin * 2;
    let out_h = ch + margin * 2;

    // Build an alpha-only buffer from the card's alpha channel, placed at margin offset
    let buf_w = out_w as usize;
    let buf_h = out_h as usize;
    let mut alpha_buf = vec![0u8; buf_w * buf_h];

    for cy in 0..ch {
        for cx in 0..cw {
            // Place shadow silhouette shifted by (offset_x, offset_y) relative to center
            let dx = (margin as i32 + offset_x + cx as i32) as usize;
            let dy = (margin as i32 + offset_y + cy as i32) as usize;
            if dx < buf_w && dy < buf_h {
                let a = card.get_pixel(cx, cy)[3];
                alpha_buf[dy * buf_w + dx] = a;
            }
        }
    }

    // Blur the alpha buffer
    gaussian_blur_alpha(&mut alpha_buf, buf_w, buf_h, shadow_blur);

    // Compose: start with outer background canvas, paint blurred shadow, then overlay card
    let mut output: RgbaImage = ImageBuffer::from_pixel(out_w, out_h, outer_bg);

    // Paint blurred shadow (black, modulated by blurred alpha * opacity) onto outer background
    for y in 0..out_h {
        for x in 0..out_w {
            let a = alpha_buf[y as usize * buf_w + x as usize] as f32 / 255.0;
            let sa = a * shadow_opacity;
            if sa > 0.0 {
                let dst = output.get_pixel(x, y);
                let da = dst[3] as f32 / 255.0;
                let out_a = sa + da * (1.0 - sa);
                if out_a > 0.0 {
                    // Shadow color is black (0,0,0) with alpha = sa
                    let r = (dst[0] as f32 * da * (1.0 - sa)) / out_a;
                    let g = (dst[1] as f32 * da * (1.0 - sa)) / out_a;
                    let b = (dst[2] as f32 * da * (1.0 - sa)) / out_a;
                    output.put_pixel(
                        x,
                        y,
                        Rgba([r as u8, g as u8, b as u8, (out_a * 255.0) as u8]),
                    );
                }
            }
        }
    }

    // Composite the card on top at (margin, margin)
    for cy in 0..ch {
        for cx in 0..cw {
            let src = card.get_pixel(cx, cy);
            let sa = src[3] as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }
            let dx = cx + margin;
            let dy = cy + margin;
            let dst = output.get_pixel(dx, dy);
            let da = dst[3] as f32 / 255.0;

            // Standard Porter-Duff "source over" compositing
            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                continue;
            }
            let r = (src[0] as f32 * sa + dst[0] as f32 * da * (1.0 - sa)) / out_a;
            let g = (src[1] as f32 * sa + dst[1] as f32 * da * (1.0 - sa)) / out_a;
            let b = (src[2] as f32 * sa + dst[2] as f32 * da * (1.0 - sa)) / out_a;
            output.put_pixel(
                dx,
                dy,
                Rgba([r as u8, g as u8, b as u8, (out_a * 255.0) as u8]),
            );
        }
    }

    output
}
