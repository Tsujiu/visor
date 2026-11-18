# VISOR — ARQUITETURA TÉCNICA SSL.COM eSIGNER CLOUD HSM
## Avaliação de Integração, Mecanismos CKA vs. CodeSignTool e Governança Criptográfica

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Provedor de Certificação:** SSL.com (eSigner for Code)  
**Conformidade:** CA/Browser Forum Baseline Requirements (Hardware Crypto HSM FIPS 140-2 Level 3)

---

## 1. Contexto Executivo e Requisitos da Fase 5

A Fase 4 do projeto VISOR avaliou os provedores comerciais globais de assinatura digital de código e concluiu que o **SSL.com eSigner** representa a solução de menor atrito e maior viabilidade para desenvolvedores sediados no Brasil, pois:
1. Elimina a necessidade de remessa postal internacional de tokens físicos USB FIPS (sujeitos a retenção alfandegária e tributação na Receita Federal).
2. Permite emissão para desenvolvedores independentes (Pessoa Física / IV) e empresas (Pessoa Jurídica / OV / EV).
3. A chave privada permanece permanentemente resguardada em Módulo de Segurança de Hardware em Nuvem (**Cloud HSM FIPS 140-2 Level 3**).

> [!IMPORTANT]
> **Princípio Fundamental de Não-Exportabilidade da Chave:**  
> Ao contrário do modelo arcaico baseado em arquivos `.pfx` em disco, no SSL.com eSigner **a chave privada nunca é exportada nem trafegada pela rede**.  
> O pipeline de compilação envia apenas o resumo criptográfico (digest SHA-256) do binário para o HSM remoto, que aplica a assinatura e devolve a estrutura CMS/PKCS #7 assinada para embutimento no cabeçalho PE do Windows.

---

## 2. Comparativo Técnico: eSigner CKA vs. eSigner CodeSignTool

A SSL.com disponibiliza duas abordagens oficiais para automação de assinatura em pipelines de CI/CD:

| Critério de Avaliação | eSigner CKA (Cloud Key Adapter) | eSigner CodeSignTool (CLI / Java) | Decisão de Arquitetura VISOR |
| :--- | :--- | :--- | :---: |
| **Mecanismo Operacional** | Driver KSP / CNG nativo do Windows | Utilitário CLI autônomo baseado em Java | **eSigner CKA Escolhido** |
| **Integração com Ferramentas** | `signtool.exe` do Windows SDK nativo | Script proprietário `CodeSignTool.bat` | **CKA integra diretamente com `signtool`** |
| **Integração com Inno Setup** | Suporte transparente via KSP / SignTool | Requer chamadas em lote pós-compilação | **CKA é nativo no ecossistema Win32** |
| **Dependências de Ambiente** | Nenhuma (binário executável puro CKA) | Requer runtime Java (JRE/JDK 11+) | **CKA tem menor footprint no runner** |
| **Localização do Certificado** | Montado em `Cert:\CurrentUser\My` | Acessado via API REST direta | **CKA permite consulta via PowerShell** |
| **Desempenho no CI/CD** | Altíssimo (autentica uma vez por job) | Inicialização da JVM a cada comando | **CKA reduz tempo total de compilação** |
| **Segurança da Chave Privada**| Reside no Cloud HSM FIPS 140-2 L3 | Reside no Cloud HSM FIPS 140-2 L3 | **Idêntica (HSM remoto seguro)** |

### Conclusão e Seleção Técnica:
A arquitetura selecionada para o VISOR é o **eSigner CKA (Cloud Key Adapter)**. Por utilizarmos runners Windows oficiais (`windows-latest`) e o compilador de instaladores Inno Setup 6, o CKA disponibiliza a chave do Cloud HSM diretamente para o `signtool.exe` nativo do Windows SDK através da API Cryptography Next Generation (CNG), permitindo assinar com `/sha1 <thumbprint>` e carimbo de tempo oficial RFC 3161 (`http://ts.ssl.com`).

---

## 3. Modalidades de Certificado e Titularidade Jurídica

A emissão do certificado junto à SSL.com deve respeitar estritamente a identidade jurídica do titular do software:

1. **Individual Validation (IV Code Signing — Pessoa Física):**
   - Emitido em nome pessoal do desenvolvedor.
   - Requisitos: Envio de documento oficial de identidade com foto (Passaporte ou CNH) e reconhecimento facial por vídeo (KYC).
   - O campo *Subject* da assinatura exibirá o nome civil do operador.
2. **Organization Validation (OV Code Signing — Pessoa Jurídica / MEI / LTDA):**
   - Emitido em nome da empresa ou microempreendimento registrado.
   - Requisitos: Comprovante de CNPJ ativo, contrato social ou Certificado de Condição de Microempreendedor Individual (CCMEI), e confirmação de linha telefônica comercial.
   - O campo *Subject* exibirá a razão social ou nome fantasia comercial.
3. **Extended Validation (EV Code Signing — Pessoa Jurídica Avançada):**
   - Máximo nível de validação empresarial; concede reputação instantânea no Microsoft Defender SmartScreen.
   - Requer pessoa jurídica constituída e verificação de existência jurídica formal.

> [!WARNING]
> **Aviso de Titularidade e Validação:**  
> O projeto VISOR não presume aceitação tácita de documentos brasileiros sem a confirmação por escrito da equipe de vetting da SSL.com. Antes da contratação comercial definitiva, deve-se confirmar formalmente com o suporte da SSL.com o aceite dos documentos específicos da modalidade escolhida (Pessoa Física com passaporte brasileiro ou Pessoa Jurídica com CNPJ/MEI).

---

## 4. Gerenciamento da Chave Local do CKA (`master.key`)

O utilitário `eSignerCKATool.exe` requer a definição de uma senha mestre temporária (`-m <MASTER_KEY>`) para criptografar a sessão local de comunicação do KSP no runner:

### Diretrizes Estritas de Segurança:
1. **Geração Efêmera em Tempo de Execução:** A senha mestre local é gerada dinamicamente durante o job do GitHub Actions através de gerador pseudo-aleatório criptográfico (`[System.Guid]::NewGuid().ToString("N")`).
2. **Nunca Versionada:** A chave mestre local jamais é comitada, salva em repositório ou exportada como artefato.
3. **Destruição Atômica:** No step de conclusão do job (bloco `always()`), o comando `eSignerCKATool.exe unload` é disparado e quaisquer arquivos temporários de sessão em `%APPDATA%\eSignerCKA` são sumariamente removidos.
4. **Isolamento Total:** A chave privada de assinatura propriamente dita permanece no HSM da SSL.com, inacessível ao runner.
