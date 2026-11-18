# VISOR — ARQUITETURA E IMPLEMENTAÇÃO DA ASSINATURA AUTHENTICODE
## Pipeline de Assinatura, Validação Criptográfica e Ordem de Empacotamento v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Target:** `x86_64-pc-windows-msvc`  
**Ferramentas Oficiais:** Microsoft Windows SDK `signtool.exe`, PowerShell `Get-AuthenticodeSignature`

---

## 1. Princípios de Segurança e Ciclo de Vida da Assinatura PE

A assinatura digital Microsoft Authenticode para executáveis Windows em formato PE/COFF (*Portable Executable*) anexa uma estrutura de assinatura PKCS #7 / CMS no diretório de segurança da imagem PE (`IMAGE_DIRECTORY_ENTRY_SECURITY`).

> [!IMPORTANT]
> **Imutabilidade e Geração de Hashes (Regra de Ouro da Release):**  
> A aplicação da assinatura digital altera bytes fundamentais do cabeçalho PE e anexa a tabela de certificados ao final do arquivo.  
> Consequentemente:
> **O hash SHA-256 de um executável ou instalador ASSINADO é completamente diferente do hash do arquivo NÃO-ASSINADO.**  
> Qualquer cálculo de hash realizado antes da assinatura final torna-se imediatamente inválido e corrompe o manifesto de integridade.

---

## 2. Ordem Estrita de Execução no Pipeline CI/CD

Para garantir que o executável embutido no pacote portátil e no instalador seja exatamente o mesmo binário assinado, o pipeline do GitHub Actions deve seguir rigorosamente a sequência de 12 etapas:

```text
[1. Compilar target/release/visor.exe]
         │
[2. Validar STATIC CRT (ausência de VCRUNTIME140.dll)]
         │
[3. Executar 278 testes automatizados em Release]
         │
[4. Assinar visor.exe com signtool.exe] ◄────────────── Primeira Assinatura
         │
[5. Verificar assinatura de visor.exe com signtool verify /pa]
         │
[6. Empacotar Visor-Portable-v*.zip (contendo o visor.exe já assinado)]
         │
[7. Compilar instalador Inno Setup (embutindo o visor.exe já assinado)]
         │
[8. Assinar Visor-Setup-v*-x64.exe com signtool.exe] ◄ Segundo Assinatura
         │
[9. Verificar assinatura do instalador com signtool verify /pa]
         │
[10. Calcular hashes SHA-256 dos artefatos FINAIS assinados]
         │
[11. Gerar manifesto canônico SHA256SUMS.txt]
         │
[12. Publicar ou disponibilizar artefatos para inspeção humana]
```

---

## 3. Especificação dos Comandos Oficiais de Assinatura (`signtool.exe`)

A ferramenta oficial do Windows SDK `signtool.exe` (localizada em `C:\Program Files (x86)\Windows Kits\10\bin\<versão>\x64\signtool.exe`) deve ser invocada com os parâmetros recomendados pela Microsoft:

### 3.1. Assinatura com Certificado Baseado em Arquivo PFX (CI/CD Efêmero)
```powershell
$signtool = (Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" | Sort-Object FullName -Descending | Select-Object -First 1).FullName

& $signtool sign `
    /fd SHA256 `
    /tr "http://timestamp.digicert.com" `
    /td SHA256 `
    /f "$certPath" `
    /p "$password" `
    "target/release/visor.exe"
```

### 3.2. Parâmetros Obrigatórios:
- `/fd SHA256`: Define o algoritmo de digest da assinatura como SHA-256 (rejeitando SHA-1 descontinuado).
- `/tr "http://timestamp.digicert.com"`: URL oficial de carimbo de tempo RFC 3161 (garante que a assinatura continue válida mesmo após a expiração do certificado digital).
- `/td SHA256`: Define o algoritmo de digest do carimbo de tempo como SHA-256.
- `/f <certPath>`: Caminho para o certificado digital temporário.
- `/p <password>`: Senha do certificado protegida via variável de ambiente secreta.

---

## 4. Procedimento de Verificação Automatizada da Assinatura

Após assinar cada arquivo, duas camadas independentes de validação devem confirmar a higidez do binário:

### 4.1. Camada 1: `signtool verify`
```powershell
& $signtool verify /pa /all /v "target/release/visor.exe"
& $signtool verify /pa /all /v "dist/Visor-Setup-v0.4.4-x64.exe"
```
- `/pa`: Aplica a política de autenticação de código padrão do Windows (Default Authenticode Verification Policy).
- `/all`: Verifica todas as assinaturas em caso de arquivos multi-assinados.
- `/v`: Modo detalhado (verbose), exibindo cadeia de certificação e dados do carimbo de tempo.
- **Critério de Sucesso:** Código de retorno `$LASTEXITCODE -eq 0`.

### 4.2. Camada 2: `Get-AuthenticodeSignature` (PowerShell)
```powershell
$sig = Get-AuthenticodeSignature "target/release/visor.exe"
if ($sig.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
    throw "ASSINATURA INVÁLIDA: Status retornado foi '$($sig.Status)'!"
}
Write-Host "Certificado emitido para: $($sig.SignerCertificate.Subject)"
Write-Host "Autoridade emissora: $($sig.SignerCertificate.Issuer)"
Write-Host "Data do carimbo de tempo: $($sig.TimeStamperCertificate.NotBefore)"
```

---

## 5. Matriz de Estados e Modos de Operação do Workflow

O pipeline deve distinguir claramente três modos operacionais, impedindo a publicação acidental de binários não assinados como se fossem versões comerciais oficiais:

| Modo Operacional | Gatilho / Condição | Comportamento de Assinatura | Autorização de Publicação Pública |
| :--- | :--- | :--- | :---: |
| **Build de Desenvolvimento** | Push em branch de feature ou PR | Assinatura ignorada silenciosamente | **BLOQUEADA** |
| **Build de Validação Sem Assinatura** | `workflow_dispatch` com `dry_run: true` e sem segredos de certificado | Emite aviso em log: *"BUILD DE VALIDAÇÃO NÃO-ASSINADO (Status: NotSigned)"* | **BLOQUEADA** |
| **Build Comercial Oficial Assinado** | `workflow_dispatch` com `dry_run: false`, confirmação `"CONFIRMAR"` e segredos configurados | Executa assinatura obrigatória de `visor.exe` e instalador; aborta se `signtool verify` falhar | **AUTORIZADA** |

> [!CAUTION]
> **Bloqueio de Falsa Conformidade:**  
> Se o operador configurar `dry_run: false` e `"CONFIRMAR"` com o objetivo de lançar a versão comercial ao público, mas as credenciais de assinatura comercial não estiverem configuradas nos segredos do repositório, o pipeline **ABORTARÁ** com erro fatal explicativo, impedindo que uma release comercial seja disponibilizada ao público como se estivesse assinada.
