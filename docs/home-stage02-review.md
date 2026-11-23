# Etapa 02 — revisão R1–R4

Registro da rodada R1–R4. Os refinamentos seguintes estão em
[Etapa 02 — R5–R9](home-stage02-r5-r9.md); as evidências desta rodada continuam preservadas.

21/11/2026, America/Sao_Paulo. Correções no preview nativo Rust/SDL2/OpenGL ES.
A conclusão técnica desta rodada não representa aprovação visual final do usuário.
A Etapa 03 permanece não iniciada.

## Estado inicial e fontes consultadas

Checkout Windows em `develop`, HEAD `806568ade6eb3e63f3462d94e50896681b7c76ca`,
com índice vazio e alterações locais das Etapas 01 e 02, preservadas integralmente.
Arquivos já modificados: `assets.md`, `assets/messages.json`, `development.md`, `package.json`,
`readme.md`, `src/main.rs`, `src/model.rs`, `src/state.rs`, `src/tests.rs`, `src/today.rs`
e `src/view.rs`. Arquivos novos já existentes: `assets/ui-icons/evolution-settings.svg`,
`docs/home-stage01.md`, `docs/home-stage02.md`, `src/home-evolution-tests.rs`,
`src/home-evolution-view.rs`, `src/home_evolution.rs` e `tests/home-evolution-smoke.mjs`.

Foram consultados `development.md`, os dois relatórios locais anteriores e, pelo conector,
os originais [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view),
[PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view)
e [revisão final aprovada](https://drive.google.com/file/d/1uVMVlf03KPKJukiMyCXlXztnqxVoafOX/view).
Os adendos recentes da decisão 84, sua retificação, revisão pós-Etapa 02 e R4 prevalecem
sobre as alternativas históricas substituídas. Nenhum original do Drive foi alterado.
As capturas anteriores `home-compact.png` e `home-expanded.png` foram inspecionadas antes
de alterar a geometria.

## IMPLEMENTADO

### R1 — clima sobre o wallpaper

Removido o fundo retangular. Ícone e texto usam a composição aberta do cabeçalho, com sombra
tipográfica. A região mantém alvo de ponteiro e indicação de foco, sem cobrir os botões globais.
Dados já disponíveis no estado meteorológico podem apresentar temperatura, condição e cidade;
sem leitura válida aparece indisponibilidade ou carregamento. Falha com leitura anterior inclui
o rótulo de última leitura. Nenhum valor meteorológico fictício é inserido nesta rodada.
Workers, origens, cache, unidades e mecanismos de rede existentes foram preservados.

### R2 — Dock reduzido e Inputs sob demanda

A escala expandida passa de 0,8 para 0,4 do tamanho permanente: metade da largura e da altura
dos cards da captura expandida anterior. A proporção continua 16:10. Capacidade, IDs, ordem
e preferências usam o tamanho permanente; a redução é apenas contextual.
Ao retornar ao Dock, a expansão se reverte e restaura o tamanho configurado.
Pressionamento de ponteiro capturado congela a geometria até a liberação.
A altura dos alvos reduzidos tem mínimo de 64 pixels lógicos, separada do tamanho do logo.
A largura compacta mínima é 76,8 pixels lógicos (51,2 pixels na captura 720p), sem sobrepor
vizinhos. O retorno direcional ao Dock restaura sua geometria em até 300 ms no perfil Equilibrado.

Inputs não geram desenho nem alvos invisíveis inicialmente ou durante a expansão do cinema.
↓ percorre todas as fileiras existentes e só então revela a seção independente de Inputs.
A janela móvel mostra até duas fileiras por vez e avança até a terceira e seguintes, sem
limitar o total de favoritos. Ao revelar Inputs, o Dock sobe para abrir espaço; ↑ retorna
ao último favorito e fecha a seção. Ao subir pelas fileiras, a janela volta progressivamente.
Sem entradas elegíveis não há seção, foco inexistente nem espaço reservado.
Inventário, visibilidade, nomes e efeito de abertura foram mantidos; HDMI do preview continua
identificado como entrada simulada.

### R3 — cabeçalho compacto

Em repouso são preservados relógio grande, data e saudação. Durante a expansão, o cabeçalho
faz transição para saudação, hora e data em uma linha. A mensagem usa o instante fornecido
ao renderer, idioma, nome, relógio de 12/24 horas, segundos, formato de data e visibilidade
individual dos widgets. A data localizada continua localizada; os formatos numéricos escolhidos
são respeitados. O horário fixo das capturas é um fixture explícito de `--time`, não o relógio
de uma sessão normal.

O perfil `balanced` mantém o tween interrompível de 320/300 ms e a transição de opacidade.
`evolution_home.motion_profile` aceita também `reduced` e `none`, com geometria instantânea;
o `reduced_motion` herdado continua prevalecendo. Não foram criados controles para o catálogo
completo de perfis nesta revisão. Com `show_top: false`, nenhum cabeçalho compacto é recriado.
Clima e ações globais permanecem em regiões superiores independentes do backdrop.

### R4 — formato horizontal exclusivo

Removido o campo e o branch `square_icons`. Os únicos tamanhos são 192×120, 224×140 e 256×160.
Não havia seletor quadrado público implementado nas Etapas 01/02; tampouco existe nesta rodada.
JSON antigo com `square_icons: true` é lido sem invalidar os demais campos, usa imediatamente
16:10 e omite o campo obsoleto na próxima gravação. Reiniciar antes de gravar também mantém
16:10, porque o campo não participa mais do layout. Favoritos e sua ordem, tamanho, atalhos
globais, nome e demais preferências permanecem intactos.

O cenário vigente `home-large-square` foi substituído pelos tamanhos horizontais, várias
fileiras e migração. Evidências antigas em `.build/home-stage01/` e `.build/home-stage02/`
são históricas e não representam opções atuais. Cards cinematográficos continuam 16:9.

### Decisão 84 preservada

Topo continua o padrão e fallback; Dock continua a alternativa persistente em Geral → Navegação.
Os dois acessos globais mantêm identidade própria, não são removíveis e podem ser reorganizados
no Dock. A mudança de posição preserva favoritos e retorno de foco. Os botões de topo continuam
disponíveis com widgets desligados. Configurações da TV e Sobre continuam em Sistema.
Cinema circular, serviços herdados, renderer e rotas de Voltar foram preservados.

## Arquivos alterados nesta revisão

- `src/home_evolution.rs`: preferências, migração por leitura, cabeçalho, escala e viewport.
- `src/home-evolution-view.rs`: clima aberto, cabeçalho compacto e composição da seção inferior.
- `src/home-evolution-tests.rs`: migração, foco, restauração e interrupção.
- `src/main.rs`: `--capture-size` para capturas 720p reais e comandos de desenho no dump do preview.
- `tests/home-evolution-smoke.mjs`: cenários gráficos, persistência, resoluções e diagnósticos.
- `development.md`, `readme.md`, nota histórica em `docs/home-stage02.md` e este relatório:
  comportamento vigente e limitações.

`assets/homebrew-description.html` foi revisado: continua descrevendo os recursos efetivos
da versão da TV, incluindo backgrounds enviados e atalhos webhook. As correções são exclusivas
do preview; não se anuncia sua disponibilidade na TV. Não houve alteração de vetores/fontes
nem necessidade de regenerar assets.

## TESTADO NO PREVIEW

| Comando                              | Resultado efetivo                                                                                |
| ------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `pnpm format:check`                  | Passou; repetido após os últimos cenários adicionados                                            |
| `pnpm check`                         | Passou, incluindo Oxlint e Clippy de todo o workspace com todos os targets/features              |
| `pnpm test`                          | Passou: 43 casos Node, 687 do launcher e 67 do helper; um teste externo ignorado                 |
| `xvfb-run -a pnpm test:home-preview` | Passou em 39 cenários                                                                            |
| `xvfb-run -a pnpm test:smoke`        | Passou: smoke geral, 40 marcas SVG, clima mockado, traduções e repetição dos 39 cenários da Home |
| `git diff --check`                   | Passou                                                                                           |

O comando `pnpm test` executa os mesmos 67 testes do helper antes de executá-los novamente
no workspace; são 754 casos Rust distintos aprovados, sem contar duas vezes essa repetição.
Também houve uma passagem inicial de 79 testes Rust focados na Home.
Uma sincronização intermediária detectou arquivo mudando durante a cópia e interrompeu
aquela etapa antes de executar `pnpm test`. A sincronização estável e os comandos finais
de formato, check e testes foram executados novamente com sucesso. Nenhuma divergência
inesperada do espelho foi sobrescrita.

Fonte editável única: `C:\Projetos\LGTV\luma-evolution`. Builds e testes usam
`~/projects/luma-evolution-home-stage01`, com sincronização Windows → WSL incluindo
arquivos novos e não commitados, validação por hashes e interrupção diante de divergência
inesperada. Nenhum fonte é editado exclusivamente no espelho Linux. Somente resultados
gráficos e logs são copiados explicitamente de volta.

As capturas obrigatórias foram inspecionadas individualmente e em três pranchas de comparação.
Foram conferidos os estados normal/expandido, cabeçalho, ausência inicial de Inputs, janela
das fileiras, foco inferior, ações globais, proporções e limites de tela. Não foram identificados
cortes indevidos ou sobreposições nessas capturas. Os diagnósticos dos cenários aprovados têm
zero imagens faltantes, glifos faltantes e falhas de glifos, e framebuffer com alpha íntegro.
Os testes conferem os pixels dos ícones globais no cinema expandido, além dos alvos de foco.
As janelas e os PNGs 720p agora têm 1280×720; os demais têm 1920×1080.

### Matriz de cenários obrigatórios

| Cenários                                       | Evidência                                                                                                                         |
| ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| A–D: Home normal/expandida em 1080p e 720p     | `home-compact`, `home-expanded`, `home-720p`, `home-720p-expanded`                                                                |
| E–F: clima aberto e cabeçalho compacto         | `home-weather-transparent`, `home-compact-header`                                                                                 |
| G–I: três tamanhos horizontais                 | `home-dock-small`, `home-dock-standard`, `home-dock-large` e variantes expandidas                                                 |
| J–K: duas e três fileiras                      | `home-second-row`, `home-dock-three-rows`; a janela acompanha o foco                                                              |
| L–N: Inputs ocultos, revelados e indisponíveis | `home-inputs-hidden`, `home-inputs-revealed`, `home-inputs-after-three-rows`, `home-no-inputs`                                    |
| O–P: ações no topo e no Dock                   | `home-top-actions`, `home-dock-actions`, `home-expanded-dock-actions` e variante 720p                                             |
| Q: topo desativado                             | `home-widgets-off`, `home-expanded-widgets-off`                                                                                   |
| R: migração e persistência                     | `home-legacy-migrated`, `home-migration-saved`, `home-migration-restarted`; testes Rust dos três tamanhos                         |
| S: restauração                                 | `home-back`, `home-three-rows-restored`                                                                                           |
| T: interação e animação                        | `home-interrupted`, `home-expanded-dock-click`, perfis reduzido/sem animações e testes Rust de reversão/captura do pressionamento |

### Capturas e logs

Pasta Windows: `C:\Projetos\LGTV\luma-evolution\.build\home-stage02-review\`.
O mesmo caminho relativo existe no espelho ext4. As 14 capturas exigidas são:

- [home-compact.png](../.build/home-stage02-review/home-compact.png)
- [home-expanded.png](../.build/home-stage02-review/home-expanded.png)
- [home-weather-transparent.png](../.build/home-stage02-review/home-weather-transparent.png)
- [home-compact-header.png](../.build/home-stage02-review/home-compact-header.png)
- [home-dock-small.png](../.build/home-stage02-review/home-dock-small.png)
- [home-dock-standard.png](../.build/home-stage02-review/home-dock-standard.png)
- [home-dock-large.png](../.build/home-stage02-review/home-dock-large.png)
- [home-dock-three-rows.png](../.build/home-stage02-review/home-dock-three-rows.png)
- [home-inputs-hidden.png](../.build/home-stage02-review/home-inputs-hidden.png)
- [home-inputs-revealed.png](../.build/home-stage02-review/home-inputs-revealed.png)
- [home-top-actions.png](../.build/home-stage02-review/home-top-actions.png)
- [home-dock-actions.png](../.build/home-stage02-review/home-dock-actions.png)
- [home-widgets-off.png](../.build/home-stage02-review/home-widgets-off.png)
- [home-720p.png](../.build/home-stage02-review/home-720p.png)

Outras evidências incluem `home-720p-expanded.png`, `home-720p-expanded-dock-actions.png`,
`home-second-row.png`, `home-no-inputs.png`, `home-inputs-after-three-rows.png`,
`home-dock-actions-three-rows.png`, `home-expanded-dock-actions.png`, `home-back.png`,
as capturas de migração e os perfis de movimento. Cada cenário tem dump `.json`, diagnóstico
`-render.json` e `.log`. O dump inclui os comandos de desenho e as geometrias dos controles.
`review-sheet-1.png` a `review-sheet-3.png` e `review-sheets.json` registram as pranchas e sua ordem.

Logs de validação: `format-final.log`, `check-final.log`, `test.log`, `home-preview.log`
e `smoke.log`. Capturas `debug-expanded` e `inspect-*` são auxiliares de inspeção, não presets
adicionais da interface. Os avisos de VSync indisponível do Xvfb não representam desempenho
ou comportamento do driver da TV.

## PENDENTE e limitações conhecidas

A rodada técnica foi concluída em `develop`, com as alterações anteriores preservadas e sem
commit/push. Não restaram testes falhando. A aprovação visual do usuário e as limitações abaixo
permanecem separadas da conclusão técnica.

- Aprovação visual final pelo usuário após esta entrega.
- Integração meteorológica completa da nova Home e demais pendências registradas nas Etapas
  01/02; esta rodada apresenta indisponibilidade honestamente quando não há dados.
- Controles completos dos perfis de animação e layouts alternativos ainda não implementados
  na base do preview continuam pendentes, sem ampliar a tarefa para a Etapa 03.
- O Dock expandido contém logos menores. A navegação direcional retorna ao tamanho normal;
  a legibilidade à distância e o tamanho físico dos alvos exigem avaliação no aparelho.
  As legendas longas dos atalhos no Dock reduzido ficam pequenas, especialmente em 720p;
  os ícones continuam reconhecíveis no preview e os rótulos recuperam o tamanho ao navegar
  para o Dock. O fator solicitado não foi substituído por uma escala maior.

## DEPENDENTE DE VALIDAÇÃO NA TV

Magic Remote físico, abertura/troca real de entradas, lifecycle, desempenho, contraste e leitura
à distância na LG. SDL/Xvfb comprova apenas o preview desktop. Não houve contato com TV,
instalação de IPK, Home Takeover, alteração de firmware/hooks, commit, push ou publicação.
Não foram feitos build ARM ou empacotamento. Próximo passo: revisão visual do usuário;
a Etapa 03 exige autorização explícita posterior.
