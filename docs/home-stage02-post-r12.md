# Etapa 02 — correções visuais pós-R10–R12

A implementação está restrita ao preview desktop em Rust/SDL2/OpenGL ES. O checkpoint
de 22/11/2026, opção C, rege estas correções. A aprovação visual global da Etapa 02
continua pendente.
[UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view) ·
[PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)

## Implementado

- O cinema compacto usa a escala temporária solicitada no preview por
  `preview-cinema-size:70` ou `preview-cinema-size:100`. O teste alterna o foco entre
  o Dock e a Área 1 e mede a mesma largura nos dois casos. Com a Área 2 expandida,
  o candidato de 100% passa a 70% antes do ajuste ao espaço disponível. O código limita
  a altura pela área segura. Se o espaço ficar abaixo de 40% da altura de referência
  mais a margem do carrossel, a faixa cede espaço durante esse estado. O foco na Área 3
  restaura a composição cinematográfica expandida e a referência de 100%.
- Calendário e previsão aparecem juntos na Home expandida. Hora, data, saudação, dias,
  mês e previsões são informativos. O contorno acompanha o bloco elegível inteiro.
  O destaque azul no dia atual marca a data de referência. A Área 1 mantém seus atalhos.
- A roda percorre as áreas visíveis no preview. Na Área 2, o estado visual pode existir
  sem seleção. A primeira seta após a roda seleciona o primeiro bloco elegível. O
  ponteiro seleciona o bloco apontado. OK e clique sobre um bloco elegível registram uma
  abertura simulada. Voltar retorna ao Dock. Ao perder o alvo de calendário, o foco
  passa ao clima se elegível. Ao perder o app de clima, a Área 2 segue informativa.
  Pela roda, os Inputs aparecem abaixo da última fileira e o retorno recupera a posição
  anterior do Dock.

| App de calendário simulado | App de clima simulado | Alvos              |
| -------------------------- | --------------------- | ------------------ |
| Disponível                 | Disponível            | Calendário e clima |
| Ausente                    | Disponível            | Clima              |
| Disponível                 | Ausente               | Nenhum             |
| Ausente                    | Ausente               | Nenhum             |

| Arquivo                                                         | Alteração                                               |
| --------------------------------------------------------------- | ------------------------------------------------------- |
| `src/home_evolution.rs`, `src/state.rs`                         | Navegação, roda, ativação e recuperação do foco         |
| `src/home-evolution-widgets.rs`                                 | Geometria da faixa de cinema e dos blocos               |
| `src/home-evolution-view.rs`, `src/home-evolution-details.rs`   | Renderização dos blocos e dos detalhes informativos     |
| `src/main.rs`                                                   | Fixtures e diagnóstico da sessão nativa                 |
| `src/home-evolution-tests.rs`, `tests/home-evolution-smoke.mjs` | Regressões de estado, eventos SDL e verificação gráfica |
| `readme.md`, `development.md`                                   | Referências para a revisão atual                        |

Para abrir o candidato de 100% e experimentar os dois blocos com o teclado ou o ponteiro
no preview desktop, use o comando abaixo.

```sh
pnpm dev --home-preview --script home,preview-cinema-size:100,preview-weather-ready,preview-widget-apps:both
```

## Testado

| Comando                                                            | Resultado                                                                                                               |
| ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `cargo build --locked -p luma --no-default-features`               | Build desktop concluído.                                                                                                |
| `pnpm format:check` e `pnpm check`                                 | Formatação, versões, Oxlint e Clippy passaram.                                                                          |
| `cargo test --locked -p luma --no-default-features home_evolution` | 35 testes passaram.                                                                                                     |
| `pnpm test`                                                        | 702 testes do launcher, 67 do helper e 43 do Node passaram. O teste que acessa o servidor da Apple permaneceu ignorado. |
| `xvfb-run -a pnpm test:home-preview`                               | 136 cenários de renderização, navegação e rotação passaram.                                                             |
| Cenários pós-R12 em Xvfb                                           | 27 cenários passaram no build final.                                                                                    |
| Roda e Inputs, com eventos SDL em 720p                             | Duas cenas confirmaram a última fileira e a restauração da janela anterior do Dock.                                     |
| `xvfb-run -a node tests/smoke.mjs`                                 | Smoke geral passou. Inclui navegação, ponteiro SDL, persistência e captura 4K.                                          |
| `xvfb-run -a node tests/brands-smoke.mjs`                          | 40 aplicativos com arte SVG renderizados.                                                                               |
| `xvfb-run -a node tests/weather-smoke.mjs`                         | Smoke de clima passou com servidor local de simulação.                                                                  |
| `xvfb-run -a node tests/translations-smoke.mjs`                    | Smoke de idiomas, glifos e escalas passou.                                                                              |
| `git diff --check`                                                 | Passou.                                                                                                                 |

Os cinco scripts de `test:smoke` foram executados separadamente em Xvfb e reutilizaram o binário desktop com `LUMA_SMOKE_SKIP_BUILD=1`.
Os cenários focados e os testes de Inputs foram repetidos após os ajustes finais de
enquadramento do Dock pela roda e de confirmação do alvo capturado no clique.
Os estados JSON e os pixels das cenas verificam o encaixe do calendário mensal e
das nove previsões, a proporção 16:9, as áreas de clique e a separação entre as
regiões. Os diagnósticos da matriz registram as duas resoluções e zero recursos
ou glifos ausentes. As dez capturas foram inspecionadas visualmente.

A primeira execução da Home detectou uma comparação instável de preferências por
causa da semente e do horário iniciais do papel de parede. A fixture agora fixa os
dois campos. A matriz também confere o arquivo isolado após cada execução.
[Log inicial](../.build/home-stage02-post-r12/home-preview-inicial.log) ·
[Log da suíte completa](../.build/home-stage02-post-r12/home-preview.log) ·
[Log dos cenários focados](../.build/home-stage02-post-r12/focused.log) ·
[Log de Inputs e roda](../.build/home-stage02-post-r12/inputs-wheel-check.log)

## Evidências

O diretório `.build/home-stage02-post-r12/` contém as capturas nativas, os estados JSON
e os diagnósticos do renderer. As cenas usam data fixa e rotação desativada para comparar
as dimensões com os mesmos títulos. A janela e o framebuffer foram renderizados na
resolução indicada em cada coluna.

| Cena                                     | Foco         | Apps simulados | 1080p                                                                             | 720p                                                                             |
| ---------------------------------------- | ------------ | -------------- | --------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Home compacta a 100%                     | Dock         | Ambos          | [PNG](../.build/home-stage02-post-r12/1080p-01-home-compacta-100.png)             | [PNG](../.build/home-stage02-post-r12/720p-01-home-compacta-100.png)             |
| Área 2 expandida pela roda, cinema a 70% | Somente área | Ambos          | [PNG](../.build/home-stage02-post-r12/1080p-02-area2-roda-cinema-70.png)          | [PNG](../.build/home-stage02-post-r12/720p-02-area2-roda-cinema-70.png)          |
| Área 2 com dois alvos                    | Calendário   | Ambos          | [PNG](../.build/home-stage02-post-r12/1080p-03-area2-ambos-calendario-focado.png) | [PNG](../.build/home-stage02-post-r12/720p-03-area2-ambos-calendario-focado.png) |
| Área 2 com um alvo                       | Clima        | Só clima       | [PNG](../.build/home-stage02-post-r12/1080p-04-area2-so-clima-focado.png)         | [PNG](../.build/home-stage02-post-r12/720p-04-area2-so-clima-focado.png)         |
| Área 2 sem alvo acionável                | Somente área | Só calendário  | [PNG](../.build/home-stage02-post-r12/1080p-05-area2-so-calendario-sem-foco.png)  | [PNG](../.build/home-stage02-post-r12/720p-05-area2-so-calendario-sem-foco.png)  |

Em coordenadas lógicas, a largura e a altura de um card medem 204,1 × 114,8 px no
candidato de 70% e 291,6 × 164,0 px no de 100%, um aumento linear de 42,9%. Na cena
com a Área 2 expandida, os cards medem 204,1 × 114,8 px, uma redução linear de 30%
em relação à Home compacta de 100%. Em 720p, o renderer aplica a escala física de 2/3
sobre a mesma composição lógica.
[Comparativo 1080p](../.build/home-stage02-post-r12/comparativo-1080p.png) ·
[Comparativo 720p](../.build/home-stage02-post-r12/comparativo-720p.png)

Os quadros complementares mostram a escala de 100% com foco na Área 1, a composição
restaurada ao focar o cinema e a expansão dos widgets com ambos os apps ausentes.
[Área 1 em foco](../.build/home-stage02-post-r12/cinema-compacto-100-foco-area1.png) ·
[Cinema expandido](../.build/home-stage02-post-r12/cinema-expandido-apos-area2.png) ·
[Ambos ausentes](../.build/home-stage02-post-r12/area2-nenhum-app.png)

## Parcial e pendente

- As escalas candidatas de 100% e 70%, o contorno dos blocos e a legibilidade em
  ambos os tamanhos seguem para revisão visual. A referência histórica de 70% continua
  a escala sem fixture. Os ajustes de comparação existem somente em memória. A rodada
  preserva as preferências e a regra de restauração. O controle persistente da escala
  compacta previsto em B27 ainda não integra o preview.
- O inventário e o lançamento de apps específicos de calendário e clima carecem de
  associação verificada no projeto. A execução desktop demonstra o roteamento mediante
  fixtures e uma confirmação explícita de abertura simulada. Os dados horários e diários
  usados nas capturas também são simulados, independentemente da disponibilidade dos apps.
- A roda foi verificada com eventos SDL no desktop. Seu comportamento físico no Magic
  Remote e a visualização em TV seguem sem validação nesta rodada.

## Impacto

A fonte Windows está em `develop`, com as mudanças locais e os relatórios históricos
preservados. O espelho ext4 foi atualizado a partir do Windows com verificação de branch,
commit, hashes e modos. A geometria do Dock, a ordem e a capacidade por fileira, os
atalhos globais, os Inputs e os indicadores R12 conservaram as regras existentes.
A redução contextual do Dock também conserva a fórmula preexistente do preview, que
chega a 40% no cinema expandido. O padrão de 70% e o controle persistente descritos
em B24/B25 seguem como diferença preexistente do protótipo.
A descrição Homebrew continua descrevendo funcionalidades da distribuição.

O `git status` registra `develop` dois commits locais à frente de `origin/develop`,
como no início da revisão. O `git diff --stat` dos arquivos rastreados contabiliza
11 arquivos, 933 inserções e 37 remoções, incluindo o trabalho anterior. O único
arquivo novo fora de `.build/` é este relatório. Os módulos de Home já não rastreados
foram editados em seus caminhos existentes. As evidências ficam em `.build/`.
[`git status --short --branch`](../.build/home-stage02-post-r12/git-status.txt) ·
[`git diff --stat`](../.build/home-stage02-post-r12/git-diff-stat.txt)

A execução ficou no desktop em `develop`. Os originais do Drive, o histórico Git,
a distribuição, a integração webOS e a Etapa 03 ficaram fora da execução.
