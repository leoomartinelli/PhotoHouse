# Checkpoint da tela de carregamento aprovada

Este checkpoint preserva exatamente a tela do commit `005c3a9d`, aprovada na conversa:
Housinho colorido, fundo #242525, Roboto, contorno verde #BFFE00 de 1,5 ponto
recuado 2 pontos, cantos de 14 pontos, animação e contagem sincronizadas de 4,16s,
com o último quadro parado em 100% antes de abrir o editor.

Referência estável: `chore/checkpoint-tela-carregamento-2026-10-17`.
O checkpoint só acrescenta este guia ao código já salvo; não inclui as alterações
posteriores da tela inicial nem os arquivos que estavam em edição na interface.

## Recuperar só a abertura

Em uma branch de trabalho, examine ou extraia estes arquivos a partir da referência:

- `apps/photocraft/src/startup_splash.rs`: janela, layout, cronômetro e animação.
- `apps/photocraft/STARTUP.md`: comportamento e tipografia local.
- `docs/design/photohouse-loading/assets/housinho-frames.webp`: quadros da animação.
- `docs/design/photohouse-loading/assets/README.md`: origem dos assets.
- `docs/design/photohouse-loading/`: estudo visual e fontes de geração, se necessários.
- `ATTRIBUTION.md`: mantenha os créditos pertinentes aos assets recuperados.

Exemplo para recuperar apenas o módulo e seus quadros, sem restaurar o restante da interface:

```powershell
git restore --source=chore/checkpoint-tela-carregamento-2026-10-17 -- apps/photocraft/src/startup_splash.rs apps/photocraft/STARTUP.md docs/design/photohouse-loading/assets/housinho-frames.webp
```

Depois, reaplique somente os pontos de integração de `apps/photocraft/src/main.rs`
conforme esta referência: `mod startup_splash`, tratamento do processo filho no começo
de `main` e chamada de `before_editor()` antes de criar o editor. Não substitua todo
`main.rs` ao reaproveitar em outra versão. Confira também as APIs já usadas do crate
`ui-egui` (fontes), codecs e ícone da janela. Compile e teste após a integração.

Roboto é carregada de `fonts/Roboto-Regular.ttf` e `fonts/Roboto-Bold.ttf` ao lado
do executável. As fontes e sua licença são arquivos locais, fora do Git; sem elas,
a abertura usa a fonte padrão. O executável também não faz parte deste checkpoint.