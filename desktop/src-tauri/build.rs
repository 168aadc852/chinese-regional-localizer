fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    println!("cargo:rerun-if-changed=../ui");
    println!("cargo:rerun-if-changed=../../scripts/check_ui_translations.py");
    println!("cargo:rerun-if-env-changed=CRL_UI_PYTHON");
    let python = std::env::var_os("CRL_UI_PYTHON").unwrap_or_else(|| "python".into());
    let status = std::process::Command::new(python)
        .arg(root.join("scripts/check_ui_translations.py"))
        .status()
        .expect("UI translation validation needs Python 3.11 (or CRL_UI_PYTHON)");
    assert!(
        status.success(),
        "UI translation validation failed; do not ship incomplete or stale locale resources"
    );
    tauri_build::build()
}
