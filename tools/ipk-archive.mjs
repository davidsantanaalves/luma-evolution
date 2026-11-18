import { crc32, inflateRawSync } from 'node:zlib';

const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const utf8 = (bytes) => decoder.decode(bytes);
const zero = (bytes) => bytes.every((byte) => byte === 0);
const isControl = (char) => char < ' ' || char === '\x7f';

export function members(bytes) {
  if (bytes.subarray(0, 8).toString() !== '!<arch>\n') throw new Error('Not an IPK/ar archive.');
  const archive = new Map();
  let offset = 8;
  for (const expected of ['debian-binary', 'control.tar.gz', 'data.tar.gz']) {
    const header = Buffer.from(bytes.subarray(offset, offset + 60));
    const name = utf8(header.subarray(0, 16)).trimEnd().replace(/\/$/, '');
    const length = utf8(header.subarray(48, 58)).trim();
    const size = Number(length);
    const end = offset + 60 + size;
    if (
      header.subarray(58).toString() !== '\x60\n' ||
      name !== expected ||
      !/^\d+$/.test(length) ||
      end + (size % 2) > bytes.length ||
      (size % 2 && bytes[end] !== 10)
    ) {
      throw new Error('Unexpected or invalid IPK archive member.');
    }
    archive.set(name, { header, data: bytes.subarray(offset + 60, end) });
    offset = end + (size % 2);
  }
  if (offset !== bytes.length || archive.get('debian-binary').data.toString() !== '2.0\n')
    throw new Error('Unexpected IPK archive trailer or format version.');
  return archive;
}

// Decode one gzip member, consuming its complete header, payload and trailer.
function inflateArchive(bytes, limit) {
  const flags = bytes[3];
  if (bytes.length < 18 || bytes.subarray(0, 3).toString('hex') !== '1f8b08' || flags & 0xe0)
    throw new Error('Invalid IPK gzip member.');
  const trailer = bytes.length - 8;
  let start = 10;
  if (flags & 4) start += 2 + bytes.readUInt16LE(start);
  for (const flag of [8, 16]) {
    if (!(flags & flag)) continue;
    const end = bytes.indexOf(0, start);
    if (end < 0) throw new Error('Invalid IPK gzip header.');
    start = end + 1;
  }
  if (flags & 2) {
    if (
      start + 2 > trailer ||
      (crc32(bytes.subarray(0, start)) & 0xffff) !== bytes.readUInt16LE(start)
    )
      throw new Error('Invalid IPK gzip header checksum.');
    start += 2;
  }
  if (start >= trailer) throw new Error('Truncated IPK gzip member.');
  const { buffer, engine } = inflateRawSync(bytes.subarray(start, trailer), {
    info: true,
    maxOutputLength: limit,
  });
  if (
    start + engine.bytesWritten !== trailer ||
    buffer.length !== bytes.readUInt32LE(trailer + 4) ||
    crc32(buffer) !== bytes.readUInt32LE(trailer)
  ) {
    throw new Error('Invalid IPK gzip checksum or trailing data.');
  }
  return buffer;
}

function field(bytes) {
  const end = bytes.indexOf(0);
  if (end === -1) return utf8(bytes);
  if (!zero(bytes.subarray(end))) throw new Error('Ambiguous IPK tar field.');
  return utf8(bytes.subarray(0, end));
}

function octal(bytes) {
  const value = utf8(bytes).replace(/^[ ]+|[ \0]+$/g, '');
  if (!/^[0-7]+$/.test(value)) throw new Error('Invalid IPK tar number.');
  return Number.parseInt(value, 8);
}

function ustarPrefix(header) {
  if (header[475] !== 0) return field(header.subarray(345, 500));
  // node-tar (used by ares-package) stores atime and ctime after a short prefix.
  // The NUL separator keeps the path identical for readers of the POSIX prefix.
  for (const offset of [476, 488]) {
    const stamp = header.subarray(offset, offset + 12);
    if (!zero(stamp) && !/^[0-7]{11}\0$/.test(utf8(stamp)))
      throw new Error('Ambiguous IPK tar timestamp.');
  }
  return field(header.subarray(345, 475));
}

function entryPath(name, directory) {
  let canonical = name.startsWith('./') ? name.slice(2) : name;
  if (directory && canonical.endsWith('/')) canonical = canonical.slice(0, -1);
  if (directory && (name === '.' || name === './')) return '';
  if (
    Array.from(canonical).some((char) => isControl(char) || char === '\\' || char === ':') ||
    canonical.split('/').some((part) => !part || part === '.' || part === '..')
  ) {
    throw new Error('Unsafe or ambiguous IPK tar path: ' + name);
  }
  return canonical;
}

export function entries(bytes, limit) {
  const tar = inflateArchive(bytes, limit);
  const files = new Map();
  for (let offset = 0; offset + 512 <= tar.length;) {
    const header = tar.subarray(offset, offset + 512);
    if (zero(header)) {
      if (tar.length - offset < 1024 || (tar.length - offset) % 512 || !zero(tar.subarray(offset)))
        throw new Error('Invalid IPK tar trailer.');
      for (const [name] of files) {
        for (let slash = name.indexOf('/'); slash >= 0; slash = name.indexOf('/', slash + 1)) {
          if (files.get(name.slice(0, slash))?.directory === false)
            throw new Error('Conflicting IPK tar paths: ' + name);
        }
      }
      return { tar, files };
    }
    const size = octal(header.subarray(124, 136));
    const mode = octal(header.subarray(100, 108));
    const sum = header.reduce(
      (total, byte, index) => total + (index >= 148 && index < 156 ? 32 : byte),
      0,
    );
    const magic = header.subarray(257, 265).toString('hex');
    const directory = header[156] === 53;
    const modeType = mode - (mode % 0o1000);
    if (
      sum !== octal(header.subarray(148, 156)) ||
      (modeType !== 0 && modeType !== (directory ? 0o040000 : 0o100000)) ||
      !(directory || header[156] === 0 || header[156] === 48) ||
      (directory && size !== 0) ||
      !zero(header.subarray(157, 257)) ||
      (magic !== '7573746172003030' && magic !== '7573746172202000') ||
      !zero(header.subarray(magic === '7573746172003030' ? 500 : 345)) ||
      files.size >= 50000
    ) {
      throw new Error('Unsupported or invalid IPK tar header.');
    }
    const prefix = magic === '7573746172003030' ? ustarPrefix(header) : '';
    const basename = field(header.subarray(0, 100));
    if (!basename || (prefix && (!entryPath(prefix, false) || basename.startsWith('/'))))
      throw new Error('Invalid IPK tar path fields.');
    const name = entryPath(prefix ? prefix + '/' + basename : basename, directory);
    if (files.has(name)) throw new Error('Duplicate IPK tar path: ' + name);
    const start = offset + 512;
    const end = start + size;
    const next = start + Math.ceil(size / 512) * 512;
    if (next > tar.length || !zero(tar.subarray(end, next)))
      throw new Error('Invalid IPK tar member size or padding.');
    files.set(name, { header, data: tar.subarray(start, end), directory });
    offset = next;
  }
  throw new Error('Missing IPK tar trailer.');
}

export function controlFields(bytes) {
  const text = utf8(bytes).replaceAll('\r\n', '\n');
  if (Array.from(text).some((char) => isControl(char) && char !== '\n' && char !== '\t'))
    throw new Error('Invalid IPK control text.');
  const fields = new Map();
  let previous;
  for (const line of text.trimEnd().split('\n')) {
    if (/^[ \t]/.test(line) && previous === 'description') continue;
    const match = /^([a-z][a-z0-9-]*):[ \t]*(.*)$/i.exec(line);
    if (!match || Array.from(match[2]).some(isControl))
      throw new Error('Invalid IPK control field.');
    const key = match[1].toLowerCase();
    if (
      fields.has(key) ||
      ![
        'package',
        'version',
        'architecture',
        'description',
        'maintainer',
        'section',
        'priority',
        'installed-size',
        'webos-package-format-version',
        'webos-packager-version',
        'homepage',
        'source',
        'license',
      ].includes(key)
    ) {
      throw new Error('Unexpected or duplicate IPK control field: ' + key);
    }
    fields.set(key, match[2].trim());
    previous = key;
  }
  return fields;
}

export function manifestJson(bytes) {
  if (bytes.length > 1024 * 1024) throw new Error('IPK manifest exceeds the size limit.');
  const source = utf8(bytes);
  const result = JSON.parse(source);
  if (!result || typeof result !== 'object' || Array.isArray(result))
    throw new Error('Expected a JSON object in the IPK manifest.');
  const keys = new Set();
  let depth = 0;
  for (const token of source.matchAll(/"(?:[^"\\]|\\[\s\S])*"|[{}[\]]/g)) {
    const [text] = token;
    if (text === '{' || text === '[') depth++;
    if (text === '}' || text === ']') depth--;
    if (
      depth === 1 &&
      text.startsWith('"') &&
      /^\s*:/.test(source.slice(token.index + text.length))
    ) {
      const key = JSON.parse(text);
      if (keys.has(key)) throw new Error('Duplicate IPK manifest key: ' + key);
      keys.add(key);
    }
  }
  return result;
}
