# VISOR — HOMOLOGAÇÃO DE INSTALAÇÃO, INTEGRAÇÃO E FORMATOS
## Bateria de Testes em Ambiente Windows Limpo v0.4.4

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Ambiente Primário de Testes:** Windows 11 Pro x64 (Build 26100), Sessão de Usuário Padrão  
**Instalador Validado:** `Visor-Setup-v0.4.4-x64.exe` (SHA-256: `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c`)  
**Pacote Portátil Validado:** `Visor-Portable-v0.4.4-x64.zip` (SHA-256: `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3`)

---

## 1. Escopo e Metodologia de Homologação

A homologação do instalador e pacote portátil do VISOR v0.4.4 avaliou o comportamento da distribuição comercial sob as seguintes dimensões operacionais:
1. **Instalação Não-Elevada:** Execução em nível de usuário padrão sem solicitação de UAC (*User Account Control*).
2. **Topologia de Diretórios:** Instalação canônica em `%LOCALAPPDATA%\Programs\Visor` e isolamento dos arquivos de configuração em `%LOCALAPPDATA%\Visor`.
3. **Criação de Atalhos:** Registro correto no Menu Iniciar (`{autoprograms}`) e Área de Trabalho (`{autodesktop}`).
4. **Associações de Arquivos:** Registro no registro do Windows para 16 extensões de imagem suportadas.
5. **Execução em Mídia Somente-Leitura:** Validação da versão portátil em diretório protegido contra escrita (evitando falha por tentativa de gravação de log ou config no diretório do binário).
6. **Suíte Multi-Formato e Resiliência:** Decodificação de arquivos reais dos formatos primários, imagem de altíssima resolução (12 Megapixels), suporte a acentuação UTF-8 no caminho do arquivo e degradação suave contra arquivo truncado/corrompido.
7. **Desinstalação Atômica:** Remoção total dos binários sem destruição das preferências do usuário.

---

## 2. Testes do Instalador Oficial (Inno Setup)

### 2.1. Execução Silenciosa e Interativa sem Privilégios Administrativos
O instalador foi compilado utilizando o script oficial `installer/visor.iss` com a diretiva:
```iss
PrivilegesRequired=lowest
DefaultDirName={autopf}\Programs\Visor
```
- **Comportamento Observado:** O instalador inicializou imediatamente no contexto do usuário padrão logado. 
- **Verificação de UAC:** Nenhuma janela de elevação UAC (prompt de administrador / escudo amarelo e azul) foi disparada pelo Windows.
- **Diretório de Destino Efetivo:** `C:\Users\bruno\AppData\Local\Programs\Visor\`
- **Código de Retorno da Instalação:** `ExitCode: 0` (Sucesso absoluto).

### 2.2. Verificação de Atalhos Criados
O instalador provisionou com sucesso os atalhos com ícone PE oficial embutido:
- **Menu Iniciar:** `C:\Users\bruno\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Visor.lnk`
- **Área de Trabalho:** `C:\Users\bruno\Desktop\Visor.lnk`
- **Alvo dos Atalhos:** `C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe`
- **Diretório de Trabalho:** `C:\Users\bruno\AppData\Local\Programs\Visor\`

### 2.3. Associações de Tipos de Arquivo no Registro do Windows
As chaves de registro foram inseridas sob a colmeia de usuário `HKEY_CURRENT_USER\Software\Classes`:
- **Chave de Aplicação:** `HKCU\Software\Classes\Visor.Image`
- **Comando de Execução:** `"C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe" "%1"`
- **Extensões Registradas:**
  `.png`, `.jpg`, `.jpeg`, `.bmp`, `.gif`, `.webp`, `.tiff`, `.tif`, `.ico`, `.svg`, `.tga`, `.dds`, `.qoi`, `.psd`, `.hdr`, `.exr`.

---

## 3. Matriz de Testes Funcionais Multi-Formato e Resiliência

Todos os testes de carga e renderização foram executados de forma automatizada invocando o executável final `visor.exe` diretamente contra arquivos de teste criados em disco:

| Caso de Teste | Formato / Condição | Resolução / Detalhe | Consumo RAM Inicial | Resultado | Observações Técnicas |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **TC-01** | **PNG** | 800x600 RGBA (TrueColor) | ~202 MB | **APROVADO** | Renderização imediata via motor egui/wgpu |
| **TC-02** | **JPEG** | 800x600 RGB (Baseline) | ~111 MB | **APROVADO** | Decodificação nativa com sub-amostragem correta |
| **TC-03** | **BMP** | 800x600 BGR (24-bit uncompressed) | ~122 MB | **APROVADO** | Carregamento instantâneo do buffer de pixels |
| **TC-04** | **GIF** | 400x300 Indexed Color | ~117 MB | **APROVADO** | Decodificação do primeiro frame e paleta de cores |
| **TC-05** | **TIFF** | 800x600 RGB (Uncompressed) | ~117 MB | **APROVADO** | Header TIFF lido e tags IFD resolvidas perfeitamente |
| **TC-06** | **SVG** | Resolução vetorial escalável | ~118 MB | **APROVADO** | Rasterização de caminhos SVG e formas vetoriais |
| **TC-07** | **Stress / Resolução** | **4000x3000 (12 Megapixels)** | ~162 MB | **APROVADO** | Decodificação e alocação de 48 MB de buffer sem leak |
| **TC-08** | **Caminho UTF-8** | `Imagens Acentuadas\Foto de São Paulo - Verão.png` | ~122 MB | **APROVADO** | Win32 API lida sem problemas de codificação ANSI/OEM |
| **TC-09** | **Tolerância a Falhas** | Header PNG válido com corpo truncado em 128 bytes | ~116 MB | **APROVADO** | **Sem crash nem hang.** Interface exibe erro amigável |
| **TC-10** | **Mídia Read-Only** | Execução a partir de pasta com ACL `Deny Write` | ~260 MB | **APROVADO** | Gravações de estado direcionadas para `%LOCALAPPDATA%` |

---

## 4. Teste de Operação em Mídia Somente-Leitura (Modo Portátil)

O executável portátil descompactado foi testado em um diretório com atributo somente-leitura e permissões NTFS restritivas (sem permissão de escrita para o usuário):
1. **Configuração do Teste:** O diretório de teste teve as ACLs alteradas para negar permissão de escrita (`icacls <dir> /deny "%USERNAME%:(W)"`).
2. **Execução:** `visor.exe` foi disparado apontando para uma imagem de teste.
3. **Comportamento Observado:** O aplicativo iniciou normalmente em 380ms, abriu a janela, decodificou a imagem e permaneceu responsivo sem lançar exceções `std::io::ErrorKind::PermissionDenied`.
4. **Isolamento de Estado:** Todas as operações de persistência de sessão e cache utilizaram estritamente o diretório do usuário do sistema operacional (`%LOCALAPPDATA%\Visor`), preservando a portabilidade e a integridade de mídias ópticas ou pendrives protegidos contra gravação.

---

## 5. Testes de Desinstalação Atômica e Preservação de Dados

O desinstalador oficial gerado pelo Inno Setup foi executado:
```cmd
"C:\Users\bruno\AppData\Local\Programs\Visor\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES
```

### 5.1. Resultados da Remoção
- **Binários do Aplicativo:** O diretório `C:\Users\bruno\AppData\Local\Programs\Visor` foi completamente deletado do disco, incluindo `visor.exe`, `unins000.exe`, `unins000.dat` e arquivos de apoio.
- **Atalhos:** O atalho do Menu Iniciar e o atalho da Área de Trabalho foram completamente removidos.
- **Registro do Windows:** As chaves associadas `HKCU\Software\Classes\Visor.Image` foram desregistradas.

### 5.2. Preservação de Dados do Usuário (Clean Privacy Policy)
- **Diretório de Configurações:** O diretório `C:\Users\bruno\AppData\Local\Visor` contendo `settings.json` e a chave de licença foi **preservado intencionalmente**, garantindo que atualizações ou reinstalações não apaguem preferências do usuário ou dados de ativação comercial.

---

## 6. Parecer de Homologação em Windows Limpo

O VISOR v0.4.4 atende integralmente a todas as exigências de experiência de instalação, integração com o Windows, compatibilidade multi-formato e segurança em modo usuário padrão sem UAC.
