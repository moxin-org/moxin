use anyhow::Result;
use makepad_widgets::math_f32::{vec4, Vec4};

pub const BYTES_PER_MB: f64 = 1_048_576.0; // (1024^2)
pub const HUGGING_FACE_BASE_URL: &str = "https://huggingface.co";

pub fn format_model_size(size: &str) -> Result<String> {
    let size_mb = size.parse::<f64>()? / BYTES_PER_MB;

    if size_mb >= 1024.0 {
        Ok(format!("{:.2} GB", size_mb / 1024.0))
    } else {
        Ok(format!("{:.2} MB", size_mb as i32))
    }
}

pub fn format_model_downloaded_size(size: &str, progress: f64) -> Result<String> {
    let size_mb = (size.parse::<f64>()? / BYTES_PER_MB) * progress / 100.0;

    if size_mb >= 1024.0 {
        Ok(format!("{:.2} GB", size_mb / 1024.0))
    } else {
        Ok(format!("{:.2} MB", size_mb as i32))
    }
}

pub fn hugging_face_model_url(model_id: &str) -> String {
    format!("{}/{}", HUGGING_FACE_BASE_URL, model_id)
}

/// Convert from hex color notation to makepad's Vec4 color.
/// Ex: Converts `0xff33cc` into `vec4(1.0, 0.2, 0.8, 1.0)`.
pub fn hex_rgb_color(hex: u32) -> Vec4 {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    vec4(r, g, b, 1.0)
}

/// Removes dashes, file extension, and capitalizes the first letter of each word.
pub fn human_readable_name(model_filename: &str) -> String {
    let name = model_filename
        .to_lowercase()
        .replace("-", " ")
        .replace(".gguf", "")
        .replace("chat.", " ")
        .replace("chat", "")
        .replace("_k", "_K")
        .replace("_m", "_M")
        .replace("_l", "_L");

    let name = name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    name
}
