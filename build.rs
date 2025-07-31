use std::env;
use std::process::Command;

fn main() {
    // Проверяем, что сборка идёт под Windows
    if env::var("TARGET").unwrap().contains("windows") {
        println!("🔧 Подготовка ресурсов для Windows...");

        // Определяем правильный windres для Linux
        let windres = if cfg!(target_os = "linux") {
            "x86_64-w64-mingw32-windres" // Для Linux кросс-компиляции
        } else {
            "windres" // Для нативной Windows
        };

        // Запускаем компиляцию .rc → .res
        let status = Command::new(windres)
            .args(&["shroombrella.rc", "-O", "coff", "-o", "shroombrella.res"])
            .status()
            .expect(&format!(
                "❌ {} не найден. Установите mingw-w64-tools: sudo apt install mingw-w64-tools",
                windres
            ));

        if !status.success() {
            panic!("❌ {} failed", windres);
        }

        // Передаём .res линковщику
        println!("cargo:rustc-link-arg=shroombrella.res");
    }
}
