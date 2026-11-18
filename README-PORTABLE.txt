==============================================================================
                         VISOR 0.4.1 — VERSAO PORTATIL
==============================================================================

Bem-vindo a versao portatil do VISOR!

Esta distribuicao foi criada para permitir o uso imediato do VISOR a partir
de qualquer pasta, unidade de rede ou dispositivo USB (pen drive), sem a
necessidade de instalacao ou privilegios de administrador.


INSTRUCOES RAPIDAS
------------------------------------------------------------------------------
1. Execucao:
   Basta dar duplo clique em `visor.exe` para iniciar o aplicativo.
   O executavel pode ser executado a partir de caminhos contendo espacos e
   caracteres acentuados ou Unicode.

2. Abertura de Imagens:
   - Arraste imagens diretamente para a janela do aplicativo.
   - Ou utilize a linha de comando: `visor.exe "C:\Caminho\Para\Imagem.png"`
   - Ou associe arquivos pelo menu "Abrir com..." do Windows Explorer.

3. Associacoes de Arquivos:
   A versao portatil NAO altera o registro do Windows automaticamente.
   Caso deseje que o VISOR apareca na lista de aplicativos recomendados
   do Windows para o seu usuario, execute via terminal:
      visor.exe --registrar-padrao

4. Persistencia de Configuracoes:
   Para proteger suas personalizacoes e garantir compatibilidade com as
   diretrizes do Windows, o VISOR armazena o arquivo de configuracoes em:
      %LOCALAPPDATA%\Visor\settings.json
   (Mesmo local utilizado pela versao instalada).


CONTEUDO DESTE PACOTE
------------------------------------------------------------------------------
- visor.exe           : Executavel oficial do VISOR (64 bits, nativo).
- CHANGELOG.md        : Historico completo de mudancas e novidades da versao.
- LEIAME.txt          : Documentacao completa de recursos, requisitos e suporte.
- README-PORTABLE.txt : Este arquivo de instrucoes da versao portatil.


VERIFICACAO DE INTEGRIDADE (SHA-256)
------------------------------------------------------------------------------
Consulte o arquivo oficial SHA256SUMS.txt no local de download para verificar
o hash de integridade deste pacote.
==============================================================================
