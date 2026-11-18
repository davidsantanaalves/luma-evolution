import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import fs from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { syncVersions } from '../tools/sync-version.mjs';
import {
  applyVersion,
  buildNumber,
  numberOf,
  releaseTag,
  releaseTitle,
  versionOf,
} from '../tools/build-number.mjs';
import { previousRelease, releaseNotes } from '../tools/release-notes.mjs';
import { normalizeAndVerifyPackage } from '../tools/ipk.mjs';

const root = path.resolve(import.meta.dirname, '..');
const metadata = ['appinfo.json', 'Cargo.toml', 'Cargo.lock', 'readme.md'];

async function fixture(t, version = '2.0.4') {
  const scratch = path.join(root, '.build');
  await fs.mkdir(scratch, { recursive: true });
  const directory = await fs.mkdtemp(path.join(scratch, 'release-test-'));
  t.after(() => fs.rm(directory, { recursive: true, force: true }));
  const project = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
  project.version = '1.2.3';
  await fs.writeFile(path.join(directory, 'package.json'), JSON.stringify(project));
  await Promise.all(
    metadata.map((filename) =>
      fs.copyFile(path.join(root, filename), path.join(directory, filename)),
    ),
  );
  await syncVersions(directory);
  project.version = version;
  await fs.writeFile(path.join(directory, 'package.json'), JSON.stringify(project));
  return directory;
}

const readMetadata = (directory) =>
  Promise.all(metadata.map((filename) => fs.readFile(path.join(directory, filename), 'utf8')));

test('version sync changes only release versions and is idempotent', async (t) => {
  const directory = await fixture(t, '3.1.0');
  const before = await readMetadata(directory);
  await assert.rejects(syncVersions(directory, { check: true }), /out of sync/);
  assert.deepEqual(await readMetadata(directory), before, 'check mode must not write files');

  assert.deepEqual((await syncVersions(directory)).changed, metadata);
  const after = await readMetadata(directory);
  const previous = JSON.parse(before[0]).version;
  assert.equal(JSON.parse(after[0]).version, '3.1.0');
  assert.equal(after[0], before[0].replace(`"version": "${previous}"`, '"version": "3.1.0"'));
  assert.equal(after[1], before[1].replace(`version = "${previous}"`, 'version = "3.1.0"'));
  assert.equal(
    after[2],
    before[2].replace(/^(name = "luma"\r?\nversion = ")[^"]+/m, (_, prefix) => `${prefix}3.1.0`),
    'dependency versions and helper crates must be preserved',
  );
  assert.equal(
    after[3],
    before[3].replaceAll(
      `com.davidsantana.lumaevolution_${previous}_arm.ipk`,
      'com.davidsantana.lumaevolution_3.1.0_arm.ipk',
    ),
  );
  assert.deepEqual((await syncVersions(directory, { check: true })).changed, []);
  assert.deepEqual((await syncVersions(directory)).changed, []);
});

test('invalid release versions and missing packages fail before writing metadata', async (t) => {
  for (const version of ['3.0.0-beta.1', '3.0', '03.0.0']) {
    const directory = await fixture(t, version);
    const before = await readMetadata(directory);
    await assert.rejects(syncVersions(directory), /numeric major.minor.patch/);
    assert.deepEqual(await readMetadata(directory), before);
  }
  const directory = await fixture(t);
  const lockfile = path.join(directory, 'Cargo.lock');
  await fs.writeFile(
    lockfile,
    (await fs.readFile(lockfile, 'utf8')).replace('name = "luma"', 'name = "other"'),
  );
  const before = await readMetadata(directory);
  await assert.rejects(syncVersions(directory), /one luma package in Cargo.lock/);
  assert.deepEqual(await readMetadata(directory), before);
});

test('a build is numbered by its minute in Chisinau and versioned by the same digits', () => {
  // 13:30 UTC is 15:30 in Chisinau in winter (EET) and 16:30 in summer (EEST).
  assert.equal(buildNumber(new Date('2026-11-08T13:30:00Z')), '2611081530');
  assert.equal(buildNumber(new Date('2026-07-01T13:30:00Z')), '2607011630');
  assert.equal(buildNumber(new Date('2026-12-31T22:05:00Z')), '2701010005');
  assert.equal(versionOf('2611081530'), '26.1108.1530');
  // No leading zeros, as the updater and Cargo require.
  assert.equal(versionOf('2701010005'), '27.101.5');
  assert.equal(versionOf('2701010000'), '27.101.0');
  for (const number of ['261108153', '2613081530', '2611082460', 'build']) {
    assert.throws(() => versionOf(number), /Invalid build number/);
  }
});

test('a build version names its release; versions before builds keep theirs', () => {
  for (const number of ['2611081530', '2701010005', '2701010000', '2612312359']) {
    assert.equal(numberOf(versionOf(number)), number);
  }
  assert.equal(releaseTag('26.1108.1530'), 'build-2611081530');
  assert.equal(releaseTitle('26.1108.1530'), 'Luma Evolution build #2611081530');
  assert.equal(numberOf('1.0.0'), null);
  assert.equal(releaseTag('1.0.0'), 'v1.0.0');
  assert.equal(releaseTitle('1.0.0'), 'Luma Evolution 1.0.0');
});

test('later builds are newer versions, and every build is newer than 1.0.0', () => {
  const compare = (a, b) => {
    const [x, y] = [a, b].map((version) => version.split('.').map(Number));
    return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
  };
  const builds = ['2611081530', '2611081531', '2611090005', '2612010000', '2701010000'];
  const versions = builds.map(versionOf);
  for (let index = 1; index < versions.length; index++) {
    assert.ok(compare(versions[index], versions[index - 1]) > 0, versions[index]);
  }
  for (const version of versions) assert.ok(compare(version, '1.0.0') > 0);
});

test('applying a build version writes it into package.json and every synced file', async (t) => {
  const directory = await fixture(t, '1.2.3');
  await syncVersions(directory);
  await applyVersion(directory, '26.1108.1530');
  assert.equal((await syncVersions(directory, { check: true })).version, '26.1108.1530');
  const [appinfo, , , readme] = await readMetadata(directory);
  assert.equal(JSON.parse(appinfo).version, '26.1108.1530');
  assert.match(readme, /com\.davidsantana\.lumaevolution_26\.1108\.1530_arm\.ipk/);
});

test('feed artifacts use the Evolution identity and release origin even in a fork checkout', async (t) => {
  const directory = await fixture(t, '1.2.3');
  const manifest = JSON.parse(await fs.readFile(path.join(directory, 'appinfo.json'), 'utf8'));
  assert.equal(manifest.id, 'com.davidsantana.lumaevolution');
  assert.equal(manifest.main, 'luma');
  assert.equal(manifest.title, 'Luma Evolution');
  const filename = `${manifest.id}_${manifest.version}_arm.ipk`;
  const fakeIpk = Buffer.from('fixture IPK bytes');
  await Promise.all(
    ['.build/package', 'assets', 'dist', 'tools'].map((dir) =>
      fs.mkdir(path.join(directory, dir), { recursive: true }),
    ),
  );
  const copies = [
    'assets/homebrew-description.html',
    'tools/build-number.mjs',
    'tools/repository.mjs',
    'tools/sync-version.mjs',
  ];
  await Promise.all([
    ...copies.map((file) => fs.copyFile(path.join(root, file), path.join(directory, file))),
    fs.writeFile(path.join(directory, '.build/package', manifest.largeIcon), 'fixture icon'),
    fs.writeFile(path.join(directory, 'dist', filename), fakeIpk),
  ]);
  execFileSync(process.execPath, [path.join(directory, 'tools/repository.mjs')], {
    env: { ...process.env, GITHUB_REPOSITORY: 'eluz1ve/lg-luma-plus', GITHUB_OUTPUT: '' },
  });
  const feed = JSON.parse(await fs.readFile(path.join(directory, 'dist/repo.json'), 'utf8'));
  const [entry] = feed.packages;
  const origin = 'https://github.com/davidsantanaalves/luma-evolution';
  assert.equal(entry.id, manifest.id);
  assert.equal(entry.manifest.id, manifest.id);
  assert.equal(entry.manifest.sourceUrl, origin);
  assert.equal(entry.manifest.ipkUrl, `${origin}/releases/download/v1.2.3/${filename}`);
  assert.equal(entry.manifest.ipkSize, fakeIpk.length);
  assert.equal(entry.manifest.ipkHash.sha256, createHash('sha256').update(fakeIpk).digest('hex'));
});

const manifest = JSON.parse(await fs.readFile(path.join(root, 'appinfo.json'), 'utf8'));
const filename = [manifest.id, manifest.version, 'arm'].join('_') + '.ipk';
const appRoot = 'usr/palm/applications/' + manifest.id + '/';
const packageRoot = 'usr/palm/packages/' + manifest.id + '/';

function tarEntry(name, content = '', type = '0', mode = '0000644', timestamps = '') {
  const data = Buffer.from(content);
  const header = Buffer.alloc(512);
  header.write(name);
  header.write(mode + '\0', 100);
  header.write(data.length.toString(8).padStart(11, '0') + '\0', 124);
  header.write(type, 156);
  header.write('ustar\0' + '00', 257);
  header.write(timestamps, 476, 24);
  header.fill(32, 148, 156);
  const sum = header.reduce((total, byte) => total + byte, 0);
  header.write(sum.toString(8).padStart(6, '0') + '\0 ', 148);
  return Buffer.concat([header, data, Buffer.alloc((512 - (data.length % 512)) % 512)]);
}

function arEntry(name, bytes) {
  const header = Buffer.alloc(60, ' ');
  header.write(name);
  header.write(String(bytes.length), 48);
  header.write('\x60\n', 58);
  return Buffer.concat([header, bytes, Buffer.alloc(bytes.length % 2, 10)]);
}

function control(id = manifest.id, version = manifest.version) {
  return 'Package: ' + id + '\nArchitecture: arm\nVersion: ' + version + '\n';
}

function payload() {
  const appFiles = [
    manifest.icon,
    manifest.largeIcon,
    'splash.png',
    'licenses/sdl2.txt',
    'licenses/lucide.txt',
    'assets/font-metrics.json',
    'assets/font-license.txt',
    'assets/font-cjk-license.txt',
    ...[350, 400, 450, 500, 550, 600, 650, 800].flatMap((weight) => [
      'assets/font-' + weight + '.ttf',
      'assets/font-display-' + weight + '.ttf',
    ]),
    'assets/font-cjk-400.ttf',
    'assets/font-cjk-500.ttf',
    'assets/font-kr-400.ttf',
    'assets/font-kr-500.ttf',
    ...[
      'dunes',
      'midnight',
      'san-francisco',
      'summit',
      'slate',
      'forest',
      'prism',
      'yosemite',
      'northern-lights',
    ].flatMap((name) => [
      'assets/wallpapers/' + name + '.jpg',
      'assets/thumbnails/' + name + '.jpg',
      'splash/' + name + '.png',
    ]),
  ];
  const arm = Buffer.alloc(52);
  arm.set([0x7f, 0x45, 0x4c, 0x46, 1, 1]);
  arm.writeUInt16LE(3, 16);
  arm.writeUInt16LE(40, 18);
  const mesh = Buffer.alloc(20 + 3 * 36);
  mesh.write('LUMAVEC1');
  mesh.writeUInt32LE(3, 16);
  return new Map([
    ...appFiles.map((name) => [appRoot + name, 'fixture']),
    [appRoot + 'luma', arm],
    [appRoot + 'luma-helper', arm],
    [
      appRoot + 'lib/libSDL2-2.0.so.0',
      Buffer.concat([
        arm,
        Buffer.from('SDL Wayland video driver\0APPID environment variable is not set\0'),
      ]),
    ],
    [appRoot + 'autostart.sh', '#!/bin/sh\nexit 0\n'],
    [appRoot + 'home.sh', '#!/bin/sh\nexit 0\n'],
    [appRoot + 'appinfo.json', JSON.stringify(manifest)],
    [appRoot + 'assets/version.txt', manifest.version],
    [appRoot + 'assets/vector-artwork.json', '{"version":1,"assets":["logos/test.lvec"]}'],
    [appRoot + 'assets/logos/test.lvec', mesh],
    [
      packageRoot + 'packageinfo.json',
      JSON.stringify({
        id: manifest.id,
        version: manifest.version,
        app: manifest.id,
      }),
    ],
  ]);
}

function packageFixture({
  controls = [tarEntry('./control', control(), '0', '0000644', '15277307247\0'.repeat(2))],
  data = payload(),
  extras = [],
  arExtras = [],
} = {}) {
  const tar = (entries) => gzipSync(Buffer.concat([...entries, Buffer.alloc(1024)]));
  return Buffer.concat([
    Buffer.from('!<arch>\n'),
    arEntry('debian-binary', Buffer.from('2.0\n')),
    arEntry('control.tar.gz', tar(controls)),
    arEntry(
      'data.tar.gz',
      tar([...Array.from(data, ([name, content]) => tarEntry(name, content)), ...extras]),
    ),
    ...arExtras,
  ]);
}

test('the IPK verifier accepts one matching install and checks the filename and POSIX scripts', () => {
  const input = packageFixture({
    extras: [tarEntry(appRoot + 'legacy/', '', '5', '0040777')],
  });
  const normalized = normalizeAndVerifyPackage(input, manifest, filename);
  assert.deepEqual(normalizeAndVerifyPackage(normalized, manifest, filename), normalized);
  assert.throws(() => normalizeAndVerifyPackage(input, manifest, 'different.ipk'), /filename/);
  for (const script of ['home.sh', 'autostart.sh']) {
    const data = payload();
    data.set(appRoot + script, '#!/bin/sh\r\nexit 0\r\n');
    assert.throws(
      () => normalizeAndVerifyPackage(packageFixture({ data }), manifest, filename),
      /LF line endings/,
    );
  }
});

test('the IPK verifier rejects foreign, conflicting or ambiguous metadata', () => {
  const upstream = 'com.eluz1ve.lumaplus';
  const data = payload();
  const cases = [
    [{ controls: [tarEntry('./control', control(upstream))] }, /app ID/],
    [{ controls: [tarEntry('./control', control(manifest.id, '9.0.0'))] }, /control version/],
    [
      { controls: [tarEntry('control', control()), tarEntry('./control', control(upstream))] },
      /Duplicate/,
    ],
    [
      { controls: [tarEntry('./control', control()), tarEntry('././control', control(upstream))] },
      /ambiguous/,
    ],
    [{ controls: [tarEntry('control', control() + 'pAcKaGe: ' + upstream + '\n')] }, /duplicate/],
    [
      { controls: [tarEntry('control', control()), tarEntry('postinst', '#!/bin/sh\n')] },
      /installation scripts/,
    ],
    [
      { extras: [tarEntry('usr/palm/applications/' + upstream + '/appinfo.json', '{}')] },
      /payload path/,
    ],
    [{ extras: [tarEntry('./' + appRoot + 'appinfo.json', '{}')] }, /Duplicate/],
    [{ extras: [tarEntry(appRoot + 'assets', 'conflict')] }, /Conflicting/],
    [
      {
        data: new Map(data).set(
          appRoot + 'appinfo.json',
          JSON.stringify({ ...manifest, version: '9.0.0' }),
        ),
      },
      /app metadata/,
    ],
    [
      {
        data: new Map(data).set(
          appRoot + 'appinfo.json',
          '{"id":"' + upstream + '",' + JSON.stringify(manifest).slice(1),
        ),
      },
      /Duplicate IPK manifest key/,
    ],
    [
      {
        data: new Map(data).set(
          packageRoot + 'packageinfo.json',
          JSON.stringify({ id: manifest.id, version: manifest.version, app: upstream }),
        ),
      },
      /packageinfo/,
    ],
    [{ arExtras: [arEntry('data.tar.gz', Buffer.alloc(0))] }, /trailer/],
  ];
  for (const [options, pattern] of cases) {
    assert.throws(
      () => normalizeAndVerifyPackage(packageFixture(options), manifest, filename),
      pattern,
    );
  }
});

test('the IPK verifier rejects traversal, links and tar metadata with extraction side effects', () => {
  const extras = [
    tarEntry('/etc/passwd', 'absolute'),
    tarEntry(appRoot + '../other/appinfo.json', '{}'),
    tarEntry(appRoot + 'a//file', 'alias'),
    tarEntry(appRoot + 'symlink', '', '2'),
    tarEntry(appRoot + 'hardlink', '', '1'),
    tarEntry(appRoot + 'device', '', '3'),
    tarEntry(appRoot + 'setuid', 'file', '0', '0004755'),
    tarEntry(appRoot + 'typed-setuid', 'file', '0', '0104755'),
    tarEntry(appRoot + 'incompatible-type', 'file', '0', '0040644'),
    tarEntry('PaxHeader', '20 path=/etc/passwd\n', 'x'),
    tarEntry(appRoot + 'timestamp', 'file', '0', '0000644', '/etc/passwd\0'),
  ];
  for (const entry of extras) {
    assert.throws(
      () => normalizeAndVerifyPackage(packageFixture({ extras: [entry] }), manifest, filename),
      /path|tar (header|timestamp)/,
    );
  }
});
test('the foundation workflow builds artifacts with publication disabled', async () => {
  const ci = await fs.readFile(path.join(root, '.github/workflows/ci.yml'), 'utf8');
  assert.match(ci, /echo "publish=false" >> "\$GITHUB_OUTPUT"/);
  assert.match(ci, /needs\.release-gate\.outputs\.publish == 'true'/);
  assert.match(ci, /github\.repository == 'davidsantanaalves\/luma-evolution'/);
});

const commits = {
  'build-2611081530': 'b'.repeat(40),
  'build-2611091200': 'c'.repeat(40),
};
const release = (tag_name, published_at, extra = {}) => ({
  tag_name,
  name: tag_name,
  published_at,
  draft: false,
  target_commitish: 'main',
  ...extra,
});

test('the release before a build is the newest published one it descends from', () => {
  const releases = [
    release('v1.0.0', '2026-11-04T04:50:20Z', {
      name: 'Luma Evolution 1.0.0',
      target_commitish: 'a'.repeat(40),
    }),
    release('build-2611081530', '2026-11-08T13:50:00Z'),
    release('build-2611091200', '2026-11-09T10:20:00Z'),
    release('build-2611091300', '2026-11-09T11:20:00Z', { draft: true }),
  ];
  const resolve = (tag) => commits[tag] ?? null;
  const found = previousRelease(releases, resolve, () => true);
  assert.equal(found.release.tag_name, 'build-2611091200');
  assert.equal(found.commit, 'c'.repeat(40));
  // A build on another line of history is passed over.
  const other = previousRelease(releases, resolve, (commit) => commit !== 'c'.repeat(40));
  assert.equal(other.release.tag_name, 'build-2611081530');
  // Before builds, a release counts from the commit it was published at, never its tag.
  const first = previousRelease(releases.slice(0, 1), resolve, () => true);
  assert.equal(first.commit, 'a'.repeat(40));
  assert.equal(
    previousRelease([release('v0.9.0', '2026-10-01T00:00:00Z')], resolve, () => true),
    null,
  );
});

test('release notes list the commits since the release before, linked to their comparison', () => {
  const notes = releaseNotes({
    subjects: [
      'Lay the page under an overlay down as one picture',
      'Launch wallpaper fade at 60 Hz',
    ],
    repository: 'davidsantanaalves/luma-evolution',
    base: 'e7b6ae64f92157d4928eb5ab6b8590bafdc51888',
    head: 'd6fc86c0000000000000000000000000000000000',
    previous: 'Luma Evolution 1.0.0',
  });
  assert.equal(
    notes,
    [
      '### Changes',
      '',
      '- Lay the page under an overlay down as one picture',
      '- Launch wallpaper fade at 60 Hz',
      '',
      'Since Luma Evolution 1.0.0: [e7b6ae6...d6fc86c](https://github.com/davidsantanaalves/luma-evolution/compare/e7b6ae6...d6fc86c)',
      '',
    ].join('\n'),
  );
  assert.match(
    releaseNotes({ subjects: [], repository: 'a/b', base: null, head: 'f' }),
    /- Rebuilt/,
  );
});
