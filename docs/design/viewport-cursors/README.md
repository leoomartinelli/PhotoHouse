# Validação local — cursores da viewport (#23)

`difference-preview.png` é gerada pelo teste GPU
`viewport_cursor::tests::gpu_difference_inverts_background_and_overlays`.

Colunas: preto, branco, azul. Linhas: padrão, mover, duplicar com Alt.
A linha vertical vermelha representa um elemento da interface sobre o fundo;
onde o cursor a cobre, ela deve ficar ciano. Isso verifica composição sobre
o framebuffer final, sem amostragem do documento ou alterações destrutivas.

Os SVGs e suas atribuições estão em `assets/cursors/`.
