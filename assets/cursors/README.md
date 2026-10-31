# Cursores PhotoHouse — issue #23

SVGs originais fornecidos pelo autor da solicitação para esta implementação:

- `cursor_padrão.svg`: ponteiro padrão na viewport.
- `cursor_move.svg`: ferramenta Move.
- `cursor_copy_alt.svg`: duplicação com Alt.

Os arquivos são preservados sem alterações. Aplicam-se as licenças de contribuição
do projeto, MIT OR Apache-2.0, reproduzidas nesta pasta.

A renderização usa o canal alfa como máscara branca em Difference: o interior
inverte as cores já desenhadas, incluindo guias, seleção e controles. O cinza
original do SVG não é usado como tinta opaca. Isso mantém o contorno do desenho
e permite a inversão de contraste solicitada.

O tamanho base da seta é 26 pontos, independente do zoom do documento; as três
pranchetas usam a mesma escala e hotspots individuais para alinhar suas pontas.
As máscaras são rasterizadas e enviadas uma vez ao inicializar o renderer.
O cursor acompanha os frames da interface, pois a composição Difference precisa
do framebuffer. Cursores de resize, rotação, texto e outras ferramentas continuam
com sua função específica. Fora da viewport o cursor do sistema é restaurado.

Difference funciona também com composição de documento na CPU quando a janela
usa wgpu. Hosts sem renderer wgpu usam os SVGs com composição normal.

Validação: `cargo test -p photocraft-ui-egui --lib viewport_cursor`.
