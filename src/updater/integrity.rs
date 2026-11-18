//! Verificação criptográfica de integridade de arquivos via SHA-256.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

/// Interpreta o conteúdo textual de um manifesto `SHA256SUMS.txt`.
///
/// Suporta os formatos padrão GNU:
/// - `<hash_hex>  <filename>` (dois espaços)
/// - `<hash_hex> *<filename>` (modo binário)
/// - `<hash_hex> <filename>`  (espaço simples)
///
/// Retorna um mapa onde as chaves são os nomes dos arquivos (case-insensitive para Windows)
/// e os valores são as somas SHA-256 em hexadecimal minúsculo.
pub fn parse_sha256sums(manifest_text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let text = manifest_text.trim_start_matches('\u{feff}');
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Divide a linha pelo primeiro espaço
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
            let clean_hash = hash.trim().to_ascii_lowercase();
            // Remove eventual indicador binário '*' no início do nome
            let clean_filename = filename.trim().trim_start_matches('*').to_string();

            if clean_hash.len() == 64 && clean_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                map.insert(clean_filename.to_ascii_lowercase(), clean_hash);
            }
        }
    }
    map
}

/// Calcula a soma SHA-256 de um arquivo em disco de forma eficiente e em blocos de memória.
pub fn calculate_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Valida se o arquivo no caminho especificado confere com a soma SHA-256 esperada.
///
/// Retorna `Ok(true)` se o hash corresponder exatamente.
/// Retorna `Ok(false)` se o arquivo for lido com sucesso mas o hash divergir.
/// Retorna `Err(err)` se houver falha de I/O na leitura do arquivo.
pub fn verify_file_integrity(path: &Path, expected_hex: &str) -> Result<bool, std::io::Error> {
    let calculated = calculate_file_sha256(path)?;
    Ok(calculated.eq_ignore_ascii_case(expected_hex.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_sha256sums() {
        let sample = "\
7ef8fec7fdd6f4630891843cd532be552a41c54c34f6758185fe696ff82c1c9d  Visor-Setup-v0.3.2-x64.exe
0d1895a399cbcc6e91bd34bb80b0a987d17b990dafbec8ac907a17793640bb93 *Visor-Portable-v0.3.2-x64.zip
# Linha de comentário
INVALID_HASH  SomeFile.exe
";
        let parsed = parse_sha256sums(sample);
        assert_eq!(parsed.len(), 2);
        assert_eq!(
            parsed.get("visor-setup-v0.3.2-x64.exe").unwrap(),
            "7ef8fec7fdd6f4630891843cd532be552a41c54c34f6758185fe696ff82c1c9d"
        );
        assert_eq!(
            parsed.get("visor-portable-v0.3.2-x64.zip").unwrap(),
            "0d1895a399cbcc6e91bd34bb80b0a987d17b990dafbec8ac907a17793640bb93"
        );
        assert!(parsed.get("somefile.exe").is_none());
    }

    #[test]
    fn test_parse_sha256sums_handles_utf8_bom() {
        let sample = "\u{feff}7ef8fec7fdd6f4630891843cd532be552a41c54c34f6758185fe696ff82c1c9d  Visor-Setup-v0.3.2-x64.exe\n";
        let parsed = parse_sha256sums(sample);
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed.get("visor-setup-v0.3.2-x64.exe").unwrap(),
            "7ef8fec7fdd6f4630891843cd532be552a41c54c34f6758185fe696ff82c1c9d"
        );
    }

    #[test]
    fn test_calculate_and_verify_sha256() {
        let temp_dir = std::env::temp_dir().join("visor_test_integrity");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test_sample.bin");

        // Conteúdo "VISOR TEST DATA 12345"
        let data = b"VISOR TEST DATA 12345";
        let mut f = File::create(&test_file).unwrap();
        f.write_all(data).unwrap();
        drop(f);

        // SHA-256 de "VISOR TEST DATA 12345":
        // echo -n "VISOR TEST DATA 12345" | sha256sum -> d99497d510cbb2cb5a86a60db272449a60e0a53b5160c50fa1b0a8fdfa3ce893
        let mut hasher = Sha256::new();
        hasher.update(data);
        let expected_hash = format!("{:x}", hasher.finalize());

        let calculated = calculate_file_sha256(&test_file).unwrap();
        assert_eq!(calculated, expected_hash);

        // Verificação positiva
        assert!(verify_file_integrity(&test_file, &expected_hash).unwrap());
        assert!(verify_file_integrity(&test_file, &expected_hash.to_uppercase()).unwrap());

        // Verificação negativa
        let bad_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!verify_file_integrity(&test_file, bad_hash).unwrap());

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
