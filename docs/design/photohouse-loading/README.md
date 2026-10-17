# Tela de carregamento do PhotoHouse — estudo visual

Estudo de design da issue #5. Esta pasta continua como referência de revisão.
A versão escura também está integrada experimentalmente à abertura nativa do
desktop: veja `apps/photocraft/STARTUP.md`. O HTML é uma referência visual
independente e não é executado ou incorporado ao editor.

## Abrir o preview

Abra `index.html` no navegador ou sirva esta pasta com um servidor estático.
O percentual e os quadros do Housinho compartilham uma reprodução de 4,16 segundos.
000% mostra o primeiro quadro; 100% mostra o último quadro e interrompe a reprodução.
Use **Repetir animação** para reiniciar ambos. O progresso ainda é demonstrativo;
não mede a inicialização do editor. Com movimento reduzido, mostra diretamente
100% e o quadro final, sem animar.

Compare as versões **Preto** e **Verde claro** pelos botões abaixo da janela.
As duas usam a mesma linha de tempo; alternar a versão mantém o progresso atual.

## Direção atual

- Duas versões: preto com texto claro e verde claro `#DDFDE6` com texto preto.
- No modo claro, a escrita antes neon e a barra usam `#1C7E67`.
- A luz e a borda externas continuam verde neon `#BFFE00` nas duas versões.
- Rodapé em ambas: **The House** Studio, com apenas “The House” em negrito,
  seguido de “Versão 0.8.2”, conforme `workspace.package.version` em `Cargo.toml`.
- Família Roboto; carregamento à esquerda e percentual ajustado à largura da barra.
- Housinho à direita, com a ilustração inteira e suas proporções preservadas.
- Janela de proporção 3:2, próxima do estudo inicial, com GIF vertical 720 × 832.
- Arte encostada no topo, lateral e base, sem margens ou moldura própria.
- Composição em coluna em janelas estreitas.
- Textos auxiliares, divisória central e rodapé do primeiro estudo removidos.

Roboto é solicitado ao Google Fonts apenas neste estudo e há fallback para Arial.
Nenhum arquivo de fonte é incluído. O GIF foi recriado do vídeo **ANIMAÇÃO.mp4**
fornecido pelo solicitante (912 × 1054, aproximadamente 4,17 segundos).
A conversão usa 720 × 832, 12,5 quadros/s, redimensionamento Lanczos e paleta
otimizada de 256 cores. O GIF ocupa aproximadamente 6,46 MiB. O vídeo original
não é modificado nem incluído no repositório.

Para evitar quantização repetida e controlar a parada, os quadros de exibição
também são gerados diretamente do MP4, em cores completas. O preview desenha os 52 quadros de
`assets/housinho-frames.webp` em canvas, usando o mesmo progresso da contagem.
O sprite contém oito colunas e sete linhas de 720 × 832; as últimas quatro células ficam vazias.
Usa WebP com qualidade 88 e ocupa aproximadamente 3,67 MiB.
O GIF em loop não é mais reproduzido diretamente. A arte estática anterior segue
versionada como referência.

## Verificação e geração dos quadros

Execute `node docs/design/photohouse-loading/check.cjs` na raiz do repositório.
O teste verifica as duas versões em 0%, 50%, 99% e 100%, a ausência de loop ao
concluir, o reinício, movimento reduzido e falha ao carregar os quadros.

Para regenerar os assets: `python docs/design/photohouse-loading/build_frames.py "caminho/ANIMAÇÃO.mp4"`.
Requer Pillow e FFmpeg (ou `imageio-ffmpeg`) apenas como ferramentas de desenvolvimento;
sem dependência do editor. No preview da conversa há uma cópia do sprite em 360 × 416
por quadro, para caber no limite de tamanho; o preview do repositório usa 720 × 832.

## Próximos commits nesta PR

- Refinar e aprovar a composição.
- Refinar o encaixe e o ritmo do GIF com o solicitante.
- Testar e refinar a abertura nativa Rust/egui, incluindo a transição ao editor.
- Rever medição das etapas reais da inicialização, movimento reduzido e empacotamento.
  A versão local solicitada reproduz a animação até 100% antes de abrir o editor.

## Arte e referência

Referência de carregamento: <https://vitorkloy.vercel.app/>.
O screenshot do Photoshop fornecido pelo usuário orientou apenas a disposição;
nenhum asset ou código desse produto foi incluído.

Veja `assets/README.md` e o registro em `ATTRIBUTION.md` para a origem do Housinho.
