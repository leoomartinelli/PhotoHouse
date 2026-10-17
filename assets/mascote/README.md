# Mascote animado do PhotoHouse

O rolinho de pintura com olhos, o mesmo desenho do ícone do app, em SVGs animados (CSS + um pequeno script).

## Animações

| Arquivo | O que faz | Prévia |
|---|---|---|
| `photohouse-mascote-parado.svg` | Flutua, pisca e olha em volta | `preview-parado.gif` |
| `photohouse-mascote-parado-com-fundo.svg` | O mesmo, com o quadrado verde-claro do ícone atrás | `preview-parado-com-fundo.gif` |
| `photohouse-mascote-digitando.svg` | Flutua, pisca e olha pra baixo, como se digitasse | — |
| `photohouse-mascote-pensando.svg` | Flutua, pisca e dá pulinhos | `preview-pulinho.gif` |
| `photohouse-mascote-olhinho-fino.svg` | Aperta os olhos (olhinho fino) | `preview-olhinho-fino.gif` |
| `photohouse-mascote-olhinho-fino-horizontal.svg` | Olhinho fino na horizontal | `preview-olhinho-fino-horizontal.gif` |
| `photohouse-mascote-rodando.svg` | Gira no próprio eixo 5 vezes e fica tonto | `preview-rodando.gif` |

`teste-mouse.html` abre todos lado a lado para testar.

## Como usar

- Os olhos seguem o mouse. Isso só funciona com o SVG em `<object>` ou inline no HTML. Em `<img>` o script não roda e o mascote não acompanha o mouse.
- No app (egui) os SVGs não rodam como estão: as animações precisam ser reimplementadas em Rust, desenhando e animando as formas ou com texturas pequenas por quadro, dentro do `max_texture_side` da GPU. Esses arquivos são a referência (é a base da tela de abertura do #8).
