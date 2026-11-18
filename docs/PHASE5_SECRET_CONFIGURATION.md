# VISOR — ESPECIFICAÇÃO DE SEGREDOS DO eSIGNER NO GITHUB ACTIONS
## Guia de Mapeamento de Credenciais, Gestão de Chaves e Segurança de CI/CD

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Local de Configuração:** Repositório Privado `Tsujiu/visor` > *Settings > Secrets and variables > Actions*

---

## 1. Mapeamento de Segredos de Assinatura

Para viabilizar a assinatura automatizada de executáveis e instaladores via SSL.com eSigner Cloud HSM, as seguintes variáveis secretas devem ser registradas na plataforma do GitHub:

| Nome do Segredo | Descrição e Finalidade | Formato Esperado | Ambiente Aplicável |
| :--- | :--- | :--- | :---: |
| **`ESIGNER_USERNAME`** | Nome de usuário ou e-mail cadastrado na conta comercial da SSL.com | Texto simples (ex: `dev@empresa.com`) | SANDBOX / PROD |
| **`ESIGNER_PASSWORD`** | Senha de acesso à conta da SSL.com | Texto alfanumérico seguro | SANDBOX / PROD |
| **`ESIGNER_TOTP_SECRET`** | Chave secreta de autenticação de dois fatores (2FA) gerada na ativação do eSigner | String Base32 (ex: `JBSWY3DPEHPK3PXP`) | SANDBOX / PROD |
| **`ESIGNER_ENVIRONMENT`** | Seletor de ambiente da API eSigner | `SANDBOX` ou `PROD` | Global (padrão: `SANDBOX`) |
| **`ESIGNER_CREDENTIAL_ID`** *(Opcional)* | Identificador de credencial quando a conta possui múltiplos certificados | GUID / UUID | Opcional |

> [!IMPORTANT]
> **Preservação de Segredos Existentes:**  
> O segredo **`VISOR_DOWNLOADS_TOKEN`** (utilizado para publicar releases no repositório público `Tsujiu/visor-downloads`) deve permanecer **intacto e isolado**. Ele não deve ser reutilizado para autenticação na SSL.com.

---

## 2. Como Obter a Chave `ESIGNER_TOTP_SECRET` na SSL.com

1. Acesse o portal da SSL.com (`https://www.ssl.com` ou `https://cs-try.ssl.com` para Sandbox).
2. Localize seu certificado validado de Code Signing e clique em **Enroll to eSigner**.
3. O portal exibirá um QR Code para aplicativos autenticadores (Google Authenticator, Microsoft Authenticator).
4. Clique no link **"Show Secret Key"** ou copie o código alfanumérico textual abaixo do QR Code.
5. Esta string textual Base32 é o valor exato a ser cadastrado na secret `ESIGNER_TOTP_SECRET`.
6. O eSigner CKA utiliza esse segredo para calcular dinamicamente o código TOTP no instante exato da assinatura, eliminando qualquer intervenção humana durante o build.

---

## 3. Diretrizes Estritas de Proteção contra Vazamento de Credenciais

Para preservar a conformidade e a segurança do projeto:

1. **Nunca versionar arquivos contendo credenciais:** Nenhuma secret deve constar em arquivos `.env`, scripts `.ps1` ou manifests commitados no Git.
2. **Mascaramento Automático em Logs:** O GitHub Actions mascara nativamente valores injetados via Secrets (`***`). Mensagens de debug do script não devem interpolar senhas em strings de saída.
3. **Eliminação de Sessão Efêmera:** Conforme demonstrado no workflow, qualquer arquivo de configuração intermediário é gerado em pasta temporária (`$env:TEMP`) e deletado com `Remove-Item` imediatamente após a execução do `signtool`.
4. **Comportamento quando as Secrets Estão Ausentes:**
   - Em builds de desenvolvimento ou testes `dry_run: true`, o pipeline detecta a ausência de credenciais e opera no modo **`BUILD DE VALIDAÇÃO NÃO-ASSINADO`** com saída limpa.
   - Em disparos de publicação de produção (`dry_run: false` e `"CONFIRMAR"`), o workflow **BLOQUEIA** a execução com erro fatal, impedindo que executáveis não assinados sejam publicados por engano.
