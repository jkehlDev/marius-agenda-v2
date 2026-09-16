use agenda_core::{AgendaConfig, AgendaIllustrations, IllustrationSlot, effective_data_dir, resolve_data_dir};
use std::time::{SystemTime, UNIX_EPOCH};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageEncoder};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Decode(#[from] image::ImageError),
}

#[derive(Clone, Copy)]
pub enum ImageRole {
    Cover,
    Day,
    Activities,
}

fn max_size(role: ImageRole) -> u32 {
    match role {
        ImageRole::Cover => 1400,
        ImageRole::Day => 900,
        ImageRole::Activities => 1000,
    }
}

/// Convert any image to high-contrast B&W WebP with pure white background.
pub fn process_illustration(
    input_path: &Path,
    output_path: &Path,
    role: ImageRole,
) -> Result<(), ImageError> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let max = max_size(role);
    let mut img = image::open(input_path)?;
    img = flatten_on_white(img);
    img = img.grayscale();
    img = normalize_contrast(img);
    img = resize_inside(img, max);
    save_webp(&img, output_path)?;
    Ok(())
}

fn flatten_on_white(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let mut canvas = image::RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut canvas, &img.to_rgba8(), 0, 0);
    DynamicImage::ImageRgba8(canvas)
}

fn normalize_contrast(img: DynamicImage) -> DynamicImage {
    // Approximate Sharp: .normalize() + .linear(1.35, -(128 * 0.35))
    let gray = img.to_luma8();
    let mut min = 255u8;
    let mut max = 0u8;
    for p in gray.pixels() {
        let v = p.0[0];
        min = min.min(v);
        max = max.max(v);
    }
    let range = (max - min).max(1) as f32;
    let mut out = image::GrayImage::new(gray.width(), gray.height());
    for (x, y, p) in gray.enumerate_pixels() {
        let norm = ((p.0[0] - min) as f32 / range) * 255.0;
        let boosted = norm * 1.35 - (128.0 * 0.35);
        let v = boosted.clamp(0.0, 255.0) as u8;
        out.put_pixel(x, y, image::Luma([v]));
    }
    DynamicImage::ImageLuma8(out)
}

fn resize_inside(img: DynamicImage, max: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w <= max && h <= max {
        return img;
    }
    let scale = (max as f32 / w as f32).min(max as f32 / h as f32);
    let nw = (w as f32 * scale).round().max(1.0) as u32;
    let nh = (h as f32 * scale).round().max(1.0) as u32;
    img.resize(nw, nh, FilterType::Lanczos3)
}

fn save_webp(img: &DynamicImage, output_path: &Path) -> Result<(), ImageError> {
    let rgba = img.to_rgba8();
    let mut buf = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut buf);
    encoder
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(image::ImageError::from)?;
    fs::write(output_path, buf)?;
    Ok(())
}

fn resolve_source_path(root: &Path, data_dir: &Path, source_path: &str) -> PathBuf {
    let p = Path::new(source_path);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let from_data = data_dir.join(source_path);
    if from_data.exists() {
        return from_data;
    }
    let from_default = resolve_data_dir(root).join(source_path);
    if from_default.exists() {
        return from_default;
    }
    root.join(source_path)
}

fn ensure_slot_processed(
    root: &Path,
    data_dir: &Path,
    slot: &IllustrationSlot,
    role: ImageRole,
    key: &str,
) -> Result<IllustrationSlot, ImageError> {
    if slot.source_path.is_none() {
        return Ok(IllustrationSlot {
            source_path: None,
            print_path: None,
        });
    }

    let print_rel = format!("images/print/{}.webp", key);
    let print_abs = data_dir.join(&print_rel);
    let source_abs = resolve_source_path(root, data_dir, slot.source_path.as_deref().unwrap());

    match process_illustration(&source_abs, &print_abs, role) {
        Ok(()) => Ok(IllustrationSlot {
            source_path: slot.source_path.clone(),
            print_path: Some(print_rel),
        }),
        Err(err) => {
            eprintln!("⚠️  Impossible de traiter {}: {}", slot.source_path.as_deref().unwrap_or(""), err);
            Ok(slot.clone())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IllustrationSlotId {
    Cover,
    Activities,
    Weekday(u8),
}

fn copy_into_uploads(data_dir: &Path, source: &Path) -> Result<String, ImageError> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .unwrap_or("png");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let rel = format!("images/uploads/{}.{}", nanos, ext);
    let dest = data_dir.join(&rel);
    fs::create_dir_all(dest.parent().unwrap())?;
    fs::copy(source, &dest)?;
    Ok(rel)
}

/// Import a user-selected image into uploads and refresh the processed print slot.
pub fn import_illustration_file(
    root: &Path,
    data_dir: &Path,
    config: &AgendaConfig,
    slot_id: IllustrationSlotId,
    source_file: &Path,
) -> Result<AgendaConfig, ImageError> {
    let rel = copy_into_uploads(data_dir, source_file)?;
    let mut config = config.clone();
    let new_slot = IllustrationSlot {
        source_path: Some(rel),
        print_path: None,
    };
    match slot_id {
        IllustrationSlotId::Cover => config.illustrations.cover = new_slot,
        IllustrationSlotId::Activities => config.illustrations.activities = new_slot,
        IllustrationSlotId::Weekday(wd) => {
            config.illustrations.set_weekday_slot(wd, new_slot);
        }
    }
    process_config_illustrations(root, &config, Some(data_dir))
}

/// Process all configured illustrations; returns updated config with printPath filled.
pub fn process_config_illustrations(
    root: &Path,
    config: &AgendaConfig,
    data_dir_override: Option<&Path>,
) -> Result<AgendaConfig, ImageError> {
    let data_dir = effective_data_dir(root, data_dir_override);
    let cover = ensure_slot_processed(
        root,
        &data_dir,
        &config.illustrations.cover,
        ImageRole::Cover,
        "cover",
    )?;
    let activities = ensure_slot_processed(
        root,
        &data_dir,
        &config.illustrations.activities,
        ImageRole::Activities,
        "activities",
    )?;

    let mut by_weekday = config.illustrations.by_weekday.clone();
    for wd in 0..7u8 {
        if !config.school_days.get(wd) {
            continue;
        }
        let slot = by_weekday
            .get(&wd.to_string())
            .cloned()
            .unwrap_or_else(|| IllustrationSlot {
                source_path: None,
                print_path: None,
            });
        let processed = ensure_slot_processed(
            root,
            &data_dir,
            &slot,
            ImageRole::Day,
            &format!("weekday-{}", wd),
        )?;
        by_weekday.insert(wd.to_string(), processed);
    }

    Ok(AgendaConfig {
        illustrations: AgendaIllustrations {
            cover,
            activities,
            by_weekday,
        },
        ..config.clone()
    })
}
