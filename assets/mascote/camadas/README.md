# Camadas do mascote (app nativo)

Recortes de `../photohouse-mascote-parado.svg`, sem CSS nem script, usados pelo widget egui
`crates/ui-egui/src/mascote.rs`:

- `corpo.svg`: o rolinho sem os olhos.
- `olho-esquerdo.svg` e `olho-direito.svg`: cada olho sozinho.

Todos usam as mesmas unidades do SVG original; o `viewBox` de cada um é a caixa da camada
(um teste confere que bate com o código). Se o desenho mudar, recorte de novo e atualize os
`viewBox` em `LAYERS`.
