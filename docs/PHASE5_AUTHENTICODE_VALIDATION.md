# VISOR — VALIDAÇÃO DA ASSINATURA AUTHENTICODE E REPUTAÇÃO
## Critérios de Conformidade, Verificação Dupla e SmartScreen para Releases Comerciais

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Algoritmo de Digest:** SHA-256  
**Servidor de Carimbo de Tempo:** `http://ts.ssl.com` (RFC 3161)

---

## 1. Critérios de Aprovação Criptográfica da Assinatura

Para que um binário executável (`visor.exe`) ou instalador (`Visor-Setup-v0.4.4-x64.exe`) seja considerado apto para distribuição comercial, ele deve atender a **cinco critérios simultâneos e inegociáveis**:

1. **Status de Validade Estrita:** O status retornado pelo Windows CryptoAPI deve ser exatamente `Valid` (`SignatureStatus::Valid`).
2. **Identidade do Publicador (Subject):** O campo `Subject` do certificado deve corresponder exatamente à entidade jurídica ou pessoal proprietária do VISOR (ex: `CN = Projeto Visor LTDA` ou `CN = [Nome do Desenvolvedor]`).
3. **Cadeia de Confiança Pública (Issuer):** O emissor deve ser uma Autoridade Certificadora pública do *Microsoft Trusted Root Certificate Program* (ex: `SSL.com Root Certification Authority RSA / ECC`).
4. **Carimbo de Tempo Confiável (RFC 3161 Timestamp):** A assinatura deve conter carimbo de tempo válido emitido por autoridade reconhecida (`http://ts.ssl.com`). Isso garante que o software continue funcionando sem avisos de expiração mesmo após o término do período de validade do certificado digital.
5. **Integridade Estrutural PE/COFF:** A tabela de certificados (`IMAGE_DIRECTORY_ENTRY_SECURITY`) deve cobrir a totalidade dos bytes executáveis, garantindo que nenhum byte foi adulterado após a assinatura.

---

## 2. Metodologia de Verificação Dupla Automatizada

O pipeline de release engineering executa duas camadas complementares de verificação no runner Windows:

### 2.1. Camada 1: `signtool verify` (Windows SDK Nativo)
```powershell
& $signtool verify /pa /all /v "target/release/visor.exe"
& $signtool verify /pa /all /v "dist/Visor-Setup-v0.4.4-x64.exe"
```
- `/pa`: Aplica a *Default Authenticode Verification Policy*, simulando exatamente como o kernel do Windows e o Windows Defender avaliarão o arquivo quando o usuário der duplo clique.
- `/all`: Inspeciona todas as assinaturas presentes no arquivo.
- `/v`: Emite os detalhes completos da cadeia de certificados intermediários.
- **Critério:** Código de retorno `$LASTEXITCODE -eq 0`.

### 2.2. Camada 2: `Get-AuthenticodeSignature` (PowerShell Script Engine)
```powershell
$sig = Get-AuthenticodeSignature -FilePath "target/release/visor.exe"
if ($sig.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
    throw "ASSINATURA INVÁLIDA: Status = $($sig.Status) | Mensagem = $($sig.StatusMessage)"
}
Write-Host "Certificado Válido:"
Write-Host "  Subject:      $($sig.SignerCertificate.Subject)"
Write-Host "  Issuer:       $($sig.SignerCertificate.Issuer)"
Write-Host "  Thumbprint:   $($sig.SignerCertificate.Thumbprint)"
Write-Host "  Timestamp:    $($sig.TimeStamperCertificate.NotBefore)"
```

---

## 3. Rejeição Estrita de Assinaturas de Teste (Sandbox) para Produção

A SSL.com disponibiliza um ambiente Sandbox gratuito (`cs-try.ssl.com`) para validar a integração e os scripts de pipeline.

> [!CAUTION]
> **Bloqueio de Assinatura Sandbox em Releases Comerciais:**  
> Os certificados emitidos pelo Sandbox da SSL.com pertencem a uma Autoridade Certificadora de Teste (**SSL.com Test Root CA**) que **NÃO é confiável** por padrão nas instalações normais do Windows.  
> Se um binário assinado no Sandbox for distribuído para usuários finais:
> 1. O Windows exibirá aviso em vermelho informando que o certificado raiz não é confiável.
> 2. O SmartScreen bloqueará o aplicativo com severidade máxima.
> 
> **Diretriz de Segurança:**  
> O modo `SANDBOX` destina-se exclusivamente a validar o pipeline de build e o adaptador CKA. **Nenhum artefato assinado em Sandbox pode ser publicado no repositório de distribuição pública `Tsujiu/visor-downloads`.**

---

## 4. Expectativa Realista sobre o Microsoft Defender SmartScreen

A contratação e implementação de uma assinatura comercial com certificado padrão (IV ou OV) é um marco essencial de profissionalização, mas **não extingue imediatamente os avisos do SmartScreen no primeiro dia**.

### Fatores de Construção de Reputação:
- **Certificados Recém-Emitidos:** Possuem identidade comprovada, mas histórico zero de telemetria na nuvem da Microsoft.
- **Maturação Orgânica:** Conforme usuários baixam o instalador oficial assinado e confirmam a execução ("Mais informações" -> "Executar assim mesmo"), o serviço SmartScreen registra que os binários assinados por aquela chave não apresentam comportamento nocivo.
- **Persistência da Reputação:** Uma vez consolidada, a reputação fica atrelada ao **Certificado / Identidade do Publicador**, beneficiando automaticamente as futuras versões e atualizações subsequentes do VISOR.
