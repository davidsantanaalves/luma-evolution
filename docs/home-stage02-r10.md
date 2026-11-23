# Etapa 02 — correção estrutural R10

A R10 corrige a hierarquia da Home Evolution em resposta à reprovação visual de R5–R9.
A revisão visual da nova implementação cabe ao usuário.

## Auditoria

A fonte Windows estava em `develop`, HEAD `806568ade6eb3e63f3462d94e50896681b7c76ca`,
com dois commits locais preexistentes acima da referência `origin/develop`. O índice estava
vazio. Foram preservadas as alterações locais e os arquivos novos das rodadas anteriores.
Os fontes diretamente afetados foram copiados para `.build/home-stage02-r10/before/`.
Os relatórios anteriores e suas capturas continuam disponíveis para comparação.

Foram consultados `agents.md`, `development.md` e os relatórios `home-stage01.md`,
`home-stage02.md`, `home-stage02-review.md` e `home-stage02-r5-r9.md`.
Os três documentos oficiais foram obtidos pelo conector Google Drive em leitura.
Os adendos mais recentes de R10 prevalecem sobre a organização de R6–R8 e sobre os registros
históricos de aprovação. A decisão 84 mantém Topo como padrão e Dock como alternativa.

- [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view)
- [PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)
- [LUMA_EVOLUTION_UX_REVISAO_FINAL_APROVADA.md](https://drive.google.com/file/d/1uVMVlf03KPKJukiMyCXlXztnqxVoafOX/view)

A rodada anterior agrupava clima e ações na mesma linha, desenhava uma placa glass atrás de
cada ícone e abria calendário e previsão em modais separados. Quando a Área 2 recebia foco,
apareciam apenas resumo, relógio ampliado e indicação de OK. A redução de 10% no Dock e a
mudança das miniaturas eram discretas. O carregamento meteorológico, calendário localizado,
preferências, carrossel e temporizador já existiam e foram reaproveitados.

## IMPLEMENTADO

- Área 1 independente com ícones vetoriais à direita. O vetor focado cresce em torno do
  centro do hitbox e recebe luz aplicada apenas à silhueta. O rótulo temporário tem sombra
  de texto e ocupa sua própria linha. Os dois hitboxes preservam posição e tamanho.
- Área 2 abaixo da faixa de ações. Hora, data, saudação e calendário começam à esquerda.
  Condição, temperatura, localização, títulos e colunas de previsão se alinham à direita.
  Foco direcional ou apontar para um widget abre as duas metades juntas na Home.
- Calendário com mês/ano, sete dias da semana, a grade inteira e o dia atual destacado.
  OK inicia navegação dentro da grade. Setas atravessam semanas e meses. As setas de mês
  também aceitam clique. Voltar dentro da grade devolve foco ao relógio.
- No clima com fixture pronto, OK seleciona a primeira hora. Esquerda/direita percorrem
  a linha, cima/baixo alternam entre horas e dias, e Voltar retorna ao widget de clima.
  Sem dados de previsão, a Home mostra o estado meteorológico e a mensagem de indisponibilidade.
- Setas para a esquerda/direita alternam as metades. Cima a partir dos widgets leva às
  ações, quando presentes. Baixo retorna ao cinema com o foco cinematográfico habitual.
  Voltar a partir de um widget restaura o favorito memorizado e as proporções de repouso.
  Calendário e clima recolhem juntos. O foco da Área 2 pausa a rotação, mantém o título e
  recolhe Inputs. O Dock mantém ordem, capacidade por tamanho e cards 16:10.
- Os atalhos em Dock mantêm seus cards, proteção contra remoção e reorganização. Quando a
  Área 1 fica sem alvos, a Área 2 usa o espaço liberado sob o identificador do preview.
  Quando a configuração herdada inclui o cartão Hoje, seu acesso permanece disponível.
  As três opções de animação usam os tweens existentes.

| Arquivo                                                         | Função                                                                                            |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `src/home-evolution-widgets.rs`                                 | `header_geometry`, `evolution_geometry`, `evolution_widget_direction`, `evolution_calendar_month` |
| `src/home-evolution-view.rs`                                    | `top_buttons`, `widgets`, `right_line`, redução de miniaturas em `cinema`                         |
| `src/home-evolution-details.rs`                                 | `calendar`, `forecast` e foco nas células dentro da cena                                          |
| `src/home_evolution.rs`                                         | `set_widgets_open`, navegação, hover, expansão e elegibilidade da rotação                         |
| `src/state.rs`                                                  | Ativação e Voltar, com foco contextual da previsão                                                |
| `src/main.rs`                                                   | Diagnóstico dos comandos desenhados e hover exclusivo do preview                                  |
| `src/home-evolution-tests.rs`, `tests/home-evolution-smoke.mjs` | Geometria, navegação, regressão, pixels dos ícones e capturas                                     |

O README e a documentação de desenvolvimento apontam para R10. A descrição do Homebrew
foi revisada e continua descrevendo recursos da distribuição. A Home Evolution segue em
preview desktop, com preferências isoladas.

## Matriz de conformidade

**CORRIGIDO** identifica uma divergência de R5–R9 resolvida. **CONFORME** identifica
comportamento mantido. **PARCIAL** indica dependência da fonte de dados.
Os números de evidência remetem às capturas listadas abaixo.

| Critério                                  | Situação  | Implementação e verificação                                                                            |
| ----------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------ |
| 1. Área 1 acima da Área 2                 | CORRIGIDO | `header_geometry`, teste `checkHeader`, 01 e guias                                                     |
| 2. Ações à direita                        | CONFORME  | `header_geometry`, borda x=1840, 02–04                                                                 |
| 3. Ícones sem fundo                       | CORRIGIDO | `top_buttons`, comandos e pixels de referência, 02–04                                                  |
| 4. Sem rótulo permanente                  | CONFORME  | `top_buttons`, `checkActions` em repouso e saída do hover, 02                                          |
| 5. Aumento real                           | CORRIGIDO | `top_buttons`, tamanho vetorial +68,75%, pixels medidos, 03–04                                         |
| 6. Glow na silhueta                       | CORRIGIDO | `top_buttons`, cópias vetoriais tingidas, 03–04                                                        |
| 7. Rótulo abaixo                          | CORRIGIDO | `action_label`, bounds medidos, 03–04                                                                  |
| 8. Clima abaixo                           | CORRIGIDO | `header_geometry`, `checkHeader`, 01, 05 e 12                                                          |
| 9. Alinhamento interno do clima à direita | CORRIGIDO | `right_line` e `forecast`, geometria dos textos, 05, 06 e 08                                           |
| 10. Relógio e data à esquerda             | CONFORME  | `widgets`, 05–07                                                                                       |
| 11. Expansão no foco                      | CORRIGIDO | `evolution_direction` e `evolution_pointer`, 06 e `clock-pointer-hover.png`                            |
| 12. Mês completo sem OK                   | CORRIGIDO | `calendar`, 30 dias no cenário de novembro, 06                                                         |
| 13. Previsão horária sem OK               | PARCIAL   | `forecast`, seis horários em 06 e 09 com fixture identificado                                          |
| 14. Próximos dias sem OK                  | PARCIAL   | `forecast`, três dias com mínimas/máximas em 06 e 09 com fixture identificado                          |
| 15. Calendário e clima no mesmo quadro    | CORRIGIDO | `widgets`, `both` em 1080p/720p, 06 e 09                                                               |
| 16. Cinema reduzido                       | CORRIGIDO | `evolution_geometry` e `cinema`, medidas relativas, 10                                                 |
| 17. Dock reduzido                         | CORRIGIDO | `evolution_geometry`, escala e ordem, 11                                                               |
| 18. Inputs ocultos até depois do Dock     | CONFORME  | `set_widgets_open` e navegação do Dock, 06, 16–17                                                      |
| 19. Navegação interna                     | CORRIGIDO | `evolution_widget_direction`, dia/mês/horas/dias, 07 e `forecast-navigation.png`                       |
| 20. Voltar restaura a Home                | CORRIGIDO | `State::back`, `widgets-back-to-dock.png`                                                              |
| 21. Ações no Dock sem duplicatas          | CONFORME  | `evolution_top_targets` e `evolution_dock_items`, 13                                                   |
| 22. 720p sem sobreposição                 | CONFORME  | Coordenadas lógicas e captura física, 09 e cenas 720p de foco                                          |
| 23. Perfis de animação                    | CONFORME  | `evolution_tick`, transição interrompida, `top-button-reduced.png` e `top-button-none.png`             |
| 24. Simulação identificada                | CONFORME  | `widgets` e `forecast`, 06, 08 e 09                                                                    |
| 25. Regressão R1–R9                       | CONFORME  | Regressão da Home, calendário/previsões atualizados para R10, 12, 13, 16, 17 e `cinema-progress-*.png` |

## TESTADO NO PREVIEW

O espelho ext4 `~/projects/luma-evolution-home-stage01` foi sincronizado a partir
da fonte Windows, incluindo as mudanças locais e arquivos novos. A sincronização confere
branch, HEAD, hashes e modos. Divergências inesperadas interrompem a cópia.
A renderização foi feita pelo binário nativo sob Xvfb com software GL. A data determinística
é 20/11/2026 às 20:41. O progresso do cinema usa tempo monotônico real.

| Comando                              | Resultado observado                                                                                                                                                                |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pnpm format:check`                  | Aprovado na fonte final sincronizada. Log `format-final.log`.                                                                                                                      |
| `pnpm check`                         | Aprovado na fonte final sincronizada. Versões, Oxlint e Clippy. Log `check-final.log`.                                                                                             |
| `pnpm test`                          | 767 testes Rust distintos aprovados (700 launcher, 67 helper) e 43 Node. O helper roda novamente no workspace. Um teste externo da Apple permanece ignorado. Log `test-final.log`. |
| `xvfb-run -a pnpm test:home-preview` | 102 cenários aprovados, incluindo os temporizadores reais de R9. Log `home-preview-delivery.log`.                                                                                  |
| `xvfb-run -a pnpm test:smoke`        | Aprovado. Smoke geral, 40 marcas SVG, clima mockado, traduções e 102 cenários da Home. Log `smoke-delivery.log`.                                                                   |
| `git diff --check`                   | Aprovado.                                                                                                                                                                          |

A bateria focada inicial passou com 39 cenários. Depois da inspeção em 720p, foi corrigida
a saída dos rótulos do Dock reduzido para fora de sua moldura. Os rótulos agora recolhem com
a redução em qualquer uma das duas expansões. O preview aprovou separadamente os três quadros
finais em 720p, de repouso, contração e retorno. O comando dedicado da Home iniciou antes dessa
correção visual. O último smoke recompilou a versão com a correção antes de executar a mesma
suíte de 102 cenários da Home.

### Capturas

Todas ficam em `.build/home-stage02-r10/`. Os dumps JSON registram limites das regiões,
comandos da faixa superior, controles, foco, seleção e progresso de rotação.

| Nº  | Arquivo                                     |
| --- | ------------------------------------------- |
| 01  | `01-home-neutral-five-areas.png`            |
| 02  | `02-top-actions-idle-no-background.png`     |
| 03  | `03-top-apps-focused-no-background.png`     |
| 04  | `04-top-settings-focused-no-background.png` |
| 05  | `05-widgets-compact-left-right.png`         |
| 06  | `06-widgets-expanded-both-visible.png`      |
| 07  | `07-widgets-expanded-calendar-left.png`     |
| 08  | `08-widgets-expanded-weather-right.png`     |
| 09  | `09-widgets-expanded-720p.png`              |
| 10  | `10-cinema-reduced-by-top-focus.png`        |
| 11  | `11-dock-reduced-by-top-focus.png`          |
| 12  | `12-home-cinema-expanded.png`               |
| 13  | `13-home-global-actions-in-dock.png`        |
| 14  | `14-widgets-disabled.png`                   |
| 15  | `15-cinema-disabled.png`                    |
| 16  | `16-inputs-hidden.png`                      |
| 17  | `17-inputs-scroll-revealed.png`             |

A captura 06, também disponível como `area2-focused-calendar-weather-simultaneous.png`,
usa `home,preview-weather-ready,up,up`, sem OK. Em um quadro, mostra Área 1, calendário
completo, previsão horária, próximos dias, cinema e Dock reduzidos. A captura 07 entra
na grade e a 08 leva foco ao clima, com ambas as metades visíveis. As capturas 10 e 11
reproduzem o estado expandido para comparação direta com 01.

`comparacao-r5-r9-r10.png` reúne repouso, widgets e Configurações em foco.
`guias-geometricas.png` sobrepõe as guias à captura 06 usando apenas o dump de diagnóstico.
A interface final não renderiza essas guias.

### Medições lógicas a 1920×1080

- As duas metades começam em y=100, abaixo das ações y=28–88, com 12 px de intervalo.
  A esquerda começa em x=80, a direita termina em x=1840. Quando os atalhos estão no Dock
  e não há ação Hoje, a Área 2 começa em y=74. Clima e ícones usam a mesma margem direita.
- Os centros dos ícones são x=1730 e x=1810 com hitboxes de 60×60. O vetor cresce de
  32×32 para 54×54, +68,75% em cada eixo, com o vizinho permanecendo em 32×32.
  O rótulo usa y=92–120 e seus limites horizontais acompanham o texto dentro da área segura.
  O foco em uma ação desce a Área 2 de y=100 para y=132; a saída restaura y=100.
- A Região 2 expandida com Topo ocupa y=100–604. O centro cinematográfico passa de 173 a
  128 px de altura. As miniaturas passam de 114 a 75 px de altura (−34,21%). O Dock padrão
  passa de 224×140 para 129,92×81,20 por card (−42%). A ordenação não muda.
- A escala física em 720p é 2/3 da lógica. O calendário, as seis horas e os três dias
  cabem lado a lado. O tamanho das colunas de previsão e dos rótulos foi conferido na captura.

Os pixels claros da silhueta foram medidos no framebuffer 1080p. As caixas estão no formato
`x, y, largura, altura` e o limiar de medição é RGB ≥ 222.

| Ação          | Ícone em repouso | Ícone em foco    | Crescimento da silhueta             | Caixa do rótulo         |
| ------------- | ---------------- | ---------------- | ----------------------------------- | ----------------------- |
| Todos os Apps | 1717, 45, 26, 26 | 1708, 36, 44, 44 | +69,2% em ambos os eixos            | 1652,68; 92; 154,64; 28 |
| Configurações | 1798, 45, 24, 26 | 1789, 36, 42, 44 | +75,0% em largura, +69,2% em altura | 1493,94; 92; 346,06; 28 |

As 17 capturas obrigatórias foram inspecionadas em resolução nativa. O comparativo mostra
que o clima saiu da linha das ações, o conteúdo detalhado ocupa ambas as metades e o foco
dos ícones cresceu sem placas. A caixa de texto de Configurações termina na margem segura,
12 px acima da Área 2. A captura 17 mostra a última fileira incompleta e Inputs abaixo
dela. A inspeção também incluiu as guias de alinhamento e o comparativo.

O primeiro teste gráfico confundiu branco puro com o tom claro opaco do renderer.
A asserção foi corrigida para selecionar os vetores pela opacidade. Os testes também
comparam os pixels nos cantos dos hitboxes com a mesma área do wallpaper quando as
ações estão no Dock, além de verificar que a faixa superior não emite comandos de fundo.

## PARCIAL

O serviço meteorológico atual oferece leitura corrente para a localização configurada.
Temperatura, condição, unidade, cache e atualização continuam usando o serviço existente.
As seis horas e os três dias adicionais no preview são fixtures explicitamente rotulados.
A navegação por dias/horas opera sobre esses dados disponíveis. Uma fonte real futura
é necessária para alimentar essas previsões com dados reais. Os estados sem dados já são
tratados pela Home. Os parâmetros numéricos de foco e espaçamento são escolhas de implementação
para revisão visual.

## PENDENTE

Revisão visual do usuário em 1080p e 720p. A Etapa 03 não foi iniciada.

## DEPENDENTE DE TESTES NA TV

Magic Remote físico, overscan, leitura à distância, desempenho e ciclo de vida dos apps na LG C1
dependem de teste na TV. A revisão ocorreu localmente em `develop`, sem commit, push, release,
instalação de IPK ou acesso à TV.
