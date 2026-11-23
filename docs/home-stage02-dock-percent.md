# Dock pós-R13 — exibição de 100%

Correção pontual autorizada em 23/11/2026 no checkout Windows da branch `develop`, commit-base
`806568a`. O defeito foi registrado nas capturas de 100% do [relatório da R13](home-stage02-r13-aceite.md).

Confirmei a cadeia da pasta canônica `LGTV/gpt-context-lg-luma-evolution` e consultei os originais.
O checkpoint de 23/11/2026, opção D, fixa a apresentação de 50% da referência cinematográfica
com a Área 2 expandida, inclusive para um perfil compacto de 40%. Consultei também
[agents.md](../agents.md) e [development.md](../development.md).
[UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view) ·
[PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)

## Correção

O campo do Dock tinha 60 px lógicos. Com a fonte existente de 24 px, as métricas do projeto
indicam que `100%` ocupa 65,33 px. Em `scale_settings`, a caixa do percentual do Dock passa a
usar os 66 px entre as duas setas. Na cena da configuração, a caixa parte de x=895 e termina
em x=961. A altura de 29 px e a fonte foram mantidas.

O diff funcional contém somente essa alteração em `src/view.rs`. A geometria de Cinema, os
alvos das setas e as regras de 40%–100% em passos de 5 pontos percentuais seguem as mesmas.
Este relatório é o único arquivo novo fora de `.build/`.
[Conferência SHA-256 dos fontes anteriores e das seis capturas do Dock da R13](../.build/home-stage02-dock-percent/integridade.txt)

## Evidências

Os dois binários foram compilados no espelho WSL/ext4 a partir do checkout Windows, antes e
depois da correção. As capturas usam o preview SDL2/OpenGL ES 2.0 em Xvfb/Mesa llvmpipe,
relógio fixo, conteúdo de preview e arquivos isolados de preferências. As pranchas mostram
capturas do mesmo roteiro e os detalhes ampliam o campo de 100% dos PNGs originais.

| Evidência                                      | 1920×1080                                                                    | 1280×720                                                                    |
| ---------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Prancha antes/depois de 40%, 45% e 100%        | [PNG](../.build/home-stage02-dock-percent/comparativo-1080p.png)             | [PNG](../.build/home-stage02-dock-percent/comparativo-720p.png)             |
| Detalhe ampliado de 100%                       | [PNG](../.build/home-stage02-dock-percent/detalhe-100-1080p.png)             | [PNG](../.build/home-stage02-dock-percent/detalhe-100-720p.png)             |
| 40% após a correção                            | [PNG](../.build/home-stage02-dock-percent/1080p-depois-40.png)               | [PNG](../.build/home-stage02-dock-percent/720p-depois-40.png)               |
| 45% após a correção                            | [PNG](../.build/home-stage02-dock-percent/1080p-depois-45.png)               | [PNG](../.build/home-stage02-dock-percent/720p-depois-45.png)               |
| 100% após a correção                           | [PNG](../.build/home-stage02-dock-percent/1080p-depois-100.png)              | [PNG](../.build/home-stage02-dock-percent/720p-depois-100.png)              |
| Clique na seta direita avança a 45%            | [PNG](../.build/home-stage02-dock-percent/1080p-depois-ponteiro-45.png)      | [PNG](../.build/home-stage02-dock-percent/720p-depois-ponteiro-45.png)      |
| Direcional seleciona Aplicar com 45% na prévia | [PNG](../.build/home-stage02-dock-percent/1080p-depois-selecao-aplicar.png)  | [PNG](../.build/home-stage02-dock-percent/720p-depois-selecao-aplicar.png)  |
| Clique na seta esquerda retorna de 100% a 95%  | [PNG](../.build/home-stage02-dock-percent/1080p-depois-seta-esquerda-95.png) | [PNG](../.build/home-stage02-dock-percent/720p-depois-seta-esquerda-95.png) |

[Diferenças de pixels](../.build/home-stage02-dock-percent/comparacao-pixels.json) ·
[Captura da Área 2 expandida com 40% salvo](../.build/home-stage02-dock-percent/720p-cinema-area2-expandida.png)

O símbolo `%` aparece inteiro em 100% e fica separado das setas nas duas resoluções. O comparativo
pixel a pixel encontrou zero diferenças fora do campo nos três valores. A geometria dos alvos
coincidiu e as capturas do menu Cinema foram idênticas nas duas versões. A Área 2 no perfil
compacto de 40% manteve a sequência aprovada de 40% → 50% → 40%, com backdrop compacto e o
valor salvo preservado.

## Validação

| Comando ou cenário                                                 | Resultado                                                                                                 | Log                                                                                                                   |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Build desktop anterior e posterior à correção                      | PASS, dois binários desktop no espelho ext4                                                               | [Antes](../.build/home-stage02-dock-percent/build-antes.log), [depois](../.build/home-stage02-dock-percent/build.log) |
| `pnpm format:check`                                                | PASS em rustfmt e Oxfmt                                                                                   | [Log](../.build/home-stage02-dock-percent/format.log)                                                                 |
| `pnpm check`                                                       | PASS em versões, Oxlint e Clippy                                                                          | [Log](../.build/home-stage02-dock-percent/check.log)                                                                  |
| `cargo test --locked -p luma --no-default-features home_evolution` | PASS, 39 testes                                                                                           | [Log](../.build/home-stage02-dock-percent/home-unit.log)                                                              |
| Roteiro SDL/OpenGL ES 2.0 de comparação, navegação e seleção       | PASS, 25 capturas incluindo navegação por direcional, cliques, geometria de Cinema e Área 2               | [Log](../.build/home-stage02-dock-percent/visual.log)                                                                 |
| Smoke de regressão B24/B25 em 720p                                 | PASS, 10 cenários de foco, prévia, Aplicar, Cancelar, Voltar, restauração, reabertura e falha de gravação | [Log](../.build/home-stage02-dock-percent/regression.log)                                                             |
| `git diff --check`                                                 | PASS, saída zero                                                                                          | [Log](../.build/home-stage02-dock-percent/git-diff-check.txt)                                                         |

A primeira execução detectou uma diferença fora do campo porque a textura de vidro ainda estava
sendo carregada em um dos quadros. O roteiro passou a aguardar a textura depois que as animações
se estabilizam. [Execução inicial](../.build/home-stage02-dock-percent/visual-inicial.log) ·
[Roteiro de captura](../.build/home-stage02-dock-percent/capture.cjs)

## Escopo da evidência

As medidas, a inspeção visual e o teste de persistência se referem ao preview desktop. Magic
Remote físico, integração real com webOS e renderização da LG C1 exigem validação em hardware.
A aprovação visual global da Etapa 02 permanece pendente.

[Estado Git inicial](../.build/home-stage02-dock-percent/git-status-antes.txt) ·
[Estado Git final](../.build/home-stage02-dock-percent/git-status-depois.txt) ·
[Diff exclusivo desta correção](../.build/home-stage02-dock-percent/diff-correcao.txt)
