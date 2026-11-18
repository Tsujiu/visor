# VISOR — UI DESIGN SYSTEM & GUIA DE ESTILO WINDOWS 11

**Documento:** Fundação do Design System da Interface  
**Versão:** 1.0.0  
**Data:** 06/11/2026  
**Status:** Oficial  
**Público-alvo:** Engenharia de Interface, Arquitetura de Software e Design de Produto  

---

## 1. Filosofia e Princípios de Design

O VISOR é projetado como um visualizador de imagens moderno, extremamente rápido e minimalista, inspirado na linguagem visual do **Windows 11 (Fluent Design System)**, sem simular incorretamente chamadas de DWM ou prometer recursos de composição que o backend gráfico multiplataforma não executa nativamente.

### Princípios Fundamentais:

1. **Protagonismo Absoluto da Imagem:**
   * A fotografia/gráfico é o centro das atenções.
   * O canvas de exibição ocupa a maior área possível da tela (próximo de 100%).
   * Barras de ferramentas, controles e contadores não devem competir visualmente com a imagem.
2. **Superfícies Discretas e Neutras (Quiet Surfaces):**
   * Superfícies escuras ligeiramente dessaturadas (`#141416` a `#202024`), evitando tons de cinza azulados estourados ou fundos pretos absolutos que causem "recorte" forçado.
   * Transparências sutis aplicadas apenas a controles flutuantes (cápsula) sobre a imagem.
3. **Bordas Suaves e Translúcidas (Subtle 1px Borders):**
   * Em vez de sombras pesadas ou bordas escuras grossas, os contornos utilizam traços de 1px com transparência de 8% a 15% (`rgba(255, 255, 255, 0.08)` a `0.15`), conferindo acabamento premium e distinção de profundidade sutil.
4. **Cantos Suavemente Arredondados (Rounded Corners):**
   * Cantos de controles, cards, menus e painéis seguem a métrica do Windows 11:
     * Botões e inputs: raio de 5–6 px.
     * Cards, menus e dialogs: raio de 8–12 px.
     * Toolbar flutuante: cápsula com raio total (pill: 24 px).
5. **Tipografia com Hierarquia Serena:**
   * Textos principais em branco suave (`#F4F4F6`), evitando ofuscamento.
   * Metadados secundários em tom atenuado (`#A0A0A8`).
   * Atalhos de teclado e valores terciários em tom discreto (`#686872`).

---

## 2. Paleta de Cores e Superfícies (Tokens)

### 2.1 Superfícies e Camadas (Dark Mode)

| Token | Hex / RGBA | Finalidade / Aplicação |
| :--- | :--- | :--- |
| `SURFACE_BASE` | `#141416` | Fundo principal da janela e área externa do canvas. |
| `SURFACE_PANEL` | `#1E1E22` | Superfície sólida de painéis encaixados (status bar, docking). |
| `SURFACE_CARD` | `#28282C` | Cards de metadados, seções internas e fundos de listas. |
| `SURFACE_FLYOUT` | `rgba(32, 32, 36, 0.94)` | Superfície da Toolbar cápsula flutuante e menus popover. |
| `SURFACE_HOVER` | `rgba(255, 255, 255, 0.07)` | Realce suave em botões sutis ao passar o mouse. |
| `SURFACE_ACTIVE`| `rgba(255, 255, 255, 0.12)` | Feedback visual ao pressionar botões ou itens de menu. |

### 2.2 Bordas e Contornos (Strokes)

| Token | Espessura / Cor | Aplicação |
| :--- | :--- | :--- |
| `BORDER_SUBTLE` | 1.0 px / `rgba(255, 255, 255, 0.08)` | Contorno da Toolbar cápsula flutuante, cards e separadores. |
| `BORDER_DEFAULT`| 1.0 px / `rgba(255, 255, 255, 0.14)` | Contorno de botões secundários, inputs e caixas de seleção. |
| `BORDER_STRONG` | 1.0 px / `rgba(255, 255, 255, 0.25)` | Delimitação de diálogos modais e janelas destacadas. |
| `BORDER_FOCUS`  | 2.0 px / `#60CDFF` | Foco de navegação por teclado e acessibilidade. |

### 2.3 Cores de Destaque (Accent Windows 11 Blue)

| Token | Hex / RGB | Aplicação |
| :--- | :--- | :--- |
| `ACCENT_DEFAULT`| `#60CDFF` (`96, 205, 255`) | Cor primária Fluent no Dark Mode: botões primários, links. |
| `ACCENT_HOVER`  | `#80DAFF` (`128, 218, 255`) | Estado hover de elementos com accent. |
| `ACCENT_ACTIVE` | `#40C0FF` (`64, 192, 255`) | Estado pressionado de botões com accent. |
| `ACCENT_MUTED`  | `rgba(96, 205, 255, 0.15)` | Fundo de seleção ativa (ex: miniatura atual no filmstrip). |

### 2.4 Tipografia e Hierarquia de Texto

| Token | Hex / RGB | Uso |
| :--- | :--- | :--- |
| `TEXT_PRIMARY`   | `#F4F4F6` | Títulos, nome do arquivo aberto, dimensões em foco. |
| `TEXT_SECONDARY` | `#A0A0A8` | Informações técnicas, metadados EXIF, status secundário. |
| `TEXT_MUTED`     | `#686872` | Dicas de atalho de teclado (`Ctrl+O`), labels inativos. |
| `TEXT_ON_ACCENT` | `#000000` | Texto de alto contraste sobre botões primários accent. |

---

## 3. Métrica de Espaçamento e Cantos Arredondados

```text
┌────────────────────────────────────────────────────────┐
│                        ESPACAMENTO                     │
│  XXS: 2px   XS: 4px   SM: 8px   MD: 12px   LG: 16px    │
│  XL: 24px   XXL: 32px                                  │
├────────────────────────────────────────────────────────┤
│                    CANTOS ARREDONDADOS                 │
│  XS: 3px (tags/badges)          SM: 5px (botões/inputs)│
│  MD: 8px (cards/menus)          LG: 12px (drawers)     │
│  PILL: 24px (toolbar flutuante em cápsula)             │
└────────────────────────────────────────────────────────┘
```

---

## 4. Hierarquia de Botões e Controles

Para evitar a poluição visual de botões cinzas retangulares pesados, o VISOR define quatro classes de botões:

1. **Botão Primário (Accent Button):**
   * **Visual:** Preenchimento com `ACCENT_DEFAULT` e texto `TEXT_ON_ACCENT`.
   * **Uso restrito:** Ações afirmativas principais (ex: "Exportar", "Confirmar conversão").
   * **Regra:** Máximo de 1 botão primário por tela/diálogo para guiar a atenção do usuário.

2. **Botão Secundário (Elevated / Standard Button):**
   * **Visual:** Fundo `SURFACE_CARD`, borda `BORDER_DEFAULT`, cantos `RADIUS_SM`.
   * **Uso:** Ações secundárias em diálogos e barras fixas.

3. **Botão Sutil / Ghost (Subtle Toolbar Button):**
   * **Visual:** Totalmente transparente em repouso.
   * **Interação:** Preenchimento com `SURFACE_HOVER` ao passar o mouse; `SURFACE_ACTIVE` ao clicar.
   * **Uso principal:** Todos os botões da **Toolbar cápsula flutuante**. Esse estilo garante que a barra não tampe visualmente a foto enquanto o usuário apenas visualiza.

4. **Item de Menu / Lista (Flyout Item):**
   * **Visual:** Alinhamento à esquerda, padding horizontal de 10px, cantos arredondados de 4px, hover sutil.

---

## 5. Arquitetura de Implementação em Rust (`src/ui/theme.rs`)

O módulo centraliza as definições em estruturas tipadas consumíveis por todo o aplicativo:

* `ThemePalette`: Conjunto unificado de tokens de cores para Dark Mode e Light Mode.
* `ThemeSpacing`: Constantes de espaçamentos em pixels.
* `ThemeRounding`: Tokens de cantos arredondados (`egui::CornerRadius`).
* `configure_visor_theme(ctx, palette)`: Função que injeta a paleta e os estilos globais de widgets no `egui::Context`, garantindo consistência em todos os painéis e janelas sem necessidade de estilização manual repetitiva.
