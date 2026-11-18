import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const source = fs.readFileSync(new URL('../tv/self-update.sh', import.meta.url), 'utf8');
function run({
  install = true,
  home = false,
  reapply = true,
  recover = true,
  launch = true,
  checksum = true,
  identity = true,
  method = 'executable',
  homeId = 'com.webos.app.home',
  pretty = false,
  stateMode = null,
  stateOwner = 0,
  fixtures = () => {},
} = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'luma-installer-'));
  const app = `${dir}/app`;
  const state = `${dir}/update`;
  const homeState = `${dir}/home`;
  const bin = `${dir}/bin`;
  for (const d of [app, homeState, bin]) fs.mkdirSync(d);
  if (stateMode !== null) {
    fs.mkdirSync(state);
    fs.chmodSync(state, stateMode);
  }
  const write = (file, contents) => fs.writeFileSync(file, contents, { mode: 0o755 });
  const attempt = 'a'.repeat(32);
  const download = (name) => {
    fs.mkdirSync(`${dir}/${name}`);
    write(`${dir}/${name}/package.ipk`, 'test package');
    return `${dir}/${name}`;
  };
  download(`luma-evolution-update-${attempt}`);
  fixtures({ dir, download });
  // tools/package.mjs and the native helper pretty-print; the saved stock manifest has its own
  // "version" and "id", which must not be mistaken for Luma's version or the Home target.
  const json = (value) => (pretty ? `${JSON.stringify(value, null, 2)}\n` : JSON.stringify(value));
  write(
    `${app}/appinfo.json`,
    json({ id: 'com.davidsantana.lumaevolution', version: '9.0.0', type: 'native' }),
  );
  write(
    `${homeState}/state.json`,
    json({ enabled: home, method, homeId, appDir: app, stock: { id: 'x', version: '1.0.0' } }),
  );
  write(`${dir}/mounts`, '');
  write(`${bin}/id`, '#!/bin/sh\necho 0\n');
  // The installer must not need the TV's Node runtime.
  write(`${bin}/node`, '#!/bin/sh\nexit 127\n');
  write(`${bin}/stat`, `#!/bin/sh\necho ${stateOwner}\n`);
  write(`${bin}/sleep`, '#!/bin/sh\n/bin/sleep 0.01\n');
  write(
    `${bin}/luna-send`,
    `#!/bin/sh
printf '%s\\n' "$*" >> '${dir}/calls'
case "$*" in
  *dev/install*) echo '${install ? '{"returnValue":true,"state":"installed"}' : '{"returnValue":false,"errorText":"disk full"}'}' ;;
  *) echo '{"returnValue":${launch}}' ;;
esac
`,
  );
  write(`${app}/home.sh`, `#!/bin/sh\necho refresh >> '${dir}/calls'\nexit ${reapply ? 0 : 1}\n`);
  write(
    `${app}/luma-helper`,
    `#!/bin/sh
[ "$1" = verify-self-update-ipk ] && [ "$3" = 9.0.0 ] &&
  [ "${'$'}{2##*/}" = com.davidsantana.lumaevolution_9.0.0_arm.ipk ] || exit 2
exit ${identity ? 0 : 1}
`,
  );
  write(
    `${homeState}/home.sh`,
    `#!/bin/sh\necho restore >> '${dir}/calls'\nexit ${recover ? 0 : 1}\n`,
  );
  const script = source
    .replace('SELF_UPDATE_ENABLED=false', 'SELF_UPDATE_ENABLED=true')
    .replaceAll('/var/lib/luma-evolution-update', state)
    .replaceAll('/var/lib/luma-evolution-home', homeState)
    .replaceAll('/media/developer/apps/usr/palm/applications/com.davidsantana.lumaevolution', app)
    .replaceAll('/proc/mounts', `${dir}/mounts`)
    .replaceAll('/var/run/luma-evolution-home', `${dir}/marker`)
    .replaceAll('/tmp/luma-evolution-update-', `${dir}/luma-evolution-update-`)
    .replaceAll('find /tmp ', `find ${dir} `);
  write(`${dir}/installer.sh`, script);
  const sha = checksum ? createHash('sha256').update('test package').digest('hex') : '0'.repeat(64);
  try {
    const result = spawnSync(
      '/bin/sh',
      [
        `${dir}/installer.sh`,
        `${dir}/luma-evolution-update-${attempt}/package.ipk`,
        sha,
        '9.0.0',
        app,
        attempt,
      ],
      {
        env: { ...process.env, PATH: `${bin}:${process.env.PATH}` },
        encoding: 'utf8',
        timeout: 10000,
      },
    );
    assert.equal(result.signal, null, result.stderr);
    const calls = fs.existsSync(`${dir}/calls`) ? fs.readFileSync(`${dir}/calls`, 'utf8') : '';
    const resultFile = `${state}/result.${attempt}.json`;
    const downloads = fs.readdirSync(dir).filter((name) => /^luma-(evolution-)?update-/.test(name));
    if (!fs.existsSync(resultFile)) return { status: null, calls, exit: result.status, downloads };
    const status = JSON.parse(fs.readFileSync(resultFile, 'utf8'));
    assert.deepEqual(status, JSON.parse(fs.readFileSync(`${state}/result.json`, 'utf8')));
    const mode = fs.statSync(state).mode & 0o777;
    return { status, calls, exit: result.status, mode, downloads };
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test('normal app update verifies the package and reports completion after launch', () => {
  const result = run();
  assert.equal(result.exit, 0);
  assert.equal(result.status.state, 'success');
  assert.match(result.calls, /dev\/install.*com\.davidsantana\.lumaevolution_9\.0\.0_arm\.ipk/);
  assert.match(result.calls, /launch.*com.davidsantana.lumaevolution/);
});
test('checksum and install failures publish retryable failure, never launch', () => {
  for (const options of [{ checksum: false }, { install: false }]) {
    const result = run(options);
    assert.equal(result.exit, 1);
    assert.equal(result.status.state, 'failed');
    assert.doesNotMatch(result.calls, /launch/);
  }
});
test('a package for a different app is refused before calling the installer', () => {
  const result = run({ identity: false });
  assert.equal(result.exit, 1);
  assert.match(result.status.message, /identity or contents mismatch/);
  assert.doesNotMatch(result.calls, /dev\/install|launch/);
});

test('the shipped self-update installer exits before reading arguments or running commands', () => {
  const shell = process.platform === 'win32' ? 'bash' : '/bin/sh';
  const result = spawnSync(shell, ['-s'], {
    input: source,
    encoding: 'utf8',
    env: { ...process.env, SELF_UPDATE_ENABLED: 'true' },
    timeout: 10000,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /self-update installation is disabled/);
});
test('both Home methods refresh from the new package and fail visibly when reapply fails', () => {
  for (const method of ['executable', 'manifest']) {
    const success = run({ home: true, method });
    assert.equal(success.status.state, 'success');
    assert.match(success.calls, /refresh/);
    const failed = run({ home: true, method, reapply: false });
    assert.equal(failed.exit, 1);
    assert.equal(failed.status.state, 'failed');
    assert.match(failed.status.message, /LG Home restored/);
    assert.match(failed.calls, /restore[\s\S]*launch.*com.webos.app.home/);
    const unrecovered = run({ home: true, method, reapply: false, recover: false });
    assert.equal(unrecovered.status.state, 'failed');
    assert.doesNotMatch(unrecovered.calls, /launch/);
  }
});
test('a state directory left with the default mode is made private, not a silent stop', () => {
  // luma-helper's app updates create /var/lib/luma-evolution-update/apps, and with it this directory.
  const result = run({ stateMode: 0o755 });
  assert.equal(result.exit, 0);
  assert.equal(result.status?.state, 'success');
  assert.equal(result.mode, 0o700);
});
test('a state directory owned by someone else is never used', () => {
  const result = run({ stateMode: 0o755, stateOwner: 1000 });
  assert.equal(result.exit, 1);
  assert.equal(result.status, null);
  assert.doesNotMatch(result.calls, /dev\/install/);
});
test('the downloaded package does not outlive its verified copy', () => {
  // Installing can end Luma before it removes the download itself.
  const result = run();
  assert.equal(result.status.state, 'success');
  assert.deepEqual(result.downloads, []);
});
test('a package that failed its checksum stays for Luma to report and remove', () => {
  const result = run({ checksum: false });
  assert.deepEqual(result.downloads, [`luma-evolution-update-${'a'.repeat(32)}`]);
});
test('only abandoned download directories are swept', () => {
  const hours = new Date(Date.now() - 2 * 3600 * 1000);
  const recent = `luma-evolution-update-${'c'.repeat(32)}`;
  const link = `luma-evolution-update-${'d'.repeat(32)}`;
  const legacy = `luma-update-${'e'.repeat(32)}`;
  let kept;
  const result = run({
    fixtures: ({ dir, download }) => {
      fs.utimesSync(download(`luma-evolution-update-${'b'.repeat(32)}`), hours, hours);
      fs.utimesSync(download('luma-evolution-update-notours'), hours, hours);
      fs.utimesSync(download(legacy), hours, hours);
      download(recent);
      kept = `${dir}/target/package.ipk`;
      fs.mkdirSync(`${dir}/target`);
      fs.writeFileSync(kept, 'someone else');
      fs.symlinkSync(`${dir}/target`, `${dir}/${link}`);
      fs.lutimesSync(`${dir}/${link}`, hours, hours);
    },
  });
  assert.equal(result.status.state, 'success');
  assert.deepEqual(
    result.downloads.sort(),
    [recent, link, legacy, 'luma-evolution-update-notours'].sort(),
  );
});
test('a rejected launch is not reported as a successful update', () => {
  assert.equal(run({ launch: false }).status.state, 'failed');
});

test('fullhome remains the launch and recovery target after an update', () => {
  const homeId = 'com.webos.app.fullhome';
  const success = run({ home: true, homeId });
  assert.equal(success.status.state, 'success');
  assert.match(success.calls, /launch.*com.webos.app.fullhome/);
  assert.doesNotMatch(success.calls, /launch.*com.webos.app.home/);
  const failed = run({ home: true, homeId, reapply: false });
  assert.match(failed.status.message, /LG Home restored/);
  assert.match(failed.calls, /restore[\s\S]*launch.*com.webos.app.fullhome/);
});

test('pretty-printed manifests and old configurations without a target are read without Node', () => {
  for (const homeId of ['com.webos.app.fullhome', undefined]) {
    const result = run({ home: true, homeId, pretty: true });
    assert.equal(result.exit, 0, result.status.message);
    assert.equal(result.status.state, 'success');
    assert.match(result.calls, new RegExp(`launch.*${homeId ?? 'com.webos.app.home'}"`));
  }
});
