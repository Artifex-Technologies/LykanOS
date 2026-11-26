//! LykanOS's look, made from what is in assets/logo (`lykanos-art logo`, `lykanos-art wallpaper`); built only with the `art`
//! feature, so the image's programs never carry an image library.
//!
//! The logo: the maze wolf's head (assets/logo/lykanos-source.png, white lines on black, and its straightened outline
//! lykanos-source.svg, both traced once from my picture lykanos-original.jpg in 2026-11; assets/logo/lykanos-eyes.png is the
//! inside of the eyes) in six tones as PNG and SVG, the duo (mint walls, Rosewood eyes lifted for dark backgrounds: the icon
//! and the splash), the splash after signing in and the icon in six sizes. Colours (decided 2026-11-07): the deep teal kept
//! from the first logo (#437569, the accent; mint #9fd1c3 on dark backgrounds) and Rosewood #65000b, BlackArch's kind of red;
//! Rosewood is too dark to read on a dark background, so the eyes on dark use it lifted (#b0122a).
//!
//! The wallpaper: deep teal, darker at the edges, a soft lighter field in the middle, a glow low on the right and Rosewood
//! glowing up from the lower left (#101c1c edges, #24413c middle, #437569 glow; the reference picture chosen 2026-11-07).
//! Placeholder art for the look. A little noise keeps the gradient from banding on 8-bit screens.

use image::{imageops::FilterType, GrayImage, ImageBuffer, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use std::path::Path;

/// The logo's tones.
pub const TONES: [(&str, [u8; 3]); 6] = [
    ("white", [255, 255, 255]),
    ("mint", [159, 209, 195]),
    ("green", [67, 117, 105]),
    ("rosewood", [101, 0, 11]),
    ("grey", [142, 142, 147]),
    ("black", [0, 0, 0]),
];
/// Rosewood lifted so it reads on a dark background: the duo's eyes.
pub const EYE_ON_DARK: [u8; 3] = [176, 18, 42];
const ICON_SIZES: [u32; 6] = [512, 256, 128, 64, 48, 32];
const SPLASH: u32 = 340;

type Result<T> = std::result::Result<T, String>;

/// Brightness becomes opacity: white lines on black become opaque shapes on transparency, with the dark noise and the soft
/// edge's last grey cut off (levels 14% to 86%).
pub fn opacity(v: u8) -> u8 {
    let (lo, hi) = (0.14 * 255.0, 0.86 * 255.0);
    let v = f32::from(v);
    if v <= lo {
        0
    } else if v >= hi {
        255
    } else {
        ((v - lo) * 255.0 / (hi - lo)) as u8
    }
}

/// One colour with `alpha` as its opacity.
fn tinted(alpha: &GrayImage, rgb: [u8; 3]) -> RgbaImage {
    ImageBuffer::from_fn(alpha.width(), alpha.height(), |x, y| {
        Rgba([rgb[0], rgb[1], rgb[2], alpha.get_pixel(x, y)[0]])
    })
}

/// `top` over `under` (Porter-Duff over, straight alpha), with Pillow's alpha_composite arithmetic (7 bits of precision, its
/// rounding; where `top` is fully transparent the result is `under` as it is), so the duo comes out as the Python script made
/// it, byte for byte.
pub fn over(top: &RgbaImage, under: &RgbaImage) -> RgbaImage {
    const BITS: u32 = 7;
    let div255 = |a: u32| ((a >> 8) + a) >> 8;
    ImageBuffer::from_fn(top.width(), top.height(), |x, y| {
        let (s, d) = (
            top.get_pixel(x, y).0.map(u32::from),
            under.get_pixel(x, y).0.map(u32::from),
        );
        if s[3] == 0 {
            return *under.get_pixel(x, y);
        }
        let blend = d[3] * (255 - s[3]);
        let outa255 = s[3] * 255 + blend;
        let coef1 = s[3] * 255 * 255 * (1 << BITS) / outa255;
        let coef2 = 255 * (1 << BITS) - coef1;
        let c = |i: usize| (div255(s[i] * coef1 + d[i] * coef2 + (0x80 << BITS)) >> BITS) as u8;
        Rgba([c(0), c(1), c(2), div255(outa255 + 0x80) as u8])
    })
}

/// A smaller copy, Lanczos, with the colours weighted by their opacity (as Pillow does), so transparent pixels' colour does
/// not bleed into the edges.
pub fn shrink(im: &RgbaImage, size: u32) -> RgbaImage {
    // in 0..1, as the resizer takes floating-point pictures
    let pre: image::Rgba32FImage = ImageBuffer::from_fn(im.width(), im.height(), |x, y| {
        let p = im.get_pixel(x, y).0;
        let a = f32::from(p[3]) / 255.0;
        let c = |v: u8| f32::from(v) / 255.0 * a;
        Rgba([c(p[0]), c(p[1]), c(p[2]), a])
    });
    let small = image::imageops::resize(&pre, size, size, FilterType::Lanczos3);
    ImageBuffer::from_fn(size, size, |x, y| {
        let p = small.get_pixel(x, y).0;
        let a = p[3].clamp(0.0, 1.0);
        let c = |v: f32| {
            if a > 0.0 {
                (v / a * 255.0).round().clamp(0.0, 255.0) as u8
            } else {
                0
            }
        };
        Rgba([c(p[0]), c(p[1]), c(p[2]), (a * 255.0).round() as u8])
    })
}

fn save(im: &impl SaveTo, path: &Path) -> Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    im.save_to(path)
        .map_err(|e| format!("{}: {e}", path.display()))
}

trait SaveTo {
    fn save_to(&self, p: &Path) -> image::ImageResult<()>;
}

impl SaveTo for RgbaImage {
    fn save_to(&self, p: &Path) -> image::ImageResult<()> {
        self.save(p)
    }
}

impl SaveTo for RgbImage {
    fn save_to(&self, p: &Path) -> image::ImageResult<()> {
        self.save(p)
    }
}

fn gray(path: &Path) -> Result<GrayImage> {
    let im = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .into_luma8();
    Ok(if im.dimensions() == (1024, 1024) {
        im
    } else {
        image::imageops::resize(&im, 1024, 1024, FilterType::Lanczos3)
    })
}

/// The logo set from assets/logo into assets/logo and the profile (repo is this checkout).
pub fn logo(repo: &Path) -> Result<()> {
    let assets = repo.join("assets/logo");
    let shipped = repo.join("profile/airootfs/usr/share/lykanos/logo");
    let icons = repo.join("profile/airootfs/usr/share/icons/hicolor");
    let walls = gray(&assets.join("lykanos-source.png"))?;
    let alpha: GrayImage =
        ImageBuffer::from_fn(1024, 1024, |x, y| Luma([opacity(walls.get_pixel(x, y)[0])]));
    let svg = std::fs::read_to_string(assets.join("lykanos-source.svg"))
        .map_err(|e| format!("lykanos-source.svg: {e}"))?;
    if !svg.contains("fill=\"#000000\"") {
        return Err("lykanos-source.svg has no fill=\"#000000\" to colour".into());
    }
    for (name, rgb) in TONES {
        let png = assets.join(format!("lykanos-{name}.png"));
        save(&tinted(&alpha, rgb), &png)?;
        let colour = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
        let svg_path = assets.join(format!("lykanos-{name}.svg"));
        std::fs::write(
            &svg_path,
            svg.replace("fill=\"#000000\"", &format!("fill=\"{colour}\"")),
        )
        .map_err(|e| format!("{}: {e}", svg_path.display()))?;
        for f in [&png, &svg_path] {
            let to = shipped.join(f.file_name().unwrap_or_default());
            std::fs::create_dir_all(&shipped)
                .and_then(|_| std::fs::copy(f, &to))
                .map_err(|e| format!("{}: {e}", to.display()))?;
        }
    }
    // the duo: mint walls over the red inside of the eyes
    let eyes = gray(&assets.join("lykanos-eyes.png"))?;
    let duo = over(&tinted(&alpha, TONES[1].1), &tinted(&eyes, EYE_ON_DARK));
    save(&duo, &assets.join("lykanos-duo.png"))?;
    save(&duo, &shipped.join("lykanos-duo.png"))?;
    // the splash after signing in, the size Labyrinth's is; the icon in each size (os-release's LOGO)
    save(&shrink(&duo, SPLASH), &shipped.join("lykanos-splash.png"))?;
    for s in ICON_SIZES {
        save(
            &shrink(&duo, s),
            &icons.join(format!("{s}x{s}/apps/lykanos.png")),
        )?;
    }
    println!("logo set written to {} and the profile", assets.display());
    Ok(())
}

/// A blurred shape as a mask: `inside(x, y)` gives the shape's coverage (0..255) at a pixel's centre, then a Gaussian blur of
/// `sigma` pixels (three box blurs, as fast at any size).
fn soft_mask(w: u32, h: u32, sigma: f32, inside: impl Fn(f32, f32) -> u8) -> Vec<f32> {
    let mut m: Vec<f32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| f32::from(inside(x as f32 + 0.5, y as f32 + 0.5)))
        .collect();
    for r in box_radii(sigma) {
        box_blur(&mut m, w as usize, h as usize, r);
    }
    m
}

/// The three box radii whose blurs together come closest to a Gaussian of `sigma`.
pub fn box_radii(sigma: f32) -> [usize; 3] {
    let n = 3.0;
    let ideal = ((12.0 * sigma * sigma / n) + 1.0).sqrt();
    let mut wl = ideal.floor() as i64;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;
    let m = ((12.0 * sigma * sigma - n * (wl * wl) as f32 - 4.0 * n * wl as f32 - 3.0 * n)
        / (-4.0 * wl as f32 - 4.0))
        .round() as i64;
    let size = |i: i64| (if i < m { wl } else { wu }).max(1) as usize;
    [(size(0) - 1) / 2, (size(1) - 1) / 2, (size(2) - 1) / 2]
}

/// A box blur of radius `r`, rows then columns, the edges extended.
pub fn box_blur(m: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 {
        return;
    }
    let mut line = vec![0.0f32; w.max(h)];
    let pass = |get: &dyn Fn(usize) -> f32, n: usize, out: &mut [f32]| {
        let at = |i: isize| get(i.clamp(0, n as isize - 1) as usize);
        let span = (2 * r + 1) as f32;
        let mut sum: f32 = (-(r as isize)..=r as isize).map(at).sum();
        for (i, o) in out.iter_mut().enumerate().take(n) {
            *o = sum / span;
            sum += at(i as isize + r as isize + 1) - at(i as isize - r as isize);
        }
    };
    for y in 0..h {
        let row: Vec<f32> = m[y * w..(y + 1) * w].to_vec();
        pass(&|i| row[i], w, &mut line[..w]);
        m[y * w..(y + 1) * w].copy_from_slice(&line[..w]);
    }
    for x in 0..w {
        let col: Vec<f32> = (0..h).map(|y| m[y * w + x]).collect();
        pass(&|i| col[i], h, &mut line[..h]);
        for y in 0..h {
            m[y * w + x] = line[y];
        }
    }
}

/// `colour` over `im` where the mask is (0..255).
fn paint(im: &mut RgbImage, mask: &[f32], colour: [u8; 3]) {
    let w = im.width() as usize;
    for (x, y, p) in im.enumerate_pixels_mut() {
        let a = (mask[y as usize * w + x as usize] / 255.0).clamp(0.0, 1.0);
        for i in 0..3 {
            p[i] = (f32::from(colour[i]) * a + f32::from(p[i]) * (1.0 - a)).round() as u8;
        }
    }
}

fn ellipse(x0: f32, y0: f32, x1: f32, y1: f32, fill: u8) -> impl Fn(f32, f32) -> u8 {
    let (cx, cy, rx, ry) = (
        (x0 + x1) / 2.0,
        (y0 + y1) / 2.0,
        (x1 - x0) / 2.0,
        (y1 - y0) / 2.0,
    );
    move |x, y| {
        if ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0 {
            fill
        } else {
            0
        }
    }
}

fn rounded(x0: f32, y0: f32, x1: f32, y1: f32, r: f32, fill: u8) -> impl Fn(f32, f32) -> u8 {
    move |x, y| {
        if x < x0 || x > x1 || y < y0 || y > y1 {
            return 0;
        }
        let dx = (x0 + r - x).max(x - (x1 - r)).max(0.0);
        let dy = (y0 + r - y).max(y - (y1 - r)).max(0.0);
        if dx * dx + dy * dy <= r * r {
            fill
        } else {
            0
        }
    }
}

/// The wallpaper, w by h.
pub fn wallpaper(w: u32, h: u32) -> RgbImage {
    const EDGE: [u8; 3] = [16, 28, 28];
    const MIDDLE: [u8; 3] = [36, 65, 60];
    const GLOW: [u8; 3] = [67, 117, 105];
    const ROSEWOOD: [u8; 3] = [101, 0, 11];
    const SHADE: [u8; 3] = [10, 18, 18];
    let (fw, fh) = (w as f32, h as f32);
    let mut im = RgbImage::from_pixel(w, h, Rgb(EDGE));
    // the lighter field: a wide rounded shape, very soft
    paint(
        &mut im,
        &soft_mask(
            w,
            h,
            fh * 0.12,
            rounded(fw * 0.16, fh * 0.08, fw * 0.84, fh * 0.94, fh * 0.3, 230),
        ),
        MIDDLE,
    );
    // the glow low on the right
    paint(
        &mut im,
        &soft_mask(
            w,
            h,
            fh * 0.14,
            ellipse(fw * 0.70, fh * 0.55, fw * 1.08, fh * 1.15, 170),
        ),
        GLOW,
    );
    // Rosewood glowing up from the lower left, the teal's counterpart
    paint(
        &mut im,
        &soft_mask(
            w,
            h,
            fh * 0.16,
            ellipse(-fw * 0.12, fh * 0.50, fw * 0.34, fh * 1.18, 200),
        ),
        ROSEWOOD,
    );
    // a darker band down each side
    paint(
        &mut im,
        &soft_mask(w, h, fw * 0.04, |x, _| {
            if x <= fw * 0.05 || x >= fw * 0.95 {
                200
            } else {
                0
            }
        }),
        SHADE,
    );
    // noise against banding: -1 to +2 per pixel, the same on every channel, the same every time (a fixed seed)
    let mut seed: u64 = 7;
    for p in im.pixels_mut() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let n = (seed % 4) as i16 - 1;
        for c in p.0.iter_mut() {
            *c = (i16::from(*c) + n).clamp(0, 255) as u8;
        }
    }
    im
}

/// The wallpaper saved to `out` (by default the profile's).
pub fn wallpaper_to(out: &Path, w: u32, h: u32) -> Result<()> {
    save(&wallpaper(w, h), out)?;
    println!("{}: {w}x{h}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_becomes_opacity_between_the_levels() {
        assert_eq!(opacity(0), 0);
        assert_eq!(opacity(35), 0, "dark noise cut off");
        assert_eq!(opacity(220), 255, "the soft edge's last grey made solid");
        assert_eq!(opacity(128), 128, "as the Python script's levels");
    }

    #[test]
    fn over_puts_the_walls_on_the_eyes() {
        let top = RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                Rgba([159, 209, 195, 255])
            } else {
                Rgba([159, 209, 195, 0])
            }
        });
        let under = RgbaImage::from_pixel(2, 1, Rgba([176, 18, 42, 255]));
        let o = over(&top, &under);
        assert_eq!(
            o.get_pixel(0, 0),
            &Rgba([159, 209, 195, 255]),
            "a wall covers the eye"
        );
        assert_eq!(
            o.get_pixel(1, 0),
            &Rgba([176, 18, 42, 255]),
            "the eye shows where no wall is"
        );
        let half = over(
            &RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 128])),
            &RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0])),
        );
        assert_eq!(
            half.get_pixel(0, 0),
            &Rgba([255, 255, 255, 128]),
            "over nothing, as it is"
        );
    }

    #[test]
    fn a_shrunk_logo_keeps_its_colour_at_the_edges() {
        // mint on transparent black: Lanczos on straight alpha would darken the edge; weighted by opacity it stays mint
        let im = RgbaImage::from_fn(64, 64, |x, _| {
            if x < 32 {
                Rgba([159, 209, 195, 255])
            } else {
                Rgba([0, 0, 0, 0])
            }
        });
        let s = shrink(&im, 16);
        for x in 0..16 {
            let p = s.get_pixel(x, 8);
            if p[3] > 8 {
                // Lanczos rings a little at a hard edge (as Pillow's does); a straight-alpha resize would darken it far more
                assert!((i16::from(p[1]) - 209).abs() <= 6, "x {x}: {p:?}");
            }
        }
    }

    #[test]
    fn the_blur_keeps_an_even_field_and_spreads_an_edge() {
        let (w, h) = (40, 4);
        let mut m = vec![100.0; w * h];
        box_blur(&mut m, w, h, 5);
        assert!(
            m.iter().all(|v| (v - 100.0).abs() < 1e-3),
            "an even field stays even"
        );
        let mut step: Vec<f32> = (0..w * h)
            .map(|i| if i % w < 20 { 255.0 } else { 0.0 })
            .collect();
        for r in box_radii(4.0) {
            box_blur(&mut step, w, h, r);
        }
        assert!(
            step[10] > 250.0 && step[30] < 5.0 && (step[19] - 127.5).abs() < 30.0,
            "{:?}",
            &step[..w]
        );
        assert!(box_radii(4.0).iter().all(|r| *r > 0));
    }

    #[test]
    fn the_wallpaper_is_teal_with_rosewood_low_left_and_the_same_every_time() {
        let a = wallpaper(256, 144);
        assert_eq!(a, wallpaper(256, 144));
        let p = |x, y| a.get_pixel(x, y).0;
        let (middle, low_left, corner) = (p(128, 72), p(20, 140), p(0, 0));
        assert!(
            middle[1] > corner[1] + 10,
            "lighter in the middle: {middle:?} {corner:?}"
        );
        assert!(
            low_left[0] > low_left[1],
            "Rosewood low on the left: {low_left:?}"
        );
    }
}
