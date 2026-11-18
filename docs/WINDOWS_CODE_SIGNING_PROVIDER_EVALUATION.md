# VISOR — AVALIAÇÃO COMERCIAL E TÉCNICA DE PROVEDORES DE ASSINATURA DIGITAL
## Análise de Mercado para Authenticode, Cloud HSM e Desenvolvedores no Brasil

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Objetivo:** Identificar a solução de assinatura digital de código (*Code Signing*) comercialmente viável, tecnicamente robusta e compatível com as regras do ecossistema Windows e desenvolvedores no Brasil.

---

## 1. Contexto Regulatório Global (CA/Browser Forum Baseline Requirements)

Desde 1º de junho de 2023, as diretrizes internacionais do **CA/Browser Forum** (Baseline Requirements for the Issuance and Management of Publicly-Trusted Code Signing Certificates) impuseram regras estritas para **todos** os certificados de assinatura de código (tanto Standard/OV quanto Extended Validation/EV):

1. **Armazenamento de Chave Privada em Hardware Criptográfico Obrigatório:**  
   É expressamente proibido exportar chaves privadas em arquivos `.pfx` para armazenamento em disco rígido convencional. A chave privada deve residir em:
   - Um **Token FIPS 140-2/140-3 Level 2+ físico** (ex: dongle USB YubiKey FIPS / SafeNet eToken), OU
   - Um **Módulo de Segurança em Nuvem (Cloud HSM)** em conformidade com FIPS 140-2 Level 3 (ex: Azure Trusted Signing, SSL.com eSigner, Certum SimplySign, DigiCert ONE).
2. **Impacto para Desenvolvedores no Brasil:**  
   O envio internacional de tokens USB físicos para o Brasil enfrenta retenções alfandegárias na Receita Federal, cobrança de impostos de importação elevados e prazos de entrega que frequentemente superam 30 a 60 dias. **Soluções em nuvem (Cloud HSM) são amplamente preferíveis e recomendadas.**

---

## 2. Análise Detalhada dos Provedores Globais

### 2.1. Microsoft Trusted Signing (Azure Artifact Signing)
Lançado pela Microsoft em 2024 para modernizar a assinatura de código em pipelines CI/CD na nuvem Azure.

- **Arquitetura:** Serviço totalmente gerenciado na nuvem, integrado ao Azure Key Vault e GitHub Actions. Não requer envio de tokens físicos.
- **Preço Oficial:**
  - *Basic Tier:* US$ 9,99 / mês (inclui até 5.000 assinaturas por mês; US$ 0,005 por assinatura excedente).
  - *Premium Tier:* US$ 99,99 / mês (inclui até 100.000 assinaturas por mês).
- **Reputação SmartScreen:** Reputação imediata de raiz pública Microsoft no Windows Defender SmartScreen.
- **Restrição Crítica Geográfica e Cadastral (Novembro de 2026):**
  > [!WARNING]
  > **Incompatibilidade Atual para o Brasil:**  
  > De acordo com a documentação oficial da Microsoft (*Trusted Signing Supported Countries*), a validação de confiança pública (**Public Trust**) está restrita a uma lista fechada de países (EUA, Canadá, União Europeia, Reino Unido, Austrália, Japão, Singapura, etc.).  
  > **O Brasil NÃO está atualmente na lista de países aprovados para validação de identidade comercial pública.**  
  > Além disso, desenvolvedores Pessoa Física (*Individual Developers*) só são aceitos se forem residentes nos Estados Unidos ou Canadá através do Microsoft Entra Verified ID.

### 2.2. SSL.com (eSigner Cloud HSM)
Autoridade Certificadora (CA) internacional integrante do Microsoft Trusted Root Program.

- **Arquitetura:** Plataforma de nuvem **eSigner** (Cloud HSM FIPS 140-2 Level 3). Permite assinar binários Windows via linha de comando (`CodeSignTool`), API REST ou GitHub Action oficial sem envio de token USB físico.
- **Pessoa Física vs. Pessoa Jurídica:** Aceita emissão tanto para **Pessoa Física (Individual Developer / CPF)** quanto para **Pessoa Jurídica (CNPJ / MEI / LTDA)** no Brasil. A verificação para pessoa física exige envio de CNH/Passaporte, comprovante de residência e validação biométrica em vídeo.
- **Preço Estimado:**
  - Certificado Code Signing Individual / OV: ~US$ 69 a US$ 129 / ano.
  - Assinatura eSigner Cloud: Planos a partir de ~US$ 10 / mês (ou pacotes de assinaturas por lote).
- **Compatibilidade com Windows SDK:** Total suporte a `signtool.exe` através de credencial de nuvem ou utilitário de mapeamento local.
- **RFC 3161 Timestamping:** Incluso sem custo adicional.

### 2.3. Certum by Asseco (SimplySign Cloud HSM)
Uma das maiores e mais antigas CAs europeias credenciadas pela Microsoft, amplamente utilizada por desenvolvedores de software livre e pequenas empresas.

- **Arquitetura:** Serviço **SimplySign** (smartcard virtual em nuvem). Instala um mini-driver no Windows que permite ao `signtool.exe` nativo assinar executáveis como se houvesse um token físico inserido na máquina.
- **Pessoa Física vs. Pessoa Jurídica:** Oferece formalmente a modalidade *Standard Code Signing for Individuals* (Pessoa Física). Validação realizada mediante envio de documento oficial de identidade com foto (passaporte ou CNH traduzida) e reconhecimento facial.
- **Preço Estimado:**
  - Certificado SimplySign Cloud: ~€ 79 a € 149 / ano (renovação anual).
  - Sem custos ocultos por assinatura individual.
- **Vantagem Operacional:** Não requer hardware físico, contornando a alfândega brasileira.

### 2.4. DigiCert / Sectigo (Modelos Tradicionais OV / EV)
Líderes de mercado em certificação corporativa.

- **Arquitetura:** Tradicionalmente baseadas em envio de Token USB FIPS físico pelos Correios / DHL, ou soluções de nuvem corporativas voltadas a grandes corporações (*DigiCert ONE* / *Sectigo KeyDecode*).
- **Preço Estimado:**
  - Certificado OV Comercial: US$ 300 a US$ 500 / ano.
  - Certificado EV Comercial: US$ 600 a US$ 900 / ano.
  - Plataformas de Nuvem Enterprise: Custos anuais que costumam exceder US$ 1.500 / ano.
- **Pessoa Física:** Praticamente inacessível para pessoa física sem registro comercial formal. Exigem número D-U-N-S, contrato social ou CNPJ ativo.
- **Desvantagem para o Projeto:** Custo desproporcional e fricção de envio de token físico para o Brasil.

### 2.5. Análise de Viabilidade: ICP-Brasil (e-CPF / e-CNPJ)
No Brasil, desenvolvedores frequentemente questionam se certificados digitais locais da cadeia ICP-Brasil (Certisign, Serasa, Soluti, etc.) podem ser utilizados para assinatura de executáveis Windows.

> [!CAUTION]
> **Incompatibilidade Técnica do ICP-Brasil com Windows Authenticode Global:**  
> A raiz certificadora do ICP-Brasil **NÃO** faz parte da lista de raízes confiáveis do *Microsoft Root Certificate Program* para assinatura de código de uso geral.  
> Se o VISOR for assinado com um e-CPF ou e-CNPJ ICP-Brasil:
> 1. O Windows acusará certificado de autoridade desconhecida.
> 2. O SmartScreen bloqueará o executável com a tela vermelha ou azul de aplicativo perigoso/desconhecido.
> 3. Usuários fora do Brasil receberão erro fatal de certificado inválido.
> 
> *Conclusão:* Certificados ICP-Brasil são estritamente inadequados para distribuição pública de software comercial no Windows.

---

## 3. Matriz Comparativa de Critérios Técnicos e Comerciais

| Critério de Avaliação | Microsoft Trusted Signing | SSL.com (eSigner) | Certum (SimplySign) | Sectigo / DigiCert | ICP-Brasil (e-CPF) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Disponível para Brasil?** | ❌ Não (Public Trust) | ✅ **Sim** | ✅ **Sim** | ⚠️ Com restrições | ✅ Sim (local) |
| **Pessoa Física (CPF)?** | ❌ Não (Apenas EUA/CA) | ✅ **Sim (KYC)** | ✅ **Sim (KYC)** | ❌ Apenas PJ | ✅ Sim |
| **Pessoa Jurídica (CNPJ)?** | ❌ Países suportados | ✅ **Sim** | ✅ **Sim** | ✅ Sim | ✅ Sim |
| **Compatível Authenticode?** | ✅ Sim | ✅ **Sim** | ✅ **Sim** | ✅ Sim | ❌ Não reconhecido |
| **Integração Windows SDK** | ✅ Sim (Dlib) | ✅ **Sim (`signtool`)**| ✅ **Sim (`signtool`)**| ✅ Sim (`signtool`) | ⚠️ Inútil no Windows |
| **Integração GitHub Actions** | ✅ Sim (Action oficial)| ✅ **Sim (Action/CLI)**| ⚠️ Requer runner local | ⚠️ Requer Enterprise | ❌ Não suportado |
| **Proteção de Chave Privada**| Cloud HSM FIPS 140-3 | **Cloud HSM Level 3** | **Cloud HSM Level 3** | Token USB FIPS | Token USB / A1 |
| **Sem Envio Físico p/ Brasil?**| ✅ Sim | ✅ **Sim (100% Nuvem)** | ✅ **Sim (100% Nuvem)** | ❌ Token físico | ✅ Sim (A1) |
| **Timestamp RFC 3161?** | ✅ Sim | ✅ **Sim (Incluso)** | ✅ **Sim (Incluso)** | ✅ Sim | ⚠️ Próprio |
| **Custo Estimado** | ~US$ 10 / mês | **~US$ 69-129 / ano** | **~€ 79-149 / ano** | US$ 300-800 / ano | R$ 150-300 / ano |
| **Complexidade Operacional** | Baixa | **Média** | **Média** | Alta (Alfândega) | Inviável |

---

## 4. Recomendação Técnica e Estratégia Comercial para o VISOR

### Opção 1 (Recomendada no Curto Prazo — Pessoa Física / MEI no Brasil): **SSL.com eSigner**
- **Justificativa:** É a solução com menor atrito que atende desenvolvedores sediados no Brasil, permite validação de identidade como Pessoa Física (Individual) ou MEI (CNPJ), armazena a chave em Cloud HSM seguro (evitando taxas de importação de dongles físicos) e possui ferramenta de linha de comando (`CodeSignTool`) e GitHub Actions oficiais para automação contínua.

### Opção 2 (Recomendada no Médio Prazo — Quando Disponível no Brasil): **Microsoft Trusted Signing**
- **Justificativa:** Assim que a Microsoft expandir a validação pública de identidade para o Brasil, esta se tornará a solução definitiva pela integração nativa e custo altamente acessível (US$ 9,99/mês).

---

## 5. Regras Estritas de Segurança e Governança Criptográfica

Para preservar a integridade do projeto:
1. **Nunca utilizar certificados autoassinados** fingindo serem comerciais.
2. **Nunca comitar chaves privadas (`.key`, `.pem`, `.pfx`)** ou senhas em nenhum repositório do Git.
3. Todas as credenciais de assinatura devem ser injetadas exclusivamente via **GitHub Actions Secrets criptografados**.
4. Certificados temporários em disco devem ser gravados em diretórios efêmeros (`$env:TEMP`) e apagados de forma atômica e segura imediatamente após o comando `signtool`.
