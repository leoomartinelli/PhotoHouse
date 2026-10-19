---
name: atualizar-photohouse
description: Sincroniza o PhotoHouse com uma versão do PhotoCraft upstream, preservando identidade House, tema verde PhotoHouse, mascote, Liquid Glass, traduções e instalador Windows. Use quando o usuário pedir atualização ou nova sincronização do PhotoHouse.
---

# Atualizar o PhotoHouse

Esta é a referência compartilhada da equipe para integrar novidades de
<https://github.com/storytold/photocraft> ao PhotoHouse.
Leia primeiro `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md` e `GEMINI.md` na raiz.
Os comandos abaixo são executados na raiz do clone, sem caminhos absolutos.

## 1. Confirmar escopo e inventariar a base

Respeite o escopo autorizado no chat. Planejar ou criar uma issue não autoriza
iniciar o merge. Para uma versão pedida, use a tag correspondente; na issue #2,
o alvo planejado é `v0.9.0`, não o estado mais recente de `main`.

```powershell
git status --short
git branch --show-current
git remote -v
git rev-parse HEAD
```

Verifique a base House aprovada e a situação de PRs anteriores. Use branch
própria conforme `CONTRIBUTING.md`, por exemplo `chore/update-photohouse-v0.9.0`.
Não misture alterações de terceiros ou arquivos locais num `git add -A` cego.
Se houver trabalho pendente, preserve-o por backup ou commit selecionado,
de acordo com a autorização e as verificações exigidas pelo projeto.
Registre o SHA inicial e crie uma tag local de salvaguarda antes da integração:

```powershell
git tag "photohouse-pre-update-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
```

Inventarie também personalizações além das listadas aqui. Guarde exemplos
reproduzíveis e capturas da versão atual para comparação.

## 2. Buscar e comparar a versão exata

Confira que o remoto `upstream` aponta para o PhotoCraft oficial. Para v0.9.0:

```powershell
git fetch upstream tag v0.9.0
git rev-parse 'v0.9.0^{commit}'
git merge-base HEAD v0.9.0
git log --oneline HEAD..v0.9.0
git diff --stat HEAD...v0.9.0
```

Registre o SHA completo, o ancestral comum e as mudanças relevantes. Diferencie
novidades já incorporadas, mudanças novas e conflitos confirmados. Uma área
alterada pelos dois projetos é um risco a revisar, não prova de conflito.
Falhas de rede ou referência devem ser resolvidas antes de afirmar que a
comparação está completa. Não prossiga para uma referência diferente em silêncio.

## 3. Integrar na branch de trabalho

Quando a implementação estiver autorizada, inicie a integração sem criar o
commit automaticamente, permitindo revisar e validar o resultado:

```powershell
git merge --no-ff --no-commit v0.9.0
```

Pare para inspecionar o resultado e eventuais conflitos. Preserve a lógica nova
do upstream e adapte-a aos padrões House; não aceite arquivos inteiros com
`ours` ou `theirs` sem revisar as duas versões.

### Matriz de preservação

| Área | Referências e comportamento a preservar |
| --- | --- |
| Marca | `apps/photocraft`, `brand.rs`, títulos, menus, Sobre, ícones e mascote em `assets/` |
| Tema | `crates/ui-egui/src/theme.rs` e `widgets.rs`; dark #141414–#282828, acento #6FCF68 e texto/ícones legíveis |
| Liquid Glass | `crates/algo/src/liquid_glass.rs` e registro em `lib.rs`; comandos em `filters.rs`, `filters_ext.rs`, `smart_cmds.rs` e menus |
| Edição do efeito | `liquid_glass_ui.rs`, `filter_dialog.rs`, `filter_preview_ui.rs` e canvas; aplicar, editar, mover, cancelar, prévia e Undo/Redo |
| Documento | Smart Objects/Filters, silhueta vetorial, camadas inferiores, atualização do fundo e persistência das propriedades suportadas |
| Traduções | `crates/ui-egui/src/i18n/*.tsv`; chaves, cobertura e nome PhotoHouse coerentes com o código |
| Windows | `packaging/windows/`, MSI/EXE offline, assistente animado, ícones, versão e metadados |
| Governança | Políticas, templates e CI da House; créditos e licenças de origem |

Para traduções, revise diferenças de chaves e traduções antes de usar:

```powershell
.\packaging\windows\photohouse-rebrand-i18n.ps1
```

O parâmetro `-FromTheirs` desse script substitui catálogos em conflito pela
versão upstream antes do rebrand. Use-o apenas após conferir que não elimina
traduções exclusivas da House; reaplique as entradas necessárias e valide-as.

Execute a auditoria versionada:

```powershell
& .\.agents\skills\atualizar-photohouse\scripts\audit-rebrand.ps1
git diff --check
git diff --name-only --diff-filter=U
```

A auditoria produz sugestões `RENAME`/`CHECK` para revisão humana; não altera
arquivos e não certifica a completude do rebrand. Mantenha referências técnicas,
créditos e licenças que legitimamente mencionem PhotoCraft.

## 4. Validar primeiro o editor

Execute as verificações exigidas por `AGENTS.md` para as áreas alteradas:

```powershell
cargo fmt --all -- --check
cargo test --locked -p photocraft-algo -p photocraft-engine -p photocraft-ui-egui --lib --tests
cargo clippy --locked -p photocraft-algo -p photocraft-engine -p photocraft-ui-egui --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm
cargo test --locked -p photocraft-engine --test panic_hunt -- --ignored
cargo build --locked --release -p photocraft -p photocraft-cli
```

Amplie para crates e corpora afetados e execute as verificações condicionais de
paridade e desempenho previstas em `AGENTS.md`. Compare o Liquid Glass antes e
depois: prévia/aplicação, parâmetros, seleção, silhueta, fundo, movimento,
Smart Filters, cancelar, Undo/Redo e salvar/reabrir `.pcraft` antigo e novo.
Verifique profundidades/modelos de cor suportados, erros de entrada e desempenho
nos caminhos afetados. Não prometa suporte de exportação de efeitos que não foi
implementado e testado.

Faça comparação visual por snapshot offscreen ou pelo protocolo de controle,
inspecionando os PNGs conforme `docs/development.md` e `docs/control-protocol.md`.
Adapte controles novos aos tokens House. Discuta previamente mudanças relevantes
de UX que alterem o fluxo existente.

Registre comandos, resultados e limitações; diferencie falhas preexistentes.
Somente após as verificações selecione os arquivos e finalize o commit com
prefixo House, por exemplo `CHORE: integra PhotoCraft v0.9.0 no PhotoHouse`.
Não finalize um merge sem checar se ele ainda está em andamento.

## 5. Validar o instalador e revisar

Após aprovação do editor candidato, siga
`packaging/windows/PHOTOHOUSE-INSTALLER.md` e execute, quando autorizado:

```powershell
& .\packaging\windows\build-photohouse.ps1
```

Confira nomes, versão, instalação, abertura e funcionamento offline do pacote.
Mantenha os executáveis anteriores até a validação; não copie automaticamente
para caminhos externos ao clone nem sobrescreva a distribuição atual.

Apresente o diff, as novidades, as adaptações House, os conflitos resolvidos,
as capturas e os resultados. Abra PR para `main` com vínculo à issue e solicite
revisão do líder. Merge na `main`, deploy e publicação de release exigem OK
explícito. EXE/MSI pertencem aos anexos de uma Release, não ao Git.
