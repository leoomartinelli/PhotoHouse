# PhotoHouse com base PhotoCraft v0.18.0

Alvo desta sincronização: tag upstream `v0.18.0`, commit
`8a87a457abb43ac344deeb3fdb67541b91a72ff1`, publicada em 31/10/2026.
Não foi usado o estado móvel de `upstream/main`.

## Integração

A branch `chore/update-photohouse-v0.18.0` parte do estado local com os
cursores da issue #23 (PR #28), sincroniza a `main` da House e incorpora
5.788 commits upstream desde a base comum `99e1138877e6aa7e856c6883b415e708d042fc59`.
A tag local `photohouse-pre-update-v0.18.0-20261031` preserva o estado anterior.

As extrações upstream de módulos foram mantidas. Os pontos de integração do
Liquid Glass agora estão em `crates/engine/src/cmd/smart_cmds/`,
`crates/ui-egui/src/dialogs/`, `canvas/` e `panels/`, conforme a nova estrutura.
Os comandos `filter.distort.liquidGlass` e `layer.smartObjects.liquidGlass`
permanecem disponíveis, incluindo edição de filtro inteligente e movimentação
da lente sem alterar a fonte até a confirmação.

Foram preservados o tema #6FCF68, contraste dos controles, ícones, mascote e
abertura animados, três cursores SVG com Difference e fallback sem GPU,
catálogos de tradução e instalador offline Windows. Os catálogos usam as
versões upstream novas com as entradas exclusivas House reaplicadas.

O atualizador automático novo usa o repositório da House, evitando substituir
o derivado por uma distribuição upstream. Os workflows House foram mantidos;
os novos workflows e configuração de dependências upstream ficaram arquivados
em `docs/upstream-workflows/` para revisão, sem ativação automática.

A versão do empacotamento Windows passou a seguir o `Cargo.toml` do workspace;
o rodapé do assistente segue a versão do próprio pacote, sem literal 0.8.2.
Executáveis e instaladores gerados ficam fora do histórico Git.

## Ajustes de compatibilidade encontrados na validação

- Arredondamento de Oil Paint e conversão para 8 bits: o alvo Windows GNU usa
  o comportamento de `f32::round`, preservando os testes de referência nos
  valores imediatamente abaixo de meio inteiro.
- Camera Raw XMP: os novos sidecars identificam PhotoHouse; os sidecars antigos
  PhotoCraft continuam reconhecidos. A proteção de sidecars externos permanece.
- Shader de composição: o retorno padrão de `blend_channel` ficou após o
  `switch`. O FXC deste Windows recusava o HLSL gerado com retorno apenas no
  `default`, desativando a composição GPU. Os modos de mesclagem permanecem iguais.
- Testes visuais passaram a conferir os cursores próprios e o fundo pontilhado
  House. Um teste de fonte recebeu até 16 quadros para estabilizar a prévia
  (precisou de 11 na execução concorrente, acima do limite antigo de 8).
- Na revisão do PR #28 foi identificado que a seta House interceptava todos os
  estados da ferramenta Mover. A integração restringe esse desenho a `Copy`,
  `Piece` e hover sem seleção, preservando os indicadores de recorte, arrasto e
  movimento do contorno. Há teste da transição de recorte para seleção flutuante.

O build nativo inclui as 13 faces do manifesto padrão de `storytold/craft-fonts`
no commit `aced4137c022f1f587624d48e81ec77e1914a9cc`. Os SHA256 foram conferidos
contra o manifesto; OFL e atribuições acompanham o pacote. As fontes continuam
fora deste repositório. O alvo WebAssembly mantém o carregamento externo upstream.

## Validação

- Testes nativos: **8.338 aprovados, 66 ignorados**, sem falhas finais:
  aplicativo 200, algoritmos 762, motor 2.461, I/O 437 e interface 4.478.
  Incluem persistência editável de Liquid Glass em `.pcraft`, os cursores
  Difference sobre imagem/overlays e os estados da seleção.
- `panic_hunt`: aprovado com entradas adversárias para os comandos.
- Clippy nos pacotes alterados e em todos os seus alvos: aprovado com
  `-D warnings`; lint adicional do crate GPU também aprovado.
- Arquitetura: 31 crates sem violações. WebAssembly: 24 crates e codecs com
  HEIF aprovados. Formatação e whitespace do diff aprovados.
- Shader validado pelo Naga e paridade real de modos de mesclagem aprovada
  em Vulkan e DirectX 12 na GPU Intel. O teste DX12 exigiu o backend, sem
  permitir ignorar a ausência de adaptador/compositor.
- Paridade de ligação dos menus: **633/633**. Scorecard regenerado.
- Documento Liquid Glass criado pela versão 0.8.2 aberto na interface e
  convertido: pixels RGBA idênticos antes/depois, em 96×64 e 6.000×4.000.
- Teste release em 24 MP (lente 1.600×800): criar, aplicar o efeito e salvar
  `.pcraft` levou 2.083 ms antes e 644 ms depois, com pixels idênticos.
  Rodada única com compilação concorrente; não constitui baseline de desempenho.
- Mascote inicial e abertura animada conferidos por capturas nativas; a abertura
  mostra 0.18.0. Assets House comparados com a base, sem alterações de conteúdo.
- Assistente offline: três testes aprovados, incluindo GIF e payload MSI.
- Distribuição final: editor e CLI compilados em release, recursos Windows com
  nome PhotoHouse e versão 0.18.0. O executável nativo abriu o documento antigo
  na GPU, exibiu a prévia do Liquid Glass e cancelou sem sujar o documento.
- MSI validado pelo WiX e extraído administrativamente. Hashes de editor/CLI
  extraídos idênticos aos do build final; 14 licenças OFL e atribuições presentes.
  O MSI embutido no EXE tem o mesmo SHA256 do MSI validado. A prévia real do
  assistente foi capturada e conferida com o rodapé 0.18.0, sem instalar.

Artefatos offline na pasta externa `dist/2026-10-31/`:
`PhotoHouse-Setup-0.18.0-x64.exe`, `PhotoHouse-0.18.0-x64.msi` e `SHA256SUMS.txt`.
A instalação/upgrade sobre a instalação pessoal não foi executada; a validação
do pacote usou extração e prévia. Os executáveis permanecem sem assinatura
Authenticode, conforme a distribuição House anterior.

As evidências locais ficam fora do Git em `analysis/update-v0.18.0/`, na pasta
externa que contém este clone; os logs de execução ficam em `log/`.
