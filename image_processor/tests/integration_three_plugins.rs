//! Один входной PNG, три независимых прогона CLI с разными плагинами (Linux .so).
//!
//! На **Linux** перед прогоном тест сам собирает артефакты: `cargo build -p blur_plugin` и
//! `scripts/build-plugins.sh` с `TARGET_DIR` = каталог `image_processor` (тот же, что
//! `CARGO_BIN_EXE_image_processor`, в т.ч. при `CARGO_TARGET_DIR`).
//!
//! На других ОС тест — заглушка (нет `.so` под Linux ABI).
//!
//! Тот же пайплайн на своём файле в `integration_outputs/`: `bash scripts/run-integration-sample.sh`.

#[cfg(target_os = "linux")]
mod linux {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use image::RgbaImage;
    use tempfile::tempdir;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("image_processor лежит внутри воркспейса")
            .to_path_buf()
    }

    /// Каталог с `image_processor` и `lib*.so` для текущего профиля/таргета.
    fn integration_plugin_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_image_processor"))
            .parent()
            .expect("бинарник image_processor в каталоге target/.../debug")
            .to_path_buf()
    }

    fn require_linux_plugins(dir: &Path) {
        for name in ["libmirror_c.so", "libblur_plugin.so", "librotate_go.so"] {
            let p = dir.join(name);
            assert!(
                p.is_file(),
                "нет {name} в {} — проверьте `gcc`, `go` и лог `scripts/build-plugins.sh`",
                dir.display()
            );
        }
    }

    /// Собирает `libblur_plugin.so` и внешние C/Go плагины в тот же `target/.../debug`, что и CLI.
    fn ensure_plugins_built() {
        let root = workspace_root();
        let dir = integration_plugin_dir();

        let st = Command::new("cargo")
            .args(["build", "-p", "blur_plugin"])
            .current_dir(&root)
            .status()
            .unwrap_or_else(|e| panic!("не удалось запустить cargo build -p blur_plugin: {e}"));
        assert!(
            st.success(),
            "cargo build -p blur_plugin: код {:?}",
            st.code()
        );

        let script = root.join("scripts").join("build-plugins.sh");
        assert!(
            script.is_file(),
            "ожидается {}",
            script.display()
        );

        let st = Command::new("bash")
            .arg(&script)
            .current_dir(&root)
            .env("TARGET_DIR", &dir)
            .status()
            .unwrap_or_else(|e| panic!("не удалось запустить build-plugins.sh: {e}"));
        assert!(
            st.success(),
            "scripts/build-plugins.sh (нужны gcc и go): код {:?}",
            st.code()
        );
    }

    pub fn run_three_plugins() {
        let plugin_dir = integration_plugin_dir();
        ensure_plugins_built();
        require_linux_plugins(&plugin_dir);

        let tmp = tempdir().expect("tempdir");
        let input = tmp.path().join("in.png");
        let img = RgbaImage::from_pixel(16, 16, image::Rgba([40, 120, 200, 255]));
        img.save(&input).expect("save input");

        let bin = env!("CARGO_BIN_EXE_image_processor");
        let cases = [
            (
                "out_mirror.png",
                "mirror_c",
                r#"{"horizontal":true,"vertical":false}"#,
            ),
            (
                "out_blur.png",
                "blur_plugin",
                r#"{"radius":1,"iterations":1}"#,
            ),
            ("out_rot.png", "rotate_go", r#"{"clockwise":true}"#),
        ];

        for (out_name, plugin, json) in cases {
            let params = tmp.path().join(format!("params_{plugin}.txt"));
            fs::write(&params, json).unwrap();
            let out = tmp.path().join(out_name);
            let st = Command::new(bin)
                .arg(&input)
                .arg(&out)
                .arg(plugin)
                .arg(&params)
                .arg("--plugin-path")
                .arg(&plugin_dir)
                .status()
                .unwrap_or_else(|e| panic!("запуск image_processor ({plugin}): {e}"));
            assert!(st.success(), "плагин {plugin}: код выхода {:?}", st.code());
            assert!(out.is_file(), "{plugin}: нет выходного файла");
            let meta = fs::metadata(&out).unwrap();
            assert!(meta.len() > 32, "{plugin}: подозрительно маленький PNG");
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn three_plugins_one_input_three_outputs() {
    linux::run_three_plugins();
}

#[cfg(not(target_os = "linux"))]
#[test]
fn three_plugins_one_input_three_outputs() {
    // Полный прогон только на Linux (сборка C/Go .so и три вызова CLI).
}
