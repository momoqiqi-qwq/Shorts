use std::{borrow::Cow, fs, fs::File, io::BufWriter, path::{Path, PathBuf}};

use arboard::{Clipboard, ImageData};
use chrono::Local;
use image::{codecs::{gif::GifEncoder, jpeg::JpegEncoder}, DynamicImage, ImageFormat, RgbaImage};

use crate::config::{AppConfig, ExportFormat};

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub path: Option<PathBuf>,
    pub copied_to_clipboard: bool,
}

pub fn export_image(img: &RgbaImage, config: &AppConfig) -> anyhow::Result<ExportResult> {
    let mut path = None;
    if config.export.auto_save {
        let out_path = build_output_path(config)?;
        save_to_path(img, &out_path, config)?;
        if config.export.open_after_save {
            let _ = open::that(&out_path);
        }
        path = Some(out_path);
    }

    let copied_to_clipboard = if config.export.auto_copy_clipboard {
        copy_to_clipboard(img).is_ok()
    } else {
        false
    };

    Ok(ExportResult { path, copied_to_clipboard })
}

pub fn build_output_path(config: &AppConfig) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(&config.export.save_dir)?;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ext = config.export.format.extension();
    let mut filename = config
        .export
        .filename_template
        .replace("{timestamp}", &timestamp)
        .replace("{ext}", ext);
    if Path::new(&filename).extension().is_none() {
        filename.push('.');
        filename.push_str(ext);
    }
    Ok(config.export.save_dir.join(filename))
}

pub fn save_to_path(img: &RgbaImage, path: &Path, config: &AppConfig) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    match config.export.format {
        ExportFormat::Png => DynamicImage::ImageRgba8(img.clone()).save_with_format(path, ImageFormat::Png)?,
        ExportFormat::Bmp => DynamicImage::ImageRgba8(img.clone()).save_with_format(path, ImageFormat::Bmp)?,
        ExportFormat::Gif => save_gif(img, path)?,
        ExportFormat::Jpeg => save_jpeg(img, path, config.export.jpeg_quality)?,
        ExportFormat::Webp => save_webp(img, path, config.export.webp_lossless, config.export.webp_quality)?,
        ExportFormat::Avif => save_avif(img, path, config.export.avif_quality, config.export.avif_speed)?,
    }
    Ok(())
}

fn save_jpeg(img: &RgbaImage, path: &Path, quality: u8) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let rgb = DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let mut encoder = JpegEncoder::new_with_quality(writer, quality.clamp(1, 100));
    encoder.encode_image(&DynamicImage::ImageRgb8(rgb))?;
    Ok(())
}

fn save_gif(img: &RgbaImage, path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = GifEncoder::new(writer);
    encoder.encode_frame(image::Frame::new(img.clone()))?;
    Ok(())
}

fn save_webp(img: &RgbaImage, path: &Path, lossless: bool, quality: u8) -> anyhow::Result<()> {
    let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
    let memory = if lossless {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality.clamp(1, 100) as f32)
    };
    fs::write(path, &*memory)?;
    Ok(())
}

fn save_avif(img: &RgbaImage, path: &Path, quality: f32, speed: u8) -> anyhow::Result<()> {
    let pixels: Vec<ravif::RGBA8> = img
        .pixels()
        .map(|p| ravif::RGBA8 { r: p[0], g: p[1], b: p[2], a: p[3] })
        .collect();
    let encoded = ravif::Encoder::new()
        .with_quality(quality.clamp(1.0, 100.0))
        .with_speed(speed.clamp(1, 10))
        .encode_rgba(ravif::Img::new(pixels.as_slice(), img.width() as usize, img.height() as usize))?;
    fs::write(path, encoded.avif_file)?;
    Ok(())
}

pub fn copy_to_clipboard(img: &RgbaImage) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_image(ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: Cow::Owned(img.clone().into_raw()),
    })?;
    Ok(())
}
