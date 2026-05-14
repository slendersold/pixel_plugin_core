//! Плагин размытия. Параметры radius/iterations — по ТЗ.

use std::os::raw::c_char;

/// C ABI: модифицирует `rgba_data` на месте.
///
/// # Safety
/// Вызывающий обязан передать `width * height * 4` байт по указателю `rgba_data`,
/// корректные `width` и `height`, а `params` — указатель на валидную
/// NUL-терминированную C-строку UTF-8 (или пустой указатель, если контракт
/// плагина это допускает).
#[no_mangle]
pub unsafe extern "C" fn process_image(
    _width: u32,
    _height: u32,
    _rgba_data: *mut u8,
    _params: *const c_char,
) {
    // Заглушка: структура репозитория; алгоритм размытия добавляется по заданию.
}
