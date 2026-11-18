# VISOR — ANÁLISE DO MICROSOFT DEFENDER SMARTSCREEN E REPUTAÇÃO
## Identidade do Publicador, Mark of the Web (MOTW) e Maturação de Confiança

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Contexto:** Distribuição de Software Comercial no Ecossistema Windows 10 e Windows 11

---

## 1. Funcionamento e Objetivos do Microsoft Defender SmartScreen

O **Microsoft Defender SmartScreen** é um serviço integrado de inteligência e segurança na nuvem da Microsoft projetado para proteger usuários finais contra malware, softwares potencialmente indesejados (PUA) e ataques de engenharia social.

Ao contrário de um antivírus tradicional baseado exclusivamente em assinaturas heurísticas de código malicioso, o SmartScreen opera primariamente com base no conceito de **Reputação Global de Aplicações**.

---

## 2. A Mecânica do Mark of the Web (MOTW)

Quando um instalador (`.exe`) ou arquivo comprimido (`.zip`) é transferido da internet através de um navegador web (Edge, Chrome, Firefox) ou cliente de e-mail, o subsistema I/O do Windows anexa automaticamente uma *Alternate Data Stream* (ADS) NTFS chamada **`Zone.Identifier`**:

```text
[ZoneTransfer]
ZoneId=3
ReferrerUrl=https://github.com/Tsujiu/visor-downloads/releases/...
HostUrl=https://objects.githubusercontent.com/...
```
- `ZoneId=3` identifica que o arquivo originou-se da Zona de Internet não-confiável.
- É a presença deste metadado que aciona a avaliação compulsória do SmartScreen no momento em que o usuário executa o arquivo.

---

## 3. Assinatura Válida vs. Reputação SmartScreen: Distinção Fundamental

Existe um equívoco comum na indústria de desenvolvimento de software de que "basta assinar um executável com um certificado Authenticode válido para eliminar imediatamente todas as mensagens do SmartScreen".

> [!IMPORTANT]
> **Fato Técnico Comprovado pela Documentação da Microsoft:**  
> A assinatura digital Authenticode garante **Identidade** e **Integridade** (comprova quem publicou o binário e que ele não foi modificado após a assinatura).  
> A assinatura digital **NÃO** concede reputação instantânea automática em certificados de validação padrão (Standard / OV).

### 3.1. Como a Reputação se Desenvolve:
1. **Certificados Standard / OV Recém-Emitidos:**
   - Possuem identidade verificada, mas **zero histórico prévio** nos servidores de telemetria da Microsoft.
   - Quando os primeiros usuários baixam e executam o instalador assinado, o SmartScreen ainda exibirá a tela azul de aviso: *"O Windows protegeu o seu computador"*.
   - A diferença crucial é que, em vez de exibir *"Editor: Desconhecido"*, o diálogo exibirá com orgulho e transparência o **Nome Oficial do Publicador** (ex: *"Editor: Projeto Visor LTDA"*).
   - À medida que dezenas de usuários clicam em "Executar assim mesmo" e a telemetria do Defender confirma que o aplicativo não apresenta comportamento malicioso, a chave pública do certificado constrói **reputação positiva**.
   - Uma vez atingido o limiar de reputação, o aviso desaparece permanentemente para todas as versões futuras assinadas com o mesmo certificado!
2. **Certificados Extended Validation (EV) e Microsoft Trusted Signing:**
   - Por envolverem validação de identidade empresarial mais profunda e guarda de chave em HSM físico ou nuvem da própria Microsoft, esses certificados recebem **reputação positiva acelerada ou imediata** no SmartScreen desde a primeira compilação.

---

## 4. Diretrizes Éticas e Transparência com o Usuário Final

O VISOR adota uma postura de conformidade absoluta com as políticas de segurança do ecossistema Windows:

### O que o Projeto VISOR NUNCA fará:
1. **Nunca instruir usuários a desabilitar o Windows Defender ou o SmartScreen.** Desabilitar recursos de proteção expõe o computador do usuário a riscos graves e viola práticas recomendadas de segurança.
2. **Nunca tentar contornar tecnicamente o Mark of the Web.** Tentativas de burlar o SmartScreen (como empacotar instaladores dentro de imagens ISO ou arquivos compactados com senha maliciosos) são tratadas pela Microsoft como indicativo de malware e resultam no banimento da reputação do aplicativo.
3. **Nunca fazer promessas falsas de que a assinatura elimina 100% dos avisos desde o minuto zero.**

### Comunicação Correta e Transparente nas Release Notes:
Para o lançamento comercial da versão v0.4.4, a documentação pública e a página de download oficial apresentarão orientações amigáveis e honestas:

> *"O VISOR é uma aplicação legítima, segura e de código auditado. Por ser uma versão recém-lançada construindo histórico de reputação no ecossistema Windows, o Microsoft Defender SmartScreen poderá exibir um aviso azul informando 'O Windows protegeu o seu computador'.  
> Para iniciar a instalação com tranquilidade, clique em **'Mais informações'** e selecione **'Executar assim mesmo'**. Você pode conferir a integridade absoluta do arquivo a qualquer momento comparando o hash SHA-256 oficial publicado em nosso repositório."*
