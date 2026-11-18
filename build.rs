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

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.4.4".to_string());
    let mut parts = pkg_version.split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch_part = parts.next().unwrap_or("0");
    let patch: u32 = patch_part
        .split('-')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    fs::write(
        &rc_path,
        format!(
            r#"1 ICON "{ico_rc}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
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
      VALUE "FileVersion", "{pkg_version}"
      VALUE "ProductVersion", "{pkg_version}"
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
        panic!("windres falhou");
    }
    println!("cargo:rustc-link-arg={}", obj_path.display());
}

fn find_windres() -> Option<String> {
    if Command::new("windres").arg("--version").output().is_ok() {
        return Some("windres".to_owned());
    }
    let paths = [
        r"C:\msys64\ucrt64\bin\windres.exe",
        r"C:\msys64\mingw64\bin\windres.exe",
        r"C:\msys64\usr\bin\windres.exe",
    ];
    for p in paths {
        if std::path::Path::new(p).is_file() {
            return Some(p.to_owned());
        }
    }
    None
}
