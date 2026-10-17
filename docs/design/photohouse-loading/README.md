# Tela de carregamento do PhotoHouse — estudo visual

Protótipo de design da issue #5. Esta pasta é a base de revisão da feature;
a integração ao editor nativo Rust/egui será feita depois do refinamento visual.
O HTML é apenas uma referência visual independente, sem dependências de build
ou alterações no aplicativo.

## Abrir o preview

Abra `index.html` no navegador ou sirva esta pasta com um servidor estático.
O percentual avança de 000 a 100 em aproximadamente 10 segundos. Use **Repetir
animação** para reiniciar. O progresso é simulado; ainda não mede a inicialização.
Com movimento reduzido ativado no sistema, exibe diretamente o estado concluído
e usa a ilustração estática no lugar do GIF.

## Direção atual

- Fundo preto e destaques verde neon `#BFFE00`, com luz ao redor da janela.
- Família Roboto; carregamento à esquerda e percentual ajustado à largura da barra.
- Housinho à direita, com a ilustração inteira e suas proporções preservadas.
- Janela de proporção 3:2, próxima do estudo inicial, com GIF vertical 490 × 566.
- Arte encostada no topo, lateral e base, sem margens ou moldura própria.
- Composição em coluna em janelas estreitas.
- Textos auxiliares, divisória central e rodapé do primeiro estudo removidos.

Roboto é solicitado ao Google Fonts apenas neste estudo e há fallback para Arial.
Nenhum arquivo de fonte é incluído. O GIF fornecido pelo solicitante foi incorporado
sem alterações: 52 quadros de 80 ms, duração de 4,16 segundos e repetição contínua.
A arte estática anterior foi mantida para a preferência de movimento reduzido.

## Próximos commits nesta PR

- Refinar e aprovar a composição.
- Refinar o encaixe e o ritmo do GIF com o solicitante.
- Implementar o visual em Rust/egui, sem incorporar este HTML ao aplicativo.
- Conectar o indicador às etapas reais da inicialização e validar desempenho,
  movimento reduzido e transição ao editor.

## Arte e referência

Referência de carregamento: <https://vitorkloy.vercel.app/>.
O screenshot do Photoshop fornecido pelo usuário orientou apenas a disposição;
nenhum asset ou código desse produto foi incluído.

Veja `assets/README.md` e o registro em `ATTRIBUTION.md` para a origem do Housinho.
