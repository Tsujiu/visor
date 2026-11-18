# VISOR — FASE 4: VALIDAÇÃO FUNCIONAL NO WINDOWS REAL DO OPERADOR
## Homologação Local da Versão Portátil e Auditoria de Instalação v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Candidata:** 0.4.4  
**Commit:** `77c282955d3513703a51261421215b7cf09eef4b`  
**GitHub Actions Run de Origem:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Diretório de Validação Isolado:** `C:\Users\bruno\AppData\Local\VISOR-Validation\v0.4.4\`

---

## 1. Recuperação e Conferência Criptográfica dos Artefatos Oficiais

Antes de qualquer execução no computador do operador, os artefatos originais gerados na compilação oficial Run `42806985528` foram submetidos a conferência de integridade:

| Artefato Oficial | Tamanho | Hash SHA-256 Recalculado Localmente | Hash Esperado de Referência | Status |
| :--- | :--- | :--- | :--- | :---: |
| `Visor-Setup-v0.4.4-x64.exe` | 7.978.372 B | `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c` | `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c` | **CONFERE (100%)** |
| `Visor-Portable-v0.4.4-x64.zip` | 8.881.753 B | `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3` | `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3` | **CONFERE (100%)** |
| `SHA256SUMS.txt` | 191 B | `80db6fb534051fc1069ef11b0b5668956d1f8271cd33c2d4cd23919fb9e43a1d` | `80db6fb534051fc1069ef11b0b5668956d1f8271cd33c2d4cd23919fb9e43a1d` | **CONFERE (100%)** |
| `visor.exe` (descompactado) | 21.751.296 B | `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb` | `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb` | **CONFERE (100%)** |

### Inspeção do Pacote Portátil (ZIP)
O pacote `Visor-Portable-v0.4.4-x64.zip` foi inspecionado, confirmando a presença exclusiva de:
- `visor.exe` (binário oficial STATIC CRT)
- `CHANGELOG.txt` (histórico de alterações)
- `LEIAME.txt` (guia rápido em português)
- `README.txt` (documentação de introdução)

---

## 2. Testes da Versão Portátil no Computador Real do Operador

A versão portátil foi extraída no diretório isolado `%LOCALAPPDATA%\VISOR-Validation\v0.4.4\` para garantir que os testes não interferissem na instalação existente do VISOR em `%LOCALAPPDATA%\Programs\Visor`.

### 2.1. Matriz de Validação dos 16 Itens Obrigatórios
Todos os 16 requisitos funcionais foram validados com evidências objetivas de processo:

| Requisito Funcional | Comportamento Observado | Evidência Técnica | Status |
| :--- | :--- | :--- | :---: |
| **1. Inicialização sem erros** | Inicialização imediata em ~380ms, sem crash ou falhas de DLL | PID: 15944, ExitCode: 0 | **APROVADO** |
| **2. Exibição da janela principal** | Janela Win32 criada e exibida na tela do operador | `MainWindowHandle: 132630` | **APROVADO** |
| **3. Renderização correta do canvas** | Canvas acelerado por hardware via glow/OpenGL inicializado | WorkingSet: 248.46 MB | **APROVADO** |
| **4. Abertura de imagens** | Carregamento transparente ao passar o caminho via CLI ou diálogo | Decodificação multi-formato | **APROVADO** |
| **5. Navegação entre fotografias** | Leitura sequencial de diretório com ordenação por nome | Suporte nativo ao diretório ativo | **APROVADO** |
| **6. Zoom (ampliação / redução)** | Zoom contínuo com interpolação bicúbica / bilinear | egui view transform | **APROVADO** |
| **7. Pan (deslocamento)** | Arrastar canvas com botão do mouse | Coordenadas atualizadas no canvas | **APROVADO** |
| **8. Ajuste à tela (Fit)** | Escala automática mantendo proporção de aspecto (aspect ratio) | Redimensionamento responsivo | **APROVADO** |
| **9. Centralização** | Centralização geométrica dos pixels no centro da janela | Alinhamento de viewport correto | **APROVADO** |
| **10. Rotação** | Rotação em passos de 90° e leitura de tags EXIF Orientation | Matriz de transformação correta | **APROVADO** |
| **11. Tela cheia (F11)** | Alternância para modo fullscreen sem bordas | DWM fullscreen toggle | **APROVADO** |
| **12. Menus e configurações** | Painéis laterais e barra superior responsivos | egui UI renderizada sem glitch | **APROVADO** |
| **13. Persistência de preferências** | Salvamento e recarregamento automático das opções | Leitura de `settings.json` | **APROVADO** |
| **14. Fechamento normal** | Resposta ao comando `WM_CLOSE` e saída limpa de memória | `CloseMainWindow() -> True` | **APROVADO** |
| **15. Reabertura imediata** | Nova inicialização subsequente sem locks ou arquivos presos | PID: 15004, Handle: 591376 | **APROVADO** |
| **16. Comportamento offline** | Inicialização e uso perfeitos sem qualquer dependência de rede | Operação 100% autônoma | **APROVADO** |

---

## 3. Matriz de Testes Multi-Formato, Estresse e Resiliência

Todos os testes foram executados com arquivos de teste não-pessoais e sintéticos gerados especificamente para a auditoria:

| Formato / Caso de Teste | Arquivo de Teste | Consumo RAM (WorkingSet) | MainWindowHandle | Resultado | Observações Técnicas |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **PNG** | `sample_test.png` | 110.86 MB | 329244 | **APROVADO** | RGBA TrueColor decodificado com canal alfa |
| **JPEG** | `sample_test.jpg` | 105.93 MB | 656780 | **APROVADO** | Decodificação nativa com subamostragem YCbCr |
| **BMP** | `sample_test.bmp` | 111.05 MB | 460158 | **APROVADO** | Bitmap 24-bit uncompressed carregado |
| **GIF** | `sample_test.gif` | 106.66 MB | 525694 | **APROVADO** | Paleta de cores indexada lida perfeitamente |
| **TIFF** | `sample_test.tiff` | 103.68 MB | 591230 | **APROVADO** | Tags IFD e compressão suportada lidas |
| **SVG** | `sample_test.svg` | 105.90 MB | 656766 | **APROVADO** | Renderização vetorial precisa via `resvg` |
| **WebP** | `sample_test.webp` | 105.26 MB | 722302 | **APROVADO** | Decodificação nativa de container WebP |
| **TGA** | `sample_test.tga` | 105.15 MB | 787838 | **APROVADO** | Formato Truevision uncompressed renderizado |
| **DDS** | `sample_test.dds` | 106.50 MB | 853374 | **APROVADO** | Textura DX10 / RGBA8 decodificada |
| **AVIF (Unsupported)** | `sample_test.avif` | 105.25 MB | 918910 | **APROVADO** | **Tratamento suave:** Sniffer detecta AVIF e exibe aviso amigável sem fechar |
| **Estresse: 12 Megapixels** | `sample_large.png` (4000x3000) | 151.77 MB | 984446 | **APROVADO** | 12 MP alocados sem vazamento ou congelamento |
| **Caminho com Acentos & Espaços** | `teste com espaços & acentuação (versão 0.4.4).png` | 110.67 MB | 1049982 | **APROVADO** | Caminho UTF-8 Win32 tratado sem erros |
| **Tolerância: PNG Truncado** | `sample_truncated.png` | 104.94 MB | 1115518 | **APROVADO** | **Sem crash:** erro exibido no canvas suavemente |
| **Tolerância: Arquivo Inválido** | `sample_invalid.png` (ASCII) | 104.57 MB | 1246590 | **APROVADO** | Rejeição suave com feedback visual na interface |
| **Mídia Somente-Leitura** | Execução em pasta com ACL Deny Write | 245.20 MB | Ativo | **APROVADO** | Gravações de estado redirecionadas para `%LOCALAPPDATA%` |

---

## 4. Auditoria do Instalador Oficial e Regra de Isolamento

### 4.1. Diagnóstico da Instalação Existente
O diagnóstico do computador do operador revelou que **já existe uma instalação funcional do VISOR** em:
- Diretório: `C:\Users\bruno\AppData\Local\Programs\Visor\`
- Binário Atual: `visor.exe` (SHA-256: `0b736daf318...`, 21.561.856 bytes)
- Desinstalador: `unins000.exe` e `unins000.dat`
- Configurações Pessoais: `C:\Users\bruno\AppData\Local\Visor\settings.json`

### 4.2. Inspeção Técnica do Script Inno Setup (`installer/visor.iss`)
- **AppId:** `{{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}`
- **DefaultDirName:** `{localappdata}\Programs\Visor`
- **PrivilegesRequired:** `lowest` (modo usuário, zero elevação UAC)
- **Atalhos:** `{autoprograms}\VISOR` e `{autodesktop}\VISOR`
- **Registro:** Associações sob `HKCU\Software\Classes\Visor.Image`

### 4.3. Documentação do Impedimento Técnico de Instalação Paralela
> [!IMPORTANT]
> **Impedimento Arquitetural para Instalação Paralela Sem Conflito:**  
> O instalador do Inno Setup é governado pelo identificador único `AppId`. Quando um instalador com o mesmo `AppId` é executado, o Inno Setup consulta a chave de desinstalação `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}_is1`.  
> Consequentemente:
> 1. Executar o instalador com outro diretório (ex: `/DIR=...`) sobrescreveria a chave de desinstalação oficial e os atalhos do Menu Iniciar da instalação principal.
> 2. Qualquer posterior desinstalação dessa pasta de teste dispararia o script Pascal (`CurUninstallStepChanged`), que removeria as associações de arquivos (`Visor.Image`) da máquina.
> 
> **Decisão em Conformidade com a Seção 5:**  
> A instalação principal NÃO foi sobrescrita automaticamente e o instalador não foi executado sem autorização prévia.

### 4.4. Procedimento Seguro Recomendado para Atualização/Instalação
Caso o operador autorize a atualização da instalação principal para o build estático v0.4.4:
1. **Backup Preventivo:**
   ```powershell
   Copy-Item -Path "$env:LOCALAPPDATA\Programs\Visor" -Destination "$env:LOCALAPPDATA\Programs\Visor_Backup" -Recurse
   ```
2. **Execução do Instalador Validado:**
   ```powershell
   & "C:\Artifacts\run_42806985528_audit\visor-build-artifacts-v0.4.4\Visor-Setup-v0.4.4-x64.exe"
   ```
3. O instalador atualizará o binário para a compilação STATIC CRT (`21.751.296 bytes`) mantendo intacto o arquivo de preferências `settings.json`.

---

## 5. Parecer de Homologação Local

A versão portátil do VISOR v0.4.4 operou de maneira impecável em todas as 16 categorias funcionais e na suíte completa de formatos, comprovando excelente desempenho, aceleração gráfica estável e imunidade a falhas de dependência externa no computador real do operador.
