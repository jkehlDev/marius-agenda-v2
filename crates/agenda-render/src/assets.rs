use agenda_core::{AgendaConfig, IllustrationSlot, WeekdayIndex};
use agenda_core::resolve_data_dir;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn mime_by_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        ".webp" => "image/webp",
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn file_to_data_uri(abs_path: &Path) -> Option<String> {
    let bytes = fs::read(abs_path).ok()?;
    let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime = mime_by_ext(&format!(".{}", ext));
    let b64 = STANDARD.encode(bytes);
    Some(format!("data:{};base64,{}", mime, b64))
}

fn resolve_slot_file(root: &Path, slot: &IllustrationSlot) -> Option<PathBuf> {
    let candidate = slot
        .print_path
        .as_deref()
        .or(slot.source_path.as_deref())?;
    let candidate_path = Path::new(candidate);
    if candidate_path.is_absolute() {
        return candidate_path.exists().then_some(candidate_path.to_path_buf());
    }
    let data_dir = resolve_data_dir(root);
    let from_data = data_dir.join(candidate);
    if from_data.exists() {
        return Some(from_data);
    }
    let from_root = root.join(candidate);
    if from_root.exists() {
        return Some(from_root);
    }
    None
}

pub fn illustration_data_uri(root: &Path, slot: &IllustrationSlot) -> Option<String> {
    resolve_slot_file(root, slot).and_then(|p| file_to_data_uri(&p))
}

pub fn cover_image_src(root: &Path, config: &AgendaConfig) -> Option<String> {
    illustration_data_uri(root, &config.illustrations.cover)
}

pub fn activities_image_src(root: &Path, config: &AgendaConfig) -> Option<String> {
    illustration_data_uri(root, &config.illustrations.activities)
}

pub fn weekday_image_src(root: &Path, config: &AgendaConfig, weekday: WeekdayIndex) -> Option<String> {
    config
        .illustrations
        .weekday_slot(weekday)
        .and_then(|slot| illustration_data_uri(root, slot))
}

static FONT_FACE: OnceLock<String> = OnceLock::new();

pub fn script_font_face_css(root: &Path) -> String {
    FONT_FACE
        .get_or_init(|| {
            let font_path = root.join("fonts").join("MarckScript-Regular.ttf");
            let bytes = fs::read(&font_path).expect("Marck Script font missing under fonts/");
            let b64 = STANDARD.encode(bytes);
            format!(
                "@font-face {{
  font-family: \"Marck Script\";
  src: url(\"data:font/truetype;base64,{}\") format(\"truetype\");
  font-weight: 400;
  font-style: normal;
  font-display: block;
}}",
                b64
            )
        })
        .clone()
}
