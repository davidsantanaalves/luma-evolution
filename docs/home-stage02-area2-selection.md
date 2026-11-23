# Etapa 02 — seleção unificada da Área 2

O checkpoint de 22/11/2026 sobre seleção unificada e dica de abertura foi conferido
nos dois originais da fonte vinculada ao projeto. A seleção foi refinada no preview
nativo desktop. A aprovação visual global da Etapa 02 continua pendente.
[UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view) ·
[PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)

## Implementado

- Na Área 2 expandida, a moldura do calendário abrange hora, saudação, data, mês e
  grade mensal. As duas molduras medem 834 × 514 px lógicos nas cenas e compartilham
  o alinhamento vertical. Cada coluna tem um único alvo. O resumo do relógio na
  Home recolhida continua informativo.
- A frase “Pressione OK para abrir o aplicativo correspondente” usa a localização
  do launcher e aparece no rodapé do único bloco em foco após a expansão. A dica
  tem 18 px lógicos, sombra de texto e margem lateral interna. No calendário,
  alinha à esquerda. No clima, alinha à direita. O bloco permanece o único alvo
  e a dica aproveita o espaço já existente na moldura.
- A disponibilidade do app de clima continua condicionando a seleção. A roda
  mantém a possibilidade de expandir a área sem seleção. As setas e o ponteiro
  selecionam o bloco, e o OK ou clique explícito usa a abertura simulada existente.

| Arquivo                          | Alteração                                                                        |
| -------------------------------- | -------------------------------------------------------------------------------- |
| `src/home-evolution-view.rs`     | Alvo esquerdo unificado, molduras equivalentes e dica contextual                 |
| `assets/messages.json`           | Tradução da dica para os 14 idiomas já suportados                                |
| `src/home-evolution-tests.rs`    | Hit testing da seleção unificada e preservação do relógio informativo em repouso |
| `tests/home-evolution-smoke.mjs` | Cenários SDL de seleção, interações, visibilidade, encaixe e novas capturas      |
| `agents.md` e `development.md`   | Regra operacional da fonte canônica e fluxo Windows → WSL/ext4                   |

## Testes

| Comando                                                            | Resultado                                                                                                     |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| `cargo build --locked -p luma --no-default-features`               | Build desktop concluído no espelho ext4.                                                                      |
| `pnpm format:check` e `pnpm check`                                 | Formatação, versões, Oxlint e Clippy passaram.                                                                |
| `cargo test --locked -p luma --no-default-features home_evolution` | 35 testes da Home passaram.                                                                                   |
| `pnpm test`                                                        | 702 testes do launcher, 67 do helper e 43 do Node passaram. O teste que consulta a Apple permaneceu ignorado. |
| `xvfb-run -a pnpm test:home-preview --area2-selection`             | 16 cenários gráficos da Área 2 passaram.                                                                      |
| `git diff --check`                                                 | Passou.                                                                                                       |

O smoke reutilizou o binário desktop com `LUMA_SMOKE_SKIP_BUILD=1`. Verificou a mesma
geometria ao alternar entre calendário e clima, um alvo por bloco, posição e
visibilidade da dica, navegação, hover, roda, Voltar, OK e clique. Os seis PNGs
principais foram inspecionados em suas resoluções. Os diagnósticos registraram zero
imagens ou glifos ausentes. Os casos exercitados passaram sem regressões observadas.
[Log do smoke nativo](../.build/home-stage02-area2-selection/focused.log)

## Evidências

As capturas do framebuffer nativo têm diagnósticos do renderer e dumps de estado em
`.build/home-stage02-area2-selection/`. As seis cenas principais usam o mesmo
relógio fixo, a mesma composição e dados meteorológicos simulados.

| Cena                                                                | 1920×1080                                                                       | 1280×720                                                                       |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Calendário em foco, com o resumo na moldura                         | [PNG](../.build/home-stage02-area2-selection/1080p-01-calendario-unificado.png) | [PNG](../.build/home-stage02-area2-selection/720p-01-calendario-unificado.png) |
| Clima em foco, com a dica à direita                                 | [PNG](../.build/home-stage02-area2-selection/1080p-02-clima.png)                | [PNG](../.build/home-stage02-area2-selection/720p-02-clima.png)                |
| Área 2 expandida sem app de clima, com app de calendário disponível | [PNG](../.build/home-stage02-area2-selection/1080p-03-sem-app-clima.png)        | [PNG](../.build/home-stage02-area2-selection/720p-03-sem-app-clima.png)        |

Os comparativos mostram as três cenas lado a lado. Agosto de 2026 verifica o espaço
entre a última linha de um mês de seis semanas e a dica.
[Comparativo 1080p](../.build/home-stage02-area2-selection/comparativo-1080p.png) ·
[Comparativo 720p](../.build/home-stage02-area2-selection/comparativo-720p.png) ·
[Calendário de seis semanas](../.build/home-stage02-area2-selection/720p-calendario-seis-semanas.png)

## Documentação canônica

O [guia operacional](../agents.md#canonical-project-source) registra as URLs e os IDs
da pasta `LGTV`, da subpasta de documentação e dos dois arquivos originais.
O [guia de desenvolvimento](../development.md#canonical-project-source) aponta para
essa cadeia e define a consulta aos originais atualizados antes de implementar ou
alterar decisões. A rodada consultou o Drive em leitura e preservou o checkpoint
existente.

## Pendências

As molduras e a dica aguardam revisão visual em 1080p e 720p. A associação com os
apps reais de calendário e clima e o comportamento do Magic Remote físico seguem
sem validação nesta rodada. A divergência preexistente da escala do Dock em relação
a B24/B25 está registrada no [relatório anterior](home-stage02-post-r12.md).

## Impacto

A execução ficou no preview desktop da Etapa 02. As capturas e relatórios anteriores
continuam em seus caminhos originais. O novo relatório é o único arquivo adicionado
fora de `.build/`.

`develop` segue dois commits locais à frente de `origin/develop`. O diff agregado
dos arquivos rastreados soma 12 arquivos, 981 inserções e 37 remoções, incluindo
o trabalho anterior. Os módulos da Home e o script gráfico já estavam sem rastreamento
no início e foram editados em seus caminhos existentes.
[Git status](../.build/home-stage02-area2-selection/git-status.txt) ·
[Git diff --stat](../.build/home-stage02-area2-selection/git-diff-stat.txt)
