//! Договор данных между бинарём `image_processor` и плагинами (`cdylib`, символ `process_image`).
//!
//! # Буфер изображения
//! - Пиксели в формате **RGBA8**: подряд байты `R`, `G`, `B`, `A` на пиксель.
//! - Упорядочивание: **построчно**, слева направо, сверху вниз — как у [`image::RgbaImage`].
//! - Длина буфера в байтах: `width * height * 4` (см. [`rgba_buffer_byte_len`]).
//! - Плагин изменяет байты **на месте** в том же буфере; выход за границы — ошибка плагина.
//!
//! # Строка `params` (FFI `const char *`)
//! - В плагин передаётся **одна** C-строка: содержимое файла параметров из CLI после
//!   [`normalize_params_file_content`]: обрезаны пробелы/переводы строк по краям, внутренние
//!   байты не меняются.
//! - Строка **NUL-терминирована** только на границе FFI; внутри текста байт `0x00` недопустим
//!   (иначе [`crate::plugin_loader::params_cstring`] вернёт ошибку).
//! - Интерпретация текста — задача плагина; для учебных плагинов этого модуля рекомендуется
//!   **JSON UTF-8** со схемами ниже.

/// Байт на компонент пикселя; пиксель RGBA = 4 байта.
pub const RGBA_BYTES_PER_PIXEL: u64 = 4;

/// Рекомендуемый JSON для зеркала на C (`mirror_c`).
pub const MIRROR_C_PARAMS_JSON_EXAMPLE: &str = r#"{"horizontal":true,"vertical":false}"#;

/// Рекомендуемый JSON для плагина размытия на Rust (`blur_plugin`).
pub const BLUR_PARAMS_JSON_EXAMPLE: &str = r#"{"radius":2,"iterations":3}"#;

/// Поворот на 90° на Go (`rotate_go`); `clockwise` по умолчанию `true`. После плагина хост сохраняет PNG с размерами **height × width**.
pub const ROTATE_GO_PARAMS_JSON_EXAMPLE: &str = r#"{"clockwise":true}"#;

/// Текст для `clap` (`after_long_help`): краткое напоминание о договоре для оператора CLI.
pub const CLI_DATA_CONTRACT_HELP: &str = r#"Договор данных (host ↔ plugin):

Изображение:
  Формат файла входа: PNG. Пиксели в памяти: RGBA8, построчно (как image::RgbaImage).
  Размер буфера байт = width * height * 4; плагин пишет в тот же буфер (in-place).

Параметры (файл params):
  В плагин уходит одна C-строка: содержимое файла без ведущих/хвостовых пробелов и переводов строк.
  Рекомендуемый формат — JSON UTF-8, например:
  mirror_c (C):   {"horizontal":true,"vertical":false}
  blur_plugin (Rust): {"radius":2,"iterations":3}
  rotate_go (Go): {"clockwise":true} — после поворота PNG сохраняется с размерами height×width
"#;

/// Возвращает ожидаемую длину буфера RGBA в байтах или `None` при переполнении / невлезании в `usize`.
pub fn rgba_buffer_byte_len(width: u32, height: u32) -> Option<usize> {
    let pixels = (width as u64).checked_mul(height as u64)?;
    let bytes = pixels.checked_mul(RGBA_BYTES_PER_PIXEL)?;
    usize::try_from(bytes).ok()
}

/// Подготовка содержимого файла параметров к передаче в `CString` / плагин.
#[must_use]
pub fn normalize_params_file_content(raw: &str) -> &str {
    raw.trim()
}
