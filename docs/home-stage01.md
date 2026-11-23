# Home Evolution — Etapa 01

Data: 20/11/2026 (America/Sao_Paulo). Implementação inicial para revisão, sem avançar à Etapa 02.

## Fontes e precedência

A localização confirmada no Drive é **Projetos / LGTV / gpt-context-lg-luma-evolution**.
A pasta irmã `luma-evolution` contém uma cópia do código e não é a pasta das especificações.
Existem duplicatas antigas com os mesmos nomes; foram selecionados os IDs originais indicados
pela revisão final, atualizados em 20/11/2026 no horário de São Paulo:

- [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view).
- [PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view).
- [Revisão final aprovada](https://drive.google.com/file/d/1uVMVlf03KPKJukiMyCXlXztnqxVoafOX/view).
- [Pasta das especificações](https://drive.google.com/drive/folders/1mKad_DYA6zFIldXw_UVyBQHGaGm26Ggs).

Todos os três arquivos foram obtidos integralmente pelo conector. Cópias locais de consulta,
com sequências de linhas vazias compactadas, ficam em `.build/ux-reference/` (ignoradas pelo Git).
A consolidação final supera os bloqueios históricos de implementação: 83 decisões e aprovação
global concedidas. Nenhuma decisão de instalação, Home Takeover ou publicação decorre disso.
Os originais do Drive não foram alterados por esta etapa.

## Diagnóstico inicial

- Branch Windows: `develop`; HEAD `806568ade6eb3e63f3462d94e50896681b7c76ca`.
- Árvore e índice inicialmente limpos; dois commits à frente da referência local `origin/develop`.
- Commits locais: `58e2f8b` (identidade/isolamento) e `806568a` (ícone oficial).
- Não foi feito fetch: a comparação usa a referência remota já existente no checkout.
- O espelho Linux antigo está em `0082312`, com alterações da validação anterior registradas
  em seu próprio manifesto de sincronização. Foi preservado integralmente.
- Esta etapa usa `~/projects/luma-evolution-home-stage01`, clone local separado.
  O script de sincronização valida hashes, arquivos inesperados, branch e HEAD antes de copiar;
  transfere também mudanças não commitadas e novos fontes, sem transferir caches do Windows.
- Fonte editável única: `C:\Projetos\LGTV\luma-evolution`. Rustfmt opera explicitamente nesses
  fontes; compilação e testes operam somente no espelho ext4. Nenhuma sincronização de fontes
  Linux → Windows é usada. Apenas capturas e relatórios são copiados explicitamente de volta.

## Arquitetura encontrada

| Responsabilidade                      | Componentes existentes                                                                           | Reuso nesta etapa                                   |
| ------------------------------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------- |
| Inicialização, SDL e ciclo de eventos | `src/main.rs`, `src/sdl_events.rs`                                                               | Mesmo processo e thread gráfica                     |
| Cenas e composição                    | `src/view.rs`, `src/layout.rs`                                                                   | Mesmas primitivas, wallpaper, glass, fontes e logos |
| GPU e apresentação                    | `src/renderer.rs`, shaders GLES2, `damage.rs`, `layers.rs`, `ground.rs`                          | Renderer preservado                                 |
| Foco, páginas e efeitos               | `src/state.rs`, `motion.rs`                                                                      | Targets por identidade, menus e páginas herdadas    |
| Preferências                          | `src/model.rs`, `src/storage.rs`                                                                 | JSON existente, defaults compatíveis                |
| Artwork                               | `src/app_artwork.rs`, `src/branding.rs`, `vector.rs`                                             | Logos vetoriais e worker de imagens                 |
| Fundos                                | `wallpapers.rs`, `dynamic_wallpaper.rs`, `aerials.rs`, `wallpaper_cycle.rs`                      | Assets e composição existentes                      |
| Widgets                               | `today.rs`, `weather.rs`, `crypto.rs`, `cs2.rs`, `shows.rs`, `cub.rs`, `app_cards.rs` e serviços | Preservados no caminho herdado                      |
| Settings                              | `view.rs` e `state.rs`, subpáginas e preview da própria Home                                     | Acessíveis pelo menu nativo                         |
| Preview e testes                      | `pnpm dev`, scripts em `tests/`, `tools/screenshots.mjs`                                         | Binário Rust desktop e framebuffer OpenGL reais     |

Não há aplicação React/JSX/Chromium no fluxo da TV. As páginas de upload/edição já existentes
são companions servidos ao telefone/computador e não renderizam esta Home.

## Arquitetura incremental

1. **Composição:** `home-evolution-view.rs` compõe topo, cinema e Dock na mesma `Scene`.
   `home_evolution.rs::geometry` calcula os espaços usando coordenadas lógicas 1920×1080;
   o renderer existente aplica a escala/letterbox do drawable físico.
2. **Estado:** `Home` guarda seleção circular, janela visível, expansão, último favorito e timers
   somente em memória. `EvolutionContent` identifica título e ação; `App` mantém o ID real.
3. **Dock:** capacidade calculada pelo tamanho permanente do card, independente da redução
   contextual. Uma fileira inicial, segunda revelada com ↓; listas maiores usam uma janela
   vertical de até duas fileiras. Esse limite de janela é parâmetro técnico, não limite de favoritos.
4. **Cinema:** seleção circular por um item, janela deslocada apenas na borda; posições lógicas
   assinadas permitem atravessar último/primeiro na direção correta. Animação retargetável,
   sem fila. Conteúdo e ação compartilham identidade. O preview nunca tenta reproduzir filmes.
5. **Entrada:** direcionais cancelam timers e atualizam seleção imediatamente. Hover confirma
   após 250 ms e saída recolhe após 350 ms. Ponteiro interrompe deslocamento do carrossel;
   pressionamento capturado congela expansão. A escala de foco altera o desenho, não o hit target.
6. **Motion:** expansão cúbica ease-in-out, carrossel ease-out, redirecionados da posição atual.
   A preferência herdada `reduced_motion` torna essas mudanças geométricas instantâneas.
   O catálogo final de cinco perfis, curvas opcionais e Aplicar/Cancelar fica para implementação
   posterior; esta etapa não apresenta controles que prometam esses recursos.
7. **Widgets:** a região climática é uma reserva explícita sem temperaturas inventadas. A próxima
   ligação deve adaptar o worker Open-Meteo existente a atividade/visibilidade, geração e cache;
   geolocalização aproximada ainda exige fonte e tratamento de privacidade comprovados.
8. **Persistência:** `Preferences.evolution_home` armazena opções de geometria e timers com
   defaults compatíveis com JSON antigo. Foco, expansão e posições de carrossel não são serializados.
   A UI de configurações completa fica pendente; atualmente essas opções são editáveis no JSON
   isolado do preview e não são anunciadas como controles prontos na interface.

## Requisitos aprovados, parâmetros e integrações

| Categoria                         | Conteúdo                                                                                                                                                                                                                                                                                   |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Aprovado                          | Três regiões; topo/centro desativáveis; Dock centralizado sem scroll horizontal; 16:10 com opção quadrada; capacidades por dimensão; foco 1,10; cinema compacto/expandido; carrossel circular por item; foco único; Voltar recolhe/foca Dock; memória de sessão; preferências persistentes |
| Referências aprovadas             | Cards 192×120 / 224×140 / 256×160, margem inferior 56; hover 250 ms e saída 350 ms independentes; expansão 320 ms, recolhimento 300 ms, carrossel 200 ms como tempos iniciais a validar                                                                                                    |
| Parâmetros técnicos deste preview | Cinco posições na janela central; Dock reduzido a 80%; gaps de 20/22; janela vertical de até duas fileiras; recorte de arte e proporções das regiões; ajustes JSON de timers limitados a valores seguros                                                                                   |
| Não comprovado                    | Catálogo/progresso privado de Netflix/Disney+/Prime/Max; deep links universais; geolocalização aproximada; lifecycle e Magic Remote reais; desempenho/blur na GPU da C1                                                                                                                    |

### Conflitos reportados e recorte

`agents.md` descreve uma faixa única com scroll horizontal; a UX aprovada posterior exige Dock
sem scroll horizontal, com segunda fileira. A nova composição segue a UX no preview explícito,
e a composição herdada continua disponível sem a opção. Isso também preserva recursos cuja
migração ainda não cabe nesta etapa.

Os menus herdados ainda usam o fluxo anterior: OK em Todos os aplicativos abre opções,
pressionamento longo de 600 ms, reorganização com persistência a cada movimento e painel lateral.
As decisões 25–72 aprovam mudanças nesses fluxos, inclusive 1 s, popover e cancelamento transacional.
Essas mudanças **não foram implementadas** neste recorte de estrutura visual. Não considerar a Home
final aprovada nem usar este preview para validar a migração completa dos menus.

Também ficam pendentes: expansão funcional do topo/calendário, previsões e cidade automática,
rotação automática do destaque, crossfade de imagens, alternativas completas de acabamento/layout,
alto contraste/escala global da nova composição e todos os controles de settings aprovados.
O caminho herdado preserva suas funcionalidades; o novo preview ainda não é o substituto de produção.

A combinação de Dock grande/quadrado em duas fileiras deixa o cinema bastante compacto;
alguns rótulos genéricos ainda quebram em linhas pouco equilibradas. O ajuste tipográfico desse
arranjo e o tratamento final da arte cinematográfica permanecem pendentes de revisão visual.
Passar os testes de geometria/renderização não equivale à aprovação visual do redesign completo.

## Executar e revisar

No espelho Linux já sincronizado, com o toolchain configurado:

```sh
pnpm dev --home-preview
pnpm test:home-preview
```

O flag é rejeitado em builds `webos`. Mesmo quando `--preferences PATH` é fornecido, o nome
usado para gravação é `home-evolution-preview/preferences.json` sob a pasta indicada. Credenciais
companion, uploads e caches também ficam nesse subdiretório isolado. Sem um caminho explícito,
usa um subdiretório da pasta de preferências desktop. Ele não importa preferências normais. `--test-extra-apps`
continua sendo um preview explícito, e não fallback para erro de inventário.

Setas percorrem Dock/centro/topo; Voltar do centro/topo retorna ao Dock; Menu/M ou o botão Menu
abre o menu herdado para Apps, Inputs, Today e Settings. OK sobre conteúdo fictício mostra uma
mensagem de preview, sem ação de plataforma. OK sobre apps usa somente o backend desktop.

JSON de exemplo no arquivo isolado:

```json
{
  "language": "pt",
  "evolution_home": {
    "show_top": true,
    "show_center": true,
    "dock_size": "standard",
    "square_icons": false,
    "hover_ms": 250,
    "leave_ms": 350,
    "pointer_expands": true
  }
}
```

## Validação e limites

O novo smoke usa o renderer real, preferências temporárias, screenshots e diagnósticos de glyphs,
imagens e alpha; cobre compacto, expandido, Voltar, ciclo, somente Dock, clique, 720p, segunda
fileira e Dock grande quadrado. Verifica dimensões físicas de 1920×1080 e 1280×720, além do
framebuffer capturado. A suíte herdada continua incluída em `pnpm test:smoke`.

Resultados executados no espelho ext4:

- `pnpm format:check`: passou.
- `pnpm check`: passou na versão final. A primeira execução apontou um `unwrap` redundante;
  foi corrigido e o comando completo foi reexecutado com sucesso.
- `pnpm test`: 43 testes Node passaram; 676 testes do launcher e 67 do helper passaram
  (743 casos Rust distintos). Um teste de integração externa permaneceu ignorado pela suíte.
  O comando também executa os 67 testes do helper uma vez antes do workspace.
- `xvfb-run -a pnpm test:smoke`: passou no ciclo completo inicial: smoke geral, 40 marcas SVG,
  clima mockado, traduções e oito cenários da Home. Após os ajustes finais exclusivos do preview,
  foram repetidos `pnpm check`, `pnpm test` e `xvfb-run -a pnpm test:home-preview`; todos passaram.
  O smoke específico final passou nos nove cenários, incluindo o caso adicional grande/quadrado.
  O smoke completo não foi repetido depois desses últimos ajustes.
- `git diff --check`: passou.
- WSLg: janela `Luma Evolution` aberta em 1280×720, driver SDL x11, quatro amostras MSAA,
  confirmada por `xwininfo` com `Map State: IsViewable`. A sessão de verificação foi encerrada.
  Isso comprova abertura da janela; não comprova Magic Remote, lifecycle ou desempenho da C1.

Os nove novos casos Rust cobrem ciclo, geometria/capacidade, reversão de motion, foco/Voltar,
regiões desativadas, clique sem lançamento fictício, hover, estabilidade da segunda fileira e
persistência sem memória de navegação. As capturas finais foram inspecionadas visualmente;
uma sobreposição da saudação durante a expansão foi corrigida antes da execução final.

Capturas e logs copiados explicitamente para `.build/home-stage01/` no Windows:

- `home-compact.png`, `home-expanded.png`, `home-second-row.png`, `home-dock-only.png`.
- `home-back.png`, `home-circular.png`, `home-click.png`, `home-720p.png`.
- `home-large-square.png` verifica o espaço do cinema acima do Dock quadrado em duas fileiras.
- `final-format.log`, `final-check.log`, `final-test.log`, `final-home-smoke.log`.
- `smoke.log` registra as cinco suítes gráficas do ciclo completo inicial.
- Arquivos `*-render.json` registram diagnósticos das capturas.

Os fontes/manifestos/configurações alterados ou adicionados são `src/main.rs`, `src/model.rs`,
`src/state.rs`, `src/view.rs`, `src/tests.rs`, `src/home_evolution.rs`,
`src/home-evolution-view.rs`, `src/home-evolution-tests.rs`, `package.json`,
`tests/home-evolution-smoke.mjs`, `readme.md`, `development.md` e este relatório.
`assets/homebrew-description.html` foi revisada e mantida: o preview não é anunciado como
recurso disponível na TV. Os assets gerados foram reutilizados da preparação Linux anterior;
não houve mudança de SVGs/fontes nem nova integração de streaming. Rótulos específicos deste
protótipo estão em português; a localização completa da nova composição permanece pendente.

Não houve acesso à TV, instalação de IPK, modificação de hooks/firmware, commit, push ou publicação.
Não foram executados build ARM, empacotamento ou testes de hardware nesta etapa.

Para reabrir no Windows o binário validado via WSLg (script local de verificação):

```powershell
wsl -d Ubuntu-24.04 -- bash /mnt/c/Projetos/LGTV/luma-evolution/.build/home-stage01/live-preview.sh
```

Próximo passo recomendado: revisão visual desta estrutura e dos conflitos/limitações acima.
A Etapa 02 depende da revisão do usuário e não será iniciada automaticamente.
