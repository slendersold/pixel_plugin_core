//! Плагин зеркального отражения. Параметры и алгоритм — по ТЗ (TODO: horizontal/vertical).

use std::os::raw::c_char;

/// C ABI: модифицирует `rgba_data` на месте.
#[no_mangle]
pub unsafe extern "C" fn process_image(
    _width: u32,
    _height: u32,
    _rgba_data: *mut u8,
    _params: *const c_char,
) {
    // Заглушка: структура репозитория; логика зеркалирования добавляется по заданию.
}
