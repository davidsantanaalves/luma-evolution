# Luma Evolution — Etapa 02

Registro histórico da implementação anterior à revisão R1–R4. O formato quadrado e as
capturas que o apresentam foram substituídos pela [revisão atual](home-stage02-review.md);
não representam opções vigentes da Home Evolution.

21/11/2026, America/Sao_Paulo. Refinamento e validação da Home Evolution no preview nativo.

## Diagnóstico e referências

O checkout Windows entrou na etapa em `develop`, no commit `806568a`, dois commits à frente da referência local `origin/develop`. O índice estava vazio e os arquivos da Etapa 01 continuavam no diretório de trabalho. A Etapa 02 foi aplicada sobre esse estado. A compilação e os testes rodam na cópia ext4 `~/projects/luma-evolution-home-stage01`, sincronizada de Windows para WSL pelo script de validação existente.

`git fsck --connectivity-only --no-dangling` e `git diff --check` passaram.

O relatório e as capturas da Etapa 01 mostravam um painel cinematográfico dominante em repouso, rótulos partidos em cards quadrados e competição por espaço entre o cinema e a segunda fileira. Os diagnósticos disponíveis mostravam renderização em resoluções diferentes e zero imagens ou glifos ausentes nas capturas consultadas. A navegação global ainda entrava pelo botão de menu e pela barra lateral herdada.

A especificação e o contexto originais contêm a decisão 84 e sua retificação de 21/11. A posição configurável em **Topo** ou **Dock** substitui a cláusula intermediária que descartava o Dock. A revisão consolidada anterior continua registrando as decisões 1–83. [UX_SPEC_HOME.md](https://drive.google.com/file/d/1y2W1ncepcXDraTQCp4sZvHtrcNQRYNaK/view) · [PROJECT_CONTEXT.md](https://drive.google.com/file/d/1QyGzw7AnpTorWgDgomiDD6ThUOxasY6a/view) · [Revisão final aprovada](https://drive.google.com/file/d/1uVMVlf03KPKJukiMyCXlXztnqxVoafOX/view)

## Arquitetura e comportamento entregue

- **Composição.** Em repouso, o cinema ocupa uma faixa de miniaturas e libera o wallpaper. Na expansão, a arte cresce com recorte proporcional, opacidade e gradientes. Topo e Dock cedem espaço durante a mesma transição. Os valores de espaço e escala usados no preview são parâmetros técnicos.
- **Dock.** A capacidade continua baseada no tamanho escolhido, incluindo o formato quadrado. A segunda fileira aparece ao descer. Os nomes cabem em uma linha, com área reservada abaixo de cada card. A região do cinema usa o espaço restante e mantém miniaturas legíveis com duas fileiras grandes.
- **Decisão 84.** Os alvos `EvolutionGlobal` identificam ações separadamente dos IDs de aplicativos. Cada ação é apresentada em um único lugar. O padrão e o fallback do campo de posição são Topo, inclusive com widgets superiores desativados. Em **Dock**, os dois atalhos participam das fileiras. OK mantido ou clique direito em um atalho inicia a reorganização, as setas movem, OK confirma e Voltar cancela. A única ação de gerenciamento disponível para esses cards é mover. O JSON guarda posição e índices dos atalhos independentemente da lista de favoritos. Em **Geral → Navegação**, a mudança persiste e Voltar da tela de Settings restaura o foco de Configurações na posição escolhida.
- **Entradas.** A faixa abaixo do Dock reutiliza inventário, nomes, ícones, filtros de visibilidade e o efeito de abertura já existentes. As fontes HDMI do desktop estão marcadas como simuladas. A faixa desaparece sem entradas elegíveis e o Dock recupera o espaço. ↓ passa pelas fileiras dos favoritos antes de entrar na faixa. ↑ volta ao foco anterior do Dock.
- **Rotas herdadas.** A Home Evolution usa os atalhos para abrir as páginas nativas existentes de Apps e Configurações. Em **Sistema**, há seleções distintas para as Configurações da TV e Sobre. Widgets habilitados da Home antiga continuam acessíveis pelo painel **Hoje**, inclusive os antes destinados ao centro. Recentes continua pelo fluxo de Voltar e as opções dos apps continuam pelo menu contextual. As rotas de borda da Home legada permanecem nas telas que as usam.

O renderer, shaders, backend da TV, firmware, hooks de takeover e empacotamento ficaram fora da alteração. A apresentação continua disponível no modo desktop `--home-preview`, com preferências e dados de demonstração isolados. A descrição do Homebrew foi revisada e continua apresentando os recursos disponíveis na versão da TV. Os originais do Drive foram consultados sem edição.

### Arquivos alterados na Etapa 02

- `src/home_evolution.rs` e `src/home-evolution-view.rs` concentram estado, geometria, motion e composição.
- `src/state.rs`, `src/view.rs`, `src/main.rs` e `src/today.rs` integram as rotas, foco, configurações, entrada SDL e widgets herdados.
- `assets/messages.json`, `assets/ui-icons/evolution-settings.svg` e `assets.md` registram os rótulos e a engrenagem vetorial.
- `src/home-evolution-tests.rs` e `tests/home-evolution-smoke.mjs` cobrem estado e renderização.
- `docs/home-stage02.md`, `development.md` e `readme.md` registram auditoria e uso do preview.

## Divergências e validação

Clima real na nova região, catálogo de streaming, perfis completos de animação e a migração integral dos menus contextuais de aplicativos ainda são pendências registradas na Etapa 01. A validação desktop cobre teclado, ponteiro SDL, renderização e persistência. O comportamento do Magic Remote na TV e a troca física de fonte ficam para uma validação específica no hardware.

- `pnpm format:check` passou.
- `pnpm assets:vector` e a compilação desktop passaram, com 194 SVGs compilados.
- `pnpm check` passou sem avisos de Oxlint ou Clippy.
- `pnpm test` passou com 683 testes do launcher, 67 do helper e 43 testes Node. Um teste de
  integração externa continuou ignorado pela suíte.
- `xvfb-run -a pnpm test:home-preview` passou em 15 cenários, com dimensões físicas de janela
  verificadas para 1920×1080 e 1280×720.
- `xvfb-run -a pnpm test:smoke` passou com smoke geral, 40 marcas vetoriais, clima via mock local,
  traduções e os mesmos 15 cenários da Home.

Os testes adicionais cobrem fallback de preferências, alternância de posição com retorno do foco, proteção e movimentação dos cards, entradas ausentes, segunda fileira, acesso a Sobre, preservação de Hoje e navegação sem depender da borda esquerda. O smoke verifica cliques SDL, geometria, persistência em novo processo, integridade de alpha e ausência de imagens e glifos faltantes. Os cenários de clima usam mocks no smoke herdado.

As capturas foram renderizadas pelo executável nativo e inspecionadas visualmente. As revisões corrigiram o ícone de Configurações, a transição do backdrop, a organização da faixa de entradas e a quebra dos nomes nos cards quadrados. Os PNGs, diagnósticos e logs estão em `C:\Projetos\LGTV\luma-evolution\.build\home-stage02`.

- [Compacta](../.build/home-stage02/home-compact.png) e [cinema expandido](../.build/home-stage02/home-expanded.png)
- [Atalhos no topo](../.build/home-stage02/home-top-actions.png) e [atalhos no Dock](../.build/home-stage02/home-dock-actions.png)
- [Entradas em foco](../.build/home-stage02/home-inputs.png) e [sem entradas disponíveis](../.build/home-stage02/home-no-inputs.png)
- [Dock grande e quadrado em duas fileiras](../.build/home-stage02/home-large-square.png)
- [Widgets superiores desativados](../.build/home-stage02/home-widgets-off.png) e [janela de 1280×720](../.build/home-stage02/home-720p.png)

Revisar os dois posicionamentos de navegação, o espaço do cinema e a leitura do Dock quadrado antes de iniciar a próxima etapa.
