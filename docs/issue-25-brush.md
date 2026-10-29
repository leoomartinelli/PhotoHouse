# Issue #25 — implementação

Branch: `fix/25-brush-continuity`.

O renderizador de pintura agora usa a união do disco varrido entre amostras para pincéis redondos sólidos com diâmetro nominal >= 4 px e spacing <= 10%. O raio pode variar linearmente com a pressão. A distância ao volume varrido tem solução analítica; a cobertura usa união máxima, evitando que mais eventos do dispositivo escureçam a borda. Segmentos longos são divididos em trechos de até 64 px para limitar os retângulos rasterizados. Preview, cauda de smoothing e commit compartilham o mesmo caminho.

A API pública que fornece carimbos mantém seu espaçamento para ferramentas sequenciais. Pincéis minúsculos, suaves, texturizados, com scattering, dinâmica avançada, fluxo parcial, build-up ou spacing alto mantêm a rota anterior.

Ao concluir uma abertura interativa (incluindo background e prompts de perfil), e também uma abertura pela automação, a interface seleciona Move. Falhas de importação não alteram a ferramenta.

Validação visual: `cargo run --release -p photocraft-paint --example verify_issue25` gera `issue-25-brush.ppm` e mede uma imagem materializada de 6000×6000 (36 MP). Comparação usa spacing 0.100001 para forçar a rota anterior e 0.1 para a nova, com o mesmo percurso e raio. Medição inicial nesta máquina: 112 ms anterior / 188 ms contínuo para o percurso completo. Esses números incluem composição incremental e são uma amostra, não uma garantia de latência. O custo maior exige acompanhamento em hardware de menor desempenho.

Testes adicionados: cápsula geométrica de referência (20/100/300 px, horizontal/diagonal, posições subpixel); equivalência entre eventos densos/esparsos; pressão; fallback de efeitos; preview com cauda e limite de opacidade; ferramenta após abertura bem-sucedida/falha e abertura por automação.

## Verificações

- Paint: 107 testes passaram, incluindo preservação dos carimbos registrados pelas ferramentas sequenciais (Color Replacement).
- UI final: 2.694 testes passaram, 12 ignorados e nenhuma falha (`cargo test -p photocraft-ui-egui --lib --offline -- --test-threads=4`). Duas falhas de temporização na execução altamente paralela foram verificadas: os 22 testes de jobs passaram isoladamente, e a bateria completa passou com concorrência reduzida.
- Clippy nos dois crates, todos os targets, com `-D warnings`: passou.
- Build de `photocraft-ui-egui`: passou.
- Layers: 29 crates sem violações.
- Captura offscreen: arquivo aberto com Move selecionada e opções de seleção/movimentação visíveis; conferida visualmente.
- WebAssembly: todos os crates avaliados por `cargo xtask wasm` passaram (incluindo codecs/heif). Avisos de código não utilizado em arquivos não alterados foram mantidos.
- `cargo xtask perf --quick`: terminou com exit 0; os cinco benchmarks executaram. A comparação automática de regressão foi pulada por diferença de classe de máquina (baseline macOS/Apple M4 Pro, execução Windows/NVIDIA). O modo quick não aplica os orçamentos de documentos completos. Relatório local: `target/perf/results.json`.
