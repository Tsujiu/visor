# VISOR — RELATÓRIO DE INTEGRIDADE E ATUALIZAÇÕES (UPDATE CENTER)
## Auditoria da Engine de Atualizações e Autenticidade Complementar v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Componentes Auditados:** `src/updater/github.rs`, `src/updater/integrity.rs`, `src/updater/download.rs`  
**Endpoint Público de Releases:** `https://api.github.com/repos/Tsujiu/visor-downloads/releases/latest`

---

## 1. Escopo e Propósito da Auditoria

O subsistema Update Center do VISOR foi submetido a uma auditoria aprofundada de segurança para garantir que a distribuição de novas versões e atualizações ocorra com máxima resiliência contra ataques de rede, adulteração de arquivos ou injeção de pacotes maliciosos.

---

## 2. Auditoria de Controles de Rede e URLs Permitidas

No arquivo `src/updater/github.rs`, todos os assets e URLs remotas são submetidos a validação rigorosa antes de qualquer processamento:

```rust
pub fn is_trusted_url(&self) -> bool {
    let url = &self.browser_download_url;
    if !url.starts_with("https://") {
        return false;
    }
    if url.contains("..") || url.contains('@') {
        return false;
    }
    url.starts_with("https://github.com/Tsujiu/visor-downloads/releases/download/")
        || url.starts_with("https://objects.githubusercontent.com/")
}
```

### Controles Auditados e Aprovados:
1. **Imposição Estrita de HTTPS:** O cliente rejeita conexões HTTP planas.
2. **Lista Branca de Domínios Oficiais:** Apenas URLs pertencentes ao repositório oficial `Tsujiu/visor-downloads` ou ao storage oficial do GitHub (`objects.githubusercontent.com`) são autorizadas. URLs de forks ou domínios de terceiros são bloqueadas sumariamente com o erro `UntrustedRelease`.
3. **Mitigação de Path Traversal:** A função `is_safe_filename` impede a presença de sequências relativas (`../`), caracteres nulos (`\0`) e barras em nomes de arquivos de atualização.
4. **Sanitização de Strings:** Descrições e notas de versão Markdown têm caracteres de controle perigosos removidos via `sanitize_text()`.
5. **Resiliência a Timeout e Falhas de Rede:** Timeout configurado para 10 segundos (`REQUEST_TIMEOUT`); falhas de transporte são encapsuladas suavemente como `UpdateCheckError::Network` sem causar crashes na aplicação.

---

## 3. Integridade Criptográfica (SHA-256) vs. Autenticidade do Publicador

O Update Center implementa atualmente conferência de integridade através do manifesto `SHA256SUMS.txt`:

### 3.1. Como a Validação Opera:
1. Ao identificar uma nova release, o cliente localiza o asset `SHA256SUMS.txt`.
2. O arquivo de atualização (instalador ou ZIP) é baixado temporariamente.
3. O hash SHA-256 dos bytes baixados é calculado e comparado com o hash correspondente extraído do manifesto.
4. Se o hash coincidir, o pacote é promovido para instalação; caso contrário, é descartado imediatamente.

### 3.2. A Assinatura Authenticode como Camada Complementar de Autenticidade:
> [!NOTE]
> **Camadas em Profundidade:**  
> - **SHA-256:** Garante que o arquivo baixado não foi corrompido durante a transferência de rede (*Integridade de Dados*).  
> - **Authenticode:** Garante que o arquivo foi gerado e assinado pela entidade proprietária legítima (*Autenticidade de Origem*).  
> A verificação da assinatura Authenticode do executável antes da execução adiciona uma camada de proteção adicional que valida que, mesmo se o manifesto `SHA256SUMS.txt` fosse adulterado em um ataque de infraestrutura, o binário não assinado pela autoridade oficial seria rejeitado pelo Windows.

---

## 4. Resultados da Suíte Automatizada de Atualizações

A bateria de testes unitários do subsistema de atualizações foi executada em modo release:

```powershell
cargo test --release --locked --no-default-features updater::
```

**Resultado dos Testes:**
- **Total:** 28 testes executados.
- **Aprovados:** 28 testes (100% de sucesso).
- **Falhas / Erros:** 0 falhas.

Dentre os cenários testados com sucesso:
- Rejeição de nomes com Path Traversal.
- Rejeição de releases em rascunho (draft).
- Rejeição de URLs originadas de forks não autorizados.
- Comparação de versão SemVer (atual vs. remota).
- Parsing de manifesto SHA256SUMS com e sem BOM UTF-8.
- Cálculo de hash SHA-256 e validação de integridade.
- Tratamento de resposta HTTP 304 (Not Modified com ETag).
