# PhotoHouse

Editor de imagens da House de Criatividade, desenvolvido em Rust a partir do
[PhotoCraft](https://github.com/storytold/photocraft).
Nossa missão: **creative minds for the kingdom**.

Tema escuro com verde neon `#BFFE00`, mascote na tela inicial, instalador offline
para Windows e integração do efeito Liquid Glass com filtros inteligentes.
O projeto está em desenvolvimento; consulte as limitações em [docs/gaps.md](docs/gaps.md).

## Compilar e executar

Use Rust estável (mínimo 1.95). No Windows, instale Visual Studio Build Tools
com MSVC e Windows SDK. Para outros sistemas, consulte [docs/development.md](docs/development.md).

```sh
git clone https://github.com/HOUSE-DE-CRIATIVIDADE/PhotoHouse.git
cd PhotoHouse
cargo build --locked --release -p photocraft -p photocraft-cli
cargo run --locked -p photocraft
```

Os crates e binários mantêm os nomes internos `photocraft-*` para compatibilidade
com o upstream. Dependências Git estão fixadas por revisão; a primeira compilação precisa de rede.

## Ferramentas

Os nomes técnicos abaixo correspondem ao catálogo de ferramentas do editor.

<h4>🧰 67 tools</h4>
Move · Artboard · Rectangular and Elliptical Marquee · Single Row Marquee · Single Column Marquee · Lasso · Polygonal Lasso · Magnetic Lasso · Object Selection · Quick Selection · Magic Wand · Crop · Perspective Crop · Slice · Slice Select · Frame · Eyedropper · Color Sampler · Ruler · Note · Count · Remove · Spot Healing Brush · Healing Brush · Patch · Content-Aware Move · Red Eye · Brush · Pencil · Mixer Brush · Color Replacement · Clone Stamp · Pattern Stamp · History Brush · Eraser · Background Eraser · Magic Eraser · Gradient · Paint Bucket · Blur · Sharpen · Smudge · Dodge · Burn · Sponge · Pen · Freeform Pen · Curvature Pen · Add Anchor Point · Delete Anchor Point · Convert Point · Horizontal and Vertical Type · Horizontal and Vertical Type Mask · Path Selection · Direct Selection · Rectangle · Ellipse · Triangle · Polygon · Line · Custom Shape · Hand · Rotate View · Zoom

## Verificar

```sh
cargo fmt --all -- --check
cargo test --locked -p photocraft-algo -p photocraft-engine -p photocraft-ui-egui --lib --tests
cargo clippy --locked -p photocraft-algo -p photocraft-engine -p photocraft-ui-egui --all-targets -- -D warnings
cargo xtask layers
```

Mudanças compartilhadas com a Web exigem também `cargo xtask wasm`.
As verificações adicionais estão em [AGENTS.md](AGENTS.md).

## Instalador Windows

Consulte [packaging/windows/PHOTOHOUSE-INSTALLER.md](packaging/windows/PHOTOHOUSE-INSTALLER.md).
EXE e MSI são anexos de uma Release, fora do histórico do código.
O workflow de versão cria tag e Release; não gera instaladores.

## Contribuir

Siga [CONTRIBUTING.md](CONTRIBUTING.md): branch própria, PR para `main` e aprovação
do líder. Vulnerabilidades seguem [SECURITY.md](SECURITY.md).
Detalhes da publicação estão em [docs/publicacao-house.md](docs/publicacao-house.md).

## Origem e licenças

Baseado em PhotoCraft, por ArtCraft Team e seus contribuidores. PhotoHouse é uma
versão derivada mantida pela House de Criatividade. O histórico e os créditos de
origem são preservados. Documentos técnicos herdados podem mencionar PhotoCraft.

Código sob MIT ou Apache-2.0: [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)
e [NOTICE](NOTICE). Materiais de terceiros mantêm suas licenças em [ATTRIBUTION.md](ATTRIBUTION.md).
