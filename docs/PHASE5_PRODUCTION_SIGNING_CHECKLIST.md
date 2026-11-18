# VISOR — CHECKLIST DE LANÇAMENTO E ASSINATURA EM PRODUÇÃO
## Procedimento Passo a Passo para Publicação Oficial da Release v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Alvo:** `v0.4.4`  
**Status Atual:** Infraestrutura Preparada e Validada (Aguardando Credenciais de Produção)

---

## 1. Etapa de Pré-Requisitos Comerciais e Validação de Identidade (Operador)

Antes de disparar a compilação final assinada, o operador humano deve concluir as seguintes etapas externas:

- [ ] **1.1. Definição da Titularidade:** Decidir se o certificado será contratado como Pessoa Física (IV - Individual) ou Pessoa Jurídica (OV - Organization / MEI / CNPJ).
- [ ] **1.2. Confirmação com SSL.com:** Abrir ticket ou chat com o suporte da SSL.com confirmando formalmente a aceitação de documentos do Brasil (Passaporte/CNH ou CCMEI/CNPJ) para a modalidade escolhida.
- [ ] **1.3. Aquisição do Certificado:** Adquirir o plano de Code Signing com assinatura eSigner Cloud HSM.
- [ ] **1.4. Conclusão do Processo de Vetting (KYC):** Enviar os documentos solicitados e realizar a validação de identidade requerida pela autoridade certificadora.
- [ ] **1.5. Inscrição no eSigner:** No painel da SSL.com, ativar o eSigner para o certificado e capturar a chave de dois fatores (**TOTP Secret**).
- [ ] **1.6. Configuração de Secrets no Repositório Privado (`Tsujiu/visor`):**
  - [ ] `ESIGNER_USERNAME`
  - [ ] `ESIGNER_PASSWORD`
  - [ ] `ESIGNER_TOTP_SECRET`
  - [ ] `ESIGNER_ENVIRONMENT` (configurado como `PROD`)
  - [ ] `VISOR_DOWNLOADS_TOKEN` (já existente e confirmado)

---

## 2. Etapa de Teste em Ambiente Sandbox (Recomendado Antes de Produção)

- [ ] **2.1. Execução Dry-Run com Sandbox:**
  Executar o workflow `.github/workflows/release-distribution.yml` com:
  - `tag_name`: `v0.4.4`
  - `dry_run`: `true`
  - `ESIGNER_ENVIRONMENT`: `SANDBOX`
- [ ] **2.2. Verificação dos Logs do CKA:**
  Confirmar que o eSigner CKA baixou, autenticou, montou o certificado na store e executou `signtool sign` e `signtool verify` sem erros.

---

## 3. Disparo da Release Comercial Oficial (Produção)

Somente após a aprovação expressa do operador:

- [ ] **3.1. Merge do PR da Fase 5:** Realizar o merge controlado da branch `feature/visor-esigner-integration` para a branch `main`.
- [ ] **3.2. Disparo do Workflow de Publicação:**
  No GitHub Actions (`Tsujiu/visor`), executar o workflow **"Release & Distribute to Official Downloads Repo"**:
  - `tag_name`: `v0.4.4`
  - `dry_run`: `false`
  - `confirm_publication`: `CONFIRMAR`
- [ ] **3.3. Monitoramento em Tempo Real:** Acompanhar a execução das 12 etapas:
  - 278 testes automatizados aprovados.
  - Compilação STATIC CRT.
  - Assinatura Authenticode de `visor.exe`.
  - Empacotamento do ZIP portátil.
  - Compilação Inno Setup.
  - Assinatura Authenticode do `Visor-Setup-v0.4.4-x64.exe`.
  - Cálculo de `SHA256SUMS.txt` com os arquivos finais assinados.
  - Publicação no repositório público `Tsujiu/visor-downloads`.

---

## 4. Auditoria e Verificação Pós-Lançamento

Imediatamente após a conclusão do workflow:

- [ ] **4.1. Inspeção no Repositório Público:**
  Acessar `https://github.com/Tsujiu/visor-downloads/releases/tag/v0.4.4` e verificar:
  - Presença dos 3 assets: `Visor-Setup-v0.4.4-x64.exe`, `Visor-Portable-v0.4.4-x64.zip`, `SHA256SUMS.txt`.
  - Ausência de código-fonte (`.rs`, `.toml`) e símbolos de depuração (`.pdb`).
- [ ] **4.2. Download e Conferência de Hashes:**
  Baixar os artefatos em uma máquina local e conferir os hashes:
  ```powershell
  Get-FileHash Visor-Setup-v0.4.4-x64.exe -Algorithm SHA256
  Get-FileHash Visor-Portable-v0.4.4-x64.zip -Algorithm SHA256
  ```
  Confrontar com os valores publicados em `SHA256SUMS.txt`.
- [ ] **4.3. Conferência de Assinatura Authenticode:**
  ```powershell
  Get-AuthenticodeSignature Visor-Setup-v0.4.4-x64.exe
  ```
  Confirmar: `Status: Valid`, emissor público confiável e carimbo de tempo presente.
- [ ] **4.4. Teste de Consulta do Update Center:**
  Abrir uma versão anterior do VISOR conectada à internet e verificar se o aviso de nova versão disponível (`v0.4.4`) é exibido corretamente.
