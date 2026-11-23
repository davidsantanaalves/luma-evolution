# Luma Evolution — R13 da Etapa 02

Execução de 23/11/2026 no preview desktop da branch `develop`, commit-base `806568a`. A R13 foi
autorizada para execução. A **aprovação visual global da Etapa 02 continua pendente**.

Os três originais foram lidos no Google Drive. Os metadados confirmaram que pertencem à subpasta
`gpt-context-lg-luma-evolution` (`1mKad_DYA6zFIldXw_UVyBQHGaGm26Ggs`), dentro da pasta `LGTV`
(`1P0N3O1oXIyOGQZzkWMoGMUgs-kioNXcA`). Os checkpoints finais de 22/11/2026 regem os percentuais.
[UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view) ·
[PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view) ·
[Especificação R13](https://drive.google.com/file/d/1l_TUUT8TjNGWdyb_zbARMow3e-GjFhnz/view)

O histórico local inclui os relatórios [R1–R4](home-stage02-review.md),
[R5–R9](home-stage02-r5-r9.md), [R10](home-stage02-r10.md), [R11/R12](home-stage02-r11-r12.md),
[pós-R12](home-stage02-post-r12.md), [seleção unificada](home-stage02-area2-selection.md) e os
[testes de B24/B25](../.build/home-stage02-dock-scale/focused.log).
[Revisão consolidada no Drive](https://drive.google.com/file/d/19pSVUQWm1_6LF_7h2D5n73WW1hxfiB9u/view)

## Comportamento e medidas

O cinema dimensiona largura e altura pela mesma referência 16:9. A escala visual acompanha as
transições existentes, com o backdrop em repouso enquanto a Área 3 estiver compacta. Em composições
com pouco espaço, a faixa usa parte dos espaçamentos antes de limitar a altura. A margem preserva
o contorno dos widgets, a legenda e o espaço entre cards e Dock.

| Cena                                         | Cinema, px lógicos | Escala | Backdrop  | Dock, px lógicos |
| -------------------------------------------- | -----------------: | -----: | --------- | ---------------: |
| 01 Home compacta, foco no Dock               |    291,56 × 164,00 |   100% | Compacto  |  224,00 × 140,00 |
| 02 Área 1 selecionada                        |    291,56 × 164,00 |   100% | Compacto  |  224,00 × 140,00 |
| 03 Área 2 expandida, ambos os apps simulados |     145,78 × 82,00 |    50% | Compacto  |   129,92 × 81,20 |
| 04 Área 2 expandida, sem app de clima        |     145,78 × 82,00 |    50% | Compacto  |   129,92 × 81,20 |
| 05 Voltar para a Home compacta               |    291,56 × 164,00 |   100% | Compacto  |  224,00 × 140,00 |
| 06 Área 3 expandida                          |    291,56 × 164,00 |   100% | Expandido |    89,60 × 56,00 |

As medidas vêm da geometria da cena e das áreas clicáveis dos cards sem a ampliação visual de foco.
O framebuffer 1080p usa escala física 1×. Em 720p, ela é 2/3 sobre a mesma malha lógica. Os dumps
também registram o foco, o índice do título, a expansão de cada área e a posição do Dock.
[Medidas 1080p](../.build/home-stage02-r13-aceite/1080p-medidas.json) ·
[Medidas 720p](../.build/home-stage02-r13-aceite/720p-medidas.json) ·
[Conferência das 60 capturas](../.build/home-stage02-r13-aceite/verificacao-evidencias.txt)

Configurações → Interface → Cinema usa `evolution_home.compact_cinema_scale`. Um perfil que ainda
carece do campo recebe 100%. Valores válidos já salvos permanecem em vigor. A prévia do menu usa
a Home compacta e responde a setas e cliques em passos de 10 p.p. entre 40% e 100%. Restaurar
padrões pede confirmação e prepara 100% na prévia. Aplicar grava e só então atualiza o valor em uso.
Cancelar e Descartar preservam a preferência aplicada. Voltar com edição pendente oferece Aplicar,
Descartar e Continuar editando. Falha na gravação mantém a edição aberta e o valor salvo.

## Capturas nativas

Os PNGs vêm do framebuffer SDL2/OpenGL ES 2.0 no Xvfb, com diagnóstico e dump JSON de cada execução.
O roteiro usa `--home-preview-gles` e verifica o perfil OpenGL ES no relatório do renderer.
As cenas usam preferências isoladas, relógio em 20/11/2026 às 20:41, catálogo fictício e clima
identificado como simulação. Na cena sem app de clima, a previsão simulada segue informativa.

| Cena                                         | 1920×1080                                                                          | 1280×720                                                                          |
| -------------------------------------------- | ---------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| 01 Home compacta, 100%                       | [PNG](../.build/home-stage02-r13-aceite/1080p-01-home-compacta.png)                | [PNG](../.build/home-stage02-r13-aceite/720p-01-home-compacta.png)                |
| 02 Área 1 em foco, 100%                      | [PNG](../.build/home-stage02-r13-aceite/1080p-02-area1.png)                        | [PNG](../.build/home-stage02-r13-aceite/720p-02-area1.png)                        |
| 03 Área 2 com calendário selecionado, 50%    | [PNG](../.build/home-stage02-r13-aceite/1080p-03-area2.png)                        | [PNG](../.build/home-stage02-r13-aceite/720p-03-area2.png)                        |
| 04 Área 2 sem app de clima, 50%              | [PNG](../.build/home-stage02-r13-aceite/1080p-04-area2-sem-app-clima.png)          | [PNG](../.build/home-stage02-r13-aceite/720p-04-area2-sem-app-clima.png)          |
| 05 Voltar ao Dock, 100%                      | [PNG](../.build/home-stage02-r13-aceite/1080p-05-retorno-compacta.png)             | [PNG](../.build/home-stage02-r13-aceite/720p-05-retorno-compacta.png)             |
| 06 Cinema expandido, Dock a 40%              | [PNG](../.build/home-stage02-r13-aceite/1080p-06-cinema-expandido.png)             | [PNG](../.build/home-stage02-r13-aceite/720p-06-cinema-expandido.png)             |
| 07 Interface → Dock, padrão 40%              | [PNG](../.build/home-stage02-r13-aceite/1080p-07-menu-dock.png)                    | [PNG](../.build/home-stage02-r13-aceite/720p-07-menu-dock.png)                    |
| 07a Dock, primeiro passo a 45%               | [PNG](../.build/home-stage02-r13-aceite/1080p-07a-dock-passo.png)                  | [PNG](../.build/home-stage02-r13-aceite/720p-07a-dock-passo.png)                  |
| 07b Dock, máximo 100% na prévia              | [PNG](../.build/home-stage02-r13-aceite/1080p-07b-dock-maximo.png)                 | [PNG](../.build/home-stage02-r13-aceite/720p-07b-dock-maximo.png)                 |
| 08 Interface → Cinema, padrão 100%           | [PNG](../.build/home-stage02-r13-aceite/1080p-08-menu-cinema.png)                  | [PNG](../.build/home-stage02-r13-aceite/720p-08-menu-cinema.png)                  |
| 08d Cinema reaberto a 70% após Aplicar       | [PNG](../.build/home-stage02-r13-aceite/1080p-08d-cinema-reaberto.png)             | [PNG](../.build/home-stage02-r13-aceite/720p-08d-cinema-reaberto.png)             |
| 08f Restaurar prepara 100% na prévia         | [PNG](../.build/home-stage02-r13-aceite/1080p-08f-cinema-restaurado-na-previa.png) | [PNG](../.build/home-stage02-r13-aceite/720p-08f-cinema-restaurado-na-previa.png) |
| 08g Cancelar volta à interface com 70% salvo | [PNG](../.build/home-stage02-r13-aceite/1080p-08g-cinema-cancelado.png)            | [PNG](../.build/home-stage02-r13-aceite/720p-08g-cinema-cancelado.png)            |
| 08h Voltar com edição pendente               | [PNG](../.build/home-stage02-r13-aceite/1080p-08h-cinema-voltar-com-rascunho.png)  | [PNG](../.build/home-stage02-r13-aceite/720p-08h-cinema-voltar-com-rascunho.png)  |
| 08l Restauração aplicada, Home a 100%        | [PNG](../.build/home-stage02-r13-aceite/1080p-08l-cinema-restauracao-aplicada.png) | [PNG](../.build/home-stage02-r13-aceite/720p-08l-cinema-restauracao-aplicada.png) |
| Área 2, Dock grande em duas fileiras         | [PNG](../.build/home-stage02-r13-aceite/1080p-area2-dock-grande-duas-fileiras.png) | [PNG](../.build/home-stage02-r13-aceite/720p-area2-dock-grande-duas-fileiras.png) |
| Área 2, quadro intermediário da expansão     | [PNG](../.build/home-stage02-r13-aceite/1080p-area2-transicao.png)                 | [PNG](../.build/home-stage02-r13-aceite/720p-area2-transicao.png)                 |
| Área 2, clima em foco por hover              | [PNG](../.build/home-stage02-r13-aceite/1080p-area2-hover-clima.png)               | [PNG](../.build/home-stage02-r13-aceite/720p-area2-hover-clima.png)               |

**Pranchas da Home** · [1080p](../.build/home-stage02-r13-aceite/comparativo-1080p.png) ·
[720p](../.build/home-stage02-r13-aceite/comparativo-720p.png)

**Pranchas das configurações** ·
[1080p](../.build/home-stage02-r13-aceite/comparativo-configuracoes-1080p.png) ·
[720p](../.build/home-stage02-r13-aceite/comparativo-configuracoes-720p.png)

A inspeção das quatro pranchas e dos quadros da Área 2 em tamanho nativo confirmou a separação
entre widgets, faixa de cinema e Dock nas duas resoluções. Em 720p, a dica de abertura e os
títulos continuam legíveis na captura nativa. O contorno sai quando o app de clima simulado
fica indisponível.

O smoke da Home também registrou o carrossel circular, a chegada a Inputs pela última fileira,
a recuperação da posição no Dock e uma sequência de transições interrompidas em OpenGL Core.
[Carrossel](../.build/home-stage02-r13-aceite/regressao/home-circular.png) ·
[Inputs](../.build/home-stage02-r13-aceite/regressao/home-inputs-after-three-rows.png) ·
[Retorno ao Dock](../.build/home-stage02-r13-aceite/regressao/home-three-rows-restored.png) ·
[Transições](../.build/home-stage02-r13-aceite/regressao/home-interrupted.png)

## Rastreabilidade

| Decisão ou regressão                                          | Implementação / limite                                                                                                 | Teste                                                                                                             | Evidência 1080p e 720p                                                    | Resultado desktop              |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------ |
| R13, cinema compacto a 100%, inclusive com foco na Área 1     | `evolution_geometry` usa o valor padrão de `CinemaScale` com o backdrop recolhido                                      | Unidade `cinema_scale_has_one_reference_across_home_areas` e medições SDL                                         | 01, 02                                                                    | PASS                           |
| R13, Área 2 a 50% e retorno a 100%                            | Interpolação com referência comum e espaço disponível acima do Dock. Foco e título conservam o comportamento existente | Comparação de IDs, dimensões, foco e áreas clicáveis nas duas resoluções, incluindo Dock grande com duas fileiras | 03, 04, 05, duas fileiras                                                 | PASS                           |
| Área 3 expandida a 100% e Dock a 40% (B24/B25)                | Referência de cinema e parâmetros do Dock preservados. O menu compartilha o desenho com Cinema                         | Cenas 06 e 07, 40% → 45% → 100%, largura e altura no framebuffer e na miniatura                                   | 06, 07, 07a, 07b                                                          | PASS                           |
| B26/B27 e B17, escopo aplicado/restaurado                     | Preferência em `home_evolution.rs`, prévia em `view.rs`, fluxo em `state.rs` e gravação atômica em `main.rs`           | Unidade da faixa, JSON entre Aplicar, Cancelar, Restaurar, reabertura e falha, e as três escolhas de Voltar       | 08–08l                                                                    | PASS                           |
| Seleção unificada e mensagens da Área 2                       | Blocos e ações existentes em `home-evolution-view.rs` e `home_evolution.rs`                                            | Quatro combinações de apps, perda de disponibilidade, OK, hover, clique e um alvo por bloco                       | 03, 04, hover e clique de clima e cenários da suíte da Home               | PASS                           |
| Foco, ponteiro, roda, carrossel, fileiras, Inputs e animações | A escala atua na geometria e preserva a navegação e as transições do preview                                           | Suíte da Home, incluindo carrossel circular, rotação, interrupção, hitboxes, Inputs, Voltar e perfis de movimento | 02–06, transição intermediária e interrompida e cenários da suíte da Home | PASS                           |
| Preferência compacta de 40% frente à regra contextual de 50%  | A preferência salva fica intacta. A Área 2 usa a apresentação de referência                                            | Medição 40% → 50% → 40% com releitura do mesmo JSON                                                               | `compacta-personalizada-40`, `area2-com-40-salvo`, `retorno-com-40-salvo` | PARCIAL, condição de UX aberta |

## Arquivos da R13

| Arquivo                                                         | Alteração                                                                                                   |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `src/home-evolution-widgets.rs`                                 | Escala contextual do cinema, reserva de espaço e medidas da cena                                            |
| `src/home_evolution.rs`                                         | Tipo de escala 40%–100%, padrão e leitura compatível, preferência persistente e override isolado do preview |
| `src/state.rs`, `src/main.rs`                                   | Menu Cinema, foco, rascunho, confirmação, persistência e opção OpenGL ES no preview                         |
| `src/view.rs`, `assets/messages.json`                           | Prévia compacta, controles e mensagens em todos os idiomas disponíveis                                      |
| `src/home-evolution-tests.rs`, `tests/home-evolution-smoke.mjs` | Regressões Rust, roteiro SDL, medidas e pranchas em duas resoluções                                         |
| `development.md`, `readme.md`                                   | Uso da configuração de cinema no preview e vínculo com a R13                                                |
| `docs/home-stage02-r13-aceite.md`                               | Relatório e índice de evidências                                                                            |

`assets/homebrew-description.html` foi revisado para o escopo da distribuição. A R13 integra o preview
desktop da Etapa 02. Os arquivos e evidências históricos permanecem em seus caminhos.

## Validação

O checkout Windows foi a origem dos fontes. O script de sincronização existente verificou branch,
commit, hashes e modos dos arquivos no espelho ext4. O smoke amplo usou uma cópia ext4 dos mesmos
fontes para manter os PNGs históricos do espelho. A validação usou Rust 1.93.0, Node 24.21.0 e
pnpm 10.34.5. As capturas de aceite usaram SDL x11 e OpenGL ES 2.0 com Mesa llvmpipe. O smoke
amplo exercitou também o perfil OpenGL Core padrão do desktop.
[Sincronização](../.build/home-stage02-r13-aceite/sync.log)

| Comando                                                            | Resultado                                                                                                  | Log                                                             |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| `pnpm format:check`                                                | PASS em rustfmt e Oxfmt                                                                                    | [Log](../.build/home-stage02-r13-aceite/format.log)             |
| `pnpm check`                                                       | PASS em versões, Oxlint e Clippy                                                                           | [Log](../.build/home-stage02-r13-aceite/check.log)              |
| `pnpm test`                                                        | PASS. 706 testes do launcher, 67 do helper e 43 do Node. O teste que consulta a Apple permaneceu ignorado. | [Log](../.build/home-stage02-r13-aceite/test.log)               |
| `cargo test --locked -p luma --no-default-features home_evolution` | PASS em 39 testes                                                                                          | [Log](../.build/home-stage02-r13-aceite/home-unit.log)          |
| `cargo build --locked -p luma --no-default-features`               | Build desktop concluído                                                                                    | [Log](../.build/home-stage02-r13-aceite/build.log)              |
| `xvfb-run -a pnpm test:home-preview --r13`                         | PASS em 60 cenários SDL2/OpenGL ES 2.0, 30 por resolução.                                                  | [Log](../.build/home-stage02-r13-aceite/r13-gles.log)           |
| `xvfb-run -a pnpm test:home-preview`                               | PASS em 151 cenários da Home.                                                                              | [Log](../.build/home-stage02-r13-aceite/home-preview-final.log) |
| `xvfb-run -a pnpm test:smoke`                                      | PASS em renderização geral, 40 marcas, clima simulado, traduções e 151 cenários da Home.                   | [Log](../.build/home-stage02-r13-aceite/smoke-final.log)        |

A primeira rodada expôs um intervalo de teste com tipo inadequado e uma asserção que comparava
a miniatura com o diagnóstico de redesenho, que soma uma margem de 1 px por borda. A revisão
seguinte encontrou um card de 40% omitido por arredondamento e o símbolo `%` cortado no 100%
do menu Cinema. A rodada final corrige os testes, o cálculo da altura no limite e o texto do
valor de Cinema.
[Compilação inicial dos testes](../.build/home-stage02-r13-aceite/home-unit-inicial.log) ·
[Smoke focado inicial](../.build/home-stage02-r13-aceite/r13-inicial.log) ·
[Smoke do limite inferior](../.build/home-stage02-r13-aceite/r13-minimo-inicial.log)

## Pendências

- Um perfil com 40% salvo exibe cards de 116,62 × 65,60 px na Home compacta. A Área 2 expande e
  o cinema passa temporariamente a 145,78 × 82,00 px, voltando a 40% ao sair. O checkpoint fixa
  a apresentação de 50% e deixa em aberto o que fazer se ela resultar em aumento. Qualquer
  exceção para esse caso depende de uma decisão específica de UX.
- O menu do Dock corta o símbolo `%` no valor máximo pela largura da caixa de texto preexistente.
  As cenas 07, 07a e 07b e seus dumps registram 40%, 45% e 100% e as respectivas prévias. O
  escopo da R13 preserva a apresentação do Dock.
- A associação e a abertura de apps reais de calendário e clima, o Magic Remote físico e o
  desempenho na LG C1 seguem pendentes de validação em hardware.
- A aprovação visual global da Etapa 02 cabe à revisão do usuário. A Etapa 03 segue bloqueada.

## Git

Os comandos foram executados no Windows. O `git diff --stat` agrega os arquivos rastreados,
incluindo o trabalho anterior à R13. Os módulos da Home e o smoke já apareciam sem rastreamento
no início da rodada. O relatório é o único arquivo novo fora de `.build/`.
`develop` continua dois commits à frente da referência local `origin/develop`. O diff rastreado
agrega 12 arquivos, 1.701 inserções e 51 remoções. `git diff --check` saiu com código zero,
com avisos de conversão de fim de linha do Git no Windows.

- [git status --short --branch](../.build/home-stage02-r13-aceite/git-status.txt)
- [git status](../.build/home-stage02-r13-aceite/git-status-detalhado.txt)
- [git diff --stat](../.build/home-stage02-r13-aceite/git-diff-stat.txt)
- [git diff --check](../.build/home-stage02-r13-aceite/git-diff-check.txt)
- [Status inicial](../.build/home-stage02-r13-aceite/git-status-inicial.txt) ·
  [Stat inicial](../.build/home-stage02-r13-aceite/git-diff-stat-inicial.txt)
