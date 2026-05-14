//! CLI: PNG → плагин (FFI) → PNG. Полная обработка будет реализована по ТЗ модуля 4.

use std::fs;
use std::path::PathBuf;

use clap::Parser;

use image_processor::error::ProcessorError;
use image_processor::plugin_loader::{call_process_image, params_cstring};

#[derive(Parser, Debug)]
#[command(name = "image_processor", about = "Обработка PNG через динамические плагины")]
struct Args {
    /// Путь к исходному PNG
    input: PathBuf,
    /// Путь для сохранения результата (PNG)
    output: PathBuf,
    /// Имя плагина без расширения (например mirror_plugin)
    plugin: String,
    /// Файл с текстовыми параметрами для плагина
    params: PathBuf,
    /// Каталог с собранными .so/.dll плагинами
    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() -> Result<(), ProcessorError> {
    let args = Args::parse();

    if !args.input.exists() {
        return Err(ProcessorError::Args(format!(
            "входной файл не найден: {}",
            args.input.display()
        )));
    }
    if !args.params.exists() {
        return Err(ProcessorError::Args(format!(
            "файл параметров не найден: {}",
            args.params.display()
        )));
    }

    let params_str = fs::read_to_string(&args.params)?;
    let params_c = params_cstring(params_str.trim_end())?;

    let img = image::open(&args.input)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut buf = rgba.into_raw();

    // Буфер и CString должны жить до конца вызова плагина.
    call_process_image(
        &args.plugin_path,
        &args.plugin,
        width,
        height,
        buf.as_mut_ptr(),
        params_c.as_ptr(),
    )?;

    let out = image::RgbaImage::from_raw(width, height, buf)
        .ok_or_else(|| ProcessorError::Image("некорректный размер буфера после плагина".into()))?;
    out.save(&args.output)?;

    Ok(())
}
