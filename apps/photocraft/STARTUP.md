# Abertura local do PhotoHouse

O aplicativo desktop agora executa uma janela nativa de abertura antes de criar
a janela do editor. O mesmo executável inicia um processo curto com a opção
interna `--photohouse-startup-splash`; o processo principal espera sua conclusão.
Assim, o tamanho, a posição e a persistência do editor não recebem a geometria
da abertura. Não há navegador, HTML ou dependência nova em tempo de execução.

A janela usa o modo escuro aprovado: fundo `#242525`, acentos `#BFFE00`,
proporção 3:2, assinatura e versão obtida de `CARGO_PKG_VERSION`. Uma textura
compartilhada com o estudo visual fornece os 52 quadros de alta resolução.
O relógio de 4,16 segundos controla percentual e quadro; ao atingir 100%, o
quadro final permanece parado por 200 ms, a abertura fecha e o editor inicia.
Este percentual descreve a animação de abertura, não a medição das tarefas do editor.

A janela é opaca, com contorno verde `#BFFE00` de 1,5 ponto, recuado 2 pontos
para dentro e com cantos de 14 pontos, evitando o recorte externo do Windows.
Não há brilho ou margem externa. O conteúdo ocupa
toda a área de 720 × 480 pontos, com fundo escuro até as bordas e a arte à direita.

Esc ou fechar a abertura cancela o lançamento inteiro. Falha ao criar a janela
de abertura é registrada e permite abrir o editor para recuperação.

## Tipografia local

Sem adicionar fontes binárias ao Git, a abertura procura `fonts/Roboto-Regular.ttf`
e `fonts/Roboto-Bold.ttf` ao lado do executável. Na ausência delas, usa a família
padrão da interface. O teste local inclui Roboto e sua licença nessa pasta.
Fonte: <https://github.com/googlefonts/roboto-2>, Apache-2.0.
O empacotamento definitivo dessas fontes será tratado antes da distribuição.

## Verificação

- `cargo test -p photocraft --bin photocraft startup_splash`
- `cargo clippy -p photocraft --bin photocraft -- -D warnings`
- `cargo build --release -p photocraft`

Para conferir só a abertura: `PhotoHouse.exe --photohouse-startup-splash`.
Para capturas de diagnóstico da própria janela, defina
`PHOTOHOUSE_SPLASH_CAPTURE_DIR` para uma pasta de teste antes de iniciar:
ela grava `splash-000.png`, `splash-050.png` e `splash-100.png` via eframe.
Sem essa variável, nenhuma captura é feita.
