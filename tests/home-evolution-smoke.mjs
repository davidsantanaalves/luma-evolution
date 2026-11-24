// The real Rust/SDL/OpenGL renderer, with only local fictional Home content.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import sharp from 'sharp';

const root = path.resolve(import.meta.dirname, '..');
const area2Only = process.argv.includes('--area2-selection');
const r13Only = process.argv.includes('--r13');
const p01Only = process.argv.includes('--p01');
const p02Only = process.argv.includes('--p02');
const p03Only = process.argv.includes('--p03');
const gles = r13Only || p01Only || p02Only || p03Only;
const defaultOutput = p03Only
  ? path.join(root, '.build/home-stage02-p03')
  : p02Only
    ? path.join(root, '.build/home-stage02-p02')
    : p01Only
      ? path.join(root, '.build/home-stage02-p01')
      : path.join(
          root,
          '.build/home-stage02-r13-aceite',
          r13Only ? '' : area2Only ? 'area2-selection' : 'regression',
        );
const output = path.resolve(process.env.LUMA_HOME_SMOKE_OUTPUT ?? defaultOutput);
await fs.mkdir(output, { recursive: true });
const binary = path.resolve(root, process.env.CARGO_TARGET_DIR ?? 'target', 'debug/luma');
if (process.env.LUMA_SMOKE_SKIP_BUILD !== '1') {
  const build = spawnSync('cargo', ['build', '--locked', '-p', 'luma', '--no-default-features'], {
    cwd: root,
    stdio: 'inherit',
  });
  assert.equal(build.status, 0);
}
const scratch = await fs.mkdtemp(path.join(output, 'preview-'));
const normal = path.join(scratch, 'normal.json');
const isolated = path.join(scratch, 'home-evolution-preview/preferences.json');
await fs.mkdir(path.dirname(isolated), { recursive: true });
const sentinel = '{"greeting_name":"Preservar preferências normais"}\n';
await fs.writeFile(normal, sentinel);
try {
  let scenarios = 0;
  async function scenario(name, script, configuration = {}, extra = []) {
    scenarios += 1;
    if (configuration !== null) {
      await fs.writeFile(
        isolated,
        JSON.stringify({ language: 'pt', hour_24: true, show_weather: true, ...configuration }),
      );
    }
    const screenshot = path.join(output, `${name}.png`);
    const dump = path.join(output, `${name}.json`);
    const diagnostics = path.join(output, `${name}-render.json`);
    const result = spawnSync(
      binary,
      [
        gles ? '--home-preview-gles' : '--home-preview',
        '--resolution',
        '1920x1080',
        '--preferences',
        normal,
        '--time',
        '2026-11-20T20:41:00-03:00',
        ...(script === null ? [] : ['--script', script]),
        '--frames',
        '100',
        '--screenshot',
        screenshot,
        '--dump-state',
        dump,
        '--diagnostics',
        diagnostics,
        '--capture-size',
        extra.includes('1280x720') ? '1280x720' : '1920x1080',
        ...extra,
      ],
      { cwd: root, encoding: 'utf8', timeout: 120000 },
    );
    await fs.writeFile(
      path.join(output, `${name}.log`),
      `${result.stdout ?? ''}${result.stderr ?? ''}`,
    );
    assert.equal(result.status, 0, result.stderr);
    const state = JSON.parse(await fs.readFile(dump, 'utf8'));
    const render = JSON.parse(await fs.readFile(diagnostics, 'utf8'));
    if (gles) assert.match(render.graphics.version, /^OpenGL ES /);
    assert.deepEqual(
      render.window,
      extra.includes('1280x720') ? { width: 1280, height: 720 } : { width: 1920, height: 1080 },
    );
    assert.equal(render.capture.missing_images, 0, result.stderr);
    assert.equal(render.capture.missing_glyphs, 0, result.stderr);
    assert.equal(render.capture.glyph_failures, 0, result.stderr);
    const dimensions = await sharp(screenshot).metadata();
    assert.equal(dimensions.width, render.window.width);
    assert.equal(dimensions.height, render.window.height);
    assert.equal(await fs.readFile(normal, 'utf8'), sentinel);
    const stats = await sharp(screenshot).stats();
    assert.equal(stats.channels[3].min, 255);
    assert.ok(stats.channels.slice(0, 3).some((c) => c.stdev > 10));
    return state;
  }
  const capture = (name, script = 'home', config = {}, extra = []) =>
    scenario(name, `${script},await-settled,preview-finish`, config, [
      '--frames',
      '3000',
      ...extra,
    ]);
  if (p03Only) {
    await reviewP03(capture);
  } else if (p02Only) {
    await reviewP02(scenario, capture);
  } else if (p01Only) {
    await reviewP01(capture);
  } else if (r13Only) {
    await reviewR13(scenario, capture);
  } else {
    await reviewArea2Selection(capture);
    if (!area2Only) await runHomeRegression(scenario, capture);
  }
  console.log(`Native Home Evolution: ${scenarios} native rendering/navigation scenarios passed`);
} finally {
  await fs.rm(scratch, { recursive: true, force: true });
}

async function reviewP03(capture) {
  const base = 'home,preview-weather-ready,preview-widget-apps:both';
  const ids = Array.from(
    { length: 50 },
    (_, n) => `com.luma.scroll-preview.${String(n).padStart(3, '0')}`,
  );
  const nav = 'settings,right' + ',down'.repeat(7) + ',enter,down,down,enter';
  const up = (n = 1) => `preview-wheel:up:${n}`;
  const down = (n = 1) => `preview-wheel:down:${n}`;
  const area = (state, name) => {
    assert.equal(state.page, 'Home');
    assert.deepEqual(state.focus, { EvolutionArea: name });
    assert.equal(state.home_preview.simulated_widget_open, null);
  };
  const centerOf = (control) => `${control.x + control.w / 2}:${control.y + control.h / 2}`;
  for (const [resolution, size] of [
    ['1080p', '1920x1080'],
    ['720p', '1280x720'],
  ]) {
    const preview = { rotation_enabled: false, dock_size: 'compact' };
    const config = { favorites: ids, evolution_home: preview };
    const steps = (n) => ({ ...config, evolution_home: { ...preview, wheel_steps: n } });
    const shot = (name, actions, prefs = config) =>
      capture(`${resolution}-${name}`, [base, ...actions].join(','), prefs, [
        '--resolution',
        size,
        '--test-extra-apps',
        '50',
      ]);
    const near = await shot('02-dock-cinema', [up(), 'enter']);
    area(near, 'cinema');
    assert.equal(near.pointer, false);
    assert.equal(near.home_preview.expansion, 1);
    const widgets = await shot('02-cinema-widgets', [up(), up(), 'enter']);
    area(widgets, 'widgets');
    assert.equal(widgets.home_preview.top_expansion, 1);
    assert.deepEqual(near.home_preview.header.actions, widgets.home_preview.header.actions);
    const toolbar = await shot('02-widgets-toolbar', [up(3), 'enter']);
    area(toolbar, 'actions');
    assert.equal(toolbar.home_preview.top_expansion, 0);
    assert.deepEqual(toolbar.home_preview.button_amounts, [0, 0, 0]);
    const reversed = await shot('08-inversao-durante-transicao', [up(3), down(), up(), down(2)]);
    area(reversed, 'cinema');
    assert.equal(reversed.home_preview.expansion, 1);
    assert.equal(reversed.home_preview.top_expansion, 0);
    const inputs = await shot('15-inputs-depois-de-50-favoritos', [up(3), down(4), 'enter']);
    area(inputs, 'inputs');
    const geometry = inputs.home_preview.geometry;
    assert.equal(geometry.rows, 7);
    assert.ok(geometry.visible_rows < geometry.rows && geometry.offset > 0);
    assert.deepEqual(geometry.scroll, { up: [946, 28, 28, 28], down: null });

    const app = await shot('03-dock-primeiro-favorito', [
      ...Array(20).fill('right'),
      down(),
      up(),
      'left',
    ]);
    assert.deepEqual(app.focus, { App: ids[0] });
    assert.equal(app.home_preview.geometry.offset, 0);
    const control = app.home_preview.controls.find((c) => c.target.App === ids[3]);
    const pointer = await shot('04-ponteiro-modo-e-alvo-real', [
      `preview-motion:move:${centerOf(control)}`,
      up(),
    ]);
    area(pointer, 'cinema');
    assert.equal(pointer.pointer, true);
    const weather = widgets.home_preview.controls.find(
      (c) => c.target.EvolutionWidget === 'weather',
    );
    const motion = `preview-motion:move:${centerOf(weather)}`;
    const selected = await shot('04-ponteiro-seleciona-clima', [up(2), 'await-settled', motion]);
    assert.deepEqual(selected.focus, { EvolutionWidget: 'weather' });
    assert.equal(selected.home_preview.simulated_widget_open, null);
    const residual = `preview-motion:residual:${centerOf(control)}`;
    const realGap = 'preview-motion:move:1910:1050';
    area(
      await shot(
        '12-residual-e-horizontal-preservam-saldo',
        [up(2), residual, 'preview-wheel:horizontal', up()],
        steps(3),
      ),
      'cinema',
    );
    const cleared = await shot(
      '12-movimento-sem-alvo-descarta-saldo',
      [up(2), realGap, up()],
      steps(3),
    );
    assert.deepEqual(cleared.focus, { App: ids[0] });
    const directional = await shot(
      '12-direcional-descarta-saldo',
      [up(2), 'right', up()],
      steps(3),
    );
    assert.deepEqual(directional.focus, { App: ids[1] });
    const partial = await shot('06-limiar-2-incompleto', [up()], steps(2));
    assert.deepEqual(partial.focus, { App: ids[0] });
    area(
      await shot('06-limiar-2-com-flipped', [up(), 'preview-wheel:flipped'], steps(2)),
      'cinema',
    );
    area(await shot('07-limiar-3-rajada-6', [up(6)], steps(3)), 'widgets');
    area(await shot('09-inversao-2-de-3', [up(2), down(), down(2)], steps(3)), 'inputs');
    area(await shot('11-extremo-sem-credito', [up(30), down(2)], steps(3)), 'actions');
    area(await shot('11-extremo-sem-wrap', [down(30), up(2)], steps(3)), 'inputs');
    const skipped = await shot('02-areas-opcionais-desativadas', [up()], {
      ...config,
      evolution_home: { ...preview, show_top: false, show_center: false },
    });
    area(skipped, 'actions');
    const circular = await shot('15-cinema-circular-independente', [up(), 'right', 'left']);
    assert.equal(circular.home_preview.first, -1);
    assert.ok(circular.focus.EvolutionContent);
    for (const [name, opening] of [
      ['menu', 'apps,side-menu'],
      ['dialogo', 'menu'],
      ['settings', 'settings'],
      ['mover', 'left,hold'],
    ]) {
      const prefs =
        name === 'mover'
          ? { ...config, evolution_home: { ...preview, global_position: 'dock' } }
          : config;
      const before = await shot(`13-${name}-antes`, [opening], prefs);
      const after = await shot(`13-${name}-roda-isolada`, [opening, up(6), down(12)], prefs);
      assert.equal(before.modal, name === 'dialogo' || name === 'menu');
      assert.deepEqual(
        [after.page, after.focus, after.modal],
        [before.page, before.focus, before.modal],
      );
      assert.deepEqual(after.preferences.favorites, before.preferences.favorites);
      assert.equal(after.home_preview.moving, name === 'mover');
    }

    const check = (state, draft, saved) => {
      assert.equal(state.subpage.Wheel.steps, draft);
      assert.equal(state.preferences.evolution_home.wheel_steps, saved);
      assert.ok(state.ui_text.includes('Passos da roda por área'));
      assert.ok(state.ui_text.includes(String(draft)));
    };
    const initial = await shot('05-sensibilidade-padrao', [nav, up(6)]);
    check(initial, 1, 1);
    const step = initial.home_preview.controls.find((c) => c.target.WheelStepsStep === true);
    const draft = await shot('05-rascunho-2-por-clique', [nav, `click:${centerOf(step)}`], null);
    check(draft, 2, 1);
    const beforeSave = await fs.readFile(isolated, 'utf8');
    const canceled = await shot(
      '05-cancela-3',
      [nav, 'right', 'right', 'down', 'down', 'enter'],
      null,
    );
    assert.equal(canceled.subpage, 'Navigation');
    await shot('05-voltar-descarta', [nav, 'right', 'back'], null);
    assert.equal(await fs.readFile(isolated, 'utf8'), beforeSave);
    await fs.chmod(path.dirname(isolated), 0o500);
    try {
      const failed = await shot(
        '05-falha-de-gravacao',
        [nav, 'right', 'right', 'down', 'enter'],
        null,
      );
      check(failed, 3, 1);
      assert.equal(failed.focus, 'WheelApply');
      assert.equal(await fs.readFile(isolated, 'utf8'), beforeSave);
    } finally {
      await fs.chmod(path.dirname(isolated), 0o700);
    }
    for (const [n, edit] of [
      [3, ['right', 'right']],
      [2, ['left']],
      [1, ['left']],
    ]) {
      const applied = await shot(`05-aplica-${n}`, [nav, ...edit, 'down', 'enter'], null);
      assert.equal(applied.subpage, 'Navigation');
      assert.equal(applied.preferences.evolution_home.wheel_steps, n);
      assert.equal(JSON.parse(await fs.readFile(isolated, 'utf8')).evolution_home.wheel_steps, n);
      const reopened = await shot(`05-reabre-${n}`, [nav], null);
      check(reopened, n, n);
      const persisted = await shot(`05-limiar-${n}-salvo-ativo-na-home`, [up(n)], null);
      area(persisted, 'cinema');
    }
  }
}

async function reviewP02(scenario, capture) {
  const ids = Array.from(
    { length: 50 },
    (_, n) => `com.luma.scroll-preview.${String(n).padStart(3, '0')}`,
  );
  const nav = { show_top: false, show_center: false, dock_size: 'compact' };
  const dockMenu = 'home,settings,right' + ',down'.repeat(8) + ',enter';
  const at = (control) => `click:${control.x + control.w / 2}:${control.y + control.h / 2}`;
  for (const [name, resolution] of [
    ['1080p', '1920x1080'],
    ['720p', '1280x720'],
  ]) {
    const extra = ['--resolution', resolution, '--test-extra-apps', '50'];
    const initial = { favorites: ids, evolution_home: nav };
    const shot = (label, script, config = initial) =>
      capture(`${name}-${label}`, script, config, extra);
    for (const [count, midpoint] of [
      [1, 3],
      [2, 3],
      [3, 2],
    ]) {
      const config = { ...initial, favorites: ids.slice(0, 8 + count) };
      const down = await shot(`ultima-${count}-descer`, 'home,down', config);
      assert.equal(down.focus.App, ids[8]);
      const up = await shot(`ultima-${count}-subir`, 'home,down,up', config);
      assert.equal(up.focus.App, ids[midpoint]);
      assert.equal(up.home_preview.geometry.offset, 0);
    }

    const next = await shot('b-cruza-fileira', 'home,left' + ',right'.repeat(8));
    assert.equal(next.focus.App, ids[8]);
    const earlier = await shot('b-volta-fileira', 'home' + ',right'.repeat(8) + ',left', null);
    assert.equal(earlier.focus.App, ids[7]);
    const last = await shot('b-limite-global', 'home' + ',right'.repeat(53), null);
    assert.equal(last.focus.App, ids.at(-1));
    assert.equal(last.home_preview.geometry.rows, 7);
    assert.equal(last.home_preview.geometry.visible_rows, 5);
    assert.equal(last.home_preview.geometry.offset, 271);
    const resumed = await shot('b-retorno-inputs', 'home' + ',right'.repeat(49) + ',down,up', null);
    assert.deepEqual(resumed.focus, last.focus);
    assert.equal(resumed.home_preview.geometry.offset, last.home_preview.geometry.offset);

    const loss = await shot(
      'b12-perda-multipla',
      `home,right,right,preview-remove-apps:${ids.slice(2, 5).join('|')}`,
    );
    assert.equal(loss.focus.App, ids[5]);
    const fresh = await scenario(
      `${name}-b13-novo-processo`,
      null,
      { ...initial, hidden: [ids[0]] },
      extra,
    );
    assert.equal(fresh.focus.App, ids[1]);

    const settings = await shot('b33-padrao-b', dockMenu + ',down');
    const verify = (s, draft, saved) => {
      assert.equal(s.subpage.Dock.horizontal, draft);
      assert.equal(s.preferences.evolution_home.dock_horizontal, saved);
      assert.ok(s.ui_text.includes('Navegação horizontal'));
    };
    verify(settings, 'sequential', 'sequential');
    assert.ok(settings.ui_text.includes('B · Entre fileiras'));
    const before = await fs.readFile(isolated, 'utf8');
    const left = settings.home_preview.controls.find((c) => c.target.DockHorizontalStep === false);
    const stagedA = await shot(
      'b33-rascunho-a',
      dockMenu + ',down,await-settled,' + at(left),
      null,
    );
    verify(stagedA, 'stop', 'sequential');
    assert.ok(stagedA.ui_text.includes('A · Parar na fileira'));
    const discarded = await shot(
      'b33-descarta-a',
      dockMenu + ',down,left,down,down,down,enter',
      null,
    );
    assert.deepEqual(discarded.focus, { Setting: 17 });
    assert.equal(discarded.preferences.evolution_home.dock_horizontal, 'sequential');
    assert.equal(await fs.readFile(isolated, 'utf8'), before);

    await fs.chmod(path.dirname(isolated), 0o500);
    try {
      const failed = await shot('b33-falha-gravacao', dockMenu + ',down,right,down,enter', null);
      verify(failed, 'wrap_row', 'sequential');
      assert.deepEqual(failed.focus, 'DockApply');
      assert.equal(await fs.readFile(isolated, 'utf8'), before);
    } finally {
      await fs.chmod(path.dirname(isolated), 0o700);
    }
    const saved = await shot('b33-aplica-c', dockMenu + ',down,right,down,enter', null);
    assert.deepEqual(saved.focus, { Setting: 17 });
    assert.equal(
      JSON.parse(await fs.readFile(isolated, 'utf8')).evolution_home.dock_horizontal,
      'wrap_row',
    );
    const reopened = await shot('b33-reabre-c', dockMenu + ',down', null);
    verify(reopened, 'wrap_row', 'wrap_row');
    assert.ok(reopened.ui_text.includes('C · Circular na fileira'));
    const wrapLeft = await shot('c-circula-a-esquerda', 'home,left', null);
    assert.equal(wrapLeft.focus.App, ids[7]);
    const wrapRight = await shot('c-circula-a-direita', 'home,left,right', null);
    assert.equal(wrapRight.focus.App, ids[0]);
    const stop = await shot('a-para-na-borda', 'home,left' + ',right'.repeat(8), {
      ...initial,
      evolution_home: { ...nav, dock_horizontal: 'stop' },
    });
    assert.equal(stop.focus.App, ids[7]);
  }
}

async function reviewP01(capture) {
  const favorites = Array.from(
    { length: 50 },
    (_, i) => `com.luma.scroll-preview.${String(i).padStart(3, '0')}`,
  );
  const home = { show_top: false, show_center: false, dock_size: 'compact' };
  const geometry = (s) => s.home_preview.geometry;
  const controls = (s) => s.home_preview.controls;
  const go = (downs) => `home${',down'.repeat(downs)}`;
  for (const [name, resolution] of [
    ['1080p', '1920x1080'],
    ['720p', '1280x720'],
  ]) {
    const extra = ['--resolution', resolution, '--test-extra-apps', '50'];
    const shot = (caseName, script, ids, prefs = home) =>
      capture(`${name}-${caseName}`, script, { favorites: ids, evolution_home: prefs }, extra);
    const empty = await shot('dock-vazio', 'home', []);
    const [x, y, w, h] = geometry(empty).empty_hint;
    assert.ok(empty.ui_text.includes('Adicione favoritos em Todos os Apps.'));
    assert.ok(geometry(empty).pane[3] < 100);
    assert.equal(geometry(empty).rows, 0);
    assert.equal(controls(empty).filter((c) => c.target.EvolutionGlobal).length, 2);
    assert.ok(!controls(empty).some((c) => c.target.App));
    const clicked = await shot('orientacao-sem-acao', `home,click:${x + w / 2}:${y + h / 2}`, []);
    assert.equal(clicked.page, empty.page);
    assert.deepEqual(clicked.focus, empty.focus);
    const apps = controls(empty).find((c) => c.target.EvolutionGlobal === 'apps');
    const opened = await shot(
      'atalho-global-no-dock-vazio',
      `home,click:${apps.x + 30}:${apps.y + 30}`,
      [],
    );
    assert.equal(opened.page, 'Apps');
    const settings = controls(empty).find((c) => c.target.EvolutionGlobal === 'settings');
    const openedSettings = await shot(
      'configuracoes-no-dock-vazio',
      `home,click:${settings.x + 30}:${settings.y + 30}`,
      [],
    );
    assert.equal(openedSettings.page, 'Settings');
    const emptyInputs = await shot('inputs-no-dock-vazio', 'home,down', []);
    assert.ok(emptyInputs.focus.Input);
    assert.ok(emptyInputs.ui_text.includes('Adicione favoritos em Todos os Apps.'));
    const mixed = await shot('dock-vazio-com-atalhos-reais', 'home', [], {
      ...home,
      global_position: 'dock',
    });
    assert.ok(mixed.ui_text.includes('Adicione favoritos em Todos os Apps.'));
    assert.equal(controls(mixed).filter((c) => c.target.EvolutionGlobal).length, 2);

    for (const [rows, count] of [
      [1, 1],
      [2, 11],
      [3, 17],
      [4, 27],
      [5, 35],
    ]) {
      const ids = favorites.slice(0, count);
      const state = await shot(`dock-${rows}-fileiras`, go(rows - 1), ids);
      const layout = geometry(state);
      assert.equal(layout.offset, 0);
      assert.equal(layout.rows, rows);
      assert.equal(layout.visible_rows, rows);
      assert.equal(layout.empty_hint, null);
      assert.deepEqual(
        controls(state)
          .filter((c) => c.target.App)
          .map((c) => c.target.App),
        ids,
      );
      assert.ok(layout.cards.every((r) => r[2] === 192 && r[3] === 120));
      for (const rowY of new Set(layout.cards.map((r) => r[1]))) {
        const row = layout.cards.filter((r) => r[1] === rowY);
        assert.ok(Math.abs((row[0][0] + row.at(-1)[0] + row.at(-1)[2]) / 2 - 960) < 0.01);
      }
    }

    const atStart = await shot('dock-muitas-fileiras', go(0), favorites);
    const visible = await shot('foco-ja-visivel', go(4), favorites);
    const partial = await shot('foco-parcial-antes-do-ajuste', go(5), favorites);
    const last = await shot('ultima-fileira', go(6), favorites);
    const inputs = await shot('inputs-depois-do-dock', go(7), favorites);
    const restored = await shot('retorno-ao-dock', `${go(7)},up`, favorites);
    const bottom = geometry(atStart).viewport[1] + geometry(atStart).viewport[3];
    assert.equal(geometry(atStart).rows, 7);
    assert.equal(geometry(atStart).visible_rows, 5);
    assert.equal(geometry(visible).offset, geometry(atStart).offset);
    const before = geometry(atStart).cards[40];
    const focusedBeforeBottom = before[1] + before[3] + 41;
    assert.ok(before[1] < bottom && focusedBeforeBottom > bottom);
    assert.equal(geometry(partial).offset, focusedBeforeBottom - bottom);
    assert.equal(geometry(last).offset - geometry(partial).offset, 168);
    assert.equal(partial.focus.App, favorites[40]);
    assert.equal(last.focus.App, favorites[48]);
    assert.deepEqual(restored.focus, last.focus);
    assert.equal(geometry(restored).offset, geometry(last).offset);
    assert.deepEqual(geometry(restored).cards, geometry(last).cards);
    const input = controls(inputs).find((c) => c.target.Input);
    assert.equal(inputs.focus.Input, input.target.Input);
    assert.ok(input.y > geometry(inputs).pane[1] + geometry(inputs).pane[3]);
    assert.deepEqual(geometry(inputs).scroll, { up: [946, 28, 28, 28], down: null });
    for (const state of [visible, partial, last, inputs, restored]) {
      assert.deepEqual(state.home_preview.header.actions, atStart.home_preview.header.actions);
    }
  }
}

async function reviewR13(scenario, capture) {
  const home = 'home,preview-weather-ready,preview-widget-apps:both';
  const dockMenu = home + ',settings,right' + ',down'.repeat(8);
  const cinemaMenu = dockMenu + ',down,enter';
  const seeded = {
    aerials: { seed: 1, changed: 1795200000, index: 0 },
    greeting_name: 'Luma',
    evolution_home: { rotation_enabled: false },
  };
  const disk = () => fs.readFile(isolated, 'utf8');
  const nearly = (value, expected) =>
    assert.ok(Math.abs(value - expected) < 0.002, `${value} != ${expected}`);
  const cinema = (s) => s.home_preview.controls.filter((c) => c.target.EvolutionContent);
  const widget = (s) => s.home_preview.controls.filter((c) => c.target.EvolutionWidget);
  // The Settings preview is 776 logical pixels wide. Draw diagnostics include a
  // one-pixel damage margin on each edge, even for opaque card artwork.
  const previewSize = (width, height, percent) =>
    [width, height]
      .map((value) => Math.round(value * (776 / 1920) * (percent / 100) + 2))
      .join('x');

  for (const [resolution, viewport] of [
    ['1080p', '1920x1080'],
    ['720p', '1280x720'],
  ]) {
    const size = ['--resolution', viewport];
    const shot = (name, script, config = seeded) =>
      capture(`${resolution}-${name}`, script, config, size);
    const samples = [];
    function measure(name, s, percent, top, expanded = 0) {
      const h = s.home_preview;
      const cards = cinema(s);
      const cardHeight = (164 * percent) / 100;
      assert.equal(h.top_expansion, top);
      assert.equal(h.expansion, expanded);
      nearly(h.geometry.cinema_card_height, cardHeight);
      assert.ok(cards.length > 0);
      for (const c of cards) {
        nearly(c.h, cardHeight);
        nearly(c.w, (cardHeight * 16) / 9);
        assert.ok(c.x >= 80 && c.x + c.w <= 1840);
        assert.ok(c.y >= h.header.bottom && c.y + c.h <= h.geometry.pane[1]);
      }
      samples.push({
        scene: name,
        cinema_percent: percent,
        cinema_card: [cards[0].w, cards[0].h],
        dock_card: h.geometry.cards[0].slice(2),
        backdrop_expansion: h.expansion,
        widgets_expansion: h.top_expansion,
        selected: h.selected,
        focus: s.focus,
      });
    }

    const geometryScenes = [
      ['01-home-compacta', home, 100, 0, 0],
      ['02-area1', home + ',up,up,up', 100, 0, 0],
      ['03-area2', home + ',up,up', 50, 1, 0],
      [
        '04-area2-sem-app-clima',
        home + ',preview-widget-apps:calendar,preview-wheel:up,preview-wheel:up,enter',
        50,
        1,
        0,
      ],
      ['05-retorno-compacta', home + ',up,up,await-settled,back', 100, 0, 0],
      ['06-cinema-expandido', home + ',up,up,await-settled,down', 100, 0, 1],
    ];
    const states = [];
    for (const [name, script, percent, top, expanded] of geometryScenes) {
      const s = await shot(name, script);
      states.push(s);
      measure(name, s, percent, top, expanded);
      assert.deepEqual(s.home_preview.selected, states[0].home_preview.selected);
      assert.deepEqual(
        cinema(s).map((c) => c.target),
        cinema(states[0]).map((c) => c.target),
      );
    }
    const [compact, area1, area2, noWeather, returned, expanded] = states;
    assert.ok(compact.focus.App);
    assert.deepEqual(area1.focus, { EvolutionGlobal: 'apps' });
    assert.deepEqual(area2.focus, { EvolutionWidget: 'calendar' });
    assert.deepEqual(
      widget(area2).map((c) => c.target.EvolutionWidget),
      ['calendar', 'weather'],
    );
    assert.ok(area2.ui_text.includes('Pressione OK para abrir o aplicativo correspondente'));
    assert.deepEqual(noWeather.focus, { EvolutionArea: 'widgets' });
    assert.deepEqual(widget(noWeather), []);
    assert.deepEqual(noWeather.home_preview.simulated_widget_apps, {
      calendar: true,
      weather: false,
    });
    assert.equal(noWeather.home_preview.simulated_widget_open, null);
    assert.deepEqual(returned.focus, compact.focus);
    assert.deepEqual(cinema(returned), cinema(compact));
    assert.ok(expanded.focus.EvolutionContent);
    nearly(compact.home_preview.geometry.cards[0][2], 224);
    nearly(compact.home_preview.geometry.cards[0][3], 140);
    nearly(expanded.home_preview.geometry.cards[0][2], 224 * 0.4);
    nearly(expanded.home_preview.geometry.cards[0][3], 140 * 0.4);

    const twoRows = await shot(
      'area2-dock-grande-duas-fileiras',
      home + ',down,preview-wheel:up,preview-wheel:up,right',
      {
        ...seeded,
        favorites: compact.inventory.slice(0, 16).map((app) => app.id),
        evolution_home: { ...seeded.evolution_home, dock_size: 'large' },
      },
    );
    measure('area2-dock-grande-duas-fileiras', twoRows, 50, 1);
    assert.deepEqual(twoRows.focus, { EvolutionWidget: 'calendar' });
    assert.ok(twoRows.home_preview.geometry.cards.length > 6);

    const weather = widget(area2)[1];
    const position = `${weather.x + weather.w / 2}:${weather.y + weather.h / 2}`;
    const wheelToWidgets = home + ',preview-wheel:up,preview-wheel:up,await-settled';
    const hover = await shot('area2-hover-clima', wheelToWidgets + ',preview-hover:' + position);
    assert.deepEqual(hover.focus, { EvolutionWidget: 'weather' });
    assert.deepEqual(widget(hover), widget(area2));
    const clicked = await shot('area2-clique-clima', wheelToWidgets + ',click:' + position);
    assert.equal(clicked.home_preview.simulated_widget_open, 'weather');
    const interrupted = await scenario(
      `${resolution}-area2-transicao`,
      home + ',up,await-settled,up,preview-motion-capture',
      seeded,
      [...size, '--frames', '3000'],
    );
    assert.ok(
      interrupted.home_preview.top_expansion > 0 && interrupted.home_preview.top_expansion < 1,
    );
    const quick = await shot(
      'area2-transicoes-interrompidas',
      home + ',up,up,down,up,up,down,back',
    );
    measure('area2-transicoes-interrompidas', quick, 100, 0);
    assert.deepEqual(quick.focus, compact.focus);
    assert.deepEqual(quick.home_preview.selected, compact.home_preview.selected);

    const openDock = dockMenu + ',enter';
    const dockDefault = await shot('07-menu-dock', openDock);
    const dockStep = await shot('07a-dock-passo', openDock + ',right');
    const dockMax = await shot('07b-dock-maximo', openDock + ',right'.repeat(15));
    for (const [s, percent] of [
      [dockDefault, 40],
      [dockStep, 45],
      [dockMax, 100],
    ]) {
      assert.equal(s.subpage.Dock.scale, percent);
      assert.equal(s.preferences.evolution_home.cinema_dock_scale, 40);
      assert.ok(s.ui_text.includes(`${percent}%`));
      assert.ok(s.ui_text.includes('Escala do Dock com Cinema expandido'));
      assert.ok(
        s.home_preview.draw_commands.some((d) =>
          new RegExp(`^shape at \\d+,\\d+ ${previewSize(224, 140, percent)}$`).test(d),
        ),
      );
    }

    const cinemaDefault = await shot('08-menu-cinema', cinemaMenu);
    const original = await disk();
    const verifyDraft = (s, percent, stored) => {
      assert.equal(s.subpage.Cinema.scale, percent);
      assert.equal(s.preferences.evolution_home.compact_cinema_scale, stored);
      assert.ok(s.ui_text.includes(`${percent}%`));
      assert.ok(
        s.home_preview.draw_commands.some((d) =>
          new RegExp(`^image at \\d+,\\d+ ${previewSize((164 * 16) / 9, 164, percent)}$`).test(d),
        ),
      );
    };
    verifyDraft(cinemaDefault, 100, 100);
    assert.equal(JSON.parse(original).evolution_home.compact_cinema_scale, undefined);
    const clickLeft = cinemaDefault.home_preview.controls.find(
      (c) => c.target.CinemaScaleStep === false,
    );
    const clickStep = `,click:${clickLeft.x + clickLeft.w / 2}:${clickLeft.y + clickLeft.h / 2}`;
    verifyDraft(
      await shot('08a-cinema-passo-ponteiro', cinemaMenu + ',await-settled' + clickStep, null),
      90,
      100,
    );
    verifyDraft(await shot('08b-cinema-minimo', cinemaMenu + ',left'.repeat(8), null), 40, 100);
    assert.equal(await disk(), original);

    const applied = await shot(
      '08c-cinema-aplicado',
      cinemaMenu + ',left,left,left,down,enter',
      null,
    );
    assert.deepEqual(applied.focus, { Setting: 20 });
    const saved = await disk();
    assert.equal(JSON.parse(saved).evolution_home.compact_cinema_scale, 70);
    assert.equal(JSON.parse(saved).evolution_home.cinema_dock_scale, 40);
    assert.equal(JSON.parse(saved).greeting_name, seeded.greeting_name);
    const restart = await shot('08d-cinema-reaberto', home, null);
    measure('08d-cinema-reaberto', restart, 70, 0);

    const restore = cinemaMenu + ',down,down,enter';
    const prompt = await shot('08e-cinema-confirmar-restauracao', restore, null);
    assert.equal(prompt.subpage.Cinema.prompt, 'Restore');
    assert.ok(
      prompt.ui_text.includes('Restaurar o tamanho dos cards do Cinema para 100% nesta prévia?'),
    );
    const resetDraft = restore + ',up,enter';
    verifyDraft(await shot('08f-cinema-restaurado-na-previa', resetDraft, null), 100, 70);
    assert.equal(await disk(), saved);
    const canceled = await shot('08g-cinema-cancelado', resetDraft + ',down,down,down,enter', null);
    assert.deepEqual(canceled.focus, { Setting: 20 });
    assert.equal(await disk(), saved);
    const unsaved = await shot('08h-cinema-voltar-com-rascunho', resetDraft + ',back', null);
    assert.equal(unsaved.subpage.Cinema.prompt, 'Unsaved');
    for (const title of ['Aplicar', 'Descartar', 'Continuar editando'])
      assert.ok(unsaved.ui_text.includes(title));
    verifyDraft(
      await shot('08i-cinema-continuar-editando', resetDraft + ',back,enter', null),
      100,
      70,
    );
    const discard = await shot('08j-cinema-descartado', resetDraft + ',back,up,enter', null);
    assert.deepEqual(discard.focus, { Setting: 20 });
    assert.equal(await disk(), saved);

    await fs.chmod(path.dirname(isolated), 0o500);
    try {
      const failed = await shot('08k-cinema-falha-de-gravacao', resetDraft + ',down,enter', null);
      verifyDraft(failed, 100, 70);
      assert.equal(await disk(), saved);
    } finally {
      await fs.chmod(path.dirname(isolated), 0o700);
    }
    const restored = await shot(
      '08l-cinema-restauracao-aplicada',
      resetDraft + ',back,up,up,enter,home',
      null,
    );
    measure('08l-cinema-restauracao-aplicada', restored, 100, 0);
    assert.equal(JSON.parse(await disk()).evolution_home.compact_cinema_scale, 100);

    const minimum = {
      ...seeded,
      evolution_home: { ...seeded.evolution_home, compact_cinema_scale: 40 },
    };
    const minimumHome = await shot('compacta-personalizada-40', home, minimum);
    const savedMinimum = await disk();
    const minimumWidgets = await shot('area2-com-40-salvo', home + ',up,up', null);
    const minimumReturn = await shot('retorno-com-40-salvo', home + ',up,up,back', null);
    measure('compacta-personalizada-40', minimumHome, 40, 0);
    measure('area2-com-40-salvo', minimumWidgets, 50, 1);
    measure('retorno-com-40-salvo', minimumReturn, 40, 0);
    assert.equal(await disk(), savedMinimum);

    await fs.writeFile(
      path.join(output, `${resolution}-medidas.json`),
      JSON.stringify(samples, null, 2),
    );
    const sheets = [
      [
        'comparativo',
        [
          ['01-home-compacta', '1. Home compacta · cinema 100%'],
          ['02-area1', '2. Área 1 · cinema compacto 100%'],
          ['03-area2', '3. Área 2 · cinema compacto 50%'],
          ['04-area2-sem-app-clima', '4. Sem app de clima · cinema compacto 50%'],
          ['05-retorno-compacta', '5. Voltar · cinema compacto 100%'],
          ['06-cinema-expandido', '6. Cinema expandido · Dock 40%'],
          ['07-menu-dock', '7. Interface → Dock · padrão 40%'],
          ['08-menu-cinema', '8. Interface → Cinema · padrão 100%'],
        ],
      ],
      [
        'comparativo-configuracoes',
        [
          ['07-menu-dock', 'Dock · 40% padrão'],
          ['07b-dock-maximo', 'Dock · 100% máximo · passos de 5 p.p.'],
          ['08d-cinema-reaberto', 'Cinema · 70% aplicado e reaberto'],
          ['08e-cinema-confirmar-restauracao', 'Cinema · confirmar restauração'],
          ['08f-cinema-restaurado-na-previa', 'Cinema · 100% na prévia · 70% salvo'],
          ['08h-cinema-voltar-com-rascunho', 'Cinema · Voltar com edição pendente'],
        ],
      ],
    ];
    for (const [name, shots] of sheets) await r13ContactSheet(name, resolution, viewport, shots);
  }
}

async function r13ContactSheet(name, resolution, viewport, scenes) {
  const [width, height] = viewport.split('x').map((value) => Number(value) / 2);
  const caption = 36;
  const tiles = await Promise.all(
    scenes.map(async ([scene, title], index) => {
      const left = (index % 2) * width;
      const top = Math.floor(index / 2) * (height + caption);
      const input = await sharp(path.join(output, `${resolution}-${scene}.png`))
        .resize(width, height)
        .png()
        .toBuffer();
      return [
        { input, top: top + caption, left },
        {
          input: {
            text: {
              text: `<span foreground="#f3f7fb">${title}</span>`,
              font: 'DejaVu Sans 20',
              rgba: true,
            },
          },
          top: top + 7,
          left: left + 16,
        },
      ];
    }),
  );
  await sharp({
    create: {
      width: width * 2,
      height: (scenes.length / 2) * (height + caption),
      channels: 4,
      background: '#18212f',
    },
  })
    .composite(tiles.flat())
    .png()
    .toFile(path.join(output, `${name}-${resolution}.png`));
}

function compareShots(name, shots, captions, width, height) {
  return sharp({
    create: {
      width: width * shots.length,
      height: height + 54,
      channels: 4,
      background: '#18212f',
    },
  })
    .composite(
      shots.flatMap((shot, index) => [
        {
          input: {
            text: {
              text: `<span foreground="#f3f7fb">${captions[index]}</span>`,
              font: 'DejaVu Sans 32',
              rgba: true,
            },
          },
          left: index * width + 32,
          top: 11,
        },
        { input: path.join(output, shot + '.png'), left: index * width, top: 54 },
      ]),
    )
    .png()
    .toFile(path.join(output, name + '.png'));
}

async function reviewArea2Selection(capture) {
  const fixture = 'home,preview-weather-ready,preview-widget-apps:both,preview-cinema-size:100';
  const open = fixture + ',up,up';
  const wheel = ',preview-wheel:up,preview-wheel:up';
  const live = { evolution_home: { rotation_enabled: false } };
  const hint = 'Pressione OK para abrir o aplicativo correspondente';
  const right = ([x, , width]) => x + width;
  const bottom = ([, y, , height]) => y + height;
  const draw = (s) => s.home_preview.header_draws;
  const hints = (s) => draw(s).filter((d) => d.kind === 'text' && d.value === hint);
  const frames = (s) => draw(s).filter((d) => d.kind === 'shape' && d.rect[2] === 834);
  const targets = (s) => s.home_preview.controls.filter((c) => c.target.EvolutionWidget);

  function checkSelection(s, app) {
    assert.deepEqual(s.focus, { EvolutionWidget: app });
    const { clock, weather } = s.home_preview.header;
    const bounds = { calendar: clock, weather };
    assert.deepEqual(clock.slice(1), weather.slice(1));
    assert.equal(targets(s).length, 2);
    for (const target of targets(s)) {
      assert.deepEqual(
        [target.x, target.y, target.w, target.h],
        bounds[target.target.EvolutionWidget],
      );
    }
    const [x, y, w, h] = bounds[app];
    assert.deepEqual(
      frames(s).map((d) => d.rect),
      [[x - 5, y - 5, w + 10, h + 10]],
    );
    assert.equal(hints(s).length, 1);
    const [label] = hints(s);
    assert.ok(label.rect[1] > y + h - 25 && bottom(label.rect) <= y + h);
    const margin = app === 'calendar' ? label.rect[0] - x : x + w - right(label.rect);
    assert.ok(Math.abs(margin - 10) < 0.02);
    const internal = draw(s).filter(
      (d) =>
        d.kind === 'text' &&
        d.value !== hint &&
        d.rect[0] >= x &&
        d.rect[0] < x + w &&
        d.rect[1] >= y &&
        d.rect[1] < y + h,
    );
    assert.ok(internal.length > 10);
    assert.ok(internal.every((d) => bottom(d.rect) <= label.rect[1] && right(d.rect) <= x + w));
    assert.ok(
      s.home_preview.controls.every(
        (c) =>
          c.target.EvolutionWidget ||
          c.x + c.w <= x ||
          c.x >= x + w ||
          c.y + c.h <= y ||
          c.y >= y + h,
      ),
    );
    for (const label of app === 'calendar'
      ? ['20:41', 'Boa noite.', 'Sexta-feira, 20 de novembro', 'Novembro 2026', '30']
      : ['24°C · Parcialmente nublado', 'Por hora · simulação', 'Próximos dias · simulação']) {
      assert.ok(
        internal.some((d) => d.value === label),
        label,
      );
    }
  }

  for (const [resolution, width, height] of [
    ['1080p', 1920, 1080],
    ['720p', 1280, 720],
  ]) {
    const names = ['01-calendario-unificado', '02-clima', '03-sem-app-clima'].map(
      (name) => resolution + '-' + name,
    );
    const size = ['--resolution', width + 'x' + height];
    const calendar = await capture(names[0], open, live, size);
    const weather = await capture(names[1], open + ',right', live, size);
    checkSelection(calendar, 'calendar');
    checkSelection(weather, 'weather');
    assert.deepEqual(calendar.home_preview.controls, weather.home_preview.controls);
    assert.deepEqual(calendar.home_preview.header, weather.home_preview.header);
    const unavailable = await capture(
      names[2],
      fixture + ',preview-widget-apps:calendar' + wheel + ',await-settled,click:160:145,enter',
      live,
      size,
    );
    assert.deepEqual(unavailable.focus, { EvolutionArea: 'widgets' });
    assert.equal(unavailable.home_preview.top_expansion, 1);
    assert.deepEqual(unavailable.home_preview.simulated_widget_apps, {
      calendar: true,
      weather: false,
    });
    assert.equal(unavailable.home_preview.simulated_widget_open, null);
    assert.ok(unavailable.ui_text.includes('Novembro 2026'));
    assert.ok(unavailable.ui_text.includes('Por hora · simulação'));
    assert.deepEqual(targets(unavailable), []);
    assert.deepEqual(frames(unavailable), []);
    assert.deepEqual(hints(unavailable), []);
    await compareShots(
      'comparativo-' + resolution,
      names,
      ['Calendário em foco', 'Clima em foco', 'Sem app de clima · sem seleção'],
      width,
      height,
    );
  }

  const neutral = await capture('home-neutra-clock-informativo', fixture + ',click:160:145', live);
  assert.ok(neutral.focus.App);
  assert.equal(neutral.home_preview.top_expansion, 0);
  assert.equal(neutral.home_preview.simulated_widget_open, null);
  const unselected = await capture('area2-roda-sem-selecao', fixture + wheel + ',enter', live);
  assert.equal(unselected.home_preview.top_expansion, 1);
  assert.equal(unselected.home_preview.simulated_widget_open, null);
  const back = await capture('area2-voltar', open + ',right,back', live);
  assert.ok(back.focus.App);
  const lost = await capture(
    'area2-app-clima-removido',
    open + ',right,preview-widget-apps:calendar,enter',
    live,
  );
  assert.deepEqual(lost.focus, { EvolutionArea: 'widgets' });
  assert.equal(lost.home_preview.simulated_widget_open, null);
  for (const s of [neutral, unselected, back, lost]) {
    assert.deepEqual(frames(s), []);
    assert.deepEqual(hints(s), []);
  }

  const hover = await capture(
    'area2-hover-relogio',
    fixture + wheel + ',await-settled,preview-hover:160:145',
    live,
  );
  checkSelection(hover, 'calendar');
  assert.equal(hover.home_preview.simulated_widget_open, null);
  for (const [app, input] of [
    ['calendar', 'right,left,enter'],
    ['weather', 'right,enter'],
    ['calendar', 'await-settled,click:160:145'],
    ['weather', 'await-settled,click:1760:145'],
  ]) {
    const s = await capture(
      `area2-abrir-${app}-${input.endsWith('enter') ? 'ok' : 'clique'}`,
      open + ',' + input,
      live,
    );
    assert.equal(s.home_preview.simulated_widget_open, app);
    assert.equal(s.modal, false);
  }
  const sixWeeks = await capture('720p-calendario-seis-semanas', open, live, [
    '--resolution',
    '1280x720',
    '--time',
    '2026-08-31T20:41:00-03:00',
  ]);
  assert.ok(sixWeeks.ui_text.includes('Agosto 2026'));
  const lastDay = draw(sixWeeks).find((d) => d.kind === 'text' && d.value === '31');
  assert.ok(lastDay && bottom(lastDay.rect) <= hints(sixWeeks)[0].rect[1]);
}

async function runHomeRegression(scenario, capture) {
  const global = (state, name) => {
    const matches = state.home_preview.controls.filter(
      (control) => control.target.EvolutionGlobal === name,
    );
    assert.equal(matches.length, 1, `${name} must have exactly one hit target`);
    return matches[0];
  };
  const at = (control) => `click:${control.x + control.w / 2}:${control.y + control.h / 2}`;
  let favorites;
  if (process.env.LUMA_HOME_REVIEW_ONLY !== '1') {
    const compact = await scenario('home-compact', 'home');
    assert.equal(compact.home_preview.expanded, false);
    assert.ok(compact.focus.App);
    assert.ok(compact.ui_text.some((v) => v.includes('CONTEÚDO FICTÍCIO')));
    assert.ok(global(compact, 'apps').y < 100);
    assert.ok(global(compact, 'settings').y < 100);
    assert.ok(compact.home_preview.controls.every((c) => !c.target.Input));
    const expanded = await scenario('home-expanded', 'home,up,right');
    assert.equal(expanded.home_preview.expanded, true);
    assert.equal(expanded.home_preview.selected, 1);
    const firstApp = (state) => state.home_preview.controls.find((c) => c.target.App);
    assert.ok(firstApp(expanded).w < firstApp(compact).w);
    assert.ok(Math.abs(firstApp(expanded).w / firstApp(compact).w - 0.4) < 0.001);
    assert.ok(expanded.home_preview.controls.every((c) => !c.target.Input));
    assert.ok(expanded.ui_text.some((v) => v.startsWith('Boa noite, 20:41 - ')));
    for (const action of ['apps', 'settings']) {
      const control = global(expanded, action);
      const pixels = await sharp(path.join(output, 'home-expanded.png'))
        .extract({
          left: Math.round(control.x + 12),
          top: Math.round(control.y + 12),
          width: 32,
          height: 28,
        })
        .toBuffer();
      const iconStats = await sharp(pixels).stats();
      assert.ok(
        iconStats.channels.slice(0, 3).every((channel) => channel.max > 220),
        `${action} icon must actually be drawn`,
      );
    }
    await scenario('home-weather-transparent', 'home');
    await scenario('home-compact-header', 'home,up');
    await scenario('home-inputs-hidden', 'home');
    const back = await scenario('home-back', 'home,up,right,back');
    assert.equal(back.home_preview.expanded, false);
    assert.deepEqual(back.focus, compact.focus);
    assert.equal(back.home_preview.selected, 1);
    assert.equal(firstApp(back).w, firstApp(compact).w);
    const circular = await scenario('home-circular', 'home,up,left');
    assert.equal(circular.home_preview.selected, 7);
    assert.equal(circular.home_preview.first, -1);
    assert.equal(circular.modal, false);
    const clean = await scenario('home-widgets-off', 'home,up', {
      evolution_home: { show_top: false, show_center: false },
    });
    assert.equal(clean.focus.EvolutionGlobal, 'apps');
    assert.ok(global(clean, 'settings').y < 100);
    assert.ok(
      !clean.ui_text.some(
        (v) => v.includes('CONTEÚDO FICTÍCIO') || v.includes('Clima indisponível'),
      ),
    );
    const noHeader = await scenario('home-expanded-widgets-off', 'home,up', {
      evolution_home: { show_top: false },
    });
    assert.ok(!noHeader.home_preview.controls.some((c) => c.target.EvolutionWidget));
    assert.ok(global(noHeader, 'settings').y < 100);
    const top = await scenario('home-top-actions', 'home,up,up,up,right');
    assert.equal(top.focus.EvolutionGlobal, 'settings');
    assert.ok(top.ui_text.includes('Configurações do Luma Evolution'));
    const appsPage = await scenario('apps-from-top', `home,${at(global(compact, 'apps'))}`);
    assert.equal(appsPage.page, 'Apps');
    const navigate = `right,${Array(7).fill('down').join(',')},enter`;
    const changeDock = await scenario(
      'home-dock-actions',
      `home,${at(global(compact, 'settings'))},${navigate},down,enter,back,back`,
    );
    assert.equal(changeDock.page, 'Home');
    assert.equal(changeDock.focus.EvolutionGlobal, 'settings');
    assert.equal(changeDock.preferences.evolution_home.global_position, 'dock');
    assert.ok(global(changeDock, 'apps').y > 300 && global(changeDock, 'settings').y > 300);
    assert.deepEqual(changeDock.apps, compact.apps);
    const saved = JSON.parse(await fs.readFile(isolated, 'utf8'));
    assert.equal(saved.evolution_home.global_position, 'dock');
    const changeTop = await scenario(
      'home-top-restored',
      `home,${at(global(changeDock, 'settings'))},${navigate},up,enter,back,back`,
      null,
    );
    assert.equal(changeTop.focus.EvolutionGlobal, 'settings');
    assert.equal(changeTop.preferences.evolution_home.global_position, 'top');
    assert.ok(global(changeTop, 'apps').y < 100 && global(changeTop, 'settings').y < 100);
    assert.deepEqual(changeTop.apps, compact.apps);
    const expandedDock = await scenario('home-expanded-dock-actions', 'home,up', {
      evolution_home: { global_position: 'dock' },
    });
    assert.ok(global(expandedDock, 'apps').y > 300 && global(expandedDock, 'settings').y > 300);
    await scenario(
      'home-720p-expanded-dock-actions',
      'home,up',
      {
        evolution_home: { global_position: 'dock' },
      },
      ['--resolution', '1280x720'],
    );
    const target = compact.home_preview.controls.find((control) =>
      control.target.EvolutionContent?.endsWith(':2'),
    );
    assert.ok(target);
    const click = await scenario(
      'home-click',
      `home,click:${target.x + target.w / 2}:${target.y + target.h / 2}`,
    );
    assert.equal(click.modal, false);
    assert.equal(click.home_preview.selected, 2);
    assert.ok(click.ui_text.some((value) => value.includes('sem reprodução')));
    await scenario('home-720p', 'home', {}, ['--resolution', '1280x720']);
    await scenario('home-720p-expanded', 'home,up', {}, ['--resolution', '1280x720']);
    const inputs = await scenario('home-inputs-revealed', 'home,down');
    assert.equal(inputs.focus.Input, 'com.webos.app.hdmi1');
    const inputControls = inputs.home_preview.controls.filter((control) => control.target.Input);
    assert.equal(inputControls.length, 2);
    assert.ok(inputControls.every((input) => input.y > firstApp(inputs).y + firstApp(inputs).h));
    assert.ok(inputs.ui_text.includes('Entrada simulada'));
    const noInputs = await scenario('home-no-inputs', 'home,down', {
      input_modes: { 'com.webos.app.hdmi1': 'Off', 'com.webos.app.hdmi2': 'Off' },
    });
    assert.ok(noInputs.home_preview.controls.every((control) => !control.target.Input));
    assert.ok(firstApp(noInputs).y > firstApp(inputs).y);
    favorites = compact.inventory.slice(0, 15).map((app) => app.id);
    const second = await scenario('home-second-row', 'home,down', { favorites });
    assert.equal(second.focus.App, favorites[7]);
    assert.equal(second.home_scroll, 0);
    for (const [size, width, name] of [
      ['compact', 192, 'small'],
      ['standard', 224, 'standard'],
      ['large', 256, 'large'],
    ]) {
      const dock = await scenario(`home-dock-${name}`, 'home', {
        evolution_home: { dock_size: size },
      });
      assert.equal(firstApp(dock).w, width);
      assert.equal(firstApp(dock).w / firstApp(dock).h, 1.6);
      const cinema = await scenario(`home-expanded-${name}`, 'home,up', {
        evolution_home: { dock_size: size },
      });
      assert.ok(Math.abs(firstApp(cinema).w - width * 0.4) < 0.001);
    }
    // 15 favorites occupy three standard rows; the viewport must visit all of them.
    const third = await scenario('home-dock-three-rows', 'home,down,down', { favorites });
    assert.equal(third.focus.App, favorites[14]);
    assert.ok(third.home_preview.controls.every((c) => !c.target.Input));
    assert.ok(third.home_preview.controls.some((c) => c.target.App === favorites[0]));
    const lastInputs = await scenario('home-inputs-after-three-rows', 'home,down,down,down', {
      favorites,
    });
    assert.ok(lastInputs.focus.Input);
    const restored = await scenario('home-three-rows-restored', 'home,down,down,down,up,up,up', {
      favorites,
    });
    assert.equal(restored.focus.App, favorites[3]);
    assert.ok(restored.home_preview.controls.every((c) => !c.target.Input));
    const dockRows = await scenario(
      'home-dock-actions-three-rows',
      'home,down,down,down',
      {
        favorites,
        evolution_home: { global_position: 'dock', global_slots: [14, 16] },
      },
      ['--resolution', '1280x720'],
    );
    assert.ok(dockRows.focus.Input);
    assert.ok(global(dockRows, 'apps').y > 300 && global(dockRows, 'settings').y > 300);
    const legacy = await scenario('home-legacy-migrated', 'home', {
      favorites,
      greeting_name: 'Ana',
      evolution_home: { dock_size: 'large', square_icons: true },
    });
    assert.equal(firstApp(legacy).w / firstApp(legacy).h, 1.6);
    // Change a real setting to force an atomic preferences save, then reload a new process.
    await scenario(
      'home-migration-saved',
      `home,${at(global(legacy, 'settings'))},right,enter,down,enter`,
      null,
    );
    const migrated = JSON.parse(await fs.readFile(isolated, 'utf8'));
    assert.equal(migrated.evolution_home.square_icons, undefined);
    assert.deepEqual(migrated.favorites, favorites);
    assert.equal(migrated.greeting_name, 'Ana');
    assert.equal(migrated.evolution_home.dock_size, 'large');
    const restart = await scenario('home-migration-restarted', 'home', null);
    assert.equal(firstApp(restart).w / firstApp(restart).h, 1.6);
    assert.deepEqual(restart.apps, legacy.apps);
    const rapid = await scenario(
      'home-interrupted',
      'home,up,right,down,up,left,down,up,right,back',
    );
    assert.equal(rapid.home_preview.expanded, false);
    assert.deepEqual(rapid.focus, compact.focus);
    const reduced = await scenario('home-reduced-motion', 'home,up', { reduced_motion: true });
    assert.equal(reduced.home_preview.expansion, 1);
    for (const profile of ['reduced', 'none']) {
      const instant = await scenario(`home-motion-${profile}`, 'home,up', {
        evolution_home: { motion_profile: profile },
      });
      assert.equal(instant.home_preview.expansion, 1);
    }
    const clickExpanded = await scenario('home-expanded-dock-click', 'home,up,preview-click-dock');
    assert.ok(clickExpanded.focus.App);
  } else {
    const inventory = await scenario('review-inventory', 'home');
    favorites = inventory.inventory.slice(0, 15).map((app) => app.id);
  }
  function aligned(state) {
    const {
      pane: [px, py, pw, ph],
      cards,
    } = state.home_preview.geometry;
    const left = Math.min(...cards.map((r) => r[0]));
    const right = Math.max(...cards.map((r) => r[0] + r[2]));
    const top = Math.min(...cards.map((r) => r[1]));
    const bottom = Math.max(...cards.map((r) => r[1] + r[3]));
    assert.ok(Math.abs(left - px - (px + pw - right)) < 0.01);
    assert.ok(Math.abs(top - py - (py + ph - bottom)) < 0.01);
    assert.ok(Math.abs((left + right) / 2 - 960) < 0.01);
  }
  for (const [name, script, config] of [
    ['dock-standard-aligned', 'home', {}],
    ['dock-compact-aligned', 'home', { evolution_home: { dock_size: 'compact' } }],
    ['dock-large-aligned', 'home', { evolution_home: { dock_size: 'large' } }],
    ['dock-incomplete-row', 'home', { favorites: favorites.slice(0, 4) }],
    ['dock-two-rows-aligned', 'home,down', { favorites }],
    ['dock-three-rows-aligned', 'home,down,down', { favorites }],
    ['dock-actions-aligned', 'home', { evolution_home: { global_position: 'dock' } }],
    ['dock-cinema-expanded', 'home,up', {}],
  ])
    aligned(await capture(name, script, config));
  aligned(
    await capture('dock-compact-full-row', 'home', {
      favorites: favorites.slice(0, 8),
      evolution_home: { dock_size: 'compact' },
    }),
  );
  for (const size of ['compact', 'standard', 'large']) {
    aligned(
      await capture(
        `dock-${size}-720p`,
        'home,down',
        { favorites, evolution_home: { dock_size: size } },
        ['--resolution', '1280x720'],
      ),
    );
  }
  const end = (r) => r[0] + r[2];
  const feet = (r) => r[1] + r[3];
  const controls = (s) => s.home_preview.controls;
  const geometry = (s) => s.home_preview.geometry;
  const header = (s) => s.home_preview.header;
  const draws = (s) => s.home_preview.header_draws;
  const noInputs = (s) => assert.ok(!controls(s).some((c) => c.target.Input));
  const fixture = 'home,preview-weather-ready,preview-widget-apps:both,preview-cinema-size:100';
  const openWidgets = fixture + ',up,up';
  const size720 = ['--resolution', '1280x720'];
  const live = {
    aerials: { changed: 1795200000, index: 0, seed: 1 },
    evolution_home: { rotation_enabled: false },
  };
  const actionLabel = ['Todos os Apps', 'Configurações do Luma Evolution'];
  const chosen = (s, action) => (action === 0 ? global(s, 'apps') : global(s, 'settings'));

  function checkHeader(s) {
    const { actions, clock, weather, actions_bottom: bottom } = header(s);
    assert.ok(actions.length === 0 || actions.every((r) => feet(r) <= bottom));
    assert.equal(clock[1], weather[1]);
    assert.ok(weather[1] > bottom);
    assert.equal(clock[0], 80);
    assert.equal(end(weather), 1840);
    assert.ok(end(clock) < weather[0]);
    if (actions.length) assert.equal(end(actions.at(-1)), end(weather));
    const right = draws(s).filter(
      (d) =>
        d.kind === 'text' &&
        d.rect[1] >= weather[1] &&
        d.rect[0] >= weather[0] &&
        d.color[3] > 0.99,
    );
    assert.ok(right.length > 0);
    for (const line of right.filter((d) => d.rect[1] < weather[1] + 83)) {
      assert.ok(Math.abs(end(line.rect) - 1840) < 0.02, line.value);
    }
    assert.ok(
      draws(s).some((d) => d.kind === 'text' && d.value === '20:41' && d.rect[0] === 80) ||
        s.home_preview.expanded ||
        s.preferences.language !== 'pt',
    );
  }

  function checkActions(s, selected = null) {
    const { actions, clock } = header(s);
    const top = draws(s).filter((d) => d.rect[0] > 1000 && d.rect[1] < clock[1]);
    assert.ok(top.every((d) => d.kind === 'image' || d.kind === 'text'));
    const icons = top.filter((d) => d.kind === 'image' && d.color[3] === 1);
    assert.equal(icons.length, 2);
    for (const [i, icon] of icons.entries()) {
      assert.equal(icon.rect[2], i === selected ? 54 : 32);
      assert.equal(icon.rect[3], icon.rect[2]);
      assert.equal(icon.rect[0] + icon.rect[2] / 2, actions[i][0] + actions[i][2] / 2);
    }
    assert.ok(icons[0].rect[0] + icons[0].rect[2] < icons[1].rect[0]);
    const labels = top.filter((d) => d.kind === 'text');
    if (selected === null) assert.equal(labels.length, 0);
    else {
      assert.deepEqual(
        labels.map((d) => d.value),
        [actionLabel[selected]],
      );
      assert.ok(labels[0].rect[1] > feet(icons[selected].rect));
      assert.ok(feet(labels[0].rect) <= clock[1]);
      assert.ok(
        top.some(
          (d) =>
            d.kind === 'image' &&
            d.rect[2] === 54 &&
            d.color[0] > 0 &&
            d.color[3] > 0 &&
            d.color[3] < 1,
        ),
      );
      assert.equal(chosen(s, selected).w, 60);
    }
    return icons;
  }

  async function pixelIcon(name, control, scale = 1) {
    const left = Math.round(control.x * scale),
      top = Math.round(control.y * scale);
    const { data, info } = await sharp(path.join(output, name + '.png'))
      .extract({
        left,
        top,
        width: Math.round(control.w * scale),
        height: Math.round(control.h * scale),
      })
      .removeAlpha()
      .raw()
      .toBuffer({ resolveWithObject: true });
    let minX = info.width,
      minY = info.height,
      maxX = -1,
      maxY = -1;
    for (let y = 0; y < info.height; y++) {
      for (let x = 0; x < info.width; x++) {
        const p = (y * info.width + x) * 3;
        if (data[p] < 222 || data[p + 1] < 222 || data[p + 2] < 222) continue;
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
        minY = Math.min(minY, y);
        maxY = Math.max(maxY, y);
      }
    }
    assert.ok(maxX > minX && maxY > minY);
    return [left + minX, top + minY, maxX - minX + 1, maxY - minY + 1];
  }

  function both(s) {
    assert.equal(s.page, 'Home');
    assert.equal(s.modal, false);
    assert.equal(s.home_preview.expanded, false);
    assert.equal(s.home_preview.top_expansion, 1);
    assert.ok(
      s.ui_text.includes(s.preferences.language === 'zh' ? '2026年十一月' : 'Novembro 2026'),
    );
    assert.ok(
      s.ui_text.some((label) =>
        label.includes(s.preferences.language === 'zh' ? 'São Paulo' : 'Clima simulado'),
      ),
    );
    const calendar = draws(s).filter(
      (d) => d.kind === 'text' && /^\d{1,2}$/.test(d.value) && d.rect[0] < 960,
    );
    assert.equal(calendar.length, 30);
    assert.ok(calendar.every((d) => end(d.rect) <= 960 && feet(d.rect) <= header(s).bottom));
    const weather = draws(s).filter(
      (d) => d.kind === 'text' && d.rect[0] >= 1016 && d.rect[1] >= header(s).calendar[1],
    );
    assert.equal(weather.filter((d) => d.value.includes('°')).length, 9);
    assert.ok(weather.every((d) => end(d.rect) <= 1840.01 && feet(d.rect) <= header(s).bottom));
    assert.ok(!controls(s).some((c) => c.target.EvolutionArea));
    assert.ok(controls(s).some((c) => c.target.EvolutionContent));
    assert.ok(controls(s).some((c) => c.target.App));
    assert.ok(geometry(s).center[1] > header(s).bottom);
    assert.ok(feet(geometry(s).center) < geometry(s).pane[1]);
    noInputs(s);
    checkHeader(s);
  }

  const neutral = await capture('01-home-neutral-five-areas', fixture, live);
  const idle = await capture('02-top-actions-idle-no-background', fixture, live);
  checkHeader(neutral);
  checkActions(idle);
  noInputs(neutral);
  const appsName = '03-top-apps-focused-no-background';
  const settingsName = '04-top-settings-focused-no-background';
  const appsFocus = await capture(appsName, openWidgets + ',up', live);
  const settingsFocus = await capture(settingsName, openWidgets + ',up,right', live);
  assert.equal(appsFocus.focus.EvolutionGlobal, 'apps');
  assert.equal(settingsFocus.focus.EvolutionGlobal, 'settings');
  checkActions(appsFocus, 0);
  checkActions(settingsFocus, 1);
  const px = [];
  for (const [i, name] of [appsName, settingsName].entries()) {
    const action = chosen(idle, i);
    assert.deepEqual(chosen(i === 0 ? appsFocus : settingsFocus, i), action);
    const before = await pixelIcon('02-top-actions-idle-no-background', action);
    const after = await pixelIcon(name, action);
    assert.ok(after[2] > before[2] * 1.4 && after[3] > before[3] * 1.4);
    px.push({ action: i === 0 ? 'apps' : 'settings', before, after });
  }
  await fs.writeFile(path.join(output, 'icon-pixels.json'), JSON.stringify(px, null, 2));
  const compact = await capture('05-widgets-compact-left-right', fixture + ',right,right', live);
  assert.equal(compact.home_preview.top_expansion, 0);
  assert.notDeepEqual(compact.focus, neutral.focus);
  checkHeader(compact);
  const firstContent = (s) => controls(s).find((c) => c.target.EvolutionContent);
  const reducedDock = await capture('widgets-dock-scale', openWidgets, live);
  both(reducedDock);
  assert.ok(geometry(reducedDock).cards[0][2] < geometry(neutral).cards[0][2] * 0.65);
  assert.equal(reducedDock.home_preview.selected, neutral.home_preview.selected);
  assert.deepEqual(reducedDock.preferences.favorites, neutral.preferences.favorites);
  const compactHeader = await capture('12-home-cinema-expanded', fixture + ',up', live);
  checkHeader(compactHeader);
  assert.ok(compactHeader.ui_text.some((label) => label.startsWith('Boa noite, 20:41 - ')));
  const dockActions = await capture('13-home-global-actions-in-dock', fixture, {
    evolution_home: { global_position: 'dock', rotation_enabled: false },
  });
  checkHeader(dockActions);
  assert.equal(header(dockActions).actions.length, 0);
  assert.ok(actionLabel.every((label) => dockActions.ui_text.includes(label)));
  for (const action of ['apps', 'settings']) assert.ok(global(dockActions, action).y > 300);
  // On the scene with no top actions, the corner pixels under the normal button hitboxes
  // must match the neutral scene. The draw checks cover the entire top action strip.
  for (const { x, y } of header(neutral).actions.map(([x, y]) => ({ x, y }))) {
    for (const [dx, dy] of [
      [6, 6],
      [6, 52],
      [53, 8],
      [52, 52],
    ]) {
      const crop = { left: x + dx, top: y + dy, width: 2, height: 2 };
      const read = (name) =>
        sharp(path.join(output, name + '.png'))
          .extract(crop)
          .raw()
          .toBuffer();
      assert.deepEqual(
        await read('02-top-actions-idle-no-background'),
        await read('13-home-global-actions-in-dock'),
      );
    }
  }
  const noWidgets = await capture('14-widgets-disabled', 'home', {
    evolution_home: { show_top: false },
  });
  assert.ok(!controls(noWidgets).some((c) => c.target.EvolutionWidget));
  checkActions(noWidgets);
  const noCinema = await capture('15-cinema-disabled', fixture + ',up', {
    evolution_home: { show_center: false },
  });
  assert.equal(noCinema.home_preview.top_expansion, 1);
  assert.ok(!controls(noCinema).some((c) => c.target.EvolutionContent));
  assert.ok(controls(noCinema).some((c) => c.target.EvolutionWidget === 'calendar'));
  noInputs(await capture('16-inputs-hidden'));
  const visible = await capture('17-inputs-scroll-revealed', 'home,down,down,down', { favorites });
  assert.ok(controls(visible).some((c) => c.target.Input));
  const bare = await capture(
    'wallpaper-and-dock',
    'home',
    {
      evolution_home: { global_position: 'dock', show_top: false, show_center: false },
    },
    size720,
  );
  assert.ok(controls(bare).every((c) => c.target.App || c.target.EvolutionGlobal));
  const dockExpanded = await capture(
    'widgets-expanded-dock-actions-720p',
    openWidgets,
    {
      favorites,
      evolution_home: { global_position: 'dock', dock_size: 'large' },
    },
    size720,
  );
  both(dockExpanded);
  assert.equal(header(dockExpanded).actions.length, 0);
  assert.ok(actionLabel.every((label) => !dockExpanded.ui_text.includes(label)));
  const multi = await capture(
    'widgets-expanded-two-rows',
    fixture + ',down,preview-wheel:up,preview-wheel:up',
    {
      favorites,
      evolution_home: { dock_size: 'large' },
    },
  );
  both(multi);
  assert.ok(geometry(multi).cards.length > 6);
  assert.ok(Math.abs(geometry(multi).cinema_card_height - 82) < 0.002);

  const returned = await capture('widgets-back-to-dock', openWidgets + ',right,enter,back', live);
  assert.equal(returned.home_preview.top_expansion, 0);
  assert.deepEqual(returned.focus, neutral.focus);
  assert.equal(geometry(returned).cards[0][2], geometry(neutral).cards[0][2]);
  for (const weatherState of ['loading', 'error', 'unavailable']) {
    const command = weatherState === 'unavailable' ? '' : 'preview-weather-' + weatherState + ',';
    const s = await capture(
      'weather-' + weatherState,
      'home,preview-widget-apps:both,' + command + 'up,up,right',
    );
    assert.equal(s.focus.EvolutionWidget, 'weather');
    assert.ok(s.ui_text.includes('Previsões horárias e diárias indisponíveis.'));
    assert.ok(!s.ui_text.includes('Por hora · simulação'));
    assert.ok(
      s.ui_text.includes(weatherState === 'loading' ? 'Carregando clima…' : 'Clima indisponível'),
    );
  }
  both(await capture('calendar-zh', openWidgets, { language: 'zh' }));
  const atAction = (s, i) => at(chosen(s, i));
  const hover = 'preview-hover:1810:58';
  checkActions(await capture('settings-pointer-hover', fixture + ',' + hover, live), 1);
  checkActions(
    await capture('settings-pointer-exit', fixture + ',' + hover + ',preview-hover:970:400', live),
  );
  checkActions(await capture('settings-to-widgets', openWidgets + ',up,right,down,back', live));
  const clicked = await capture('apps-pointer-click', fixture + ',' + atAction(idle, 0), live);
  assert.equal(clicked.page, 'Apps');
  const interrupted = await scenario(
    'top-button-focus-transition',
    openWidgets + ',up,await-settled,right,preview-motion-capture',
    live,
    ['--frames', '3000'],
  );
  assert.ok(interrupted.home_preview.button_amounts.some((v) => v > 0 && v < 1));
  for (const profile of ['reduced', 'none']) {
    checkActions(
      await capture('top-button-' + profile, openWidgets + ',up,right', {
        evolution_home: { motion_profile: profile },
      }),
      1,
    );
  }
  for (const [name, script, selected] of [
    ['idle', fixture, null],
    ['apps', openWidgets + ',up', 0],
    ['settings', openWidgets + ',up,right', 1],
  ])
    checkActions(await capture('top-actions-' + name + '-720p', script, live, size720), selected);

  const wheelUp = ',preview-wheel:up,preview-wheel:up';
  const widgetTargets = (s) => controls(s).filter((c) => c.target.EvolutionWidget);
  const widget = (s, app) => widgetTargets(s).find((c) => c.target.EvolutionWidget === app);
  const widgetOutline = (s) => draws(s).filter((d) => d.kind === 'shape' && d.rect[2] === 834);
  const compact70 = 'home,preview-weather-ready,preview-widget-apps:both,preview-cinema-size:70';

  function checkWidgets(s, allowed, focus) {
    both(s);
    assert.deepEqual(
      widgetTargets(s).map((c) => c.target.EvolutionWidget),
      allowed,
    );
    assert.deepEqual(s.focus, focus ? { EvolutionWidget: focus } : { EvolutionArea: 'widgets' });
    assert.equal(s.home_preview.simulated_widget_open, null);
    assert.equal(widgetOutline(s).length, focus ? 1 : 0);
    for (const control of widgetTargets(s)) {
      const bounds = header(s)[control.target.EvolutionWidget === 'calendar' ? 'clock' : 'weather'];
      assert.deepEqual([control.x, control.y, control.w, control.h], bounds);
    }
  }

  for (const [resolution, size] of [
    ['1080p', []],
    ['720p', size720],
  ]) {
    const byName = async (name, script) => {
      const state = await capture(resolution + '-' + name, script, live, size);
      assert.deepEqual(JSON.parse(await fs.readFile(isolated, 'utf8')), {
        language: 'pt',
        hour_24: true,
        show_weather: true,
        ...live,
      });
      return state;
    };
    const old = await byName('home-compacta-70', compact70);
    const normal = await byName('01-home-compacta-100', fixture);
    const areaOnly = await byName('02-area2-roda-cinema-50', fixture + wheelUp + ',enter');
    const bothApps = await byName('03-area2-ambos-calendario-focado', openWidgets);
    const weatherOnly = await byName(
      '04-area2-so-clima-focado',
      fixture + ',preview-widget-apps:weather,up,up',
    );
    const noWeather = await byName(
      '05-area2-so-calendario-sem-foco',
      fixture + ',preview-widget-apps:calendar' + wheelUp + ',enter',
    );
    checkWidgets(areaOnly, ['calendar', 'weather'], null);
    checkWidgets(bothApps, ['calendar', 'weather'], 'calendar');
    checkWidgets(weatherOnly, ['weather'], 'weather');
    checkWidgets(noWeather, [], null);
    assert.deepEqual(noWeather.home_preview.simulated_widget_apps, {
      calendar: true,
      weather: false,
    });
    assert.equal(normal.home_preview.top_expansion, 0);
    assert.equal(normal.home_preview.expansion, 0);
    assert.equal(normal.home_preview.compact_cinema_percent, 100);
    assert.equal(old.home_preview.compact_cinema_percent, 70);
    assert.deepEqual(normal.preferences, old.preferences);
    assert.deepEqual(geometry(normal).cards, geometry(old).cards);
    for (const key of ['w', 'h']) {
      assert.ok(Math.abs(firstContent(normal)[key] / firstContent(old)[key] - 100 / 70) < 0.001);
      assert.ok(Math.abs(firstContent(areaOnly)[key] / firstContent(normal)[key] - 0.5) < 0.001);
    }
    for (const s of [old, normal, areaOnly, bothApps, weatherOnly, noWeather]) {
      assert.equal(s.home_preview.selected, normal.home_preview.selected);
      assert.deepEqual(
        controls(s)
          .filter((c) => c.target.EvolutionContent)
          .map((c) => c.target.EvolutionContent),
        controls(old)
          .filter((c) => c.target.EvolutionContent)
          .map((c) => c.target.EvolutionContent),
      );
      for (const c of controls(s).filter((c) => c.target.EvolutionContent)) {
        assert.ok(Math.abs(c.w / c.h - 16 / 9) < 0.001);
        assert.ok(c.x >= 80 && c.x + c.w <= 1840);
        assert.ok(c.y >= header(s).bottom && c.y + c.h < geometry(s).pane[1]);
      }
    }
    assert.ok(normal.focus.App);

    const shots = ['home-compacta-70', '01-home-compacta-100', '02-area2-roda-cinema-50'].map(
      (name) => resolution + '-' + name,
    );
    const { width, height } = await sharp(path.join(output, shots[0] + '.png')).metadata();
    const captions = [
      'Home compacta · 70% (personalizado)',
      'Home compacta · 100% (padrão)',
      'Área 2 expandida pela roda · cinema a 50%',
    ];
    await compareShots('comparativo-cinema-' + resolution, shots, captions, width, height);
  }

  const neither = await capture(
    'area2-nenhum-app',
    fixture + ',preview-widget-apps:none' + wheelUp + ',enter',
    live,
  );
  checkWidgets(neither, [], null);
  const bothAvailable = await capture(
    'area2-confirmar-calendario-ok',
    openWidgets + ',enter',
    live,
  );
  assert.equal(bothAvailable.home_preview.simulated_widget_open, 'calendar');
  const weatherAvailable = await capture(
    'area2-confirmar-clima-ok',
    openWidgets + ',right,enter',
    live,
  );
  assert.equal(weatherAvailable.home_preview.simulated_widget_open, 'weather');
  for (const app of ['calendar', 'weather']) {
    const target = widget(bothAvailable, app);
    const hover = 'preview-hover:' + (target.x + target.w / 2) + ':' + (target.y + target.h / 2);
    const hoverScript = fixture + wheelUp + ',await-settled,' + hover;
    const pointed = await capture('area2-ponteiro-' + app, hoverScript, live);
    checkWidgets(pointed, ['calendar', 'weather'], app);
    const clicked = await capture(
      'area2-clique-' + app,
      fixture + wheelUp + ',await-settled,' + at(target),
      live,
    );
    assert.equal(clicked.home_preview.simulated_widget_open, app);
    assert.equal(clicked.modal, false);
  }
  const calendarLost = await capture(
    'area2-calendario-sai-do-inventario',
    openWidgets + ',preview-widget-apps:weather',
    live,
  );
  checkWidgets(calendarLost, ['weather'], 'weather');
  const weatherLost = await capture(
    'area2-clima-sai-do-inventario',
    openWidgets + ',right,preview-widget-apps:calendar,enter',
    live,
  );
  checkWidgets(weatherLost, [], null);
  const reentry = await capture('area2-direcional-apos-roda', fixture + wheelUp + ',right', live);
  checkWidgets(reentry, ['calendar', 'weather'], 'calendar');
  const otherFocus = await capture('cinema-compacto-100-foco-area1', openWidgets + ',up', live);
  assert.equal(firstContent(otherFocus).w, firstContent(neutral).w);
  assert.equal(otherFocus.focus.EvolutionGlobal, 'apps');
  const expandedCinema = await capture('cinema-expandido-apos-area2', openWidgets + ',down', live);
  assert.equal(expandedCinema.home_preview.expansion, 1);
  assert.equal(firstContent(expandedCinema).w, firstContent(neutral).w);
  const leap = await capture('area2-fevereiro-bissexto', fixture + wheelUp, live, [
    '--time',
    '2024-02-29T20:41:00-03:00',
  ]);
  assert.ok(leap.ui_text.includes('Fevereiro 2024') && leap.ui_text.includes('29'));
  assert.ok(!leap.ui_text.includes('30'));

  const more = neutral.inventory.slice(0, 22).map((app) => app.id);
  const withoutInputs = { 'com.webos.app.hdmi1': 'Off', 'com.webos.app.hdmi2': 'Off' };
  const start = 'home';
  const middle = 'home,down,down';
  const last = middle + ',down';
  const inputRow = last + ',down';
  const scrollCases = [
    ['06-scroll-top-down-arrow', start, { favorites: more }, [false, true]],
    ['07-scroll-middle-both-arrows', middle, { favorites: more }, [true, true]],
    ['08-scroll-bottom-up-arrow', inputRow, { favorites: more }, [true, false]],
    [
      '09-scroll-all-visible-no-arrows',
      'home,down',
      { favorites: more.slice(0, 14), input_modes: withoutInputs },
      [false, false],
    ],
    ['10-scroll-last-favorites-row', last, { favorites: more }, [true, true]],
    ['11-scroll-inputs-visible', 'home,down', { favorites: more.slice(0, 3) }, [true, false]],
    ['12-scroll-no-inputs', last, { favorites: more, input_modes: withoutInputs }, [true, false]],
    [
      '13-scroll-720p',
      middle,
      { favorites: more, evolution_home: { dock_size: 'compact' } },
      [false, true],
      size720,
    ],
    [
      '14-scroll-large-dock',
      middle,
      { favorites: more, evolution_home: { dock_size: 'large' } },
      [true, true],
    ],
    [
      '15-scroll-widgets-disabled',
      middle,
      { favorites: more, evolution_home: { show_top: false, show_center: false } },
      [false, true],
    ],
    ['scroll-return-to-start', inputRow + ',up,up,up,up', { favorites: more }, [true, true]],
    [
      'scroll-dock-actions-reduced-motion',
      middle,
      { favorites: more, evolution_home: { global_position: 'dock', motion_profile: 'reduced' } },
      [false, true],
    ],
    [
      'scroll-no-animation',
      last,
      { favorites: more, evolution_home: { motion_profile: 'none' }, input_modes: withoutInputs },
      [true, false],
    ],
    ['scroll-cinema-expanded', 'home,up', { favorites: more }, [false, true]],
    ['scroll-widgets-expanded', fixture + ',up,up', { favorites: more }, [false, true]],
    ['scroll-indicators-ignore-click', 'home,click:960:1050', { favorites: more }, [false, true]],
  ];

  async function visibleChevron(name, y, scale) {
    const { data, info } = await sharp(path.join(output, name + '.png'))
      .extract({
        left: Math.round(946 * scale),
        top: Math.round(y * scale),
        width: Math.round(28 * scale),
        height: Math.round(28 * scale),
      })
      .removeAlpha()
      .raw()
      .toBuffer({ resolveWithObject: true });
    let bright = 0;
    for (let p = 0; p < data.length; p += info.channels) {
      if (data[p] > 135 && data[p + 1] > 135 && data[p + 2] > 135) bright++;
    }
    return bright >= 4;
  }

  for (const [name, script, config, [up, down], extra = []] of scrollCases) {
    const s = await capture(name, script, config, extra);
    const hints = geometry(s).scroll;
    assert.deepEqual(hints.up, up ? [946, 28, 28, 28] : null, name);
    assert.deepEqual(hints.down, down ? [946, 1036, 28, 28] : null, name);
    const scale = extra.length ? 2 / 3 : 1;
    assert.equal(await visibleChevron(name, 28, scale), up, name + ': upper pixels');
    assert.equal(await visibleChevron(name, 1036, scale), down, name + ': lower pixels');
    for (const r of [hints.up, hints.down].filter(Boolean)) {
      assert.ok(
        controls(s).every(
          (c) => c.x >= end(r) || c.x + c.w <= r[0] || c.y >= feet(r) || c.y + c.h <= r[1],
        ),
      );
    }
    if (name === '10-scroll-last-favorites-row') {
      assert.equal(s.focus.App, more.at(-1));
      noInputs(s);
    }
    if (name === '08-scroll-bottom-up-arrow' || name === '11-scroll-inputs-visible') {
      assert.ok(s.focus.Input);
    }
    if (name === 'scroll-indicators-ignore-click') assert.equal(s.focus.App, more[0]);
  }

  for (const [label, progress] of [
    ['0', 0],
    ['25', 0.25],
    ['50', 0.5],
    ['75', 0.75],
    ['100', 0.98],
  ]) {
    const s = await capture(
      `cinema-progress-${label}`,
      `home,preview-cinema,await-settled,preview-restart-cycle,await-pause:${progress}`,
    );
    assert.ok(s.home_preview.rotation_progress >= progress);
    assert.ok(s.home_preview.rotation_progress < Math.min(1, progress + 0.07));
    assert.equal(s.home_preview.rotation_paused, true);
    assert.equal(s.home_preview.rotation_advances, 0);
  }
  const switched = await capture(
    'cinema-progress-auto-switch',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-cycle,preview-pause',
  );
  assert.equal(switched.home_preview.rotation_advances, 1);
  assert.equal(
    switched.home_preview.selected,
    (switched.home_preview.rotation_trace_selected + 1) % 8,
  );
  assert.ok(switched.home_preview.rotation_progress < 0.15);
  const paused = await capture(
    'cinema-progress-paused',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-progress:0.5,preview-pointer-cinema',
  );
  assert.equal(paused.home_preview.rotation_paused, true);
  assert.ok(
    paused.home_preview.rotation_progress >= 0.5 && paused.home_preview.rotation_progress < 0.57,
  );
  const resumed = await capture(
    'cinema-progress-restarted',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-progress:0.25,preview-pointer-cinema,preview-pointer-leave,await-settled,preview-cinema',
  );
  assert.equal(resumed.home_preview.rotation_paused, false);
  assert.ok(resumed.home_preview.rotation_progress < 0.2);
  await capture('cinema-progress-disabled', 'home,up', {
    evolution_home: { rotation_enabled: false },
  });
  const single = await capture('cinema-one-title', 'home,preview-catalog-one,up');
  assert.equal(single.home_preview.rotation_advances, 0);
  assert.equal(single.home_preview.controls.filter((c) => c.target.EvolutionContent).length, 1);
  const empty = await capture('cinema-empty', 'home,preview-catalog-empty');
  assert.ok(!empty.home_preview.controls.some((c) => c.target.EvolutionContent));
  const interval = await capture(
    'cinema-custom-interval',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-cycle,preview-pause',
    { evolution_home: { rotation_seconds: 2 } },
  );
  assert.equal(interval.home_preview.rotation_advances, 1);
  await capture(
    'cinema-progress-720p',
    'home,preview-cinema,await-settled,preview-restart-cycle,await-progress:0.5,preview-pause',
    {},
    ['--resolution', '1280x720'],
  );
}
