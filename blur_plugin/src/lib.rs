//! Плагин размытия: JSON `{"radius":u32,"iterations":u32}`.
//!
//! **Box blur:** в каждой итерации пиксель заменяется средним по квадрату соседей
//! с шагом `radius` (max(|dx|,|dy|) ≤ radius), затем итерации повторяются.

use std::ffi::CStr;
use std::os::raw::c_char;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BlurParams {
    #[serde(default)]
    radius: u32,
    #[serde(default = "default_iterations")]
    iterations: u32,
}

fn default_iterations() -> u32 {
    1
}

fn rgba_len_bytes(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)
}

/// Одна итерация: **box blur** — среднее по квадрату соседей с max(|dx|,|dy|) ≤ radius (заметнее на больших PNG).
fn blur_once(width: u32, height: u32, radius: u32, src: &[u8], dst: &mut [u8]) {
    let w = width as i32;
    let h = height as i32;
    let r = radius as i32;

    for y in 0..height {
        for x in 0..width {
            let mut acc = [0u64; 4];
            let mut count: u64 = 0;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let idx = (ny as usize * width as usize + nx as usize) * 4;
                    for c in 0..4 {
                        acc[c] += src[idx + c] as u64;
                    }
                    count += 1;
                }
            }
            let out_i = (y as usize * width as usize + x as usize) * 4;
            for c in 0..4 {
                dst[out_i + c] = match (acc[c] + count / 2).checked_div(count) {
                    Some(v) => v as u8,
                    None => src[out_i + c],
                };
            }
        }
    }
}

/// C ABI: модифицирует `rgba_data` на месте.
///
/// # Safety
/// Вызывающий обязан передать `width * height * 4` байт по указателю `rgba_data`,
/// корректные `width` и `height`, а `params` — указатель на валидную
/// NUL-терминированную C-строку UTF-8 (или пустой указатель, если контракт
/// плагина это допускает).
#[no_mangle]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    if rgba_data.is_null() {
        return;
    }
    let Some(len) = rgba_len_bytes(width, height) else {
        return;
    };
    let slice = std::slice::from_raw_parts_mut(rgba_data, len);

    let params_json = if params.is_null() {
        "{}"
    } else {
        let Ok(s) = CStr::from_ptr(params).to_str() else {
            return;
        };
        if s.is_empty() {
            "{}"
        } else {
            s
        }
    };

    let Ok(cfg) = serde_json::from_str::<BlurParams>(params_json) else {
        return;
    };

    if cfg.iterations == 0 || cfg.radius == 0 {
        return;
    }

    let mut a = slice.to_vec();
    let mut b = vec![0u8; len];
    for i in 0..cfg.iterations {
        if i % 2 == 0 {
            blur_once(width, height, cfg.radius, &a, &mut b);
        } else {
            blur_once(width, height, cfg.radius, &b, &mut a);
        }
    }
    let final_buf = if cfg.iterations % 2 == 1 {
        &b[..]
    } else {
        &a[..]
    };
    slice.copy_from_slice(final_buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_preserves_uniform_color() {
        let w = 3u32;
        let h = 3u32;
        let src = vec![100u8; (w * h * 4) as usize];
        let mut dst = vec![0u8; src.len()];
        blur_once(w, h, 1, &src, &mut dst);
        assert!(dst.iter().all(|&b| b == 100));
    }

    #[test]
    fn blur_reduces_spike() {
        let w = 5u32;
        let h = 1u32;
        let mut src = vec![0u8; (w * h * 4) as usize];
        src[2 * 4..3 * 4].copy_from_slice(&[255, 255, 255, 255]);
        let mut dst = vec![0u8; src.len()];
        blur_once(w, h, 2, &src, &mut dst);
        let center = &dst[2 * 4..3 * 4];
        assert!(center[0] < 255, "пик должен смягчиться");
    }
}
