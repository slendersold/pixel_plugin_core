//! Динамическая загрузка библиотеки плагина и символа `process_image` через `libloading`.

use std::ffi::CString;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

use crate::error::ProcessorError;

pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

/// Имя файла динамической библиотеки по короткому имени плагина (без расширения).
pub fn plugin_library_filename(plugin_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{plugin_name}.dll")
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{plugin_name}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{plugin_name}.so")
    }
}

pub fn resolve_plugin_path(plugin_dir: &Path, plugin_name: &str) -> PathBuf {
    plugin_dir.join(plugin_library_filename(plugin_name))
}

/// Загружает плагин, резолвит `process_image`, вызывает и выгружает библиотеку после возврата из плагина.
pub fn call_process_image(
    plugin_dir: &Path,
    plugin_name: &str,
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) -> Result<(), ProcessorError> {
    let path = resolve_plugin_path(plugin_dir, plugin_name);
    if !path.exists() {
        return Err(ProcessorError::Plugin(format!(
            "библиотека не найдена: {}",
            path.display()
        )));
    }
    let lib = unsafe { Library::new(&path) }.map_err(|e| {
        ProcessorError::Plugin(format!("не удалось загрузить {}: {e}", path.display()))
    })?;
    let process_image: Symbol<ProcessImageFn> =
        unsafe { lib.get(b"process_image\0") }.map_err(|e| {
            ProcessorError::Plugin(format!("символ process_image не найден: {e}"))
        })?;
    unsafe {
        process_image(width, height, rgba_data, params);
    }
    Ok(())
}

/// Строка параметров для FFI: `CString` живёт до вызова плагина.
pub fn params_cstring(params: &str) -> Result<CString, ProcessorError> {
    CString::new(params.as_bytes()).map_err(|_| {
        ProcessorError::Args("в параметрах встречен внутренний нулевой байт".into())
    })
}
