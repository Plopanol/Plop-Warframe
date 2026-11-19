// Generates the third-party licence notices (NOTICES.md) of a Rust project from `cargo metadata`.
// Run from the project root: node tools/commun/generate_notices.mjs [options]; --help lists them.
// Why generated: the notices ship with every deliverable that embeds dependencies (modules/livrable.md), and
// a hand-kept list drifts from Cargo.lock; the texts come from the sources Cargo already downloaded.
// cargo-deny stays the gate on allowed licences (deny.toml); --deny runs it first, it does not supply texts.
// Exit codes: 0 written or up to date, 1 --check found a difference or cargo-deny failed, 2 error.
// Node built-ins only (D-9).

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

export const DEFAULT_PERMISSIVE = [
  '0BSD', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'BSL-1.0', 'CC0-1.0', 'ISC', 'MIT', 'Unicode-3.0',
  'Unicode-DFS-2016', 'Unlicense', 'Zlib',
];

const licenceFileName = /^(licen[cs]e|copying|unlicense|notice)/i;
const licencesDirectory = /^licen[cs]es$/i;
const genericFileName = /^(licen[cs]e|copying)(\.(md|txt))?$/i;

const licenceFilePatterns = {
  '0BSD': /0bsd/i,
  'Apache-2.0': /apache/i,
  'BSD-2-Clause': /bsd/i,
  'BSD-3-Clause': /bsd/i,
  'BSL-1.0': /boost|bsl/i,
  'CC0-1.0': /cc0/i,
  ISC: /isc/i,
  'MPL-2.0': /mpl/i,
  MIT: /mit/i,
  'Unicode-3.0': /unicode/i,
  'Unicode-DFS-2016': /unicode/i,
  Unlicense: /unlicen/i,
  Zlib: /zlib/i,
};

export function licenceAlternatives(expression) {
  return expression
    .replace(/[()]/g, ' ')
    .split(/\s+OR\s+|\s*\/\s*/i)
    .map(alternative => alternative.split(/\s+(?:AND|WITH)\s+/i).map(token => token.trim()).filter(token => token.length > 0))
    .filter(alternative => alternative.length > 0);
}

// The first alternative made only of permissive licences; otherwise the first one, whose text must still ship.
export function retainedAlternative(expression, permissive = DEFAULT_PERMISSIVE) {
  const allowed = new Set(permissive);
  const alternatives = licenceAlternatives(expression ?? '');
  return alternatives.find(alternative => alternative.every(token => allowed.has(token))) ?? alternatives[0] ?? null;
}

function patternFor(token) {
  if (licenceFilePatterns[token]) return licenceFilePatterns[token];
  const stem = token.replace(/^LicenseRef-/, '').split(/[-.]/)[0];
  return stem.length >= 3 ? new RegExp(stem.replace(/[^a-z0-9]/gi, ''), 'i') : null;
}

export function selectLicenceFiles(fileNames, retained) {
  if (retained === null) return [];
  const selected = [];
  for (const token of retained) {
    const pattern = patternFor(token);
    if (!pattern) continue;
    for (const name of fileNames) {
      if (pattern.test(path.posix.basename(name)) && !selected.includes(name)) selected.push(name);
    }
  }
  if (selected.length > 0) return selected;
  const generic = fileNames.filter(name => genericFileName.test(name));
  return generic.length === 1 ? generic : [];
}

export function listLicenceFileNames(directory) {
  const names = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.isFile() && licenceFileName.test(entry.name)) names.push(entry.name);
    else if (entry.isDirectory() && licencesDirectory.test(entry.name)) {
      for (const nested of fs.readdirSync(path.join(directory, entry.name), { withFileTypes: true })) {
        if (nested.isFile()) names.push(entry.name + '/' + nested.name);
      }
    }
  }
  return names.sort();
}

// A fence longer than any backtick run inside the text keeps the licence verbatim in Markdown.
export function fence(text) {
  const longest = [...text.matchAll(/`{3,}/g)].reduce((maximum, match) => Math.max(maximum, match[0].length), 2);
  return '`'.repeat(longest + 1);
}

// Runtime packages reachable from workspace members through normal or build edges.
export function distributedPackages(metadata) {
  const packages = new Map(metadata.packages.map(item => [item.id, item]));
  const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
  const members = new Set(metadata.workspace_members);
  const runtime = node => node.deps.filter(dependency => dependency.dep_kinds.some(kind => kind.kind !== 'dev')).map(dependency => dependency.pkg);
  const pending = [...members].flatMap(id => runtime(nodes.get(id)));
  const reached = new Set();
  while (pending.length > 0) {
    const id = pending.pop();
    if (reached.has(id) || members.has(id)) continue;
    reached.add(id);
    pending.push(...runtime(nodes.get(id)));
  }
  return [...reached].map(id => packages.get(id)).sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
}

export function collectNotices(packages, readPackageFiles, permissive = DEFAULT_PERMISSIVE) {
  const texts = [];
  const digests = new Map();
  const entries = [];
  for (const item of packages) {
    const files = readPackageFiles(item);
    const names = files.map(file => file.name);
    let license = item.license;
    let selected;
    if (!license && item.license_file) {
      const declared = path.posix.basename(item.license_file.replace(/\\/g, '/'));
      license = 'voir ' + declared;
      selected = names.filter(name => path.posix.basename(name) === declared);
    } else selected = selectLicenceFiles(names, retainedAlternative(license, permissive));
    const references = [];
    for (const name of selected) {
      const content = files.find(file => file.name === name).content;
      const digest = crypto.createHash('sha256').update(content).digest('hex');
      let index = digests.get(digest);
      if (index === undefined) {
        index = texts.length + 1;
        digests.set(digest, index);
        texts.push({ index, content, packages: [] });
      }
      const label = item.name + ' ' + item.version;
      if (!texts[index - 1].packages.includes(label)) texts[index - 1].packages.push(label);
      if (!references.includes(index)) references.push(index);
    }
    entries.push({ name: item.name, version: item.version, license: license ?? 'inconnue', references });
  }
  return { entries, texts };
}

const cell = value => String(value).replaceAll('|', '\\|');

export function renderNotices({ name, version, projectLicence, preamble, entries, texts }) {
  const covered = entries.filter(entry => entry.references.length > 0);
  const gaps = entries.filter(entry => entry.references.length === 0);
  const lines = [
    '# Notices de ' + name,
    '',
    'Document produit par `tools/commun/generate_notices.mjs` à partir de `Cargo.lock` ; ne pas le modifier à la main. Version applicative ' + version + '.',
    '',
    '## ' + name,
    '',
    projectLicence ? name + ' est publié sous licence ' + projectLicence + '. Le texte intégral figure dans le fichier `LICENSE` distribué avec le paquet, s’il existe.' : 'Licence de ' + name + ' : voir `LICENSE` ou les conditions du projet.',
    '',
    ...(preamble ? [preamble.trim(), ''] : []),
    '## Paquets distribués',
    '',
    covered.length + ' paquet(s) dont le texte de licence est reproduit en annexe, sur ' + entries.length + ' paquet(s) du graphe distribué.',
    '',
    '| Paquet | Version | SPDX | Texte |',
    '|---|---|---|---|',
    ...entries.map(entry => '| ' + cell(entry.name) + ' | ' + cell(entry.version) + ' | ' + cell(entry.license) + ' | ' +
      (entry.references.length > 0 ? entry.references.map(index => '[' + index + '](#texte-' + index + ')').join(', ') : '—') + ' |'),
    '',
    '## Lacunes amont',
    '',
    'Les paquets suivants ne publient aucun fichier de licence identifiable dans leur source ; seule leur déclaration SPDX est disponible.',
    '',
    ...(gaps.length > 0 ? gaps.map(entry => '- ' + entry.name + ' ' + entry.version + ' — ' + entry.license) : ['Aucune.']),
    '',
    '## Textes de licence',
    '',
  ];
  for (const text of texts) {
    const body = text.content.toString('utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n').replace(/\n+$/, '');
    const bar = fence(body);
    lines.push('<a id="texte-' + text.index + '"></a>', '', '### Texte ' + text.index, '', text.packages.join(', '), '', bar, body, bar, '');
  }
  return lines.join('\n').trimEnd() + '\n';
}

// --- Command line ----------------------------------------------------------------------------------------
const USAGE = `Usage : node generate_notices.mjs [options]  (depuis la racine du projet)
  --manifest-path <Cargo.toml>  manifeste du projet (défaut : Cargo.toml)
  --out <fichier>               notices produites (défaut : NOTICES.md)
  --target <triplet>            plateforme du paquet (défaut : hôte de rustc)
  --name <nom>                  nom affiché (défaut : paquet exécutable du projet)
  --licence <SPDX>              licence du projet (défaut : champ license du manifeste)
  --allow <SPDX,SPDX>           licences ajoutées à la liste des permissives
  --preamble <fichier.md>       sections propres au projet, insérées avant les paquets
  --deny                        lance d'abord « cargo deny check licenses » (non exécuté si absent)
  --check                       compare sans écrire ; code 1 si le fichier diffère`;

export function parseArguments(argv) {
  const options = { manifest: 'Cargo.toml', out: 'NOTICES.md', allow: [] };
  const valued = { '--manifest-path': 'manifest', '--out': 'out', '--target': 'target', '--name': 'name', '--licence': 'licence', '--preamble': 'preamble' };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--deny') options.deny = true;
    else if (arg === '--check') options.check = true;
    else if (arg === '--help') options.help = true;
    else if (arg === '--allow') options.allow = (argv[++i] ?? '').split(',').map(value => value.trim()).filter(Boolean);
    else if (valued[arg]) {
      if (argv[i + 1] === undefined) throw new Error('Valeur manquante pour ' + arg);
      options[valued[arg]] = argv[++i];
    } else throw new Error('Option inconnue : ' + arg);
  }
  return options;
}

const hostTriple = () => execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host:\s*(\S+)/m)?.[1];

function runDeny(manifest) {
  const probe = spawnSync('cargo', ['deny', '--version'], { encoding: 'utf8' });
  if (probe.status !== 0) return { skipped: true };
  const run = spawnSync('cargo', ['deny', '--manifest-path', manifest, 'check', 'licenses'], { encoding: 'utf8' });
  return { skipped: false, ok: run.status === 0, output: (run.stdout + run.stderr).trim() };
}

function main(argv) {
  const options = parseArguments(argv);
  if (options.help) { console.log(USAGE); return 0; }
  const manifest = path.resolve(options.manifest);
  if (!fs.existsSync(manifest)) throw new Error('Manifeste introuvable : ' + options.manifest);
  const workspace = path.dirname(manifest);
  if (options.deny) {
    const deny = runDeny(manifest);
    if (deny.skipped) console.log('cargo-deny non installé : contrôle des licences non exécuté');
    else if (!deny.ok) { console.error(deny.output); console.error('cargo-deny a refusé une licence : notices non générées'); return 1; }
    else console.log('cargo-deny : licences conformes');
  }
  const target = options.target ?? hostTriple();
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked', '--manifest-path', manifest, ...(target ? ['--filter-platform', target] : [])],
    { cwd: workspace, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] }));
  const members = metadata.packages.filter(item => metadata.workspace_members.includes(item.id));
  const primary = members.find(item => item.targets.some(t => t.kind.includes('bin'))) ?? members[0];
  const packages = distributedPackages(metadata);
  const readPackageFiles = item => {
    const directory = path.dirname(item.manifest_path);
    if (!fs.existsSync(directory)) throw new Error('Source absente du cache Cargo : ' + item.name + ' ' + item.version + ' (cargo fetch --locked)');
    const names = listLicenceFileNames(directory);
    if (item.license_file) {
      const declared = item.license_file.replace(/\\/g, '/');
      const relative = path.isAbsolute(item.license_file) ? path.relative(directory, item.license_file).replace(/\\/g, '/') : declared;
      if (fs.existsSync(path.join(directory, relative)) && !names.includes(relative)) names.push(relative);
    }
    return names.map(name => ({ name, content: fs.readFileSync(path.join(directory, name)) }));
  };
  const permissive = [...DEFAULT_PERMISSIVE, ...options.allow];
  const { entries, texts } = collectNotices(packages, readPackageFiles, permissive);
  const preamble = options.preamble ? fs.readFileSync(path.resolve(options.preamble), 'utf8').replace(/\r\n/g, '\n') : '';
  const document = renderNotices({
    name: options.name ?? primary?.name ?? path.basename(workspace),
    version: primary?.version ?? 'inconnue',
    projectLicence: options.licence ?? primary?.license,
    preamble, entries, texts,
  });
  const out = path.resolve(options.out);
  const summary = entries.length + ' paquet(s), ' + texts.length + ' texte(s) distinct(s), ' + entries.filter(entry => entry.references.length === 0).length + ' lacune(s) amont';
  if (options.check) {
    const current = fs.existsSync(out) ? fs.readFileSync(out, 'utf8').replace(/\r\n/g, '\n') : null;
    if (current !== document) { console.error(options.out + ' : absent ou différent de la génération (' + summary + ') ; relancer sans --check'); return 1; }
    console.log(options.out + ' à jour : ' + summary);
    return 0;
  }
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, document);
  console.log(options.out + ' écrit : ' + summary);
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error('Erreur : ' + (error.stderr ? String(error.stderr).trim() : error.message)); process.exitCode = 2; }
}
