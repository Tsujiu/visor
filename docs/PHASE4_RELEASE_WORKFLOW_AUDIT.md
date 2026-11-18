# VISOR — AUDITORIA DO WORKFLOW DE DISTRIBUIÇÃO E ASSINATURA DIGITAL
## Análise do Pipeline CI/CD, Gestão de Segredos e Ordem Criptográfica v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Arquivo Auditado:** `.github/workflows/release-distribution.yml`  
**Branch de Trabalho:** `feature/visor-phase4-signing`

---

## 1. Visão Geral do Pipeline de Distribuição Oficial

O workflow `.github/workflows/release-distribution.yml` é o elemento central de release engineering do VISOR. Ele executa em runner Windows oficial (`windows-latest`) e gerencia todo o ciclo de validação, compilação de alta performance com STATIC CRT, empacotamento, assinatura digital, manifesto de hashes e publicação no repositório público `Tsujiu/visor-downloads`.

---

## 2. Ordem Criptográfica de Operações no Pipeline

A auditoria implementou no workflow a sequência mandatória de 12 etapas, corrigindo o momento de cálculo dos hashes SHA-256 para que estes reflitam os arquivos finais já assinados:

```text
Etapa  1: Validação prévia de integridade da árvore Git e versão no Cargo.toml.
Etapa  2: Inspeção do estado remoto no repositório Tsujiu/visor-downloads.
Etapa  3: Instalação do toolchain Rust estável oficial (x86_64-pc-windows-msvc).
Etapa  4: Execução da bateria completa de 278 testes automatizados em Release.
Etapa  5: Compilação de produção com STATIC CRT (--release --no-default-features).
Etapa  6: Auditoria PE do binário (Magic MZ, ausência de VCRUNTIME140.dll).
Etapa  7: [NOVO] Assinatura Authenticode de visor.exe com signtool.exe e verificação.
Etapa  8: Empacotamento do arquivo portátil Visor-Portable-v*-x64.zip com visor.exe assinado.
Etapa  9: Compilação do instalador Inno Setup embutindo o visor.exe assinado.
Etapa 10: [NOVO] Assinatura Authenticode do instalador Visor-Setup-v*-x64.exe e verificação.
Etapa 11: [NOVO] Cálculo de hashes SHA-256 dos artefatos FINAIS e geração de SHA256SUMS.txt.
Etapa 12: Publicação idempotente no GitHub Releases ou exportação de artefatos de build.
```

---

## 3. Gestão e Isolamento de Segredos de CI/CD

O workflow foi projetado para assegurar que nenhuma credencial confidencial seja exposta ou gravada em logs públicos:

### 3.1. Segredo Existente: `VISOR_DOWNLOADS_TOKEN`
- **Função:** Conceder permissão de escrita estritamente limitada ao repositório público `Tsujiu/visor-downloads`.
- **Isolamento:** O token é injetado como variável de ambiente exclusivamente no passo final de release (`gh release create`), mantendo as etapas anteriores travadas sob permissão `contents: read`.

### 3.2. Novos Segredos Requeridos para Assinatura Comercial:
Para habilitar a assinatura automática quando o operador adquirir o certificado comercial, os seguintes segredos devem ser configurados em *Settings > Secrets and variables > Actions* no repositório privado `Tsujiu/visor`:

1. `SIGNING_CERTIFICATE_PFX_BASE64`:
   - Conteúdo: Certificado digital exportado em formato PFX e codificado em Base64 (`[Convert]::ToBase64String([IO.File]::ReadAllBytes('cert.pfx'))`).
   - Uso: O workflow decodifica o arquivo exclusivamente na memória ou em arquivo temporário efêmero em `$env:TEMP\<guid>.pfx`.
2. `SIGNING_CERTIFICATE_PASSWORD`:
   - Conteúdo: Senha do arquivo PFX.
   - Proteção: Mascarada automaticamente pelo GitHub Actions em qualquer log de execução.
3. `SIGNING_TIMESTAMP_URL` (Opcional):
   - Conteúdo: URL do servidor de carimbo de tempo RFC 3161 (padrão: `http://timestamp.digicert.com`).

---

## 4. Salvaguardas contra Publicação Não-Assinada Acidental

O workflow implementa uma trava de segurança em tempo de execução:

```powershell
if ($env:IS_DRY_RUN -eq "false" -and $env:CONFIRM_PUBLICATION -eq "CONFIRMAR") {
    if (-not $env:SIGNING_CERTIFICATE_PFX_BASE64) {
        throw "BLOQUEIO DE SEGURANÇA: Uma publicação de Release comercial foi disparada (dry_run=false), mas as credenciais de assinatura comercial (SIGNING_CERTIFICATE_PFX_BASE64) não estão configuradas! O pipeline recusa publicar releases comerciais sem assinatura Authenticode válida."
    }
}
```

Essa salvaguarda garante que uma versão de produção comercial jamais seja enviada ao público desprovida de sua assinatura digital verificada, a menos que o operador execute explicitamente em modo Dry-Run ou validação técnica.
