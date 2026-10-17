# Publicação do PhotoHouse

Destino: https://github.com/HOUSE-DE-CRIATIVIDADE/PhotoHouse.
Regras: https://github.com/HOUSE-DE-CRIATIVIDADE/.github.

## Primeira publicação

O destino estava vazio em 17/10/2026. A preparação ocorre em
`chore/primeira-publicacao`, preservando o histórico upstream e as alterações
locais. Nenhum push foi feito nesta preparação.

Para abrir o primeiro PR é necessário criar uma base `main`. Propõe-se um commit
mínimo de documentação e sua integração à branch de importação para manter
ancestralidade comum e permitir revisão do código completo. Essa inicialização
excepcional depende de aprovação explícita. Reconsultar o destino antes do envio.

Não enviar tags upstream em lote: elas interferem no cálculo da primeira versão.
Auditar arquivos e histórico para segredos e tamanho antes de publicar.

## Revisão e automação

Siga CONTRIBUTING.md e o template de PR. Rode lint, testes e build antes de
commitar. O líder precisa aprovar o PR; merge exige OK explícito.

O CI usa Rust em Windows e Linux, sem depender de workflows reutilizáveis da
organização upstream. As receitas anteriores ficam em `docs/upstream-workflows/`
para referência e futuras adaptações. O CI mantém os testes de corpus externo
com revisões fixadas e verificação de hashes. HEIF opcional, empacotamento ARM64
e benchmarks noturnos não fazem parte deste CI inicial.

O workflow `versao.yml` é o template da House: cria tag e Release após merge,
conforme labels `versao:major`, `versao:minor`, `versao:patch`, `versao:nenhuma`
ou título do PR. Não altera Cargo.toml nem gera instaladores.

Para o PR de importação, recomenda-se `versao:nenhuma`: a versão Cargo atual é
0.8.2 e a base automática sem tags começaria em 0.0.0. O lançamento de uma versão
distribuível deve ser alinhado em uma etapa própria.

As versões Cargo, MSI e Release devem ser alinhadas antes de distribuir um
instalador. EXE/MSI ficam como anexos da Release, fora do histórico de código.
Siga `packaging/windows/PHOTOHOUSE-INSTALLER.md` para empacotar e validar.

## Licenças

Preserve LICENSE-MIT, LICENSE-APACHE, NOTICE e atribuições de terceiros. As marcas
ArtCraft foram retiradas de `docs/brand/`; sua licença permanece como referência
histórica. Créditos em texto ao PhotoCraft e seus autores permanecem.
