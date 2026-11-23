# Etapa 02 — correções R11 e R12

R11 e R12 complementam a organização das cinco áreas entregue na R10. A revisão visual da
Home continua com o usuário.

## Auditoria

A fonte editável estava em `C:\Projetos\LGTV\luma-evolution`, branch `develop`, HEAD
`806568ade6eb3e63f3462d94e50896681b7c76ca`. Os dois commits acima da referência local
`origin/develop` são anteriores a esta rodada. A árvore já continha modificações e arquivos
novos. Os fontes diretamente afetados foram preservados em
`.build/home-stage02-r11-r12/before/`, junto com a documentação de referência anterior em
seus caminhos originais.

Foram lidos `agents.md`, `development.md` e `docs/home-stage02-r10.md`. Os dois documentos
oficiais foram consultados pelo conector do Google Drive. Seus adendos R11/R12 de 21/11/2026
confirmam o alvo de foco no resumo do relógio, a hierarquia de Voltar, a ausência de ação
aprovada para OK no relógio e os indicadores informativos vinculados à janela atual.

- [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view)
- [PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)

O alvo R10 `EvolutionTop` cobria toda a coluna esquerda. OK transferia foco para a grade,
e os espaços vazios entre controles podiam selecionar o alvo amplo. O Dock já mantinha seu
índice de primeira fileira e os limites de sua janela, mas a Home ainda não usava esses dados
para indicar o que estava oculto.

## IMPLEMENTADO

### R11 · Foco no resumo

O alvo existente `EvolutionTop` agora corresponde à faixa que contém relógio, saudação e
data, acima das células do calendário. A seleção usa contorno branco e dois traços externos
na cor de glow escolhida em Configurações. A largura do contorno acompanha o resumo visível.
O fundo continua sendo o wallpaper, e o estado de foco não desloca os widgets. Os resumos
e as células usam alvos distintos, com um único foco lógico.

- **Entrar na Área 2:** ↑ a partir do cinema ou apontar para o resumo. Calendário e clima
  expandem juntos.
- **Resumo → calendário:** ↓ entra no dia atual ou no último dia percorrido naquele mês.
  Os direcionais da grade e os controles de mês continuam funcionando.
- **Calendário → resumo:** Voltar retorna diretamente ao relógio. ↑ na primeira semana
  leva aos controles de mês e ↑ novamente retorna ao relógio.
- **Resumo ↔ clima:** → no relógio e ← no clima alternam entre as metades. OK no clima
  continua acessando a previsão quando há dados de preview. Voltar da previsão devolve
  foco ao clima.
- **Saída da Área 2:** Voltar a partir de um resumo restaura o foco do Dock e recolhe os
  widgets conforme a R10. OK sobre o relógio mantém a seleção. A ação adicional de OK
  fica pendente de definição de UX.
- **Ponteiro:** o resumo pode ser apontado ou clicado a partir de uma célula. Atravessar
  o espaço entre os controles dentro da Área 2 mantém a seleção. Sair da região recolhe
  os widgets no modo ponteiro.

### R12 · Indicadores da janela vertical

A geometria projeta o primeiro e último card elegíveis com a mesma origem e o mesmo
passo vertical usados para posicionar a janela do Dock. Se o limite do conteúdo fica
fora da janela por mais de 0,5 px lógico, a Home indica a direção correspondente. A
última fileira incompleta ainda conta como conteúdo. Para ↓, também são considerados os
Inputs elegíveis que ainda não foram revelados. A paginação horizontal de Inputs não
entra no cálculo. A mera existência de widgets ou cinema acima do Dock não ativa ↑.

| Estado real                                      | ↑                       | ↓       |
| ------------------------------------------------ | ----------------------- | ------- |
| Primeira parte visível e conteúdo abaixo         | Oculta                  | Visível |
| Janela intermediária com conteúdo acima e abaixo | Visível                 | Visível |
| Última fileira visível e Inputs ainda abaixo     | Depende da janela acima | Visível |
| Fim. Há conteúdo oculto acima e nada abaixo      | Visível                 | Oculta  |
| Todos os itens verticais elegíveis visíveis      | Oculta                  | Oculta  |

A largura e altura lógica dos dois vetores são 28 px, com centro em x=960. O superior
ocupa y=28–56, entre as colunas. O inferior ocupa y=1036–1064, abaixo do Dock e dos
Inputs, cujo limite nominal inferior é y=1024. As mesmas coordenadas lógicas valem
em 1080p e 720p. O traço é branco a 64% de opacidade com sombra leve e sem placa. A
renderização é condicional e instantânea nos três perfis. Nenhum timer, alvo de foco ou
manipulador de clique foi incluído. Durante a reorganização de favoritos, os indicadores
cedem lugar ao modo de edição e à sua faixa superior.

Mudanças nos favoritos, tamanho dos cards ou lista de Inputs passam pela geometria já
recalculada da Home. A escala contextual de cinema e widgets usa o mesmo cálculo de faixa.
O indicador inferior desaparece assim que os Inputs são revelados ou deixam de ser elegíveis.

### Arquivos alterados

| Arquivo                                                         | Alteração                                                                       |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `src/home-evolution-widgets.rs`                                 | Resumo com hitbox independente, navegação da grade e integração da geometria    |
| `src/home_evolution.rs`                                         | Limites da janela, `ScrollHints`, preservação do foco na travessia por ponteiro |
| `src/home-evolution-view.rs`                                    | Contorno do relógio e vetores informativos                                      |
| `src/state.rs`, `src/main.rs`                                   | OK e hover da Home                                                              |
| `assets/ui-icons/evolution-scroll-up.svg`                       | Chevron derivado do vetor de seta inferior já incluído                          |
| `src/home-evolution-tests.rs`, `tests/home-evolution-smoke.mjs` | Navegação, inventário, geometria, pixels e capturas                             |
| `development.md`, `readme.md`                                   | Referências para a R11/R12                                                      |

A descrição Homebrew foi revisada. Como a alteração se aplica ao preview da Home, a
listagem de recursos da distribuição ficou como estava.

## Matriz de conformidade

| Correção | Critério                                                      | Situação e evidência                                                                  |
| -------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| R11      | Relógio/saudação/data selecionáveis por direcional e ponteiro | Implementado. Capturas 01, 05 e `clock-pointer-after-calendar.png`                    |
| R11      | Destaque luminoso sem placa ou reflow                         | Implementado. Contorno dos comandos nativos e capturas 01 e 05                        |
| R11      | Alternância entre resumo, grade, clima e previsão             | Implementado. Capturas 02–04, testes direcionais e de Voltar                          |
| R11      | OK no relógio mantém a seleção e Voltar respeita a hierarquia | Implementado. Captura 01 e testes de estado                                           |
| R11      | Calendário e previsão simultâneos, foco único                 | Testado no preview. Capturas 01–05 e transição interrompível                          |
| R12      | Estado inicial, intermediário, final e tudo visível           | Testado no preview. Capturas 06–09                                                    |
| R12      | Última fileira incompleta e Inputs elegíveis ou ausentes      | Testado no preview. Capturas 10–12 e remoção de Inputs nos testes                     |
| R12      | Três tamanhos, widgets/cinema, Dock/Topo e 1080p/720p         | Testado no preview. Capturas 13–15 e cenários de combinação                           |
| R12      | Setas discretas, centralizadas, sem colisão ou eventos        | Testado no preview. Inspeção visual, amostragem de pixels e clique SDL                |
| R12      | Perfis de movimento e recálculo imediato                      | Testado no preview. Configurações reduzido/nenhum, mudanças de inventário e de escala |
| R1–R10   | Cinco áreas, persistência, cinema e rotação                   | Regressão nativa, com temporizador monotônico real e previsões de teste identificadas |

## TESTADO NO PREVIEW

A fonte Windows foi sincronizada no espelho ext4
`~/projects/luma-evolution-home-stage01` com verificação de branch, commit, hashes
e modos, incluindo fontes modificados e não rastreados. O renderer Rust/SDL2/OpenGL gerou
as capturas sob Xvfb. O relógio usa a data determinística dos testes (20/11/2026 às 20:41).
O progresso do cinema usa tempo monotônico real.

| Comando                                      | Resultado                                                                                                                    |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `pnpm format:check`                          | Passou. `format.log`                                                                                                         |
| `pnpm check`                                 | Passou. `check.log`                                                                                                          |
| `pnpm test`                                  | 703 testes do launcher, 67 do helper e 43 do Node passaram. Um teste que acessa o servidor da Apple foi ignorado. `test.log` |
| `xvfb-run -a pnpm test:home-preview`         | 125 cenários nativos R1–R12 passaram. `home-preview.log`                                                                     |
| `xvfb-run -a pnpm test:smoke`                | Passou. Renderização, clima simulado, traduções, galeria com 40 apps e nova execução dos 125 cenários da Home. `smoke.log`   |
| `git diff --check`                           | Passou.                                                                                                                      |
| `pnpm assets:vector` e build nativo de debug | Passaram. `vector.log` e `build.log`                                                                                         |
| Execução focada nas correções                | 24 cenários nativos passaram. `focused.log`                                                                                  |

Os logs ficam em `.build/home-stage02-r11-r12/`. O comando `pnpm test` executa o
helper isoladamente e novamente no workspace. A contagem registra seus 67 testes uma
vez. Os testes de clima usam um servidor local de simulação.

### Capturas

Diretório `.build/home-stage02-r11-r12/`. Cada cena tem PNG, dump de estado, diagnóstico
gráfico e log. A geometria dos indicadores está em `home_preview.geometry.scroll` nos
dumps JSON.

| Nº  | Arquivo                               |
| --- | ------------------------------------- |
| 01  | `01-clock-greeting-focused.png`       |
| 02  | `02-calendar-focused.png`             |
| 03  | `03-weather-focused.png`              |
| 04  | `04-widgets-focus-transition.png`     |
| 05  | `05-widgets-720p.png`                 |
| 06  | `06-scroll-top-down-arrow.png`        |
| 07  | `07-scroll-middle-both-arrows.png`    |
| 08  | `08-scroll-bottom-up-arrow.png`       |
| 09  | `09-scroll-all-visible-no-arrows.png` |
| 10  | `10-scroll-last-favorites-row.png`    |
| 11  | `11-scroll-inputs-visible.png`        |
| 12  | `12-scroll-no-inputs.png`             |
| 13  | `13-scroll-720p.png`                  |
| 14  | `14-scroll-large-dock.png`            |
| 15  | `15-scroll-widgets-disabled.png`      |

`scroll-indicators-comparison.png` compara os quatro estados. A prancha
`scroll-navigation-sequence.png` percorre o início, uma janela intermediária, o último
favorito e a revelação dos Inputs. As pranchas combinam quadros do renderer nativo.

As 15 capturas foram inspecionadas individualmente na resolução original. Em 1080p e
720p, o contorno permaneceu junto ao resumo, com o calendário e as duas previsões
visíveis. A cena 04 mostra um quadro intermediário da expansão com foco no clima.
Nas cenas de rolagem, os chevrons apareceram de acordo com os itens ainda alcançáveis.
Os indicadores ficaram separados dos cards, dos Inputs, dos controles e dos rótulos
nas configurações capturadas. As duas pranchas também foram inspecionadas.

## PARCIAL

A leitura climática atual continua usando o serviço existente. Previsão horária e diária
permanece simulada e identificada nas cenas de teste, por depender de uma fonte real com
esses dados.

## PENDENTE

Revisão visual pelo usuário em 1080p e 720p e eventual definição de uma ação para OK no
resumo. A Etapa 03 permanece por iniciar.

## DEPENDENTE DE TESTES NA TV

Magic Remote físico, overscan, legibilidade à distância, contraste e desempenho na LG C1
dependem de validação em hardware. A rodada ficou local em `develop`, sem commit, push,
release, instalação de IPK ou acesso à TV.
