package main

/*
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
*/
import "C"

import (
	"encoding/json"
	"unsafe"
)

type rotateCfg struct {
	Clockwise *bool `json:"clockwise"`
}

func rotateCW90(w, h int, src, dst []byte) {
	// Было изображение w×h; после CW — h×w, stride новой ширины = h.
	for sy := 0; sy < h; sy++ {
		for sx := 0; sx < w; sx++ {
			nx := h - 1 - sy
			ny := sx
			di := (ny*h + nx) * 4
			si := (sy*w + sx) * 4
			copy(dst[di:di+4], src[si:si+4])
		}
	}
}

func rotateCCW90(w, h int, src, dst []byte) {
	for sy := 0; sy < h; sy++ {
		for sx := 0; sx < w; sx++ {
			nx := sy
			ny := w - 1 - sx
			di := (ny*h + nx) * 4
			si := (sy*w + sx) * 4
			copy(dst[di:di+4], src[si:si+4])
		}
	}
}

//export process_image
func process_image(width C.uint32_t, height C.uint32_t, rgba *C.uint8_t, params *C.char) {
	if rgba == nil || width == 0 || height == 0 {
		return
	}
	w := int(width)
	h := int(height)
	total := w * h * 4

	jsonStr := "{}"
	if params != nil {
		jsonStr = C.GoString(params)
	}
	var cfg rotateCfg
	if err := json.Unmarshal([]byte(jsonStr), &cfg); err != nil {
		return
	}
	clockwise := true
	if cfg.Clockwise != nil {
		clockwise = *cfg.Clockwise
	}

	src := unsafe.Slice((*byte)(unsafe.Pointer(rgba)), total)
	tmp := make([]byte, total)

	if clockwise {
		rotateCW90(w, h, src, tmp)
	} else {
		rotateCCW90(w, h, src, tmp)
	}
	copy(src, tmp)
}

func main() {}
