# Etapa 02 — refinamentos R5–R9

21/11/2026, America/Sao_Paulo. Rodada do preview nativo; aprovação visual final pertence ao
usuário. A Etapa 03 não foi iniciada.

## Auditoria inicial e fontes

Windows em `develop`, HEAD `806568ade6eb3e63f3462d94e50896681b7c76ca`, índice vazio e trabalho
local das Etapas 01/02 e R1–R4 preservado. Já estavam modificados `assets.md`, `assets/messages.json`,
`development.md`, `package.json`, `readme.md`, `src/main.rs`, `src/model.rs`, `src/state.rs`,
`src/tests.rs`, `src/today.rs` e `src/view.rs`; já existiam os novos arquivos dos relatórios,
Home Evolution, smoke e ícone de Configurações. Uma cópia dos fontes diretamente afetados
ficou em `.build/home-stage02-r5-r9/before/` antes das edições.

Foram lidos `agents.md`, `development.md`, os relatórios `home-stage01.md`, `home-stage02.md`
e `home-stage02-review.md`, além dos originais consultados pelo conector Google Drive:

- [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view).
- [PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view).
- [Revisão final aprovada](https://drive.google.com/file/d/1uVMVlf03KPKJukiMyCXlXztnqxVoafOX/view).

Os adendos R5–R9 mais recentes e a retificação da decisão 84 prevalecem sobre trechos históricos
substituídos. Nenhum original foi alterado. As capturas R1–R4 foram consultadas antes de editar.
Calendário, previsão expandida e rotação ainda eram pendências na implementação anterior;
não foram encontrados componentes funcionais equivalentes da Home Evolution para simplesmente abrir.

## IMPLEMENTADO

### R5 — Dock alinhado

O painel usa o envelope dos cards calculados, com padding simétrico em ambos os eixos e
fileiras incompletas centralizadas em x=960. Capacidade e ordenação usam o tamanho permanente;
escala contextual não recalcula favoritos. O padding e os gaps escalam juntos. Rótulos permanecem
ancorados e desaparecem no Dock cinematográfico reduzido para evitar microtexto. O foco cresce
sobre sua posição nominal, sem redistribuir vizinhos. O desenho compartilhado de logos foi
preservado, pois já os centralizava dentro dos cards. Somente 16:10 continua disponível.
Os dumps registram `geometry.pane`, `geometry.cards`, `geometry.first` e `geometry.center`.
O cenário com três fileiras percorre o catálogo e mostra a janela das duas fileiras finais,
mantendo a seleção inteira visível; preserva o limite de fileiras simultâneas da etapa anterior.

### R6 — cabeçalho alinhado à direita

A região meteorológica termina antes dos botões presentes, com gap calculado; sem botões,
termina na margem segura direita, sem espaço reservado para ausentes. O texto e o ícone se
alinham pelo conteúdo efetivo, eliminando a cauda vazia que deslocava o clima para a esquerda.
Clima e botões compartilham a linha superior; o cabeçalho cinematográfico compacto usa a mesma
linha. Clima continua sem fundo retangular. Localização/leitura anterior são respeitadas.

### R7 — widgets expandidos e detalhes

↑ do cinema expande a região superior, com foco próprio para relógio e clima. ←/→ alternam
entre os widgets; ↓ volta ao cinema; ↑ alcança as ações globais. O Dock se reduz a 90% e as
miniaturas cinematográficas cedem espaço durante essa expansão. Esses valores são parâmetros
provisórios de implementação, não novas decisões de UX.

OK no relógio abre calendário mensal local, com mês/ano, grade, seleção e destaque do dia atual.
Direcionais percorrem dias e atravessam meses; controles de mês funcionam por ponteiro, e
Voltar/fechar restaura o widget anterior. Meses e dias da semana cobrem os 14 idiomas; Português
(Brasil) e Inglês usam semana iniciada em domingo. O calendário não acessa agendas pessoais.

OK no clima abre detalhes com leitura/localização existentes quando disponíveis. O adaptador
real atual fornece somente temperatura, condição e dia/noite. Não se fabricam previsões horárias
ou diárias para esse caminho. O preview tem fixtures em memória para pronto, carregamento e erro,
com marca explícita de clima simulado. Horas e dias adicionais existem apenas no fixture
identificado; não são persistidos no cache meteorológico real e não iniciam HTTP.

### R8 — foco contextual dos botões

Sem foco, apenas ícones compactos são desenhados. O botão selecionado ganha expansão, contorno,
glow compartilhado com o Dock e rótulo contextual. Seus alvos de ponteiro permanecem fixos.
Quando há rótulo, os widgets se deslocam coordenadamente para evitar colisão; ao sair, voltam
à posição normal. Não há faixa residual para legendas. Tweens existentes são reaproveitados,
sem novo temporizador de hover. Movimento reduzido e Sem animações resolvem geometria de imediato.

### R9 — rotação e progresso

`Rotation` mantém um único ciclo com `Instant`/`Duration`: a seleção automática e a barra usam
o mesmo elapsed monotônico. Preferências persistentes: `rotation_enabled` e `rotation_seconds`
(referência 10 segundos, faixa técnica de 1–300; entrada inválida restaura 10 sem perder favoritos).
O renderer recebe progresso limitado a 0–1, sem intervalo fixado em sua função de desenho.

A linha de 2 pixels lógicos fica na borda inferior interna do card selecionado, abaixo da legenda.
Foco direcional, ponteiro no cinema, pressionamento capturado e detalhes abertos suspendem a
rotação. Saída da interação reinicia um intervalo completo. Mudança de título, duração e retorno
à Home reiniciam o ciclo. Catálogo vazio/único e rotação desativada ocultam a barra e impedem troca.
O carrossel acompanha a seleção circular, inclusive com ponteiro fora do cinema.

O loop processa eventos antes de decidir uma troca automática: uma interação recebida no limite
do intervalo tem prioridade. A mudança manual por hover também zera o ciclo e associa a linha ao
ID lógico do conteúdo, inclusive quando a janela circular do carrossel atravessa o fim do catálogo.

A composição compacta continua sendo o repouso padrão; a rotação não força expansão espontânea.
Por consequência, entrar direcionalmente no cinema mostra o progresso pausado, conforme a UX.
O script desktop `preview-cinema` permite demonstrar o estado expandido passivo; os comandos
`await-progress`, `await-pause` e `await-cycle` aguardam o ciclo real, sem atribuir percentuais.
Esses auxiliares exigem a Home de preview desktop e não são oferecidos na TV.

### Decisão 84 e comportamento preservado

Topo continua o padrão/fallback; Dock, a alternativa persistente em Geral → Navegação. Ambos
os acessos existem exclusivamente na posição selecionada, permanecem protegidos e reorganizáveis
no Dock. Configurações da TV e Sobre continuam em Sistema. Não foram modificados o núcleo do renderer,
shaders, inventário, APIs da TV, firmware, hooks, identidade, versões ou empacotamento.
Favoritos, Inputs após a última fileira, cinema 16:9, Dock 16:10, Voltar e isolamento de preferências
continuam cobertos pela regressão anterior.

## Arquivos alterados nesta rodada

- `src/home_evolution.rs`: preferências, geometria, ciclo de rotação, foco e motion.
- `src/home-evolution-widgets.rs`: geometria do cabeçalho, calendário e navegação dos painéis.
- `src/home-evolution-view.rs`: Dock, cabeçalho, foco contextual e linha de progresso.
- `src/home-evolution-details.rs`: calendário e previsão na mesma Scene nativa.
- `src/state.rs`: alvos, OK/Voltar, retorno à Home e agendamento de animação.
- `src/main.rs`: data local do calendário, helpers restritos ao preview e diagnósticos.
- `src/home-evolution-tests.rs`, `tests/home-evolution-smoke.mjs`: testes e capturas.
- `assets/messages.json`: rótulos locais dos widgets/fixtures nos 14 idiomas suportados.
- `development.md`, `readme.md`, nota em `docs/home-stage02-review.md` e este relatório.

`assets/homebrew-description.html` foi revisado e mantido: estas alterações são de preview
desktop e não devem ser anunciadas como funcionalidades já instaladas/validadas na TV.
Não houve mudança de SVGs/fontes nem necessidade de regenerar assets.

## TESTADO NO PREVIEW

Fonte editável única: Windows. Compilação/testes: espelho ext4
`~/projects/luma-evolution-home-stage01`, sincronizado por hashes antes das validações.
Arquivos novos e alterações não commitadas são incluídos; divergências inesperadas interrompem
a sincronização. Somente artefatos são copiados explicitamente de volta.

`pnpm format:check`, `pnpm check` e `pnpm test` passaram na rodada `delivery`.
Os logs ficam em `.build/home-stage02-r5-r9/format-delivery.log`, `check-delivery.log` e
`test-delivery.log`. O teste de entrada no limite do intervalo, o retorno dos painéis e o reset
de progresso após hover estão incluídos nessa execução.

| Validação                            | Resultado observado                                                                                                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pnpm format:check`                  | Aprovado; Rust e ferramentas formatados.                                                                                                                            |
| `pnpm check`                         | Aprovado; versões consistentes, Oxlint sem avisos/erros e Clippy.                                                                                                   |
| `pnpm test`                          | 43 testes Node e 766 testes Rust distintos aprovados: 699 launcher e 67 helper. O helper roda novamente na suíte workspace; 1 teste do launcher permanece ignorado. |
| `xvfb-run -a pnpm test:smoke`        | Aprovado: smoke geral, 40 marcas, clima mockado, traduções e 88 cenários da Home.                                                                                   |
| `xvfb-run -a pnpm test:home-preview` | Passagem final aprovada: 92 cenários R1–R9, com capturas 1080p/720p; log `home-preview-delivery.log`.                                                               |

Os ajustes finais restritos à Home foram cobertos pelo check/test da rodada `delivery`, executado
durante o smoke, e por uma passagem completa dedicada da Home depois do smoke. As quatro capturas
adicionais de Dock aumentaram essa última passagem de 88 para 92 cenários.
Os testes meteorológicos usam servidor local mockado; nenhuma validação acessa a TV ou requer
dados meteorológicos de produção.

Depois da inspeção, a mensagem de previsão indisponível foi simplificada nos 14 idiomas para
evitar termos internos na interface. `pnpm format:check`, `pnpm check` e `pnpm test` passaram
novamente (`format-copy.log`, `check-copy.log`, `test-copy.log`, mesmas contagens). Os três
cenários nativos afetados — carregamento, erro e indisponibilidade — foram recapturados e
revalidados separadamente: painel correto, mensagem nova, ausência de previsão simulada,
imagens/glifos sem falhas e alpha opaco (`copy-validation.log`: 3 aprovados). O smoke completo
e os 92 cenários da Home precedem somente essa revisão de texto; não houve alteração funcional
depois deles. As três imagens e a prancha 15 foram inspecionadas novamente após a revisão.
`git diff --check` também passou no fechamento.

O único teste ignorado é
`aerials::tests::apple_serves_a_full_size_still_to_luma_trusting_only_apples_root`, que alcança
servidores da Apple. Não foi habilitado nesta rodada. Os 12 testes Rust acrescentados para R5–R9
integram os 699 do launcher, além da regressão anterior mantida.

As capturas usam SDL/OpenGL nativo, software GL sob Xvfb, preferências temporárias isoladas e
data determinística `2026-11-20T20:41:00-03:00`. Esse horário controla a apresentação de data/hora;
o progresso usa `Instant` real. O cenário aquece os assets, reinicia o mesmo ciclo de rotação e
aguarda o percentual; a pausa congela a imagem para inspeção. `cinema-progress-100.png` representa
o estado próximo de 100% (limiar de 98%), pois o limite exato já troca o título e zera a barra.
A troca e o reset têm captura separada. Não se atribuem valores artificiais ao elapsed.

Valores medidos na passagem final (intervalo de 10 segundos):

| Captura                           | Elapsed real                   | Progresso | Estado                          |
| --------------------------------- | ------------------------------ | --------- | ------------------------------- |
| `cinema-progress-0.png`           | 0,000009 s                     | ≈0%       | Pausado por foco                |
| `cinema-progress-25.png`          | 2,520932 s                     | 25,21%    | Pausado por foco                |
| `cinema-progress-50.png`          | 5,006638 s                     | 50,07%    | Pausado por foco                |
| `cinema-progress-75.png`          | 7,501869 s                     | 75,02%    | Pausado por foco                |
| `cinema-progress-100.png`         | 9,810671 s                     | 98,11%    | Pausado antes da troca          |
| `cinema-progress-auto-switch.png` | 0,091485 s do novo ciclo       | 0,91%     | Título 0 → 1; uma troca         |
| `cinema-progress-paused.png`      | 5,126917 s                     | 51,27%    | Ponteiro no cinema; zero trocas |
| `cinema-progress-restarted.png`   | 1,450461 s do ciclo reiniciado | 14,50%    | Retomado no mesmo título        |

O snapshot de retomada ocorre depois da estabilização visual, já a 1,45 segundo do novo ciclo;
o reset imediato em zero é verificado pelo teste monotônico. O catálogo circular percorre mais
de uma volta no teste unitário. O intervalo de 2 segundos tem cenário nativo próprio.

O teto de 3.000 iterações dos cenários novos é apenas proteção contra travamento; `preview-finish`
encerra cada execução após o estado esperado. Não define a duração do ciclo e não mede desempenho.

## Capturas e evidências

Diretório: `.build/home-stage02-r5-r9/`. Cada PNG tem dump de estado `.json`, diagnóstico
`-render.json` e log `.log` correspondentes. As imagens são capturas do renderer nativo;
as pranchas apenas as reduzem e acrescentam o nome do arquivo para inspeção.
Foram produzidas 92 capturas de cenário e 16 pranchas `sheet-01.png` a `sheet-16.png`.
Todas as 92 foram inspecionadas nas pranchas. Também foram abertas em resolução nativa as cenas
normal/expandida, Dock grande/incompleto/três fileiras, widgets, calendário, previsão, botões
focados, progresso de 50%/98% e os painéis/botões em 720p.

Capturas obrigatórias:

| Refinamento | Arquivos PNG                                                                                                                                                                                                                    |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R5          | `dock-standard-aligned.png`, `dock-large-aligned.png`, `dock-incomplete-row.png`, `dock-three-rows-aligned.png`, `dock-actions-aligned.png`, `dock-cinema-expanded.png`                                                         |
| R6          | `header-normal-aligned.png`, `header-cinema-compact-aligned.png`, `header-top-actions.png`, `header-dock-actions.png`                                                                                                           |
| R7          | `top-widgets-expanded.png`, `calendar-month-expanded.png`, `weather-forecast-expanded.png`, `widgets-expanded-720p.png`                                                                                                         |
| R8          | `top-buttons-idle.png`, `top-button-apps-focused.png`, `top-button-settings-focused.png`, `top-button-focus-transition.png`, `top-button-reduced-motion.png`, `top-button-no-animation.png`                                     |
| R9          | `cinema-progress-0.png`, `cinema-progress-25.png`, `cinema-progress-50.png`, `cinema-progress-75.png`, `cinema-progress-100.png`, `cinema-progress-paused.png`, `cinema-progress-restarted.png`, `cinema-progress-disabled.png` |

Evidências adicionais incluem uma fileira compacta completa, duas fileiras, três tamanhos do Dock
em 720p, calendário/previsão/botões/progresso em 720p, calendário chinês, navegação mensal, retorno
ao cabeçalho compacto, carregamento/erro/indisponibilidade, clique de ponteiro, troca automática,
intervalo de 2 segundos e catálogos vazio/único. A regressão R1–R4 também gera suas capturas
novamente nesta pasta; as originais de `.build/home-stage02-review/` continuam preservadas.

`rotation-captures.json` reúne valores reais de progresso/elapsed/pausa/trocas dos cenários R9.
`contact-sheets.json` relaciona cada captura à prancha `sheet-NN.png` usada na inspeção.

## Comparação visual e correções da rodada

As capturas originais `home-compact.png` e `home-expanded.png` de R1–R4 foram comparadas com as
novas capturas equivalentes em resolução nativa. O cabeçalho agora agrupa clima e ícones na mesma
linha, próximo à margem direita; o cinema mantém a frase compacta localizada. O Dock preserva
logos e proporções, com envelope equilibrado e menor faixa vazia sob os logos quando reduzido.

Durante a implementação foram corrigidos: alvo de clique calculado antes de concluir a transição
do Dock; ciclo iniciado antes do aquecimento de assets nas capturas temporais; retorno de painel
que perdia o estado compacto anterior; hover pendente depois de abrir um painel; e progresso
residual após mudar manualmente de título. As capturas temporais passaram a aquecer a cena antes
de reiniciar o próprio ciclo, sem substituir o relógio real. Esses problemas não são resultados
aprovados: os cenários foram reexecutados após as correções.

A inspeção técnica das 16 pranchas não apontou cortes ou colisões novos que bloqueiem a revisão.
Os rótulos superiores cabem em 1080p e 720p; a linha temporal fica abaixo do texto, dentro do card.
Os estados sem previsão apresentam mensagem curta, sem expor detalhes da implementação.
Não há correção técnica bloqueante adicional identificada nesta rodada. Permanecem provisórios
o tamanho dos destaques superiores, o acabamento dos modais e a distância de reflow do cabeçalho;
esses pontos dependem da revisão visual do usuário, e a leitura à distância depende da TV.

## PARCIAL

- Previsão horária/diária: demonstrada por fixtures identificados; adaptador real fornece somente
  leitura corrente. Geocodificação/atualização e cache existentes foram preservados.
- Controles completos de personalização da Home e frequência de rotação: os campos persistentes
  do preview são configuráveis em JSON; o catálogo completo de controles ainda não está nesta UI.
- Acabamento de painéis, métricas e rótulos novos são provisórios para revisão visual.

## PENDENTE

Aprovação visual do usuário e revisão das traduções novas; integração meteorológica
com dados de horas/dias verificáveis, sem ampliar esta rodada para a Etapa 03.

## DEPENDENTE DE TESTES NA TV

Magic Remote físico, ciclo de vida/retorno de apps reais, desempenho na LG C1, contraste e
leitura à distância. SDL/Xvfb não comprova esses pontos. Sem acesso à TV, instalação de IPK,
Home Takeover, hooks, firmware, commit, push ou publicação. Próximo passo: revisar as capturas;
a Etapa 03 continua dependendo de autorização explícita posterior.
