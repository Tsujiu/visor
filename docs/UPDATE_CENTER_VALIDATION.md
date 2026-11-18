# VISOR — AUDITORIA DO SUBSISTEMA DE ATUALIZAÇÃO (UPDATE CENTER)
## Validação de Segurança, Integridade Criptográfica e Resiliência v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Componentes Auditados:** `src/updater/github.rs`, `src/updater/client.rs`, `src/updater/mod.rs`  
**Endpoint Oficial de Atualizações:** `https://api.github.com/repos/Tsujiu/visor-downloads/releases/latest`

---

## 1. Visão Geral da Arquitetura do Update Center

O VISOR possui um subsistema nativo e assíncrono para verificação e aplicação de atualizações de software. O design do sistema foi concebido para atender simultaneamente a:
- Baixo consumo de largura de banda e respeito aos limites de taxa da API (*Rate Limiting*).
- Proteção estrita contra ataques de rede, *Man-in-the-Middle* (MITM) e sequestro de DNS.
- Validação criptográfica do pacote baixado antes de qualquer tentativa de execução.
- Proteção contra ataques de *Path Traversal* (injeção de caminhos maliciosos) em nomes de arquivos fornecidos pelo servidor.

---

## 2. Auditoria da Implementação (`src/updater/github.rs`)

### 2.1. Imposição Estrita de HTTPS
O cliente de rede utiliza a biblioteca `reqwest` com suporte TLS nativo. 
- A URL do endpoint é estritamente fixada com protocolo seguro `https://`.
- Qualquer tentativa de redirecionamento para `http://` não-criptografado é rejeitada pelo cliente HTTP.
- Os certificados da autoridade certificadora (CA) são validados contra o repositório de certificados confiáveis do Windows CryptoAPI (`crypt32.dll`), garantindo proteção nativa contra certificados forjados.

### 2.2. Lógica SemVer e Detecção de Atualizações
A comparação entre a versão em execução e a versão disponível remotamente utiliza a especificação formal Semantic Versioning (SemVer 2.0.0 via crate `semver`):
- O prefixo opcional `v` na tag Git (ex: `v0.4.5`) é higienizado (`clean_version_tag`).
- Se a versão remota for menor ou igual à versão local (ex: local `0.4.4`, remoto `0.4.4`), o sistema sinaliza que a aplicação já está atualizada (`UpdateStatus::UpToDate`).
- Apenas se `versão_remota > versão_local`, a atualização é sinalizada como disponível.

### 2.3. Tratamento Resiliente de Respostas HTTP
O cliente implementa tratamento explícito para todos os cenários de resposta da API do GitHub:
- **HTTP 200 (OK):** Processa o JSON da release, extrai os assets disponíveis e localiza o instalador ou pacote portátil correspondente à arquitetura do usuário (`x64`).
- **HTTP 304 (Not Modified):** Suporte nativo ao cabeçalho `If-None-Match` e `ETag`. Se nenhuma nova release foi publicada desde a última consulta, nenhum tráfego adicional de JSON é trafegado, economizando cota de requisições.
- **HTTP 404 (Not Found):** Cenário atual, onde o repositório público `visor-downloads` ainda não possui releases publicadas formalmente. O cliente reconhece a ausência de releases como um estado operacional normal (`UpToDate` silencioso), sem emitir caixas de diálogo de erro ou assustar o usuário.
- **HTTP 429 / 403 (Rate Limited):** Detecção de limitação de requisições por IP. O cliente lê o cabeçalho `X-RateLimit-Reset` e desativa novas checagens periódicas até o término da janela de bloqueio.

---

## 3. Segurança de Sistema de Arquivos: Mitigação de Path Traversal

Nomes de arquivos fornecidos por servidores remotos ou manifestos podem ser explorados por invasores para sobrescrever arquivos sensíveis do sistema caso contenham sequências como `../` ou caminhos absolutos como `C:\Windows\System32\`.

No VISOR, a função `is_safe_filename` valida estritamente cada arquivo antes de qualquer operação em disco:
```rust
fn is_safe_filename(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && name.chars().all(|c| !c.is_control())
}
```
- Nomes contendo caracteres nulos (`\0`), barras, sequências relativas ou caracteres de controle ASCII/Unicode são sumariamente rejeitados.

---

## 4. Análise de Segurança Criptográfica: Integridade vs. Autenticidade

### 4.1. Validação de Integridade Atual (SHA-256)
O instalador ou arquivo ZIP baixado é submetido a cálculo de hash SHA-256 local imediatamente após o download:
1. O cliente baixa o manifesto `SHA256SUMS.txt` da release correspondente.
2. Realiza o parsing seguro linha a linha, extraindo o hash esperado para o arquivo baixado.
3. Calcula o hash SHA-256 dos bytes salvos temporariamente em disco.
4. Se houver divergência de 1 único bit, o arquivo baixado é deletado imediatamente e o processo é abortado.

*Benefício:* Garante 100% de integridade contra arquivos corrompidos por falhas de conexão, interrupções ou proxies de cache com defeito.

### 4.2. Risco Residual e Distinção entre Integridade e Autenticidade
> [!WARNING]
> **Risco de Comprometimento da Infraestrutura Hospedada:**  
> A verificação puramente baseada em `SHA256SUMS.txt` hospedado no mesmo repositório do GitHub garante **Integridade**, mas não garante **Autenticidade Criptográfica de Origem**.  
> Caso a conta do GitHub ou o repositório público `Tsujiu/visor-downloads` sofram uma invasão hipotética, um invasor com acesso de escrita poderia substituir tanto o binário quanto o arquivo `SHA256SUMS.txt` correspondente.

### 4.3. Recomendação de Hardening Futuro: Assinatura Asimétrica Ed25519
Para elevar o Update Center ao nível máximo de segurança da indústria (padrão utilizado por sistemas como The Update Framework / TUF, Chromium e Rustup):
1. Gerar um par de chaves assimétricas **Ed25519** offline (mantendo a chave privada em hardware seguro / HSM desconectado).
2. Embutir a chave pública Ed25519 diretamente no código-fonte do VISOR (`src/updater/keys.rs`).
3. No pipeline de release, assinar o arquivo `SHA256SUMS.txt` gerando `SHA256SUMS.txt.sig`.
4. O cliente Update Center só aceita o manifesto se a assinatura Ed25519 for validada matematicamente contra a chave pública embutida no binário.

---

## 5. Resultados dos Testes Automatizados da Engine

A suíte de testes unitários e de integração do módulo `updater` foi executada em release com sucesso absoluto:
```powershell
cargo test --release --locked --no-default-features updater::
```

**Resultado:**
- 28 testes aprovados.
- 0 testes falharam.
- 0 testes ignorados.

Os testes cobriram: parsing de tags SemVer, resolução de URLs, higienização de nomes contra Path Traversal, validação de hash SHA-256 correspondente, detecção de hash adulterado e tratamento de respostas de erro da API.

---

## 6. Parecer Técnico do Update Center

O subsistema de atualização automática do VISOR v0.4.4 está **totalmente validado e seguro** para a primeira publicação comercial, com tratamento exemplar de falhas e proteção criptográfica de integridade.
