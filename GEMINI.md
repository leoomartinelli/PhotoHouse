# Diretrizes do PhotoHouse

PhotoHouse é a versão da House de Criatividade derivada do
[PhotoCraft](https://github.com/storytold/photocraft), escrita em Rust.

Leia [AGENTS.md](AGENTS.md), [CONTRIBUTING.md](CONTRIBUTING.md) e
[SECURITY.md](SECURITY.md). Estas diretrizes complementam as regras do projeto.

## Personalizações que devem ser preservadas

- Marca PhotoHouse em títulos, menus, Sobre, diálogos e traduções; os nomes
  internos dos crates e binários continuam `photocraft-*`.
- Tema dark (`#141414` a `#282828`) com acento verde `#6FCF68`, contraste
  legível e uso dos tokens compartilhados da interface.
- Mascote da tela inicial, ícones da House e respectivas atribuições.
- Liquid Glass: algoritmo, comandos, prévia, posicionamento no canvas,
  composição do fundo e integração com Smart Objects e Smart Filters.
- Instalador offline Windows MSI/EXE e assistente com mascote animada.
- Catálogos i18n e compatibilidade com documentos existentes.

## Atualização do upstream

O procedimento versionado está em
[.agents/skills/atualizar-photohouse/SKILL.md](.agents/skills/atualizar-photohouse/SKILL.md).
Seu script de auditoria de marca está na mesma pasta, em `scripts/`.
Todos os caminhos partem da raiz deste repositório; nenhum depende de uma pasta
de Downloads ou de uma instalação local específica.

Quando o usuário pedir uma versão, fixe a integração na tag e registre seu SHA.
Para o planejamento da [issue #2](https://github.com/HOUSE-DE-CRIATIVIDADE/PhotoHouse/issues/2),
o alvo é `v0.9.0`. Não substitua esse alvo por `upstream/main`.

Respeite a etapa autorizada: um pedido de planejamento não inicia atualização.
Após alinhamento, trabalhe em branch própria, preserve o trabalho local e
resolva as divergências mantendo as novidades upstream e as personalizações.
Valide primeiro o editor e depois o instalador; apresente evidências antes de
afirmar que uma funcionalidade foi preservada.

Commits usam os prefixos da House. Abra PR para `main`, com revisão do líder;
merge, deploy e publicação de release dependem de OK explícito. Preserve os
créditos e licenças upstream. Nunca adicione segredos ao histórico.
