use anyhow::{Context, Result, bail};
use clap::Parser;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, imageops};
use qrcode::{EcLevel, QrCode};
use rusttype::{Font, Scale, point};
use url::Url;

#[derive(Parser, Debug)]
#[command(
    name = "qrbrand",
    about = "Generate a scannable QR code PNG from a URL, optionally with a centered logo."
)]
struct Args {
    /// URL to encode (e.g. https://github.com/softwarewrighter/speed-kings)
    #[arg(short = 'u', long = "url")]
    url: String,

    /// Optional center image/logo (png/jpg)
    #[arg(short = 'i', long = "image")]
    image: Option<String>,

    /// Output PNG path
    #[arg(short = 'o', long = "out", default_value = "qrcode.png")]
    out: String,

    /// Size (in pixels) of the QR portion (square). Higher is better for video.
    #[arg(long = "size", default_value_t = 1024)]
    size: u32,

    /// Quiet zone size in modules (border). 4 is the usual minimum.
    #[arg(long = "quiet", default_value_t = 4)]
    quiet: u32,

    /// Logo size as a fraction of QR width (0.10..0.30 recommended).
    #[arg(long = "logo-scale", default_value_t = 0.20)]
    logo_scale: f32,

    /// Draw a white plate behind the logo for scan reliability.
    #[arg(long = "logo-plate", default_value_t = true)]
    logo_plate: bool,

    /// Extra padding around the logo plate (fraction of logo size).
    #[arg(long = "logo-pad", default_value_t = 0.18)]
    logo_pad: f32,

    /// Render the URL as text below the QR code.
    #[arg(
        short = 's',
        long = "show-url",
        default_value_t = false,
        conflicts_with = "alt_text"
    )]
    show_url: bool,

    /// Render alternate text below the QR code instead of the URL.
    #[arg(short = 'a', long = "alt-text", conflicts_with = "show_url")]
    alt_text: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate URL (catches missing scheme; ensures https:// etc.)
    let parsed = Url::parse(&args.url)
        .with_context(|| format!("Invalid URL: {} (did you include https:// ?)", args.url))?;

    // Generate QR with high error correction (important for logo overlays).
    let code = QrCode::with_error_correction_level(parsed.as_str().as_bytes(), EcLevel::H)
        .context("Failed to build QR code")?;

    // Render QR to RGBA image (square).
    let mut qr_img = render_qr_rgba(&code, args.size, args.quiet)?;

    // Optional logo overlay.
    if let Some(path) = args.image.as_deref() {
        overlay_logo_center(
            &mut qr_img,
            path,
            args.logo_scale,
            args.logo_plate,
            args.logo_pad,
        )?;
    }

    // Optionally add text below QR by extending the canvas height.
    let final_img = if args.show_url {
        add_url_text_below(&qr_img, parsed.as_str())?
    } else if let Some(alt_text) = &args.alt_text {
        add_url_text_below(&qr_img, alt_text)?
    } else {
        qr_img
    };

    final_img
        .save(&args.out)
        .with_context(|| format!("Failed to write output PNG: {}", args.out))?;

    eprintln!("Wrote {}", args.out);
    Ok(())
}

/// Render a QR code into an RGBA ImageBuffer of size (approximately) `size` x `size`,
/// including a quiet zone of `quiet_modules` around the code.
/// The output may be slightly smaller than `size` to keep modules crisp.
fn render_qr_rgba(
    code: &QrCode,
    size: u32,
    quiet_modules: u32,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let module_count = code.width() as u32;
    if module_count == 0 {
        bail!("QR module count is zero");
    }

    // Total modules including quiet zone border.
    let total_modules = module_count + 2 * quiet_modules;

    // Compute pixels per module. Floor to integer to keep modules crisp.
    let ppm = size / total_modules;
    if ppm < 2 {
        bail!(
            "Requested size {} too small for total modules {} (ppm={}). Increase --size.",
            size,
            total_modules,
            ppm
        );
    }

    // Actual output size (may be slightly smaller than requested to preserve crisp modules).
    let out_w = ppm * total_modules;
    let out_h = out_w;

    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);

    let mut img = ImageBuffer::from_pixel(out_w, out_h, white);

    // Draw modules.
    for y in 0..module_count {
        for x in 0..module_count {
            let is_dark = matches!(code[(x as usize, y as usize)], qrcode::Color::Dark);
            if is_dark {
                // Offset by quiet zone.
                let mx = x + quiet_modules;
                let my = y + quiet_modules;

                let px0 = mx * ppm;
                let py0 = my * ppm;

                for py in py0..(py0 + ppm) {
                    for px in px0..(px0 + ppm) {
                        img.put_pixel(px, py, black);
                    }
                }
            }
        }
    }

    Ok(img)
}

/// Overlay a logo image centered on the QR.
/// The logo is resized to `logo_scale` of QR width.
/// Optionally draws a white plate behind it to improve scan reliability.
fn overlay_logo_center(
    qr_img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    logo_path: &str,
    logo_scale: f32,
    logo_plate: bool,
    logo_pad: f32,
) -> Result<()> {
    if !(0.05..=0.35).contains(&logo_scale) {
        bail!("--logo-scale should be between ~0.05 and 0.35 for scan reliability");
    }

    let qr_w = qr_img.width();
    let qr_h = qr_img.height();
    let target_logo_w = (qr_w as f32 * logo_scale).round() as u32;
    let target_logo_h = target_logo_w; // keep square-ish; we’ll preserve aspect by fit.

    let logo = image::open(logo_path)
        .with_context(|| format!("Failed to open logo image: {}", logo_path))?;

    // Resize logo to fit within target box, preserving aspect ratio.
    let resized = resize_fit(&logo, target_logo_w, target_logo_h);

    let lw = resized.width();
    let lh = resized.height();

    let x0 = (qr_w - lw) / 2;
    let y0 = (qr_h - lh) / 2;

    // Optional white plate behind logo.
    if logo_plate {
        let pad_px = ((lw.max(lh) as f32) * logo_pad).round() as u32;
        let plate_w = lw + 2 * pad_px;
        let plate_h = lh + 2 * pad_px;

        let plate_x0 = (qr_w - plate_w) / 2;
        let plate_y0 = (qr_h - plate_h) / 2;

        draw_rect(
            qr_img,
            plate_x0,
            plate_y0,
            plate_w,
            plate_h,
            Rgba([255, 255, 255, 255]),
        );
    }

    // Composite logo onto QR (alpha-aware).
    imageops::overlay(qr_img, &resized.to_rgba8(), x0.into(), y0.into());
    Ok(())
}

/// Resize while preserving aspect ratio to fit within (max_w, max_h).
fn resize_fit(img: &DynamicImage, max_w: u32, max_h: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img.clone();
    }
    let scale_w = max_w as f32 / w as f32;
    let scale_h = max_h as f32 / h as f32;
    let scale = scale_w.min(scale_h).min(1.0);

    let new_w = (w as f32 * scale).round().max(1.0) as u32;
    let new_h = (h as f32 * scale).round().max(1.0) as u32;

    img.resize(new_w, new_h, imageops::FilterType::Lanczos3)
}

fn draw_rect(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    color: Rgba<u8>,
) {
    let max_x = (x0 + w).min(img.width());
    let max_y = (y0 + h).min(img.height());

    for y in y0..max_y {
        for x in x0..max_x {
            img.put_pixel(x, y, color);
        }
    }
}

/// Add a white band below the QR code and render the URL as text.
/// Uses an embedded font (DejaVuSans) so no OS font dependency.
fn add_url_text_below(
    qr_img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    url_text: &str,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    // Embed a widely-available, permissive font.
    // NOTE: This requires you to add the font bytes. See instructions below.
    static FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

    let font = Font::try_from_bytes(FONT_BYTES).context("Failed to load embedded font")?;

    let qr_w = qr_img.width();
    let qr_h = qr_img.height();

    // Band height heuristics: enough for one line of text with padding.
    let band_h = (qr_h as f32 * 0.18).round().max(120.0) as u32;

    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);

    let mut out = ImageBuffer::from_pixel(qr_w, qr_h + band_h, white);

    // Copy QR into top.
    imageops::overlay(&mut out, qr_img, 0, 0);

    // Determine font size so the URL fits within width with margins.
    let margin_x = (qr_w as f32 * 0.06).round().max(24.0) as u32;
    let max_text_w = qr_w.saturating_sub(2 * margin_x);

    // Start from a reasonable size and shrink until it fits.
    let mut font_px = (band_h as f32 * 0.35).round().max(18.0);
    let min_font_px = 14.0;

    loop {
        let scale = Scale::uniform(font_px);
        let text_w = measure_text_width(&font, scale, url_text);

        if text_w <= max_text_w as f32 || font_px <= min_font_px {
            break;
        }
        font_px *= 0.92;
    }

    let scale = Scale::uniform(font_px);
    let v_metrics = font.v_metrics(scale);

    // Baseline positioning: vertically centered in the band.
    let text_h = (v_metrics.ascent - v_metrics.descent).ceil();
    let band_y0 = qr_h;
    let y_center = band_y0 as f32 + (band_h as f32 / 2.0);
    let baseline_y = y_center + (text_h / 2.0) - v_metrics.descent;

    // Horizontally centered.
    let text_w = measure_text_width(&font, scale, url_text);
    let start_x = ((qr_w as f32 - text_w) / 2.0).max(margin_x as f32);

    draw_text_rgba(&mut out, &font, scale, start_x, baseline_y, url_text, black);

    Ok(out)
}

/// Measure the width of a string in pixels for a given font/scale.
fn measure_text_width(font: &Font<'_>, scale: Scale, text: &str) -> f32 {
    let mut x = 0.0;
    let mut last_id = None;

    for ch in text.chars() {
        let glyph = font.glyph(ch).scaled(scale);

        // kerning
        if let Some(prev) = last_id {
            x += font.pair_kerning(scale, prev, glyph.id());
        }
        last_id = Some(glyph.id());

        x += glyph.h_metrics().advance_width;
    }

    x
}

/// Draw a single line of text into an RGBA image buffer.
fn draw_text_rgba(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &Font<'_>,
    scale: Scale,
    start_x: f32,
    baseline_y: f32,
    text: &str,
    color: Rgba<u8>,
) {
    let mut x = start_x;
    let mut last_id = None;

    for ch in text.chars() {
        let glyph = font.glyph(ch).scaled(scale);

        if let Some(prev) = last_id {
            x += font.pair_kerning(scale, prev, glyph.id());
        }
        last_id = Some(glyph.id());

        // Save advance width BEFORE positioned() consumes the glyph
        let adv = glyph.h_metrics().advance_width;

        let positioned = glyph.positioned(point(x, baseline_y));

        if let Some(bb) = positioned.pixel_bounding_box() {
            positioned.draw(|gx, gy, v| {
                let px = bb.min.x + gx as i32;
                let py = bb.min.y + gy as i32;
                if px >= 0 && py >= 0 {
                    let (pxu, pyu) = (px as u32, py as u32);
                    if pxu < img.width() && pyu < img.height() {
                        let dst = img.get_pixel(pxu, pyu);
                        let a = (v * 255.0) as u8;
                        let blended = blend_over(*dst, color, a);
                        img.put_pixel(pxu, pyu, blended);
                    }
                }
            });
        }

        x += adv;
    }
}

/// Blend src color over dst with alpha coverage `a` (0..255), assuming src is opaque.
fn blend_over(dst: Rgba<u8>, src: Rgba<u8>, a: u8) -> Rgba<u8> {
    let inv = 255u16 - a as u16;

    let dr = dst[0] as u16;
    let dg = dst[1] as u16;
    let db = dst[2] as u16;

    let sr = src[0] as u16;
    let sg = src[1] as u16;
    let sb = src[2] as u16;

    let r = (sr * a as u16 + dr * inv) / 255;
    let g = (sg * a as u16 + dg * inv) / 255;
    let b = (sb * a as u16 + db * inv) / 255;

    Rgba([r as u8, g as u8, b as u8, 255])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_blend_over() {
        let white = Rgba([255, 255, 255, 255]);
        let black = Rgba([0, 0, 0, 255]);
        let red = Rgba([255, 0, 0, 255]);

        // Test blending black over white with full opacity
        let result = blend_over(white, black, 255);
        assert_eq!(result, black);

        // Test blending red over white with half opacity
        let result = blend_over(white, red, 128);
        assert_eq!(result[0], 255); // Red component
        assert!(result[1] < 255); // Green component reduced
        assert!(result[2] < 255); // Blue component reduced
    }

    #[test]
    fn test_measure_text_width() {
        static FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
        let font = Font::try_from_bytes(FONT_BYTES).expect("Failed to load font");
        let scale = Scale::uniform(20.0);

        let width_a = measure_text_width(&font, scale, "A");
        let width_aa = measure_text_width(&font, scale, "AA");

        assert!(width_aa > width_a);
        assert!(width_a > 0.0);
    }

    #[test]
    fn test_resize_fit() {
        // Create a small test image
        let img = ImageBuffer::from_fn(100, 200, |x, y| Rgba([x as u8, y as u8, 128, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        // Test fitting within larger bounds (should not resize)
        let resized = resize_fit(&dynamic_img, 200, 400);
        assert_eq!(resized.width(), 100);
        assert_eq!(resized.height(), 200);

        // Test fitting within smaller bounds (should resize)
        let resized = resize_fit(&dynamic_img, 50, 50);
        assert!(resized.width() <= 50);
        assert!(resized.height() <= 50);
        assert!(resized.width() > 0);
        assert!(resized.height() > 0);
    }

    #[test]
    fn test_draw_rect() {
        let mut img = ImageBuffer::from_pixel(10, 10, Rgba([0, 0, 0, 255]));
        let white = Rgba([255, 255, 255, 255]);

        draw_rect(&mut img, 2, 2, 3, 3, white);

        // Check that the rectangle was drawn
        assert_eq!(img.get_pixel(2, 2), &white);
        assert_eq!(img.get_pixel(4, 4), &white);
        // Check that pixels outside rectangle are still black
        assert_eq!(img.get_pixel(0, 0), &Rgba([0, 0, 0, 255]));
        assert_eq!(img.get_pixel(9, 9), &Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_url_validation() {
        // This is an integration test that would be run separately
        // For now, just verify the URL parsing logic would work
        use url::Url;

        let valid_url = "https://example.com";
        let parsed = Url::parse(valid_url);
        assert!(parsed.is_ok());

        let invalid_url = "not-a-url";
        let parsed = Url::parse(invalid_url);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_alt_text_feature() {
        // Test that add_url_text_below works with arbitrary text
        let img = ImageBuffer::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let test_text = "Test Alt Text";

        // This should not panic
        let result = add_url_text_below(&img, test_text);
        assert!(result.is_ok());

        let extended_img = result.unwrap();
        assert!(extended_img.height() > img.height());
        assert_eq!(extended_img.width(), img.width());
    }
}
