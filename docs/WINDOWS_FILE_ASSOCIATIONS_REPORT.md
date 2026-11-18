# Relatório de Integração com o Shell e Associações de Arquivo do Windows — VISOR v0.5.0

**Data:** 15 de Novembro de 2026  
**Ambiente Alvo:** Windows 10 e Windows 11 (builds 22H2, 23H2 e 24H2)  
**Conformidade:** Windows UCPD (*User Choice Protection Driver*) e Win32 Shell API

---

## 1. Visão Geral e Desafios no Windows 10/11

A partir do Windows 10 e especialmente com o Windows 11 (e o driver protetor de escolha do usuário — UCPD introduzido em 2024), aplicações desktop não têm permissão para sobrescrever silenciosamente as associações de arquivo padrão do usuário. Tentativas de gravar hashes diretos na chave `UserChoice` do registro acionam violação de integridade e redefinição automática do Windows para o aplicativo padrão da Microsoft (ex: Microsoft Fotos).

O VISOR implementa uma arquitetura Win32 nativa plenamente compatível:
- Registro limpo de ProgIDs, `OpenWithProgids` e `RegisteredApplications`, guiando o usuário de forma transparente para a tela nativa de configurações do Windows (`ms-settings:defaultapps`).

---

## 2. Arquitetura de Registro Win32

### 2.1. ProgIDs Estruturados
Para cada extensão suportada, o VISOR registra um identificador programático padronizado:
- **Chave Principal:** `HKCU\Software\Classes\Visor.Image.<EXT>`
- **Valor Padrão:** "Arquivo de Imagem <EXT>"
- **Ícone:** `DefaultIcon` apontando para o binário `%LOCALAPPDATA%\Programs\Visor\visor.exe,0`.
- **Comando de Abertura:**
  `shell\open\command = "%LOCALAPPDATA%\Programs\Visor\visor.exe" "%1"`

### 2.2. Integração com o Menu "Abrir com..." (`OpenWithProgids`)
Em vez de forçar a associação exclusiva, o VISOR insere seu ProgID na lista de candidatos conhecidos do Windows Explorer:
- `HKCU\Software\Classes\.<ext>\OpenWithProgids\Visor.Image.<EXT> = ""`
Isso garante que o VISOR apareça imediatamente no menu de contexto do botão direito ("Abrir com") sem gerar alertas de segurança do sistema.

### 2.3. Registro de Aplicação (`RegisteredApplications`)
O VISOR é cadastrado no registro global de capacidades de aplicativos padrão:
- **Chave de Capacidades:** `HKCU\Software\Visor\Capabilities`
  - `ApplicationName`: "VISOR"
  - `ApplicationDescription`: "Visualizador leve, rápido e profissional de imagens para Windows"
  - `FileAssociations`: Lista completa de todas as extensões mapeadas para seu respectivo ProgID.
- **Registro no Sistema:**
  `HKCU\Software\RegisteredApplications\Visor = "Software\Visor\Capabilities"`

Isso faz com que o VISOR seja listado oficialmente no menu **Configurações > Aplicativos > Aplicativos Padrão** do Windows 10 e 11.

---

## 3. Conformidade com o UCPD e Experiência do Usuário

1. **Ausência de Hijacking:** O VISOR nunca tenta calcular hashes proprietários da Microsoft na chave `UserChoice`.
2. **Deep-Link de Configuração:** Quando o usuário clica em associar arquivos na interface do VISOR, a aplicação:
   - Configura os ProgIDs e capacidades locais no registro de usuário.
   - Invoca a URI nativa do Windows:
     `ms-settings:defaultapps?registeredApp=Visor`
   - O Windows 11 abre a página dedicada do VISOR onde o operador pode definir o VISOR como padrão com apenas um clique oficial.


