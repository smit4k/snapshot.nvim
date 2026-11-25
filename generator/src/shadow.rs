use image::{ImageBuffer, Rgba, RgbaImage};

/// Single-pass horizontal box blur on an alpha-only buffer stored row-major.
fn box_blur_h(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    let diam = radius * 2 + 1;
    for y in 0..h {
        let row = y * w;
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
    // Keep each row segment contiguous while bounding the column sums to the stack.
    const STRIPE: usize = 128;
    let diam = (radius * 2 + 1) as u32;
    for first_x in (0..w).step_by(STRIPE) {
        let count = STRIPE.min(w - first_x);
        let mut sums = [0u32; STRIPE];
        let sums = &mut sums[..count];
        for (sum, &alpha) in sums.iter_mut().zip(&src[first_x..first_x + count]) {
            *sum = (radius as u32 + 1) * alpha as u32;
        }
        for row in 0..radius {
            let start = row.min(h - 1) * w + first_x;
            for (sum, &alpha) in sums.iter_mut().zip(&src[start..start + count]) {
                *sum += alpha as u32;
            }
        }
        for y in 0..h {
            let bottom = (y + radius).min(h - 1) * w + first_x;
            let top = y.saturating_sub(radius + 1) * w + first_x;
            let target = y * w + first_x;
            for (((sum, &bottom_alpha), &top_alpha), pixel) in sums
                .iter_mut()
                .zip(&src[bottom..bottom + count])
                .zip(&src[top..top + count])
                .zip(&mut dst[target..target + count])
            {
                *sum += bottom_alpha as u32;
                *pixel = (*sum / diam).min(255) as u8;
                *sum -= top_alpha as u32;
            }
        }
    }
}

/// Approximate Gaussian blur via three successive box-blur passes on an alpha buffer.
fn gaussian_blur_alpha(buf: &mut [u8], tmp: &mut [u8], w: usize, h: usize, sigma: f32) {
    for box_size in boxes_for_gauss(sigma) {
        let radius = ((box_size - 1) / 2) as usize;
        if radius == 0 {
            continue;
        }
        box_blur_h(buf, tmp, w, h, radius);
        box_blur_v(tmp, buf, w, h, radius);
    }
}

/// Compute ideal box sizes for a 3-pass box blur approximating a Gaussian with given sigma.
fn boxes_for_gauss(sigma: f32) -> [i32; 3] {
    let n = 3.0;
    let w_ideal = ((12.0 * sigma * sigma / n) + 1.0).sqrt();
    let mut wl = w_ideal.floor() as i32;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;

    let m_ideal = (12.0 * sigma * sigma - 3.0 * wl as f32 * wl as f32 - 12.0 * wl as f32 - 9.0)
        / (-4.0 * wl as f32 - 4.0);
    let m = m_ideal.round().clamp(0.0, 3.0) as usize;

    let mut sizes = [wu; 3];
    for slot in sizes.iter_mut().take(m) {
        *slot = wl;
    }
    sizes
}

fn source_over(src: Rgba<u8>, dst: Rgba<u8>) -> Rgba<u8> {
    let sa = src[3] as f32 / 255.0;
    if sa <= 0.0 {
        return dst;
    }

    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    let r = (src[0] as f32 * sa + dst[0] as f32 * da * (1.0 - sa)) / out_a;
    let g = (src[1] as f32 * sa + dst[1] as f32 * da * (1.0 - sa)) / out_a;
    let b = (src[2] as f32 * sa + dst[2] as f32 * da * (1.0 - sa)) / out_a;
    Rgba([r as u8, g as u8, b as u8, (out_a * 255.0) as u8])
}

pub fn composite_image_onto(
    output: &mut RgbaImage,
    overlay: &RgbaImage,
    offset_x: u32,
    offset_y: u32,
) {
    let (ow, oh) = overlay.dimensions();
    for y in 0..oh {
        for x in 0..ow {
            let src = *overlay.get_pixel(x, y);
            if src[3] == 0 {
                continue;
            }

            let dx = x + offset_x;
            let dy = y + offset_y;
            let pixel = if src[3] == 255 {
                src
            } else {
                source_over(src, *output.get_pixel(dx, dy))
            };
            output.put_pixel(dx, dy, pixel);
        }
    }
}

fn has_visible_alpha(buf: &[u8]) -> bool {
    buf.iter().any(|&alpha| alpha > 0)
}

pub fn shadow_blur_margin(sigma: f32) -> u32 {
    (sigma * 3.0).ceil() as u32
}

/// Compose a drop-shadow behind `card` and return the final image.
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
    let margin = shadow_blur_margin(shadow_blur) + outer_padding;
    let out_w = cw + margin * 2;
    let out_h = ch + margin * 2;

    let mut output: RgbaImage = ImageBuffer::from_pixel(out_w, out_h, outer_bg);

    if shadow_opacity > 0.0 && shadow_blur >= 0.0 {
        let buf_w = out_w as usize;
        let buf_h = out_h as usize;
        let mut alpha_buf = vec![0u8; buf_w * buf_h];

        for cy in 0..ch {
            for cx in 0..cw {
                let dx = margin as i32 + offset_x + cx as i32;
                let dy = margin as i32 + offset_y + cy as i32;
                if dx < 0 || dy < 0 {
                    continue;
                }

                let dx = dx as usize;
                let dy = dy as usize;
                if dx < buf_w && dy < buf_h {
                    alpha_buf[dy * buf_w + dx] = card.get_pixel(cx, cy)[3];
                }
            }
        }

        if has_visible_alpha(&alpha_buf) {
            if shadow_blur > 0.0 {
                let mut tmp = vec![0u8; buf_w * buf_h];
                gaussian_blur_alpha(&mut alpha_buf, &mut tmp, buf_w, buf_h, shadow_blur);
            }

            for y in 0..out_h {
                for x in 0..out_w {
                    let alpha = alpha_buf[y as usize * buf_w + x as usize];
                    if alpha == 0 {
                        continue;
                    }

                    let shadow_alpha = ((alpha as f32 / 255.0) * shadow_opacity * 255.0) as u8;
                    if shadow_alpha == 0 {
                        continue;
                    }

                    let dst = *output.get_pixel(x, y);
                    output.put_pixel(x, y, source_over(Rgba([0, 0, 0, shadow_alpha]), dst));
                }
            }
        }
    }

    composite_image_onto(&mut output, card, margin, margin);
    output
}

#[cfg(test)]
mod tests {
    use super::{
        apply_outer_shadow, box_blur_v, boxes_for_gauss, composite_image_onto, source_over,
    };
    use image::{ImageBuffer, Rgba};

    #[test]
    fn gaussian_boxes_are_stable() {
        assert_eq!(boxes_for_gauss(0.0), [1, 1, 1]);
        assert_eq!(boxes_for_gauss(1.5), [3, 3, 3]);
    }

    #[test]
    fn vertical_blur_preserves_existing_values_across_stripes() {
        let width = 131;
        let height = 3;
        let src: Vec<u8> = (0..width * height).map(|i| (i * 93) as u8).collect();
        let mut dst = vec![0; src.len()];

        for radius in [1, 4] {
            box_blur_v(&src, &mut dst, width, height, radius);
            let diameter = radius * 2 + 1;
            for (i, &pixel) in dst.iter().enumerate() {
                let (x, y) = (i % width, i / width);
                let sum: usize = (0..diameter)
                    .map(|offset| {
                        let row = (y + offset).saturating_sub(radius).min(height - 1);
                        usize::from(src[row * width + x])
                    })
                    .sum();
                // The original rolling accumulator kept an extra first-row sample.
                let expected = ((sum + usize::from(src[x])) / diameter).min(255);
                assert_eq!(usize::from(pixel), expected, "radius {radius}, ({x}, {y})");
            }
        }
    }

    #[test]
    fn source_over_matches_expected_alpha_blend() {
        let src = Rgba([200, 100, 50, 128]);
        let dst = Rgba([10, 20, 30, 255]);
        let out = source_over(src, dst);

        assert_eq!(out[3], 255);
        assert!(out[0] > dst[0]);
        assert!(out[1] > dst[1]);
    }

    #[test]
    fn composite_image_onto_respects_offsets_and_alpha() {
        let background = Rgba([0, 0, 0, 255]);
        let mut base = ImageBuffer::from_pixel(5, 3, background);
        let overlay =
            ImageBuffer::from_fn(3, 1, |x, _| Rgba([255, 0, 0, [255, 128, 0][x as usize]]));

        composite_image_onto(&mut base, &overlay, 2, 1);

        assert_eq!(*base.get_pixel(0, 0), background);
        assert_eq!(*base.get_pixel(2, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*base.get_pixel(3, 1), Rgba([128, 0, 0, 255]));
        assert_eq!(*base.get_pixel(4, 1), background);
    }

    #[test]
    fn outer_shadow_keeps_card_visible_without_shadow() {
        let card = ImageBuffer::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let output = apply_outer_shadow(&card, 0.0, 0.0, 0, 0, Rgba([1, 2, 3, 255]), 1);

        assert_eq!(output.dimensions(), (4, 4));
        assert_eq!(*output.get_pixel(1, 1), Rgba([255, 255, 255, 255]));
        assert_eq!(*output.get_pixel(0, 0), Rgba([1, 2, 3, 255]));
    }
}
