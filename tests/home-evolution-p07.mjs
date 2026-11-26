import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';

const desktop = { evolution_home: { rotation_enabled: false } };
const home = 'home,preview-weather-ready,preview-widget-apps:both';
const open = (row) => 'home,settings,right' + ',down'.repeat(row) + ',enter';
const controls = (state) => state.home_preview.controls;
const geometry = (state) => state.home_preview.geometry;
const targets = (state, variant) => controls(state).filter((control) => control.target[variant]);
const appId = (i) => `com.luma.scroll-preview.${String(i).padStart(3, '0')}`;
const near = (actual, expected) =>
  assert.ok(Math.abs(actual - expected) < 0.003, `${actual} != ${expected}`);
const flags = (state) => state.preferences.evolution_home;
const script = (value) => value + ',await-settled,preview-finish';

function globalActions(state, onTop) {
  const actions = targets(state, 'EvolutionGlobal');
  assert.deepEqual(actions.map((c) => c.target.EvolutionGlobal).sort(), ['apps', 'settings']);
  assert.ok(actions.every((c) => c.y < 100 === onTop));
  return actions;
}

function cinemaSize(state, factor) {
  near(geometry(state).cinema_card_height, 164 * factor);
  const cards = targets(state, 'EvolutionContent');
  assert.ok(cards.length > 0);
  for (const card of cards) {
    near(card.h, 164 * factor);
    near(card.w / card.h, 16 / 9);
  }
}

export async function reviewP07({ scenario, isolated, output }) {
  const include = process.argv
    .filter((arg) => arg.startsWith('--case='))
    .map((arg) => arg.slice(7));
  const log = [];
  async function check(name, steps, config, assertState, options = {}) {
    if (include.length && !include.some((prefix) => name.startsWith(prefix))) return;
    const { size = '1920x1080', image = false, frames = script(steps), extra = [] } = options;
    try {
      const state = await scenario(
        name,
        frames,
        config,
        ['--frames', '3000', '--resolution', size, ...extra],
        image,
      );
      await assertState(state);
      log.push({ name, size, image, result: 'pass' });
      console.log(`PASS ${name}`);
    } catch (error) {
      log.push({ name, size, image, result: 'fail', reason: String(error.message) });
      console.error(`FAIL ${name}: ${error.message}`);
    }
    await fs.writeFile(path.join(output, 'assertions-p07.json'), JSON.stringify(log, null, 2));
  }

  const min = { evolution_home: { ...desktop.evolution_home, compact_cinema_scale: 40 } };
  const ids = Array.from({ length: 50 }, (_, i) => appId(i));
  const large = {
    favorites: ids,
    evolution_home: {
      ...desktop.evolution_home,
      show_top: false,
      show_center: false,
      dock_size: 'compact',
    },
  };
  for (const [label, size] of [
    ['1080p', '1920x1080'],
    ['720p', '1280x720'],
  ]) {
    await check(
      `${label}-cinema-40`,
      home,
      min,
      (s) => {
        cinemaSize(s, 0.4);
        assert.equal(s.home_preview.top_expansion, 0);
        assert.equal(flags(s).compact_cinema_scale, 40);
        const [card] = geometry(s).cards;
        near(card[2], 224);
        near(card[3], 140);
        globalActions(s, true);
        assert.equal(targets(s, 'Input').length, 0);
      },
      { size },
    );
    await check(
      `${label}-widgets-50`,
      home + ',up,up',
      min,
      async (s) => {
        cinemaSize(s, 0.5);
        assert.equal(s.home_preview.expansion, 0);
        assert.equal(s.home_preview.top_expansion, 1);
        assert.deepEqual(
          targets(s, 'EvolutionWidget').map((c) => c.target.EvolutionWidget),
          ['calendar', 'weather'],
        );
        assert.ok(s.ui_text.includes('Novembro 2026'));
        assert.ok(s.ui_text.includes('Próximos dias · simulação'));
        const prefs = JSON.parse(await fs.readFile(isolated, 'utf8'));
        assert.equal(prefs.evolution_home.compact_cinema_scale, 40);
        globalActions(s, true);
      },
      { size, image: true },
    );
    await check(
      `${label}-retorno-40`,
      home + ',up,up,back',
      min,
      async (s) => {
        cinemaSize(s, 0.4);
        assert.equal(s.home_preview.top_expansion, 0);
        assert.ok(s.focus.App);
        assert.equal(
          JSON.parse(await fs.readFile(isolated, 'utf8')).evolution_home.compact_cinema_scale,
          40,
        );
      },
      { size },
    );
    await check(
      `${label}-dock-100`,
      open(8) + ',right',
      {
        evolution_home: { ...desktop.evolution_home, cinema_dock_scale: 95 },
      },
      (s) => {
        assert.ok(s.ui_text.includes('100%'));
        assert.equal(s.subpage.Visual.draft.dock_scale, 100);
        assert.equal(flags(s).cinema_dock_scale, 95);
      },
      { size, image: true },
    );
    await check(
      `${label}-dock-50-e-inputs`,
      'home' + ',down'.repeat(7),
      large,
      (s) => {
        assert.deepEqual(s.preferences.favorites, ids);
        assert.equal(geometry(s).rows, 7);
        assert.ok(geometry(s).visible_rows < geometry(s).rows);
        const input = targets(s, 'Input')[0];
        assert.equal(s.focus.Input, input.target.Input);
        assert.ok(input.y > geometry(s).pane[1] + geometry(s).pane[3]);
        assert.deepEqual(geometry(s).scroll, { up: [946, 28, 28, 28], down: null });
        globalActions(s, true);
      },
      { size, extra: ['--test-extra-apps', '50'], image: size === '1280x720' },
    );
  }

  await check(
    'cinema-padrao-e-dock-reduzido',
    home + ',up,left',
    desktop,
    (s) => {
      cinemaSize(s, 1);
      assert.equal(s.home_preview.selected, 7);
      assert.equal(s.home_preview.first, -1);
      assert.equal(s.home_preview.expansion, 1);
      assert.equal(s.home_preview.rotation_advances, 0);
      const [card] = geometry(s).cards;
      near(card[2], 89.6);
      near(card[3], 56);
      assert.ok(s.ui_text.includes('Nosso lugar'));
    },
    { image: true },
  );
  await check(
    'home-compacta-padrao',
    home,
    desktop,
    (s) => {
      cinemaSize(s, 1);
      assert.equal(s.home_preview.expansion, 0);
      assert.equal(flags(s).compact_cinema_scale, 100);
      assert.equal(flags(s).cinema_dock_scale, 40);
    },
    { image: true },
  );
  await check(
    'cinema-um-apos-perda-de-id',
    home + ',up,right,preview-catalog-one',
    desktop,
    (s) => {
      assert.equal(s.home_preview.selected, 0);
      const [card] = targets(s, 'EvolutionContent');
      assert.equal(targets(s, 'EvolutionContent').length, 1);
      assert.deepEqual(s.focus, card.target);
      assert.equal(s.home_preview.rotation_advances, 0);
    },
  );
  await check(
    'cinema-vazio-apos-perda-de-id',
    home + ',up,right,preview-catalog-empty',
    desktop,
    (s) => {
      assert.equal(targets(s, 'EvolutionContent').length, 0);
      assert.equal(geometry(s).cinema_card_height, 0);
      assert.equal(s.home_preview.expansion, 0);
      assert.ok(s.focus.App);
    },
  );
  await check(
    'cinema-pausa-ciclo-b6',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-pause:0.25',
    {
      evolution_home: { rotation_seconds: 5 },
    },
    (s) => {
      assert.ok(s.home_preview.rotation_progress >= 0.25 && s.home_preview.rotation_progress < 1);
      assert.equal(s.home_preview.rotation_paused, true);
      assert.equal(s.home_preview.rotation_advances, 0);
    },
  );
  await check(
    'cinema-retoma-ciclo-b6',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-pause:0.25,back',
    {
      evolution_home: { rotation_seconds: 5 },
    },
    (s) => {
      assert.equal(s.home_preview.rotation_paused, false);
      assert.ok(s.home_preview.rotation_progress < 0.25);
      assert.equal(s.home_preview.rotation_advances, 0);
      assert.ok(s.focus.App);
    },
  );
  await check(
    'cinema-modos-spotlight-e-strip',
    home,
    { evolution_home: { ...desktop.evolution_home, cinema_layout: 'spotlight' } },
    (s) => {
      assert.ok(s.ui_text.includes('Além das dunas'));
      const titleCount = s.ui_text.filter((value) => value === 'Além das dunas').length;
      assert.ok(titleCount >= 2, 'spotlight must expose a title above the matching card');
    },
  );
  await check(
    'cinema-strip-expandido',
    home + ',up',
    { evolution_home: { ...desktop.evolution_home, cinema_layout: 'strip' } },
    (s) => {
      assert.equal(s.ui_text.filter((value) => value === 'Além das dunas').length, 1);
      assert.ok(s.focus.EvolutionContent);
    },
  );
  await check(
    'layout-atalhos-e-fallback-clima',
    'home,preview-weather-error,preview-widget-apps:calendar,preview-wheel:up:2',
    {
      evolution_home: {
        ...desktop.evolution_home,
        global_position: 'dock',
        widgets_layout: 'right',
        weather_details: 'minimal',
      },
    },
    (s) => {
      globalActions(s, false);
      assert.equal(targets(s, 'EvolutionWidget').length, 0);
      assert.ok(s.ui_text.includes('Clima indisponível'));
      assert.equal(s.home_preview.top_expansion, 1);
    },
  );
  await check(
    'layout-barra-e-dock-minimalista',
    'home,preview-weather-ready',
    {
      evolution_home: {
        ...desktop.evolution_home,
        show_center: false,
        widgets_layout: 'compact',
        dock_theme: 'minimal',
        dock_position: 'raised',
        dock_size: 'large',
        dock_labels: 'hidden',
      },
    },
    (s) => {
      assert.equal(targets(s, 'EvolutionContent').length, 0);
      const [r] = geometry(s).cards;
      near(r[2], 256);
      near(r[3], 160);
      near(geometry(s).pane[1] + geometry(s).pane[3], 960);
      assert.ok(!s.ui_text.includes('Netflix'));
      globalActions(s, true);
    },
    { size: '1280x720', image: true },
  );
  await check(
    'dock-vazio-e-inputs-desativados',
    'home,down',
    {
      favorites: [],
      input_modes: { 'com.webos.app.hdmi1': 'Off', 'com.webos.app.hdmi2': 'Off' },
      evolution_home: { ...desktop.evolution_home, show_top: false, show_center: false },
    },
    (s) => {
      assert.equal(geometry(s).rows, 0);
      assert.equal(targets(s, 'Input').length, 0);
      assert.ok(s.ui_text.includes('Adicione favoritos em Todos os Apps.'));
      globalActions(s, true);
    },
  );
  await check(
    'dock-foco-apos-input-e-remocao',
    'home' + ',down'.repeat(7) + ',up,preview-remove-apps:' + ids[48],
    large,
    (s) => {
      assert.deepEqual(s.focus, { App: ids[49] });
      assert.equal(geometry(s).rows, 7);
      assert.equal(s.preferences.favorites.at(-1), ids[49]);
    },
    { size: '1280x720', extra: ['--test-extra-apps', '50'] },
  );

  await check(
    'roda-home-redireciona-e-cancela-hover',
    home + ',preview-wheel:up:1,right,preview-pointer-cinema,right,preview-motion:residual:300:380',
    {
      evolution_home: {
        ...desktop.evolution_home,
        hover_ms: 600,
        motion_profile: 'cinematic',
        motion_curve: 'elastic',
      },
    },
    (s) => {
      assert.equal(s.pointer, false);
      assert.ok(s.focus.EvolutionContent);
      assert.equal(s.home_preview.expansion, 1);
    },
  );
  await check(
    'ok-mantido-na-animacao-de-foco',
    'home,preview-press-dock,await-app-options,preview-release-dock',
    {
      evolution_home: {
        ...desktop.evolution_home,
        hold_ms: 500,
        motion_profile: 'cinematic',
        motion_speed: 50,
      },
    },
    (s) => {
      assert.equal(s.modal, true);
      assert.equal(s.focus, 'OpenApp');
    },
  );
  await check(
    'retorno-neutro-com-id-do-dock',
    home + ',right,up,preview-pointer-cinema,home-key',
    desktop,
    (s) => {
      assert.deepEqual(s.focus, { App: 'youtube.leanback.v4' });
      assert.equal(s.pointer, false);
      assert.equal(s.home_preview.expansion, 0);
      assert.equal(s.home_preview.top_expansion, 0);
    },
  );
  const seeded = {
    language: 'pt',
    hour_24: true,
    show_weather: true,
    ...desktop,
    aerials: { seed: 1, changed: 1795200000, index: 0 },
  };
  await check(
    'roda-local-durante-rascunho',
    open(8) + ',right,back,preview-wheel:down:1',
    seeded,
    async (s) => {
      assert.equal(s.focus, 'DockApply');
      assert.equal(s.subpage.Visual.prompt, 'Unsaved');
      assert.equal(flags(s).cinema_dock_scale, 40);
      assert.equal(await fs.readFile(isolated, 'utf8'), JSON.stringify(seeded));
    },
  );
  await check(
    'cinema-rascunho-descartado',
    open(9) + ',left,back,enter,back,up,enter',
    desktop,
    (s) => {
      assert.deepEqual(s.focus, { Setting: 20 });
      assert.equal(flags(s).compact_cinema_scale, 100);
    },
  );

  await check(
    'cinema-persistencia-aplicar',
    open(9) + ',right,back,up,up,enter,home',
    min,
    async (s) => {
      cinemaSize(s, 0.5);
      assert.equal(
        JSON.parse(await fs.readFile(isolated, 'utf8')).evolution_home.compact_cinema_scale,
        50,
      );
    },
  );
  await check('cinema-persistencia-reabrir', open(9), null, (s) => {
    assert.equal(s.subpage.Visual.draft.cinema_scale, 50);
    assert.equal(flags(s).compact_cinema_scale, 50);
  });
  await check(
    'layout-clima-detalhado-direita',
    'home,preview-weather-ready',
    {
      evolution_home: {
        ...desktop.evolution_home,
        widgets_layout: 'right',
        weather_details: 'detailed',
      },
    },
    (s) => {
      assert.ok(s.home_preview.header.clock[0] > 80);
      assert.ok(s.ui_text.some((line) => line.startsWith('Sensação de 25')));
    },
  );
  await check(
    'cinema-giro-completo-b6',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-cycle,preview-pause',
    { evolution_home: { rotation_seconds: 5 } },
    (s) => {
      assert.equal(s.home_preview.rotation_advances, 1);
      assert.equal(s.home_preview.selected, (s.home_preview.rotation_trace_selected + 1) % 8);
    },
  );

  const preserved = {
    ...seeded,
    favorites: ['youtube.leanback.v4'],
    greeting_name: 'Ana',
    evolution_home: { rotation_enabled: false, compact_cinema_scale: 40, hold_ms: 1700 },
  };
  const disk = () => fs.readFile(isolated, 'utf8');
  await check(
    'restauracao-global-aplicada-e-reaberta',
    open(11) + ',up,enter,enter,home',
    preserved,
    async (s) => {
      cinemaSize(s, 1);
      assert.deepEqual(s.preferences.favorites, preserved.favorites);
      assert.equal(s.preferences.greeting_name, 'Ana');
      assert.equal(flags(s).hold_ms, 1700);
      assert.equal(JSON.parse(await disk()).evolution_home.compact_cinema_scale, 100);
    },
  );
  if (
    !include.length ||
    include.some((prefix) => 'gravacao-atomica-preserva-rascunho'.startsWith(prefix))
  ) {
    await fs.writeFile(isolated, JSON.stringify({ language: 'pt', ...preserved }));
    const before = await disk();
    await fs.chmod(path.dirname(isolated), 0o500);
    try {
      await check(
        'gravacao-atomica-preserva-rascunho',
        open(8) + ',right,back,up,up,enter',
        null,
        async (s) => {
          assert.equal(s.subpage.Visual.draft.dock_scale, 45);
          assert.equal(flags(s).cinema_dock_scale, 40);
          assert.ok(s.ui_text.some((text) => text.startsWith('Não foi possível salvar')));
          assert.equal(await disk(), before);
        },
      );
    } finally {
      await fs.chmod(path.dirname(isolated), 0o700);
    }
  }
  const animations = open(3) + ',right,right,down,right,down,left';
  await check(
    'animacoes-demo-combinada',
    animations,
    desktop,
    (s) => {
      assert.equal(s.subpage.Visual.draft.motion_profile, 'cinematic');
      assert.equal(s.subpage.Visual.draft.motion_curve, 'elastic');
      assert.equal(s.subpage.Visual.draft.motion_speed, 90);
      assert.equal(s.home_preview.motion_sample_active, true);
      assert.equal(flags(s).motion_profile, 'balanced');
    },
    {
      size: '1280x720',
      image: true,
      frames: animations + ',await-motion-sample:330,preview-motion-capture',
    },
  );
  await check('animacoes-demo-terminada', animations, desktop, (s) => {
    assert.equal(s.home_preview.motion_sample_active, false);
    assert.equal(flags(s).motion_profile, 'balanced');
  });
  assert.ok(log.length > 0, 'Nenhum cenário P07 selecionado');
  const failures = log.filter((s) => s.result === 'fail');
  assert.equal(failures.length, 0, failures.map((s) => s.name).join(', '));
}
