//! Associação por usuário. No Windows 11 24H2+ o Explorer lê UserChoiceLatest.
//! O hash é o do SFTA 2.0.1 (MIT), que lê o sal nesta máquina e não desliga o UCPD.

use std::fs;
use std::path::Path;
use std::process::Command;

use winreg::enums::*;
use winreg::RegKey;

use crate::catalog;

const PROG_ID: &str = "Visor.Image";
const SCRIPT: &str = include_str!("../third_party/SFTA.ps1");

#[derive(Clone, Debug)]
pub struct AssocReport {
    pub lines: Vec<String>,
    pub png_ok: bool,
}

impl AssocReport {
    pub fn summary(&self) -> String {
        let mut text = self.lines.join("\n");
        if self.png_ok {
            text.push_str("\nPNG abre no Visor.");
        } else {
            text.push_str("\nO Windows não aceitou o Visor como padrão de PNG.");
        }
        text
    }
}

pub fn register_as_default(exe: &Path) -> AssocReport {
    let mut lines = Vec::new();
    if let Err(err) = write_capabilities(exe) {
        lines.push(format!("registro do programa: {err}"));
    } else {
        lines.push("Visor.Image registrado para este usuário".to_owned());
    }
    let extensions = catalog::verified_extensions();
    match run_sfta(exe, &extensions) {
        Ok(output) => {
            for line in output.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    lines.push(line.to_owned());
                }
            }
        }
        Err(err) => lines.push(err),
    }
    let png = read_progid(".png");
    let png_ok = png.as_deref() == Some(PROG_ID);
    lines.push(match png {
        Some(value) => format!("UserChoiceLatest de .png = {value}"),
        None => "UserChoiceLatest de .png não foi lido".to_owned(),
    });
    AssocReport { lines, png_ok }
}

pub fn current_png_progid() -> Option<String> {
    read_progid(".png")
}

fn write_capabilities(exe: &Path) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let exe = exe.display().to_string();
    let (prog, _) = hkcu
        .create_subkey(format!("Software\\Classes\\{PROG_ID}"))
        .map_err(|err| err.to_string())?;
    prog.set_value("", &"Imagem do Visor")
        .map_err(|err| err.to_string())?;
    let (icon, _) = hkcu
        .create_subkey(format!("Software\\Classes\\{PROG_ID}\\DefaultIcon"))
        .map_err(|err| err.to_string())?;
    icon.set_value("", &format!("{exe},0"))
        .map_err(|err| err.to_string())?;
    let (command, _) = hkcu
        .create_subkey(format!("Software\\Classes\\{PROG_ID}\\shell\\open\\command"))
        .map_err(|err| err.to_string())?;
    command
        .set_value("", &format!("\"{exe}\" \"%1\""))
        .map_err(|err| err.to_string())?;
    let (caps, _) = hkcu
        .create_subkey(format!("Software\\Classes\\{PROG_ID}\\Capabilities"))
        .map_err(|err| err.to_string())?;
    caps.set_value("ApplicationName", &"Visor")
        .map_err(|err| err.to_string())?;
    caps.set_value("ApplicationDescription", &"Visualizador de imagens")
        .map_err(|err| err.to_string())?;
    let (assoc, _) = hkcu
        .create_subkey(format!(
            "Software\\Classes\\{PROG_ID}\\Capabilities\\FileAssociations"
        ))
        .map_err(|err| err.to_string())?;
    for ext in catalog::verified_extensions() {
        assoc
            .set_value(format!(".{ext}"), &PROG_ID)
            .map_err(|err| err.to_string())?;
    }
    let (registered, _) = hkcu
        .create_subkey("Software\\RegisteredApplications")
        .map_err(|err| err.to_string())?;
    registered
        .set_value("Visor", &format!("Software\\Classes\\{PROG_ID}\\Capabilities"))
        .map_err(|err| err.to_string())?;
    let (app, _) = hkcu
        .create_subkey("Software\\Classes\\Applications\\visor.exe\\shell\\open\\command")
        .map_err(|err| err.to_string())?;
    app.set_value("", &format!("\"{exe}\" \"%1\""))
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn read_progid(ext: &str) -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(format!(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\{ext}\\UserChoiceLatest\\ProgId"
        ))
        .ok()?;
    key.get_value("ProgId").ok()
}

fn run_sfta(exe: &Path, extensions: &[&str]) -> Result<String, String> {
    let dir = std::env::temp_dir().join(format!("visor-sfta-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|err| format!("pasta temporária: {err}"))?;
    let script = dir.join("SFTA.ps1");
    let runner = dir.join("register.ps1");
    fs::write(&script, SCRIPT).map_err(|err| format!("script: {err}"))?;
    let ext_list = extensions
        .iter()
        .map(|ext| format!("'.{ext}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let exe_ps = exe.display().to_string().replace('\'', "''");
    let script_ps = script.display().to_string().replace('\'', "''");
    let body = format!(
        r#"$ErrorActionPreference = 'Continue'
. '{script_ps}'
$exe = '{exe_ps}'
foreach ($e in @({ext_list})) {{
  try {{
    $out = Register-FTA -ProgramPath $exe -Extension $e -ProgId '{PROG_ID}' -Icon "$exe,0" | Out-String
    Write-Output ("OK $e " + $out.Trim())
  }} catch {{
    Write-Output ("FAIL $e " + $_.Exception.Message)
  }}
}}
"#
    );
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend(body.as_bytes());
    fs::write(&runner, bytes).map_err(|err| format!("script: {err}"))?;
    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&runner)
        .output()
        .map_err(|err| format!("PowerShell: {err}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let err = String::from_utf8_lossy(&output.stderr);
    if !err.trim().is_empty() {
        text.push('\n');
        text.push_str(err.trim());
    }
    let _ = fs::remove_dir_all(&dir);
    if text.trim().is_empty() {
        return Err("o registro não devolveu resultado".to_owned());
    }
    Ok(text)
}

