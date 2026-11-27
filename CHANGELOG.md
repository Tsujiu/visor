# Registro de Alterações (Changelog) — VISOR

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

## [1.0.2] - 2026-11-27

### Adicionado
- **Suporte Completo à Flag `--remover-padrao`:**
  - Implementada a remoção limpa e idempotente dos registros do VISOR no catálogo de aplicativos do usuário (`HKCU`), cumprindo integralmente a documentação do utilitário de linha de comando.

### Modificado
- **Desinstalador Completo para Todas as 96+ Extensões Suportadas:**
  - Atualização do script Inno Setup para limpar automaticamente o registro de todas as extensões de imagem, RAW, vetores e texturas registradas pela versão atual.
- **Central do VISOR e Textos Dinâmicos de Versão:**
  - Remoção de menções estáticas à versão antiga nos cartões multilíngues da Central do VISOR e atualização completa da documentação (`LEIAME.txt`, `README-PORTABLE.txt` e READMEs).
- **Refatoração e Limpeza Estrutural do Código:**
  - Eliminação do módulo legado baseado em PowerShell (`SFTA.ps1`), reduzindo o tamanho do executável, e tratamento explícito de falhas ao iniciar o instalador de atualização.

### Corrigido
- **Qualidade de Código & Zero Avisos de Compilação:**
  - Correção de avisos de compilador, modernização de trechos clippy e validação completa da integridade do binário em tempo de execução.

## [1.0.1] - 2026-11-19

### Corrigido
- **Compatibilidade Nativa Definitiva no Windows (Eliminação de runtimes MinGW/GCC):**
  - Padronização integral da compilação e empacotamento no target oficial `x86_64-pc-windows-msvc` com runtime C/C++ estático (`+crt-static`).
  - O VISOR agora é 100% autocontido, iniciando perfeitamente em computadores Windows 10 e 11 limpos sem depender de MinGW, GCC, MSYS2 ou pacotes redistribuíveis externos.
- **Abertura Segura de Navegador via Win32 ShellExecuteW:**
  - Correção dos cliques em links externos (GitHub Oficial e Central do VISOR), acionando com confiabilidade o navegador padrão do usuário.
- **Diálogo Elegante de Apoio e Doações:**
  - Remoção de glifos ausentes nos botões e interface modal com links diretos para Brasil (PIX / InfinitePay) e Global (Stripe USD).

### Modificado
- **Ícone Oficial Refinado Fluent Design:**
  - Ícone do aplicativo com cantos arredondados no padrão Windows 11 (squircle) e transparência limpa.
- **Pipeline de Distribuição e Instalador Oficial Inno Setup:**
  - O instalador oficial consome exclusivamente binários MSVC estáticos auditados por verificação PE de dependências.

## [1.0.0] - 2026-11-17 — Lançamento Oficial

### Adicionado
- **Lançamento Oficial 1.0.0:**
  - Instalação oficial e otimizada para Windows 10 e Windows 11 com atualizações automáticas e silenciosas.
  - Experiência completa, rápida e segura para visualização de fotos do dia a dia e fluxos profissionais.
- **Internacionalização Completa em 9 Idiomas:**
  - Suporte nativo e 100% offline a Português do Brasil, Inglês, Espanhol, Francês, Alemão, Italiano, Russo, Chinês Simplificado e Japonês.
  - Detecção automática do idioma do sistema operacional e seletor manual na tela de Configurações.
- **Zoom Padrão em 100% (Tamanho Real 1:1):**
  - Abertura de imagens e navegação por padrão na escala nativa pixel por pixel, com preferência configurável em Configurações.
  - Alternância instantânea entre 100% e Ajustar à Janela com tecla F, duplo clique ou barra de ferramentas.
- **Biblioteca Abrangente com Mais de 75 Formatos de Imagem:**
  - Suporte completo a fotos convencionais, formatos modernos (AVIF, JXL), negativos digitais RAW de câmeras profissionais, arquivos vetoriais (SVG, DXF), arquivos de edição gráfica (PSD, XCF, CDR) e texturas.

### Modificado
- **Estabilidade e Confiabilidade Definitivas:**
  - Aplicativo 100% offline, sem anúncios, sem telemetria e com total respeito à sua privacidade.
  - Inicialização instantânea, baixo consumo de memória em repouso e navegação fluida em pastas com milhares de fotos.
- **Design Moderno Fluent Design:**
  - Tema escuro elegante, barra de ferramentas adaptável e painel técnico de informações fotográficas (EXIF).

## [0.4.4] - 2026-11-10

### Adicionado
- **Distribuição Segura e Confiável:**
  - Pacotes oficiais assinados com canal protegido de atualizações e verificação de integridade.

## [0.4.3] - 2026-11-09

### Modificado
- **Instalador Oficial Simplificado:**
  - Atualização direta e automática com um clique na janela de aviso, garantindo transição transparente para versões mais recentes.

### Adicionado
- **Notificação Discreta de Atualizações:**
  - Aviso visual elegante e não intrusivo quando houver novas versões disponíveis, com opções para atualizar imediatamente ou adiar.

## [0.4.2] - 2026-11-09

### Adicionado
- **Central de Informações Redesenhada:**
  - Diálogos com cantos arredondados, fundo escurecido imersivo e navegação organizada em abas: Destaques, Formatos e Segurança.
  - Atalhos de acesso rápido a novidades através do menu da barra de ferramentas.

### Modificado
- **Apresentação Clara de Atualizações:**
  - Visualização categorizada com identificadores visuais para novas funções, melhorias e correções.

### Segurança
- **Proteção Avançada de Integridade:**
  - Validação estrita de pacotes recebidos, assegurando proteção contra arquivos corrompidos ou adulterados.

## [0.4.1] - 2026-11-09

### Corrigido
- **Aprimoramentos de Layout e Tipografia:**
  - Quebra automática de texto em títulos e descrições longas, eliminando cortes nas margens.
  - Centralização suave de janelas flutuantes e botões de fechamento intuitivos.

### Modificado
- **Interface e Navegação Aprimoradas:**
  - Destaques visuais com resumos de novidades e barra de progresso suave durante transferências.

## [0.4.0] - 2026-11-09

### Adicionado
- **Verificação Inteligente de Versões:**
  - Consulta automática em segundo plano sem qualquer lentidão na exibição de fotos.
  - Frequência de verificação configurável nas Configurações do aplicativo.
  - Notificações com resumo das melhorias e opção de adiar ou verificar manualmente.

### Segurança
- **Verificação Rigorosa de Autenticidade:**
  - Garantia de download exclusivo de fontes oficiais e verificação contra alterações.
  - Zero coleta de dados pessoais, fotos locais ou hábitos de navegação.

### Modificado
- **Persistência de Preferências do Usuário:**
  - Salvamento automático de todas as configurações sem perda de preferências entre sessões.

## [0.3.3] - 2026-11-08

### Corrigido
- **Consistência Visual dos Menus Suspensos:**
  - Correção dos marcadores de seleção nos menus de interpolação e fundo da imagem, garantindo renderização limpa e uniforme de todos os itens.

## [0.3.2] - 2026-11-08

### Adicionado / Corrigido
- **Gerenciador de Aplicativo Padrão do Windows:**
  - Integração nativa com o Windows 10 e 11 para definir o VISOR como visualizador padrão com facilidade.
  - Detecção em tempo real dos formatos associados e pendentes.
  - Seção dedicada nas Configurações para gerenciar associações de arquivos com um clique.

## [0.3.1] - 2026-11-08

### Corrigido
- **Estabilidade do Espaço de Visualização:**
  - Otimização do dimensionamento da tira inferior de miniaturas em pastas com muitas imagens, garantindo que o visualizador central permaneça amplo e sem deformações.

## [0.3.0] - 2026-11-08

### Lançamento da Nova Geração do VISOR
- **Velocidade Extrema:** Abertura instantânea de imagens e carregamento inteligente em segundo plano sem travar a interface.
- **Tira de Miniaturas (Filmstrip):** Navegação ágil pelas imagens da pasta com visualização fiel das proporções.
- **Painel Técnico de Metadados (EXIF):** Exibição de informações detalhadas da câmera, exposição, lente e perfis de cor.
- **Modo Pixel-Perfect:** Alternância para nitidez absoluta em pixel art, texturas e ilustrações técnicas.
- **Integração com o Windows:** Suporte completo a copiar com transparência (Ctrl+C), menu "Abrir com..." e envio seguro para a Lixeira.

## [0.2.2] - 2026-11-06

### Adicionado
- **Navegação Fluida e Controles de Zoom:**
  - Suporte a rolagem do mouse para zoom suave ou navegação entre fotos.
  - Fundo personalizável com temas xadrez claro, xadrez escuro, cores sólidas ou seletor de cores.
  - Menus de contexto rápidos com atalhos de teclado e integração com o Explorador de Arquivos.

### Segurança e Conformidade
- Aplicativo totalmente seguro, 100% autônomo, sem telemetria e sem exigir privilégios de administrador.
