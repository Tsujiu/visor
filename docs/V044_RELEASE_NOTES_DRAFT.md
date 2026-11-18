# VISOR v0.4.4 — PRIMEIRA RELEASE COMERCIAL OFICIAL
## Visualizador de Imagens Moderno, Autocontido e Ultrarrápido para Windows

**Versão:** `v0.4.4`  
**Data Prevista de Publicação:** Novembro de 2026  
**Status:** Release Candidate Aprovado  
**Público-Alvo:** Usuários domésticos, profissionais de criação e ambientes corporativos Windows 10/11 x64

---

## Destaques da Versão v0.4.4

### 🚀 1. Executável 100% Autocontido (Static CRT Linkage)
A versão v0.4.4 introduz a vinculação estática do runtime C da Microsoft (`-C target-feature=+crt-static`). O VISOR agora é totalmente independente do pacote *Microsoft Visual C++ Redistributable*. Ele executa instantaneamente em qualquer máquina Windows 10 ou Windows 11 limpa, sem mensagens de erro de `VCRUNTIME140.dll` ausente.

### 🛡️ 2. Instalação em Modo Usuário (Zero UAC)
O novo instalador Inno Setup foi projetado para instalar no diretório do usuário (`%LOCALAPPDATA%\Programs\Visor`) sem exigir privilégios de administrador ou acionar confirmações irritantes do UAC (*User Account Control*). Instale com um clique mesmo em computadores corporativos restritos.

### 🖼️ 3. Suporte Completo a 16 Formatos de Imagem
Visualize imagens com renderização acelerada por GPU, navegação fluida, zoom suave e alto desempenho em formatos modernos e legados:
- **Padrão:** PNG, JPEG, BMP, GIF, WebP, ICO.
- **Profissional / Fotografia:** TIFF, SVG, PSD, HDR, EXR.
- **Texturas e Jogos:** DDS, TGA, QOI.

### 💼 4. Fundação Comercial e Transparência
- **Edição Gratuita (Free):** Acesso irrestrito a todos os recursos essenciais e decodificadores visuais.
- **Edição Profissional (Pro):** Sem anúncios, recursos avançados de produtividade e suporte prioritário.
- **Privacidade Absoluta:** O motor de anúncios em modo Free é totalmente isolado e anônimo, sem rastreamento de comportamento nem coleta de dados pessoais.

### 🔄 5. Update Center Integrado
Verificação automática e segura de novas versões através do canal oficial do GitHub Releases, com conferência de integridade criptográfica SHA-256 antes da instalação.

---

## Registro Detalhado de Alterações (Changelog)

### Adicionado
- **Static CRT:** Configuração `.cargo/config.toml` forçando vinculação `/MT` para compatibilidade universal no Windows.
- **Instalador Inno Setup v6.3.3:** Empacotamento profissional com atalhos no Menu Iniciar, Área de Trabalho e associações de arquivos registradas por usuário.
- **Tratamento Seguro de Atualizações:** Mitigação robusta contra ataques de Path Traversal (`is_safe_filename`) e suporte a HTTP 304 (ETag caching).
- **Suíte de Testes Expandida:** Bateria de testes de release ampliada para 278 testes unitários e de integração com cobertura total dos decodificadores e monetização.

### Corrigido
- **Erro de Runtime em VM Limpa:** Eliminada a dependência externa de `VCRUNTIME140.dll` e das bibliotecas dinâmicas `api-ms-win-crt-*`.
- **Compilação de Release:** Implementada trava em tempo de compilação impedindo vazamento de features de simulação local para binários de produção.
- **Metadados PE:** Sincronização estrita de versão `0.4.4.0` nos cabeçalhos PE do executável Windows e no instalador.

---

## Tabela Oficial de Hashes Criptográficos SHA-256

Antes de executar os arquivos baixados, você pode conferir a integridade utilizando o comando nativo do PowerShell `Get-FileHash <arquivo>`:

| Arquivo de Distribuição | Tamanho Exato | Hash SHA-256 Autêntico |
| :--- | :--- | :--- |
| **`Visor-Setup-v0.4.4-x64.exe`** | 7.978.372 bytes | `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c` |
| **`Visor-Portable-v0.4.4-x64.zip`** | 8.881.753 bytes | `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3` |
| **`visor.exe`** *(executável interno)* | 21.751.296 bytes | `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb` |

---

## Requisitos do Sistema

- **Sistema Operacional:** Windows 10 (versão 1809 ou superior) ou Windows 11 (todas as versões).
- **Arquitetura:** Processador x86-64 (AMD64 / Intel 64 bits).
- **Memória RAM:** Mínimo de 512 MB livres (recomendado 2 GB para visualização de imagens ultra-resolução).
- **Placa de Vídeo / Gráficos:** Suporte a DirectX 11 / OpenGL 3.3 ou superior para aceleração de renderização via GPU.
- **Dependências Externas:** **Nenhuma.** O aplicativo não necessita de Java, .NET Framework ou Visual C++ Redistributable.

---

## Instruções de Instalação e Execução

### Opção 1: Instalador Oficial (Recomendado)
1. Baixe o arquivo `Visor-Setup-v0.4.4-x64.exe`.
2. Dê um duplo clique no instalador e siga as instruções na tela.
3. Não são necessários privilégios de administrador.

### Opção 2: Versão Portátil (Sem Instalação)
1. Baixe o arquivo `Visor-Portable-v0.4.4-x64.zip`.
2. Extraia o conteúdo para uma pasta de sua preferência (ex: em um pendrive ou pasta Documentos).
3. Execute diretamente o `visor.exe`.

---

## Nota sobre o Microsoft Defender SmartScreen

> [!NOTE]
> Por se tratar do lançamento inicial da versão v0.4.4, os novos executáveis estão em processo de maturação da reputação global na nuvem de segurança da Microsoft.
> Ao baixar pelo navegador e executar o arquivo pela primeira vez, o Windows Defender SmartScreen poderá exibir a janela:
> **"O Windows protegeu o seu computador"**.
> 
> **Para prosseguir com a instalação:**
> 1. Clique no link **"Mais informações"** (*More info*).
> 2. Clique no botão **"Executar assim mesmo"** (*Run anyway*).
> 
> Você pode verificar a autenticidade do instalador a qualquer momento comparando o hash SHA-256 fornecido na tabela acima.
