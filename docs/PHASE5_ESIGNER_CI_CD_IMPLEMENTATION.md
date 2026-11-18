# VISOR — IMPLEMENTAÇÃO DO eSIGNER CKA NO PIPELINE CI/CD
## Guia de Integração para GitHub Actions com Windows Runner e SignTool

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Workflow:** `.github/workflows/release-distribution.yml`  
**Runner Oficial:** `runs-on: windows-latest`

---

## 1. Topologia da Integração no Workflow

O pipeline de release engineering do VISOR foi desenhado para orquestrar a assinatura remota através do utilitário oficial **eSigner CKA** fornecido pela SSL.com.

### Diagrama de Execução:
```text
[Runner windows-latest]
         │
[1. Baixar pacote oficial SSL.COM-eSigner-CKA]
         │
[2. Gerar Master Key efêmera e configurar credenciais (Username, Password, TOTP)]
         │
[3. eSignerCKATool.exe load -> Certificado remoto carregado em Cert:\CurrentUser\My]
         │
[4. Obter Thumbprint do Certificado de Code Signing]
         │
[5. SignTool sign /sha1 <thumbprint> /tr http://ts.ssl.com -> target/release/visor.exe]
         │
[6. SignTool verify /pa /all /v -> Validação Authenticode imediata]
         │
[7. Empacotar Visor-Portable-v*.zip (contendo o visor.exe já assinado)]
         │
[8. Inno Setup compila Visor-Setup-v*-x64.exe (embutindo o visor.exe assinado)]
         │
[9. SignTool sign /sha1 <thumbprint> -> dist/Visor-Setup-v*-x64.exe]
         │
[10. SignTool verify /pa /all /v -> Validação do Instalador final]
         │
[11. eSignerCKATool.exe unload -> Descarregar chaves e apagar dados de sessão]
         │
[12. Calcular hashes SHA-256 e gerar SHA256SUMS.txt com os arquivos assinados]
```

---

## 2. Implementação Passo a Passo em PowerShell

### 2.1. Instalação e Inicialização do CKA
```powershell
# 1. Download do pacote oficial fixado e autenticado
$ckaZip = Join-Path $env:TEMP "eSigner_CKA_Setup.zip"
$ckaDir = Join-Path $env:TEMP "eSigner_CKA_Files"
Invoke-WebRequest -Uri "https://github.com/SSLcom/eSignerCKA/releases/download/v1.1.2/SSL.COM-eSigner-CKA_1.1.2.zip" -OutFile $ckaZip
Expand-Archive -Path $ckaZip -DestinationPath $ckaDir -Force
Remove-Item $ckaZip -Force

# Localiza o diretório do utilitário eSignerCKATool.exe
$ckaTool = (Get-ChildItem -Path $ckaDir -Filter "eSignerCKATool.exe" -Recurse | Select-Object -First 1).FullName
$ckaBinDir = (Get-Item $ckaTool).DirectoryName
```

### 2.2. Configuração e Autenticação com TOTP
```powershell
# Gera chave mestra temporária exclusiva para esta execução
$masterKey = [System.Guid]::NewGuid().ToString("N") + "V1s0r!"

# Define o ambiente operacional (sandbox vs product)
$mode = if ($env:ESIGNER_ENVIRONMENT -eq "PROD") { "product" } else { "sandbox" }

# Configura a senha mestre
& $ckaTool config -m "$masterKey"

# Registra credenciais com autenticação de dois fatores (TOTP Secret)
& $ckaTool config -u "$env:ESIGNER_USERNAME" -p "$env:ESIGNER_PASSWORD" -s "$env:ESIGNER_TOTP_SECRET" -m "$masterKey" -mode $mode

# Monta o certificado do Cloud HSM no repositório de certificados do Windows
& $ckaTool load
```

### 2.3. Resolução Dinâmica do Thumbprint
```powershell
$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Select-Object -First 1
if (-not $cert) {
    throw "CERTIFICADO NÃO ENCONTRADO: O eSigner CKA não carregou o certificado na store Cert:\CurrentUser\My!"
}
$thumbprint = $cert.Thumbprint
Write-Host "Certificado eSigner carregado com sucesso. Subject: $($cert.Subject) | Thumbprint: $thumbprint"
```

### 2.4. Assinatura com SignTool do Windows SDK
```powershell
$signtool = (Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" | Sort-Object FullName -Descending | Select-Object -First 1).FullName

# Assinatura de visor.exe
& $signtool sign /debug /fd sha256 /tr "http://ts.ssl.com" /td sha256 /sha1 "$thumbprint" "target/release/visor.exe"

# Assinatura do Instalador Oficial
& $signtool sign /debug /fd sha256 /tr "http://ts.ssl.com" /td sha256 /sha1 "$thumbprint" "dist/Visor-Setup-v0.4.4-x64.exe"
```

### 2.5. Descarregamento e Descarte de Sessão (`always()`)
```powershell
if (Test-Path $ckaTool) {
    Write-Host "Descarregando contexto do eSigner CKA..."
    & $ckaTool unload
    Remove-Item $ckaDir -Recurse -Force -ErrorAction SilentlyContinue
}
```

---

## 3. Gestão dos Modos Operacionais (TEST vs. PROD)

| Modo | Variável `ESIGNER_ENVIRONMENT` | Endpoint SSL.com | Timestamp Server | Destino dos Artefatos |
| :--- | :---: | :---: | :---: | :--- |
| **TEST / SANDBOX** | `SANDBOX` | `cs-try.ssl.com` | `http://ts.ssl.com` | Artefatos internos do CI (não publicar) |
| **PROD / PRODUCTION**| `PROD` | `cs.ssl.com` | `http://ts.ssl.com` | GitHub Releases público oficial |

> [!CAUTION]
> **Proteção contra Troca Acidental de Ambiente:**  
> O pipeline adota `SANDBOX` como padrão defensivo. Para assinar em produção (`PROD`), o operador deve configurar explicitamente a Secret `ESIGNER_ENVIRONMENT=PROD`, além de definir `dry_run: false` e `confirm_publication: "CONFIRMAR"`.
