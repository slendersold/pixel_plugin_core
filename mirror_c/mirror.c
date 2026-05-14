/*
 * Плагин зеркалирования (чистый C), FFI process_image.
 * params: JSON UTF-8, например {"horizontal":true,"vertical":false}
 */
#include <ctype.h>
#include <stdint.h>
#include <string.h>

static int json_bool(const char *json, const char *key, int default_val) {
    char pattern[80];
    size_t key_len = strlen(key);
    if (key_len > 40) {
        return default_val;
    }
    memcpy(pattern, "\"", 1);
    memcpy(pattern + 1, key, key_len);
    pattern[1 + key_len] = '"';
    pattern[2 + key_len] = '\0';

    const char *hit = strstr(json, pattern);
    if (!hit) {
        return default_val;
    }
    const char *colon = strchr(hit + strlen(pattern), ':');
    if (!colon) {
        return default_val;
    }
    const char *p = colon + 1;
    while (*p && isspace((unsigned char)*p)) {
        p++;
    }
    if (strncmp(p, "true", 4) == 0) {
        return 1;
    }
    if (strncmp(p, "false", 5) == 0) {
        return 0;
    }
    return default_val;
}

static void mirror_horizontal(uint32_t width, uint32_t height, uint8_t *data) {
    size_t w = (size_t)width;
    size_t h = (size_t)height;
    size_t stride = w * 4;
    for (size_t y = 0; y < h; y++) {
        size_t row = y * stride;
        for (size_t x = 0; x < w / 2; x++) {
            size_t l = row + x * 4;
            size_t r = row + (w - 1 - x) * 4;
            for (size_t k = 0; k < 4; k++) {
                uint8_t t = data[l + k];
                data[l + k] = data[r + k];
                data[r + k] = t;
            }
        }
    }
}

static void mirror_vertical(uint32_t width, uint32_t height, uint8_t *data) {
    size_t w = (size_t)width;
    size_t h = (size_t)height;
    size_t stride = w * 4;
    for (size_t y = 0; y < h / 2; y++) {
        size_t row_a = y * stride;
        size_t row_b = (h - 1 - y) * stride;
        for (size_t x = 0; x < stride; x++) {
            uint8_t t = data[row_a + x];
            data[row_a + x] = data[row_b + x];
            data[row_b + x] = t;
        }
    }
}

void process_image(uint32_t width, uint32_t height, uint8_t *rgba_data,
                   const char *params) {
    if (!rgba_data || width == 0 || height == 0) {
        return;
    }

    const char *json = params ? params : "{}";
    int horizontal = json_bool(json, "horizontal", 0);
    int vertical = json_bool(json, "vertical", 0);

    if (horizontal) {
        mirror_horizontal(width, height, rgba_data);
    }
    if (vertical) {
        mirror_vertical(width, height, rgba_data);
    }
}
