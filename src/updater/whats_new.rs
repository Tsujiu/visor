//! Parser e extrator do histórico de novidades (Changelog) para exibição offline e remota.

use crate::updater::version::{CURRENT_VERSION, clean_version_str};

/// O arquivo CHANGELOG.md oficial é incorporado diretamente ao binário na compilação.
/// Isso garante que a janela de novidades funcione 100% offline, sem requisições ao GitHub.
pub const EMBEDDED_CHANGELOG: &str = include_str!("../../CHANGELOG.md");

/// Notas estruturadas de uma versão específica do VISOR.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VersionNotes {
    /// Número da versão (ex: "0.4.1" ou "0.3.3").
    pub version: String,
    /// Data de publicação da release, se indicada no cabeçalho (ex: "2026-11-09").
    pub date: Option<String>,
    /// Novas funcionalidades adicionadas nesta versão.
    pub added: Vec<String>,
    /// Otimizações de desempenho e melhorias de usabilidade/interface.
    pub changed: Vec<String>,
    /// Bugs corrigidos e falhas de compatibilidade resolvidas.
    pub fixed: Vec<String>,
    /// Correções de segurança, validação de limites e privacidade.
    pub security: Vec<String>,
    /// Mudanças importantes, recursos descontinuados ou alterações de comportamento.
    pub important: Vec<String>,
    /// Texto markdown bruto da seção da versão.
    pub raw_markdown: String,
}

impl VersionNotes {
    /// Retorna se há algum item documentado nas categorias.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.changed.is_empty()
            && self.fixed.is_empty()
            && self.security.is_empty()
            && self.important.is_empty()
            && self.raw_markdown.trim().is_empty()
    }

    /// Retorna o total consolidado de itens de novidades.
    pub fn total_items(&self) -> usize {
        self.added.len()
            + self.changed.len()
            + self.fixed.len()
            + self.security.len()
            + self.important.len()
    }
}

/// Categoria detectada durante o processamento do markdown.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Category {
    None,
    Added,
    Changed,
    Fixed,
    Security,
    Important,
}

/// Extrai e estrutura as notas de todas as versões contidas no texto markdown de um changelog completo.
pub fn parse_all_versions(markdown: &str) -> Vec<VersionNotes> {
    let mut versions = Vec::new();
    let mut current_notes: Option<VersionNotes> = None;
    let mut current_category = Category::None;

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Detecta cabeçalho de versão (ex: "## [0.3.3] - 2026-11-08" ou "## [v0.4.0]")
        if trimmed.starts_with("## [") {
            if let Some(notes) = current_notes.take() {
                if !notes.version.is_empty() {
                    versions.push(notes);
                }
            }

            let mut notes = VersionNotes::default();
            if let Some(close_bracket) = trimmed.find(']') {
                let v_str = &trimmed[4..close_bracket];
                notes.version = clean_version_str(v_str).to_string();

                // Procura a data após o hífen
                if let Some(dash_pos) = trimmed.find(" - ") {
                    let date_str = trimmed[dash_pos + 3..].trim();
                    if !date_str.is_empty() {
                        notes.date = Some(date_str.to_string());
                    }
                }
            }
            current_category = Category::None;
            current_notes = Some(notes);
            continue;
        }

        if let Some(ref mut notes) = current_notes {
            notes.raw_markdown.push_str(line);
            notes.raw_markdown.push('\n');

            // Detecta subtítulos de categoria (ex: "### Adicionado", "### Corrigido")
            if trimmed.starts_with("### ") || trimmed.starts_with("#### ") {
                let title = trimmed.trim_start_matches('#').trim().to_ascii_lowercase();
                if title.contains("adicionad") || title.contains("added") || title.contains("novas")
                {
                    current_category = Category::Added;
                } else if title.contains("modificad")
                    || title.contains("alterad")
                    || title.contains("changed")
                    || title.contains("melhorias")
                    || title.contains("otimizad")
                {
                    current_category = Category::Changed;
                } else if title.contains("corrigid") || title.contains("fixed") {
                    current_category = Category::Fixed;
                } else if title.contains("segurança")
                    || title.contains("security")
                    || title.contains("privacidade")
                {
                    current_category = Category::Security;
                } else if title.contains("important")
                    || title.contains("removid")
                    || title.contains("breaking")
                {
                    current_category = Category::Important;
                } else {
                    current_category = Category::None;
                }
                continue;
            }

            // Detecta linhas de tópicos principais (iniciadas por "- " ou "* " ou "• ")
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
            {
                let bullet_content = trimmed[2..].trim().to_string();
                if !bullet_content.is_empty() {
                    match current_category {
                        Category::Added => notes.added.push(bullet_content),
                        Category::Changed => notes.changed.push(bullet_content),
                        Category::Fixed => notes.fixed.push(bullet_content),
                        Category::Security => notes.security.push(bullet_content),
                        Category::Important => notes.important.push(bullet_content),
                        Category::None => {
                            // Se nenhuma categoria específica foi iniciada, agrega em Alterações
                            notes.changed.push(bullet_content);
                        }
                    }
                }
            }
        }
    }

    if let Some(notes) = current_notes {
        if !notes.version.is_empty() {
            versions.push(notes);
        }
    }

    versions
}

/// Extrai notas estruturadas a partir do corpo de uma Release do GitHub ou texto markdown de release única.
///
/// Trata variações de cabeçalhos (`#`, `##`, `###`, `####`), títulos semânticos de seções
/// e descarta blocos de manifesto técnico (como tabelas de arquivos e hashes SHA-256).
pub fn parse_release_notes_from_markdown(
    version: &str,
    date: Option<&str>,
    markdown: &str,
) -> VersionNotes {
    let mut notes = VersionNotes {
        version: clean_version_str(version).to_string(),
        date: date.map(|d| d.to_string()),
        ..Default::default()
    };
    notes.raw_markdown = markdown.to_string();

    let mut current_category = Category::None;
    let mut in_ignored_section = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Detecta cabeçalhos markdown: #, ##, ###, ####
        if trimmed.starts_with('#') {
            let header_text = trimmed.trim_start_matches('#').trim().to_ascii_lowercase();

            // Seções técnicas que devem ser ignoradas no resumo das novidades
            if header_text.contains("arquivos oficiais")
                || header_text.contains("hashes criptogr")
                || header_text.contains("manifesto")
                || header_text.contains("download")
                || header_text.contains("distribui")
            {
                in_ignored_section = true;
                current_category = Category::None;
                continue;
            } else {
                in_ignored_section = false;
            }

            // Identifica categoria semântica multilíngue
            if header_text.contains("novo")
                || header_text.contains("novas")
                || header_text.contains("novidad")
                || header_text.contains("added")
                || header_text.contains("new")
                || header_text.contains("features")
                || header_text.contains("destaque")
                || header_text.contains("highlight")
                || header_text.contains("funcionalidade")
                || header_text.contains("recurso")
                || header_text.contains("neu")
                || header_text.contains("nouveaut")
                || header_text.contains("novedad")
                || header_text.contains("новое")
                || header_text.contains("新機能")
                || header_text.contains("新增")
            {
                current_category = Category::Added;
            } else if header_text.contains("melhoria")
                || header_text.contains("otimiz")
                || header_text.contains("moderniz")
                || header_text.contains("alterad")
                || header_text.contains("changed")
                || header_text.contains("improvement")
                || header_text.contains("aprimor")
                || header_text.contains("design")
                || header_text.contains("layout")
                || header_text.contains("tipografia")
                || header_text.contains("verbesserung")
                || header_text.contains("amélioration")
                || header_text.contains("miglioramento")
                || header_text.contains("улучшения")
                || header_text.contains("改善")
                || header_text.contains("优化")
            {
                current_category = Category::Changed;
            } else if header_text.contains("corrig")
                || header_text.contains("correç")
                || header_text.contains("fixed")
                || header_text.contains("fix")
                || header_text.contains("bug")
                || header_text.contains("resolv")
                || header_text.contains("behebung")
                || header_text.contains("correction")
                || header_text.contains("correzione")
                || header_text.contains("исправления")
                || header_text.contains("修正")
                || header_text.contains("修复")
            {
                current_category = Category::Fixed;
            } else if header_text.contains("seguran")
                || header_text.contains("security")
                || header_text.contains("privacidade")
                || header_text.contains("privacy")
                || header_text.contains("integridade")
                || header_text.contains("sicherheit")
                || header_text.contains("sécurité")
                || header_text.contains("sicurezza")
                || header_text.contains("безопасность")
                || header_text.contains("セキュリティ")
                || header_text.contains("安全")
            {
                current_category = Category::Security;
            } else if header_text.contains("important")
                || header_text.contains("breaking")
                || header_text.contains("atenção")
                || header_text.contains("aviso")
                || header_text.contains("wichtig")
                || header_text.contains("attention")
                || header_text.contains("attenzione")
                || header_text.contains("важно")
                || header_text.contains("重要")
            {
                current_category = Category::Important;
            }
            continue;
        }

        if in_ignored_section {
            continue;
        }

        // Linhas de tópicos (- , * , • )
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ") {
            let bullet_content = trimmed[2..].trim().to_string();
            if !bullet_content.is_empty() {
                match current_category {
                    Category::Added => notes.added.push(bullet_content),
                    Category::Changed => notes.changed.push(bullet_content),
                    Category::Fixed => notes.fixed.push(bullet_content),
                    Category::Security => notes.security.push(bullet_content),
                    Category::Important => notes.important.push(bullet_content),
                    Category::None => {
                        // Se não estiver dentro de uma seção específica, classifica como Changed/Melhoria
                        notes.changed.push(bullet_content);
                    }
                }
            }
        }
    }

    notes
}

/// Limpa e formata um item de lista markdown, separando subtítulos destacados de descrições comuns
/// e removendo delimitadores como `**`.
pub fn parse_bullet_item(raw: &str) -> (Option<String>, String) {
    let mut trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("- ") {
        trimmed = rest.trim();
    } else if let Some(rest) = trimmed.strip_prefix("* ") {
        trimmed = rest.trim();
    } else if let Some(rest) = trimmed.strip_prefix("• ") {
        trimmed = rest.trim();
    }

    // Caso 1: Item iniciando com negrito, ex: "**Update Center:**" ou "**Título:** Descrição"
    if let Some(rest) = trimmed.strip_prefix("**") {
        if let Some(end_idx) = rest.find("**") {
            let bold_title = rest[..end_idx].trim().trim_end_matches(':').trim();
            let after_bold = rest[end_idx + 2..].trim();
            let desc = after_bold.trim_start_matches(':').trim().replace("**", "");
            if desc.is_empty() {
                return (Some(bold_title.to_string()), String::new());
            } else {
                return (Some(bold_title.to_string()), desc);
            }
        }
    }

    // Caso 2: Linha normal de texto com possíveis marcadores internos
    let cleaned = trimmed.replace("**", "");
    (None, cleaned)
}

/// Localiza as notas de lançamento para uma versão específica no changelog embutido.
pub fn get_notes_for_version(version: &str) -> Option<VersionNotes> {
    let clean = clean_version_str(version);
    let all = parse_all_versions(EMBEDDED_CHANGELOG);
    all.into_iter()
        .find(|n| clean_version_str(&n.version) == clean)
}

/// Retorna as notas da versão, consultando primeiro o changelog embutido e, se não encontrar,
/// utilizando o corpo da release remota recebida do GitHub (se fornecida).
pub fn get_notes_or_parse_remote(
    version: &str,
    date: Option<&str>,
    remote_body: Option<&str>,
) -> VersionNotes {
    if let Some(notes) = get_notes_for_version(version) {
        notes
    } else if let Some(body) = remote_body {
        let parsed = parse_release_notes_from_markdown(version, date, body);
        if parsed.total_items() > 0 {
            parsed
        } else {
            VersionNotes {
                version: clean_version_str(version).to_string(),
                date: date.map(|d| d.to_string()),
                raw_markdown: body.to_string(),
                changed: vec![
                    "Atualização com novas melhorias e correções no canal oficial.".to_string(),
                ],
                ..Default::default()
            }
        }
    } else {
        VersionNotes {
            version: clean_version_str(version).to_string(),
            date: date.map(|d| d.to_string()),
            raw_markdown: format!("## [{version}]\n- Nova versão oficial disponível."),
            added: vec![
                "Atualização com novas melhorias e correções no canal oficial.".to_string(),
            ],
            ..Default::default()
        }
    }
}

/// Retorna as notas da versão atualmente em execução no executável.
pub fn get_current_version_notes() -> VersionNotes {
    if let Some(notes) = get_notes_for_version(CURRENT_VERSION) {
        notes
    } else {
        // Fallback gracioso caso a versão atual não tenha seção no CHANGELOG ainda
        VersionNotes {
            version: CURRENT_VERSION.to_string(),
            date: None,
            added: vec!["Refinamentos gerais e atualização de versão.".to_string()],
            changed: Vec::new(),
            fixed: Vec::new(),
            security: Vec::new(),
            important: Vec::new(),
            raw_markdown: format!("## [{CURRENT_VERSION}]\n- Atualização geral do VISOR."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_embedded_changelog() {
        let versions = parse_all_versions(EMBEDDED_CHANGELOG);
        assert!(
            !versions.is_empty(),
            "O CHANGELOG.md embutido deve conter versões"
        );

        let has_recent = versions
            .iter()
            .any(|v| v.version == "0.4.1" || v.version == "0.4.0" || v.version == "0.3.3");
        assert!(has_recent, "Deve conter versões recentes do VISOR");
    }

    #[test]
    fn test_version_notes_is_empty() {
        let empty = VersionNotes::default();
        assert!(empty.is_empty());
        let non_empty = VersionNotes {
            added: vec!["Nova funcionalidade".to_string()],
            ..Default::default()
        };
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_parse_sample_categories() {
        let sample = r#"
# Changelog

## [0.4.0] - 2026-11-09

### Adicionado
- Update Center automático com verificação no GitHub
- Janela de boas-vindas com novidades pós-atualização

### Modificado
- Desempenho do polling de background aprimorado

### Corrigido
- Glifo de seleção nos menus suspensos
"#;

        let versions = parse_all_versions(sample);
        assert_eq!(versions.len(), 1);
        let v = &versions[0];
        assert_eq!(v.version, "0.4.0");
        assert_eq!(v.date.as_deref(), Some("2026-11-09"));
        assert_eq!(v.added.len(), 2);
        assert_eq!(v.changed.len(), 1);
        assert_eq!(v.fixed.len(), 1);
    }

    #[test]
    fn test_parse_github_release_markdown() {
        let body = r#"
# VISOR v0.4.1 — Pop-up Modal Elegante & Refinamento

### O que há de novo nesta versão:

#### Design Popup Card Modal & Backdrop:
- **Pop-up Card Elegante:** As janelas agora utilizam modal centralizado.
- **Fundo Escurecido:** O fundo do aplicativo recebe scrim translúcido.

#### Correções de Bugs:
- **Resolução do corte:** Corrigido problema de borda.

### Arquivos Oficiais de Distribuição:
| Arquivo | Descrição |
| :--- | :--- |
| Visor-Setup.exe | Instalador |

### Hashes Criptográficos SHA-256:
```
b86bfee4  Visor-Setup.exe
```
"#;

        let notes = parse_release_notes_from_markdown("v0.4.1", Some("2026-11-09"), body);
        assert_eq!(notes.version, "0.4.1");
        assert_eq!(notes.date.as_deref(), Some("2026-11-09"));
        assert_eq!(notes.changed.len(), 2);
        assert_eq!(notes.fixed.len(), 1);
        assert_eq!(notes.total_items(), 3);
    }

    #[test]
    fn test_parse_bullet_item_formatting() {
        let raw = "- **Sistema de Atualização:** Permite checar versões automaticamente.";
        let (title, desc) = parse_bullet_item(raw);
        assert_eq!(title.as_deref(), Some("Sistema de Atualização"));
        assert_eq!(desc, "Permite checar versões automaticamente.");

        let raw2 = "• Linha simples de texto sem negrito.";
        let (title2, desc2) = parse_bullet_item(raw2);
        assert!(title2.is_none());
        assert_eq!(desc2, "Linha simples de texto sem negrito.");
    }
}
