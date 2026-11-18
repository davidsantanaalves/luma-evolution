import { gzipSync } from 'node:zlib';
import { controlFields, entries, manifestJson, members } from './ipk-archive.mjs';
import { verifyWebosSdl } from './webos-runtime.mjs';

const controlLimit = 256 * 1024;
const payloadLimit = 384 * 1024 * 1024;
export const packagedScripts = ['autostart.sh', 'home.sh'];

export function verifyPosixScript(bytes, name) {
  if (
    !bytes ||
    !bytes.subarray(0, 10).equals(Buffer.from('#!/bin/sh\n')) ||
    bytes.includes(13) ||
    bytes.includes(0)
  )
    throw new Error(`Packaged ${name} must be a POSIX shell script with LF line endings.`);
}

export function normalizeAndVerifyPackage(bytes, manifest, filename, expectedScripts) {
  if (filename !== `${manifest.id}_${manifest.version}_arm.ipk`)
    throw new Error('IPK filename must match the intended app ID, version and architecture.');
  if (bytes.length > 96 * 1024 * 1024) throw new Error('IPK exceeds the size limit.');
  const archive = members(bytes);
  const controlArchive = archive.get('control.tar.gz');
  const controlContents = entries(controlArchive.data, controlLimit);
  const control = controlContents.files.get('control');
  if (
    !control ||
    control.directory ||
    [...controlContents.files].some(([name]) => name && name !== 'control')
  )
    throw new Error('IPK must contain one regular control file and no installation scripts.');
  const fields = controlFields(control.data);
  if (fields.get('architecture') !== 'arm')
    throw new Error('The IPK must declare Architecture: arm.');
  if (fields.get('package') !== manifest.id)
    throw new Error('The IPK control file must declare the app ID from appinfo.json.');
  if (fields.get('version') !== manifest.version)
    throw new Error('The IPK control version must match appinfo.json.');

  const member = archive.get('data.tar.gz');
  const { tar, files } = entries(member.data, payloadLimit);
  const root = `usr/palm/applications/${manifest.id}`;
  const packageRoot = `usr/palm/packages/${manifest.id}`;
  const packagePath = `${packageRoot}/packageinfo.json`;
  const directories = new Set([
    '',
    'usr',
    'usr/palm',
    'usr/palm/applications',
    'usr/palm/packages',
    root,
    packageRoot,
  ]);
  const owned = new Map();
  for (const [name, file] of files) {
    if (name.startsWith(root + '/')) owned.set(name.slice(root.length + 1), file);
    else if (
      !(directories.has(name) && file.directory) &&
      !(name === packagePath && !file.directory)
    )
      throw new Error('Unexpected IPK payload path: ' + name);
  }
  const packageFile = files.get(packagePath);
  const packageInfo = packageFile && manifestJson(packageFile.data);
  if (
    packageInfo?.id !== manifest.id ||
    packageInfo.version !== manifest.version ||
    packageInfo.app !== manifest.id ||
    Object.keys(packageInfo).some((key) => !['id', 'version', 'app'].includes(key))
  )
    throw new Error('Packaged packageinfo.json must describe only the intended app and version.');
  const read = (name) => {
    const file = owned.get(name);
    return file && !file.directory ? file.data : undefined;
  };
  if (
    [...owned.keys()].some((name) =>
      /^assets\/(app-icons(?:\/|$)|icons\.json$|colors\.json$)/.test(name),
    )
  )
    throw new Error('Retired dumped artwork must not be shipped in the TV package.');
  const appinfo = read('appinfo.json');
  const actual = appinfo && manifestJson(appinfo);
  if (
    actual?.id !== manifest.id ||
    actual?.version !== manifest.version ||
    actual.type !== 'native' ||
    actual.main !== manifest.main
  )
    throw new Error('Packaged app metadata does not match the source.');
  if (JSON.stringify(actual.requiredACG) !== JSON.stringify(manifest.requiredACG))
    throw new Error('Packaged app permissions do not match the source.');
  if (actual.icon !== manifest.icon || actual.largeIcon !== manifest.largeIcon)
    throw new Error('Packaged icon paths do not match the source.');
  const vectorManifest = read('assets/vector-artwork.json');
  const vectorAssets = vectorManifest && manifestJson(vectorManifest);
  if (
    vectorAssets?.version !== 1 ||
    !Array.isArray(vectorAssets.assets) ||
    !vectorAssets.assets.length
  )
    throw new Error('The IPK is missing its vector artwork manifest.');
  for (const required of [
    manifest.main,
    manifest.icon,
    manifest.largeIcon,
    ...packagedScripts,
    'luma-helper',
    'splash.png',
    'lib/libSDL2-2.0.so.0',
    'licenses/sdl2.txt',
    'licenses/lucide.txt',
    'assets/font-metrics.json',
    'assets/font-license.txt',
    'assets/font-cjk-license.txt',
    'assets/version.txt',
    ...[350, 400, 450, 500, 550, 600, 650, 800].flatMap((weight) => [
      `assets/font-${weight}.ttf`,
      `assets/font-display-${weight}.ttf`,
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
      `assets/wallpapers/${name}.jpg`,
      `assets/thumbnails/${name}.jpg`,
      `splash/${name}.png`,
    ]),
    ...vectorAssets.assets.map((name) => {
      if (typeof name !== 'string' || !/^(logos|ui-icons)\/[a-z0-9-]+\.lvec$/.test(name))
        throw new Error('Invalid packaged vector asset path.');
      return `assets/${name}`;
    }),
  ]) {
    if (!read(required)?.length) throw new Error(`Package is missing ${required}.`);
  }
  for (const name of packagedScripts) {
    const script = read(name);
    verifyPosixScript(script, name);
    if (expectedScripts && !script.equals(expectedScripts.get(name)))
      throw new Error(`Packaged ${name} differs from the checked script.`);
  }
  verifyWebosSdl(read('lib/libSDL2-2.0.so.0'));
  for (const name of [manifest.main, 'luma-helper']) {
    const executable = read(name);
    if (
      !executable ||
      executable.length < 52 ||
      executable.subarray(0, 4).toString('hex') !== '7f454c46' ||
      executable[4] !== 1 ||
      executable[5] !== 1 ||
      executable.readUInt16LE(18) !== 40 ||
      (executable.readUInt32LE(36) & 0x400) !== 0
    )
      throw new Error(`Packaged ${name} must be an ARM soft-float ELF.`);
  }
  const version = read('assets/version.txt')?.toString().trim();
  if (version !== manifest.version)
    throw new Error('Packaged assets were generated for another version.');
  for (const [name, file] of owned) {
    if (!name.endsWith('.lvec')) continue;
    if (
      file.data.length < 20 ||
      file.data.subarray(0, 8).toString() !== 'LUMAVEC1' ||
      file.data.readUInt32LE(16) === 0 ||
      file.data.readUInt32LE(16) % 3 !== 0 ||
      file.data.length !== 20 + file.data.readUInt32LE(16) * 36
    )
      throw new Error(`Invalid packaged vector geometry: ${name}`);
  }
  const executables = new Set(
    [manifest.main, 'luma-helper', ...packagedScripts].map((name) => `${root}/${name}`),
  );
  for (const [name, file] of files) {
    normalizeHeader(file, file.directory || executables.has(name));
  }
  for (const file of controlContents.files.values()) normalizeHeader(file, file.directory);
  controlArchive.data = gzipSync(controlContents.tar, { level: 9 });
  member.data = gzipSync(tar, { level: 9 });
  const parts = [Buffer.from('!<arch>\n')];
  for (const entry of archive.values()) {
    entry.header.write(String(entry.data.length).padEnd(10, ' '), 48, 10, 'ascii');
    parts.push(entry.header, entry.data);
    if (entry.data.length % 2) parts.push(Buffer.from('\n'));
  }
  console.log(
    `Verified IPK contents: ${owned.size} Luma entries; app metadata, assets, scripts and modes checked.`,
  );
  return Buffer.concat(parts);
}

function normalizeHeader({ header }, executable) {
  header.write(executable ? '0000755\0' : '0000644\0', 100, 8, 'ascii');
  header.write('0000000\0'.repeat(2), 108, 16, 'ascii');
  header.fill(0, 265, 329);
  if (header[475] === 0) header.fill(0, 476, 500);
  header.fill(32, 148, 156);
  const sum = header.reduce((total, byte) => total + byte, 0);
  header.write(`${sum.toString(8).padStart(6, '0')}\0 `, 148, 8, 'ascii');
}
