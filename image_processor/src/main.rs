//! CLI: PNG → плагин (FFI `process_image`) → PNG.

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use image::ImageFormat;
use image::ImageReader;

use image_processor::contract::{
    normalize_params_file_content, rgba_buffer_byte_len, CLI_DATA_CONTRACT_HELP,
};
use image_processor::error::ProcessorError;
use image_processor::plugin_loader::{call_process_image, params_cstring};

#[derive(Parser, Debug)]
#[command(
    name = "image_processor",
    version,
    about = "Обработка PNG через динамические плагины (FFI process_image).",
    arg_required_else_help = true,
    after_long_help = CLI_DATA_CONTRACT_HELP
)]
struct Args {
    /// Путь к исходному PNG
    #[arg(value_name = "INPUT_PNG")]
    input: PathBuf,

    /// Путь для сохранения результата (PNG)
    #[arg(value_name = "OUTPUT_PNG")]
    output: PathBuf,

    /// Имя плагина без расширения (например mirror_c, blur_plugin, rotate_go)
    #[arg(value_name = "PLUGIN")]
    plugin: String,

    /// Текстовый файл с параметрами для плагина (см. --help после «Договор данных»)
    #[arg(value_name = "PARAMS_FILE")]
    params: PathBuf,

    /// Каталог с собранными плагинами (.so / .dylib / .dll)
    #[arg(
        long = "plugin-path",
        value_name = "DIR",
        default_value = "target/debug"
    )]
    plugin_path: PathBuf,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// После успешного `rotate_go` буфер — изображение **height × width** (поворот на 90°).
/// Меняем размеры сохранения только если params — **JSON-объект** (как в плагине); иначе буфер не трогали.
fn rotate_go_save_dimensions(plugin: &str, params_json: &str, w: u32, h: u32) -> (u32, u32) {
    if plugin != "rotate_go" {
        return (w, h);
    }
    match serde_json::from_str::<serde_json::Value>(params_json.trim()) {
        Ok(serde_json::Value::Object(_)) => (h, w),
        _ => (w, h),
    }
}

fn run() -> Result<(), ProcessorError> {
    let args = Args::parse();

    if !args.input.is_file() {
        return Err(ProcessorError::Args(format!(
            "входной PNG не найден или не файл: {}",
            args.input.display()
        )));
    }
    if !args.params.is_file() {
        return Err(ProcessorError::Args(format!(
            "файл параметров не найден или не файл: {}",
            args.params.display()
        )));
    }
    if !args.plugin_path.is_dir() {
        return Err(ProcessorError::Args(format!(
            "каталог плагинов не найден или не каталог: {}",
            args.plugin_path.display()
        )));
    }

    let params_str = fs::read_to_string(&args.params)?;
    let params_normalized = normalize_params_file_content(&params_str);
    let params_c = params_cstring(params_normalized)?;

    let reader = ImageReader::open(&args.input)?.with_guessed_format()?;
    if reader.format() != Some(ImageFormat::Png) {
        return Err(ProcessorError::Args(format!(
            "ожидается PNG по содержимому/расширению, получено: {}",
            args.input.display()
        )));
    }
    let img = reader.decode()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut buf = rgba.into_raw();

    let expected = rgba_buffer_byte_len(width, height).ok_or_else(|| {
        ProcessorError::Image("слишком большое изображение (переполнение размера буфера)".into())
    })?;
    if buf.len() != expected {
        return Err(ProcessorError::Image(format!(
            "внутренняя ошибка: ожидалось {expected} байт RGBA, фактически {}",
            buf.len()
        )));
    }

    // `buf` и `params_c` должны жить до возврата из `process_image`.
    call_process_image(
        &args.plugin_path,
        &args.plugin,
        width,
        height,
        buf.as_mut_ptr(),
        params_c.as_ptr(),
    )?;

    let (save_w, save_h) =
        rotate_go_save_dimensions(&args.plugin, params_normalized, width, height);
    let out = image::RgbaImage::from_raw(save_w, save_h, buf)
        .ok_or_else(|| ProcessorError::Image("некорректный размер буфера после плагина".into()))?;
    out.save(&args.output)?;

    Ok(())
}
