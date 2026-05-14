//! Плагин размытия. Параметры radius/iterations — по ТЗ.

use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn process_image(
    _width: u32,
    _height: u32,
    _rgba_data: *mut u8,
    _params: *const c_char,
) {
    // Заглушка: структура репозитория; алгоритм размытия добавляется по заданию.
}
