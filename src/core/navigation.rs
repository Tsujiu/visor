//! Listagem de imagens em diretórios e ordenação de nomes de arquivo.
//!
//! Implementa ordenação natural (Natural Sort / Numeric Sort) compatível
//! com o Windows Explorer (StrCmpLogicalW), garantindo que arquivos numerados
//! como "foto1.jpg", "foto2.jpg" e "foto10.jpg" sejam percorridos na sequência
//! intuitiva e correta, e não na ordem puramente lexicográfica ("foto1", "foto10", "foto2").

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

/// Lista e ordena todas as imagens suportadas no diretório especificado.
pub fn list_images(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_image_path(path))
        .collect();
    files.sort_by(|a, b| cmp_name(a, b));
    files
}

/// Verifica se o caminho aponta para um formato navegável pelo catálogo do VISOR.
pub fn is_image_path(path: &Path) -> bool {
    visor::browsable(path)
}

/// Compara dois caminhos de arquivo usando ordenação natural (case-insensitive e numérica).
pub fn cmp_name(a: &Path, b: &Path) -> Ordering {
    let left = a
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    let right = b
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();

    natural_cmp(&left, &right).then_with(|| a.cmp(b))
}

/// Algoritmo canônico de ordenação natural pura:
/// - Agrupa sequências de dígitos e as compara pelo seu valor numérico.
/// - Compara sequências de caracteres alfabéticos de forma insensível a maiúsculas/minúsculas.
/// - Não depende de chamadas FFI ou bibliotecas externas.
pub fn natural_cmp(s1: &str, s2: &str) -> Ordering {
    let mut chars1 = s1.chars().peekable();
    let mut chars2 = s2.chars().peekable();

    while let (Some(&c1), Some(&c2)) = (chars1.peek(), chars2.peek()) {
        if c1.is_ascii_digit() && c2.is_ascii_digit() {
            // 1. Extração contígua dos dígitos de ambas as strings
            let mut num1 = String::new();
            while let Some(&c) = chars1.peek() {
                if c.is_ascii_digit() {
                    num1.push(chars1.next().unwrap());
                } else {
                    break;
                }
            }

            let mut num2 = String::new();
            while let Some(&c) = chars2.peek() {
                if c.is_ascii_digit() {
                    num2.push(chars2.next().unwrap());
                } else {
                    break;
                }
            }

            // 2. Compara o valor numérico (removendo zeros à esquerda)
            let val1 = num1.trim_start_matches('0');
            let val2 = num2.trim_start_matches('0');

            let num_cmp = match val1.len().cmp(&val2.len()) {
                Ordering::Equal => val1.cmp(val2),
                other => other,
            };

            if num_cmp != Ordering::Equal {
                return num_cmp;
            }

            // 3. Desempate se os números forem de mesmo valor (ex: "01" vs "1")
            if num1.len() != num2.len() {
                return num1.len().cmp(&num2.len());
            }
        } else {
            let ch1 = chars1.next().unwrap();
            let ch2 = chars2.next().unwrap();

            // Comparação insensível a maiúsculas/minúsculas
            let low1 = ch1.to_ascii_lowercase();
            let low2 = ch2.to_ascii_lowercase();

            let cmp = low1.cmp(&low2);
            if cmp != Ordering::Equal {
                return cmp;
            }
        }
    }

    // Se uma das strings terminou antes, a menor vem primeiro
    match (chars1.peek().is_some(), chars2.peek().is_some()) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_natural_cmp_numeric_sequence() {
        assert_eq!(natural_cmp("foto1.jpg", "foto2.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("foto2.jpg", "foto10.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("foto9.jpg", "foto10.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("foto10.jpg", "foto2.jpg"), Ordering::Greater);
        assert_eq!(natural_cmp("foto100.png", "foto20.png"), Ordering::Greater);
    }

    #[test]
    pub fn test_natural_cmp_case_insensitivity() {
        assert_eq!(natural_cmp("FOTO1.jpg", "foto1.jpg"), Ordering::Equal);
        assert_eq!(natural_cmp("Foto1.JPG", "foto2.jpg"), Ordering::Less);
        assert_eq!(natural_cmp("imagem_A.png", "imagem_b.png"), Ordering::Less);
    }

    #[test]
    pub fn test_natural_cmp_multiple_numbers() {
        assert_eq!(natural_cmp("v1.2.3.png", "v1.2.10.png"), Ordering::Less);
        assert_eq!(
            natural_cmp("render_01_frame_2.png", "render_01_frame_10.png"),
            Ordering::Less
        );
        assert_eq!(
            natural_cmp("render_02_frame_1.png", "render_01_frame_99.png"),
            Ordering::Greater
        );
    }

    #[test]
    pub fn test_cmp_name_path_sorting() {
        let mut list = vec![
            PathBuf::from(r"C:\photos\img10.png"),
            PathBuf::from(r"C:\photos\img1.png"),
            PathBuf::from(r"C:\photos\img2.png"),
            PathBuf::from(r"C:\photos\img20.png"),
            PathBuf::from(r"C:\photos\img3.png"),
        ];

        list.sort_by(|a, b| cmp_name(a, b));

        assert_eq!(list[0], PathBuf::from(r"C:\photos\img1.png"));
        assert_eq!(list[1], PathBuf::from(r"C:\photos\img2.png"));
        assert_eq!(list[2], PathBuf::from(r"C:\photos\img3.png"));
        assert_eq!(list[3], PathBuf::from(r"C:\photos\img10.png"));
        assert_eq!(list[4], PathBuf::from(r"C:\photos\img20.png"));
    }

    #[test]
    pub fn test_find_initial_index() {
        let files = vec![PathBuf::from(r"C:\img1.png"), PathBuf::from(r"C:\img2.png")];
        let target = PathBuf::from(r"C:\img2.png");
        let idx = files.iter().position(|p| p == &target);
        assert_eq!(idx, Some(1));
    }

    #[test]
    pub fn test_is_image_path() {
        assert!(is_image_path(Path::new("test.png")));
        assert!(is_image_path(Path::new("test.jpg")));
        assert!(is_image_path(Path::new("test.cdr")));
        assert!(is_image_path(Path::new("test.dng")));
        assert!(!is_image_path(Path::new("test.txt")));
        assert!(!is_image_path(Path::new("test.exe")));
    }
}
