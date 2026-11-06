use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let ico = manifest.join("assets").join("moldura.ico");
    println!("cargo:rerun-if-changed={}", ico.display());
    if !ico.is_file() {
        panic!("ícone ausente: {}", ico.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let rc_path = out_dir.join("visor.rc");
    let obj_path = out_dir.join("visor-icon.o");
    let ico_rc = ico.to_string_lossy().replace('\\', "/");
    fs::write(
        &rc_path,
        format!(
            r#"1 ICON "{ico_rc}"
1 VERSIONINFO
FILEVERSION 0,2,2,0
PRODUCTVERSION 0,2,2,0
FILEFLAGSMASK 0x3f
FILEFLAGS 0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "000004b0"
    BEGIN
      VALUE "FileDescription", "Visualizador de imagens"
      VALUE "ProductName", "Visor"
      VALUE "FileVersion", "0.2.2"
      VALUE "ProductVersion", "0.2.2"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0, 0x04b0
  END
END
"#
        ),
    )
    .expect("script do ícone");

    let windres = find_windres().unwrap_or_else(|| "windres".to_owned());
    let status = Command::new(&windres)
        .arg(&rc_path)
        .args(["-O", "coff", "-o"])
        .arg(&obj_path)
        .status()
        .unwrap_or_else(|err| panic!("windres não encontrado ({windres}): {err}"));
    if !status.success() {
        panic!("windres falhou ao incorporar o ícone");
    }
    println!("cargo:rustc-link-arg-bins={}", obj_path.display());
}

fn find_windres() -> Option<String> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join("windres.exe");
        if candidate.is_file() {
            return Some(candidate.display().to_string());
        }
    }
    None
}
