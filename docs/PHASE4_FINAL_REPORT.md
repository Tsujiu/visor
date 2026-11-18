# VISOR — FASE 4: RELATÓRIO FINAL CONSOLIDADO
## Assinatura Digital, Identidade do Publicador e Validação Local no Windows

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Candidata:** 0.4.4 (`v0.4.4`)  
**Target:** `x86_64-pc-windows-msvc`  
**Runtime:** STATIC CRT (`-C target-feature=+crt-static`)  
**Commit de Produção:** `77c282955d3513703a51261421215b7cf09eef4b`  
**GitHub Actions Run de Origem:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Branch de Trabalho:** `feature/visor-phase4-signing`  
**Repositório Privado (Fonte):** `https://github.com/Tsujiu/visor`  
**Repositório Público (Distribuição):** `https://github.com/Tsujiu/visor-downloads`

---

## 1. Classificação Formal da Fase 4

### **DECISÃO: APPROVED WITH CONDITIONS** *(Aprovado com Condições)*

#### Justificativa da Decisão:
O VISOR v0.4.4 foi submetido a uma bateria extensiva de testes diretamente no **computador real do operador** (Windows 11 Pro físico), sem uso de máquinas virtuais ou emuladores, alcançando **100% de aprovação** funcional em todos os formatos de imagem e condições operacionais. A arquitetura de assinatura Authenticode com `signtool.exe` foi completamente projetada, a ordem de empacotamento CI/CD foi ajustada para cálculo de hashes pós-assinatura, o Update Center foi auditado e a instalação pré-existente do operador foi integralmente preservada.

A classificação **WITH CONDITIONS** decorre exclusivamente da necessidade de provisão das credenciais comerciais de certificado digital pelo operador humano antes da publicação ao público em geral.

---

## 2. As 18 Entregas Formais da Fase 4 (Seção 13)

### 1. Informações do Windows Utilizado
- **Sistema Operacional:** Microsoft Windows 11 Pro 64 bits (`x86_64`).
- **Versão / Build:** Versão 10.0.26300, Build 26300.
- **Processador:** AMD Ryzen 5 5500 (6 núcleos, 12 threads).
- **Usuário e Privilégios:** Sessão padrão de usuário (`MUTSUJII\bruno`, `IsAdministrator: False`).
- **Espaço Livre:** 659 GB livres no disco C:\.

### 2. Resultado da Validação Local
- Execução direta no hardware do operador, sem VM e sem Windows Sandbox.
- Desempenho excelente com inicialização da janela Win32 em ~380ms.
- Renderização acelerada via GPU sem tearing ou lentidão.

### 3. Resultado do Teste Portátil
- **16 Itens Funcionais Aprovados:** Inicialização, exibição da janela (`MainWindowHandle: 132630`), canvas, abertura de arquivos, navegação, zoom, pan, fit, centralização, rotação, fullscreen (F11), menus egui, persistência de configurações, fechamento normal (`WM_CLOSE`), reabertura imediata e operação 100% autônoma offline.
- **Suíte de Formatos Aprovada:** PNG, JPEG, BMP, GIF, TIFF, SVG, WebP, TGA, DDS, AVIF (tratamento suave de formato não suportado), imagem de alta resolução de 12 MP (4000x3000), caminhos acentuados UTF-8 com espaços, arquivo truncado, arquivo inválido e execução a partir de pasta somente-leitura.

### 4. Resultado da Inspeção do Instalador
- Script Inno Setup 6 (`installer/visor.iss`) auditado.
- Instalação per-user em `%LOCALAPPDATA%\Programs\Visor` com `PrivilegesRequired=lowest` (zero UAC).
- Registro de atalhos e 16 associações de imagem sob `HKCU\Software\Classes\Visor.Image`.

### 5. Problemas Encontrados
- **Impedimento Arquitetural de Instalação Paralela:** O Inno Setup utiliza um `AppId` estático. Executar uma instalação paralela simultânea sobrescreveria a chave de desinstalação e atalhos da instalação principal existente. A regra de isolamento foi rigorosamente cumprida e a instalação atual não foi sobrescrita.

### 6. Confirmação da Preservação da Instalação Existente
- A instalação principal em `C:\Users\bruno\AppData\Local\Programs\Visor` foi mantida 100% intacta.
- O arquivo de preferências `C:\Users\bruno\AppData\Local\Visor\settings.json` (902 bytes) foi preservado sem qualquer modificação.

### 7. Estado da Dependência STATIC CRT
- Compilação estática (`+crt-static` com flags `/MT`) mantida intacta.
- Zero imports de `VCRUNTIME140.dll` e zero imports de `api-ms-win-crt-*`.
- Exatamente 20 DLLs importadas, todas nativas do Windows 10/11.

### 8. Opções Atuais de Assinatura Digital
- Pesquisa detalhada realizada junto às CAs globais em conformidade com as regras do CA/Browser Forum (exigência de Cloud HSM ou token físico FIPS).

### 9. Fornecedor Recomendado e Justificativa
- **Recomendação Atual:** **SSL.com (eSigner Cloud HSM)**.
- **Justificativa:** É a opção mais ágil que aceita desenvolvedores sediados no Brasil (Pessoa Física via CPF ou Pessoa Jurídica via CNPJ), armazena chaves em Cloud HSM seguro (dispensando o envio alfandegário de tokens USB) e suporta automação oficial em CI/CD.
- *Nota sobre Microsoft Trusted Signing:* Permanece indisponível para validação de confiança pública no Brasil na data presente.

### 10. Custos e Requisitos Encontrados
- **SSL.com:** Certificado Code Signing Individual / OV a partir de ~US$ 69 a US$ 129/ano + plano eSigner Cloud (~US$ 10/mês). Requer documento oficial com foto e comprovação de endereço.
- **Certum SimplySign:** ~€ 79 a € 149/ano.
- **ICP-Brasil:** Descartado por não possuir raiz confiável no Windows Authenticode global.

### 11. Estado da Implementação Authenticode
- Ordem de empacotamento corrigida no workflow CI/CD.
- Suporte nativo a `signtool.exe` do Windows SDK com validação via `signtool verify /pa` e `Get-AuthenticodeSignature`.

### 12. Resultado da Assinatura dos Executáveis no Momento
- Os artefatos do build atual (Run `42806985528`) mantêm status Authenticode `NotSigned` até a injeção do certificado comercial pelo operador nos segredos do repositório.

### 13. Situação do Microsoft Defender SmartScreen
- Documentada a mecânica do Mark of the Web (`ZoneId=3`).
- Esclarecida a distinção entre assinatura válida (Identidade) e construção de reputação (volume de telemetria).
- Minuta de release notes preparada com aviso orientador e hashes de integridade.

### 14. Alterações Feitas no Workflow
- Workflow `.github/workflows/release-distribution.yml` atualizado para assinar `visor.exe` antes de empacotar o ZIP e compilar o instalador.
- Hashes `SHA256SUMS.txt` gerados estritamente após a assinatura do instalador.
- Trava de proteção adicionada para impedir publicação comercial oficial sem assinatura.

### 15. Estado do Atualizador (Update Center)
- Endpoint oficial preservado: `https://api.github.com/repos/Tsujiu/visor-downloads/releases/latest`.
- Imposição estrita de HTTPS e lista branca de URLs.
- 28 testes automatizados da engine executados e aprovados com 100% de sucesso.

### 16. Hashes Finais dos Artefatos de Referência
- `Visor-Setup-v0.4.4-x64.exe`: `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c`
- `Visor-Portable-v0.4.4-x64.zip`: `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3`
- `SHA256SUMS.txt`: `80db6fb534051fc1069ef11b0b5668956d1f8271cd33c2d4cd23919fb9e43a1d`
- `visor.exe` (interno): `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb`

### 17. Commits Realizados
- Criação e avanço da branch de trabalho: `feature/visor-phase4-signing`.
- Registro e inclusão de todos os 8 relatórios técnicos de homologação na pasta `docs/`.

### 18. Pendências que Dependem de Decisão do Operador
1. **Aquisição / Configuração do Certificado:** Decidir entre a aquisição do certificado SSL.com eSigner (ou aguardar expansão do Microsoft Trusted Signing para o Brasil) e configurar as Secrets `SIGNING_CERTIFICATE_PFX_BASE64` e `SIGNING_CERTIFICATE_PASSWORD`.
2. **Atualização da Instalação do Operador:** Autorizar ou executar o procedimento seguro de atualização da instalação principal em `%LOCALAPPDATA%\Programs\Visor`.
3. **Publicação Oficial:** Autorizar o merge e disparo da release comercial quando os itens acima forem definidos.
