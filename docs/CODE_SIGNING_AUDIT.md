# VISOR — AUDITORIA DE ASSINATURA DIGITAL (AUTHENTICODE) E SMARTSCREEN
## Diagnóstico de Reputação, Impacto na Experiência do Usuário e Roteiro de Implementação

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Artefatos Auditados:**  
- `Visor-Setup-v0.4.4-x64.exe` (SHA-256: `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c`)
- `visor.exe` (SHA-256: `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb`)

---

## 1. Diagnóstico Atual da Assinatura Digital

A verificação formal de assinatura digital Authenticode foi executada no ambiente PowerShell nativo do Windows:

```powershell
Get-AuthenticodeSignature -FilePath "C:\Artifacts\run_42806985528_audit\visor-build-artifacts-v0.4.4\Visor-Setup-v0.4.4-x64.exe"
Get-AuthenticodeSignature -FilePath "C:\Artifacts\run_42806985528_audit\unpacked_portable\visor.exe"
```

### Resultados da Auditoria:
| Arquivo | Status Authenticode | StatusMessage | SignerCertificate |
| :--- | :---: | :---: | :---: |
| `Visor-Setup-v0.4.4-x64.exe` | **NotSigned** | O arquivo especificado não está assinado. | `$null` |
| `visor.exe` | **NotSigned** | O arquivo especificado não está assinado. | `$null` |

### Causa e Contexto
Nenhum certificado digital de assinatura de código (*Code Signing Certificate*) comercial foi anexado aos binários nesta versão de lançamento v0.4.4. A ausência de certificado é comum e esperada em projetos de software no início do ciclo comercial antes da contratação formal de uma autoridade certificadora (CA).

---

## 2. Mecânica do Microsoft Defender SmartScreen e MOTW

Quando um usuário baixa um arquivo executável da internet através de qualquer navegador moderno (Edge, Chrome, Firefox), o sistema operacional Windows anexa uma Alternate Data Stream (ADS) NTFS chamada **Mark of the Web (MOTW)**:
```text
Zone.Identifier:
[ZoneTransfer]
ZoneId=3 (Internet)
```

### 2.1. Comportamento do SmartScreen ao Abrir Arquivo Unsigned com MOTW
Ao tentar executar um instalador sem assinatura ou com assinatura de baixa reputação:
1. O Windows Defender SmartScreen consulta o serviço de reputação na nuvem da Microsoft.
2. Como o executável possui um novo hash SHA-256 e não possui assinatura de editor conhecido, o Windows bloqueia a execução preventiva e exibe a tela azul modal:
   > *"O Windows protegeu o seu computador"*  
   > *"O Microsoft Defender SmartScreen impediu a inicialização de um aplicativo não reconhecido. A execução deste aplicativo pode colocar o computador em risco."*
   > *Editor: Desconhecido*  
   > *Aplicativo: Visor-Setup-v0.4.4-x64.exe*

### 2.2. Como o Usuário Consegue Executar
O usuário precisa clicar manualmente no link **"Mais informações"** (*More info*) e, em seguida, clicar no botão **"Executar assim mesmo"** (*Run anyway*).

---

## 3. Matriz Comparativa de Certificados e Reputação SmartScreen

| Tipo de Certificado / Assinatura | Custo Estimado | Requisitos de Emissão | Comportamento no SmartScreen |
| :--- | :---: | :--- | :--- |
| **Sem Assinatura (Estado Atual v0.4.4)** | R$ 0 / Grátis | Nenhum | **Aviso Azul de Bloqueio** até que centenas de usuários aprovem manualmente |
| **Certificado Padrão (OV - Organization Validation)** | ~US$ 200 - 400 / ano | CNPJ / Identidade corporativa validada | Aviso Azul inicial; constrói reputação gradualmente com volume |
| **Certificado Estendido (EV - Extended Validation)** | ~US$ 500 - 900 / ano | Verificação rigorosa + Token FIPS USB físico ou HSM na nuvem | **Reputação instantânea**; sem aviso azul desde o primeiro download |
| **Azure Trusted Signing (Recomendado)** | ~US$ 9,99 / mês | Conta corporativa Microsoft Azure + CNPJ verificado | **Reputação instantânea**, sem necessidade de token USB físico |

---

## 4. Mitigações para o Lançamento Comercial da Versão v0.4.4

Como o lançamento v0.4.4 será realizado antes da integração do certificado pago, as seguintes ações operacionais devem ser adotadas no material de distribuição pública:

1. **Documentação Explícita nas Release Notes e no Site:**
   Incluir um aviso transparente e orientador com captura de tela instruindo os usuários sobre o SmartScreen:
   > *"Por ser uma nova versão lançada recentemente e em fase de maturação de reputação no ecossistema Windows, o SmartScreen pode exibir uma tela de aviso. Para instalar, clique em 'Mais informações' e depois em 'Executar assim mesmo'."*
2. **Disponibilização Pública dos Hashes SHA-256:**
   Publicar ostensivamente a tabela de hashes SHA-256 em todas as páginas de download para que usuários avançados e empresas possam confirmar a autenticidade dos binários através do comando `Get-FileHash`.

---

## 5. Roteiro Técnico para Integração do Azure Trusted Signing (Fase 4)

Para eliminar definitivamente qualquer aviso do SmartScreen a partir da versão v0.4.5, recomenda-se a adoção do **Microsoft Azure Trusted Signing** (anteriormente *Azure Code Signing*):

### Vantagens do Azure Trusted Signing:
- Emissão direta pela Microsoft (Microsoft Identity Verification).
- Concede reputação imediata no Windows Defender SmartScreen.
- Não requer envio de tokens físicos USB FIPS (Dongles).
- Integração nativa e perfeita com GitHub Actions através da Action oficial `@azure/trusted-signing-action`.
- Custo acessível (modelo pré-pago por assinatura mensal em torno de US$ 10).

### Exemplo de Etapa a ser Inserida no `.github/workflows/release-distribution.yml`:
```yaml
- name: Assinar Binários com Azure Trusted Signing
  uses: azure/trusted-signing-action@v0.4.0
  with:
    azure-tenant-id: ${{ secrets.AZURE_TENANT_ID }}
    azure-client-id: ${{ secrets.AZURE_CLIENT_ID }}
    azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
    endpoint: 'https://eus.codesigning.azure.net/'
    trusted-signing-account-name: 'visor-software'
    certificate-profile-name: 'visor-release'
    files: |
      target/x86_64-pc-windows-msvc/release/visor.exe
      dist/Visor-Setup-v0.4.4-x64.exe
```

---

## 6. Parecer Técnico Final sobre Assinatura

A ausência de assinatura digital não afeta a funcionalidade, o desempenho, a estabilidade ou a segurança intrínseca do software. Contudo, trata-se de um ponto de fricção na experiência do usuário final que deve ser tratado via comunicação transparente no lançamento v0.4.4 e mitigado definitivamente na infraestrutura da Fase 4.
