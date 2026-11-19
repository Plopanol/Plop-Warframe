// Renders the third-party licence notices of the Maven libraries an Android build ships, from the
// artifacts.json inventory of the Licensee Gradle plugin (app.cash.licensee). Run from the project root:
// node tools/commun/generate_gradle_notices.mjs --artifacts <artifacts.json> [options]; --help lists them.
// Why generated: notices ship with every deliverable that embeds dependencies (modules/livrable.md), and a
// hand-kept list drifts from the dependency graph. Licensee is the gate on allowed licences (allow() in the
// Gradle build); this tool only renders. Maven artifacts rarely carry their licence file, so texts come from
// licence directories: <dir>/<groupId>/<artifactId>.txt first (licences that embed a copyright, such as
// BSD or MIT), then <dir>/<SPDX>.txt (licences without one, such as Apache-2.0).
// Exit codes: 0 written or up to date, 1 --check found a difference, 2 error (including a missing text).
// Node built-ins only (D-9).

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const DEFAULT_TITLE = 'Bibliothèques Android (Maven)';

const coordinates = item => item.groupId + ':' + item.artifactId;

// Licence directories are searched in order; the artifact-specific text wins over the generic SPDX one.
export function findLicenceText(item, directories, exists = fs.existsSync) {
  for (const directory of directories) {
    const specific = path.join(directory, item.groupId, item.artifactId + '.txt');
    if (exists(specific)) return { file: specific, identifier: (item.spdxLicenses ?? [])[0]?.identifier ?? 'propre' };
  }
  for (const licence of item.spdxLicenses ?? []) {
    for (const directory of directories) {
      const generic = path.join(directory, licence.identifier + '.txt');
      if (exists(generic)) return { file: generic, identifier: licence.identifier };
    }
  }
  return null;
}

export function collectNotices(artifacts, directories, readText = file => fs.readFileSync(file, 'utf8'), exists = fs.existsSync) {
  const sorted = [...artifacts].sort((a, b) => coordinates(a).localeCompare(coordinates(b)) || a.version.localeCompare(b.version));
  const texts = [];
  const digests = new Map();
  const entries = [];
  const missing = [];
  for (const item of sorted) {
    const spdx = (item.spdxLicenses ?? []).map(licence => licence.identifier);
    const unknown = (item.unknownLicenses ?? []).map(licence => [licence.name, licence.url].filter(Boolean).join(' — '));
    const found = findLicenceText(item, directories, exists);
    let reference = null;
    if (found) {
      const content = readText(found.file).replace(/^﻿/, '').replace(/\r\n/g, '\n').replace(/^\n+/, '').replace(/\n+$/, '');
      const digest = crypto.createHash('sha256').update(content).digest('hex');
      reference = digests.get(digest);
      if (reference === undefined) {
        reference = texts.length + 1;
        digests.set(digest, reference);
        texts.push({ index: reference, identifier: found.identifier, content, artifacts: 0 });
      }
      texts[reference - 1].artifacts++;
    } else if (spdx.length > 0) missing.push(coordinates(item) + ' ' + item.version + ' (' + spdx.join(', ') + ')');
    entries.push({ coordinates: coordinates(item), version: item.version, spdx, unknown, reference });
  }
  return { entries, texts, missing };
}

// A fence longer than any backtick run inside the text keeps the licence verbatim in Markdown.
export function fence(text) {
  const longest = [...text.matchAll(/`{3,}/g)].reduce((maximum, match) => Math.max(maximum, match[0].length), 2);
  return '`'.repeat(longest + 1);
}

const cell = value => String(value).replaceAll('|', '\\|');

// Anchors are prefixed so that the section can follow other notices in the same file without collisions.
export function renderSection({ title, entries, texts }) {
  const gaps = entries.filter(entry => entry.unknown.length > 0 || entry.reference === null);
  const lines = [
    '## ' + title,
    '',
    'Section produite par `tools/commun/generate_gradle_notices.mjs` à partir de l’inventaire du plugin Gradle Licensee ; ne pas la modifier à la main. ' +
      entries.length + ' artefact(s), ' + texts.length + ' texte(s) de licence distinct(s).',
    '',
    '| Artefact | Version | SPDX | Texte |',
    '|---|---|---|---|',
    ...entries.map(entry => '| ' + cell(entry.coordinates) + ' | ' + cell(entry.version) + ' | ' + cell(entry.spdx.join(', ') || '—') + ' | ' +
      (entry.reference === null ? '—' : '[' + entry.reference + '](#gradle-texte-' + entry.reference + ')') + ' |'),
    '',
    '### Lacunes amont',
    '',
    'Artefacts sans licence SPDX reconnue ou sans texte disponible ; seule leur déclaration Maven est connue.',
    '',
    ...(gaps.length > 0 ? gaps.map(entry => '- ' + entry.coordinates + ' ' + entry.version + ' — ' + (entry.unknown.join(' ; ') || entry.spdx.join(', '))) : ['Aucune.']),
    '',
    '### Textes de licence',
    '',
  ];
  for (const text of texts) {
    const bar = fence(text.content);
    lines.push('<a id="gradle-texte-' + text.index + '"></a>', '', '#### Texte ' + text.index + ' — ' + text.identifier, '', bar, text.content, bar, '');
  }
  return lines.join('\n').trimEnd() + '\n';
}

// --- Command line ----------------------------------------------------------------------------------------
const USAGE = `Usage : node generate_gradle_notices.mjs --artifacts <artifacts.json> [options]  (depuis la racine du projet)
  --artifacts <fichier>   inventaire Licensee (build/.../reports/licensee/<variante>/artifacts.json)
  --out <fichier>         notices produites (défaut : NOTICES.md)
  --append                ajoute la section à la fin de --out existant au lieu de le remplacer
  --texts <dossier>       dossier de textes, consulté avant le dossier commun ; répétable
  --title <titre>         titre de la section (défaut : ${DEFAULT_TITLE})
  --check                 compare sans écrire ; code 1 si le fichier diffère`;

export function parseArguments(argv) {
  const options = { out: 'NOTICES.md', texts: [], title: DEFAULT_TITLE };
  const valued = { '--artifacts': 'artifacts', '--out': 'out', '--title': 'title' };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--append') options.append = true;
    else if (arg === '--check') options.check = true;
    else if (arg === '--help') options.help = true;
    else if (arg === '--texts' || valued[arg]) {
      if (argv[i + 1] === undefined) throw new Error('Valeur manquante pour ' + arg);
      if (arg === '--texts') options.texts.push(argv[++i]);
      else options[valued[arg]] = argv[++i];
    } else throw new Error('Option inconnue : ' + arg);
  }
  return options;
}

const commonTexts = path.join(path.dirname(fileURLToPath(import.meta.url)), 'licences');

function main(argv) {
  const options = parseArguments(argv);
  if (options.help) { console.log(USAGE); return 0; }
  if (!options.artifacts) throw new Error('--artifacts manquant (--help pour l’usage)');
  if (!fs.existsSync(options.artifacts)) throw new Error('Inventaire introuvable : ' + options.artifacts + ' (lancer d’abord la tâche Licensee de la variante)');
  const artifacts = JSON.parse(fs.readFileSync(options.artifacts, 'utf8'));
  if (!Array.isArray(artifacts)) throw new Error('Inventaire inattendu : tableau d’artefacts attendu dans ' + options.artifacts);
  const directories = [...options.texts.map(dir => path.resolve(dir)), commonTexts];
  const { entries, texts, missing } = collectNotices(artifacts, directories);
  if (missing.length > 0) {
    throw new Error('Texte de licence absent pour : ' + missing.join(' ; ') +
      '. Ajouter <dossier>/<groupId>/<artifactId>.txt (licence avec copyright) ou <dossier>/<SPDX>.txt, dans --texts ou le dossier commun licences/');
  }
  const section = renderSection({ title: options.title, entries, texts });
  const out = path.resolve(options.out);
  let document = section;
  if (options.append) {
    if (!fs.existsSync(out)) throw new Error('--append : fichier à compléter introuvable : ' + options.out);
    document = fs.readFileSync(out, 'utf8').replace(/\r\n/g, '\n').trimEnd() + '\n\n' + section;
  }
  const summary = entries.length + ' artefact(s), ' + texts.length + ' texte(s) distinct(s), ' +
    entries.filter(entry => entry.unknown.length > 0 || entry.reference === null).length + ' lacune(s) amont';
  if (options.check) {
    const current = fs.existsSync(out) ? fs.readFileSync(out, 'utf8').replace(/\r\n/g, '\n') : null;
    const expected = options.append ? null : document;
    const upToDate = options.append ? current !== null && current.endsWith(section) : current === expected;
    if (!upToDate) { console.error(options.out + ' : absent ou différent de la génération (' + summary + ') ; relancer sans --check'); return 1; }
    console.log(options.out + ' à jour : ' + summary);
    return 0;
  }
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, document);
  console.log(options.out + (options.append ? ' complété : ' : ' écrit : ') + summary);
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error('Erreur : ' + error.message); process.exitCode = 2; }
}
