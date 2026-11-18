# VISOR — Linha de Base de Desempenho e Recursos (Performance Baseline)

**Data da Coleta:** 06/11/2026  
**Ambiente de Teste:** Windows (x86_64), Rust Toolchain `stable-x86_64-pc-windows-gnu`  
**Compilador:** `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1`

---

## 1. Métricas de Compilação e Binário

| Métrica | Valor Coletado | Observações |
| :--- | :--- | :--- |
| **Tamanho do Executável (Debug)** | `226.176.512 bytes` (~215,7 MB) | Contém símbolos completos de debuginfo e dependências de compilação. |
| **Tamanho do Executável (Release)** | `17.776.640 bytes` (~16,95 MiB) | Gerado com `lto = "thin"` e `strip = "symbols"`. |
| **Tempo de Execução `cargo check`** | `8,92 segundos` | Verificação completa de tipos e sintaxe com `--all-targets --all-features`. |
| **Tempo de Execução `cargo test`** | `1,21 segundos` | Execução dos 17 testes unitários e de integração existentes. |
| **Taxa de Sucesso dos Testes** | **100%** (17/17 testes aprovados) | 13 testes em `lib.rs`, 4 testes em `main.rs`. |

---

## 2. Linha de Base de Concorrência e Threads

### 2.1 Modelo Atual de Criação de Threads
No código auditado, o aplicativo utiliza criação ad-hoc de threads nativas do Windows sem pool ou limite superior:

```text
Ação do Usuário               Thread Criada                        Tempo de Vida
-------------------------------------------------------------------------------------
Navegar para foto N           std::thread::Builder ("visor-decode") Termina ao fim da decodificação
Miniatura do Filmstrip [0..6] std::thread::Builder ("visor-thumb")  Uma thread por miniatura visível
Exportar imagem única         std::thread::spawn                   Termina ao salvar arquivo
Exportar lote de arquivos     std::thread::spawn                   Processa todos os arquivos do lote
Registrar no Windows          std::thread::spawn                   Invoca PowerShell em background
```

### 2.2 Diagnóstico de Overhead
* **Comportamento em Navegação Rápida:** Ao manter pressionada a seta `→` em uma pasta com centenas de fotos, o loop principal dispara uma nova thread para cada arquivo selecionado, enquanto threads de miniaturas competem simultaneamente por I/O e CPU.
* **Falta de Cancelamento:** Uma thread de decodificação que iniciou o processamento de uma imagem pesada de 40 MB continuará alocando memória e processando pixels mesmo que o usuário já tenha avançado 10 fotos à frente.
* **Meta de Melhoria:** Substituir a criação contínua por um **Thread Pool Fixo** (tamanho = `num_cpus` limitado a no máximo 4 threads de worker) com fila de prioridade e cancelamento atômico imediato (`AtomicBool`).

---

## 3. Linha de Base de Memória e Limites Alocativos

O código estabelece limites defensivos em [`src/model.rs`](../src/model.rs#L119-L125) e [`src/raster.rs`](../src/raster.rs#L15-L21) para prevenir esgotamento de memória por arquivos maliciosos ou gigantescos:

| Limite / Constante | Valor no Código | Finalidade Técnica |
| :--- | :--- | :--- |
| `MAX_FILE` | `512 MB` (536.870.912 bytes) | Arquivos maiores são rejeitados de imediato antes da leitura. |
| `MAX_ALLOC` | `512 MB` | Teto de alocação de buffer de imagem para descompactadores. |
| `MAX_DIM` | `40.000 px` | Largura ou altura máxima admitida por imagem. |
| `MAX_PIXELS` | `80.000.000 pixels` (80 MP) | Rejeita imagens com área desproporcional. |
| `MAX_SIDE` (Downscale) | `8.192 px` | Lado máximo permitido para textura na GPU. Imagens maiores sofrem downscale para renderização, preservando o original para exportação. |
| `MAX_FRAMES` | `180 quadros` | Limite de animação para GIFs/APNGs longos a fim de evitar estouro de RAM. |

---

## 4. Eficiência de Renderização (egui / Glow)

* **Backend Gráfico:** `eframe::Renderer::Glow` (OpenGL leve).
* **Taxa de Atualização:** Orientada a eventos (*reactive UI*). O aplicativo só repinta a tela quando:
  1. O usuário interage (mouse, teclado, redimensionamento).
  2. Uma animação (GIF/APNG) solicita repintura após o delay do quadro (`request_repaint_after`).
  3. Uma thread de decodificação conclui e envia mensagem via canal (`mpsc`).
* **Taxa de Repouso:** 0% de uso de CPU quando o aplicativo está estático (sem animação e sem interação).
* **Filtro de Textura Atual:** Apenas `TextureOptions::LINEAR` (bilinear). A adição de suporte a `NEAREST` terá custo de processamento zero, pois ambas são funções nativas de amostragem na GPU.

---

## 5. Protocolo de Coleta e Metodologia para as Próximas Fases

Para validar os ganhos de desempenho nas fases subsequentes (Fase 1 a Fase 6), serão coletadas e comparadas as seguintes métricas padronizadas:

1. **Tempo de Inicialização a Frio (Cold Startup):**
   * Medição do tempo decorrido entre a execução do binário e a primeira renderização do frame.
   * *Meta:* < 150 ms em inicialização padrão.
2. **Latência de Troca de Imagem (Image Switch Latency):**
   * Medição do tempo entre o pressionamento de `←`/`→` e a exibição do frame:
     * Sem cache (leitura direta do disco).
     * Com o novo Cache LRU (memória RAM).
   * *Meta:* < 3 ms para imagens em cache; < 30 ms para imagens de 1080p em disco SSD.
3. **Pico de Threads Ativas:**
   * Contagem de threads do processo durante navegação rápida por 50 imagens consecutivas.
   * *Meta:* Manter contagem de threads constante no limite do pool (máx. 6 a 8 threads), sem picos descontrolados.
4. **Pegada de Memória (Resident Set Size - RSS):**
   * Uso de memória RAM após navegar por 100 imagens variadas (JPEGs, PNGs pesados, animações).
   * *Meta:* Consumo de memória estável com limpeza automática garantida pelo teto do Cache LRU.
