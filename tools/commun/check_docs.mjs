// Checks that task, bug, ADR and question registers match their files, and that the next-task pointer
// is valid. Run from the project root, no argument: node tools/commun/check_docs.mjs [--fix].
// Why a script: fiche status is the authority and the index copies it; drift is mechanical to detect and
// to repair, so agents should not hand-edit index tables.
// Exit codes: 0 consistent, 1 inconsistencies, 2 error. Node built-ins only (D-9).
// Formats come from socle v6 and gabarits/docs/; legacy archives/ folders stay valid (D-14).

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const NO_NEXT_TASK = 'Aucune tâche planifiée.';

export const LAYOUTS = {
  TASK: {
    prefix: 'TASK', dir: 'docs/tasks',
    statuses: ['À cadrer', 'À faire', 'En cours', 'Bloquée', 'À valider', 'Terminée', 'Reportée', 'Abandonnée'],
    sections: [
      { title: 'Actives', statuses: ['En cours', 'Bloquée', 'À valider'], insert: 'asc' },
      { title: 'À faire', statuses: ['À cadrer', 'À faire'], insert: 'asc' },
      { title: 'Reportées', statuses: ['Reportée'], insert: 'asc' },
      { title: 'Terminées', statuses: ['Terminée', 'Abandonnée'], insert: 'first' },
    ],
    empty: 'Aucune.', keepEmptyTable: false,
  },
  BUG: {
    prefix: 'BUG', dir: 'docs/bugs',
    statuses: ['Signalé', 'Confirmé', 'En cours', 'Bloqué', 'Corrigé à vérifier', 'Clos', 'Doublon', 'Rejeté'],
    sections: [
      { title: 'Actifs', statuses: ['Signalé', 'Confirmé', 'En cours', 'Bloqué', 'Corrigé à vérifier'], insert: 'asc' },
      { title: 'Fermés', statuses: ['Clos', 'Doublon', 'Rejeté'], insert: 'first' },
    ],
    empty: 'Aucun.', keepEmptyTable: true,
  },
  ADR: {
    prefix: 'ADR', dir: 'docs/adr',
    statuses: ['Proposée', 'Acceptée', /^Remplacée par ADR-\d{3}$/, 'Rejetée', 'Obsolète'],
    sections: null,
    empty: 'Aucune décision enregistrée.', keepEmptyTable: true,
  },
};

const PLACEHOLDER = /^Aucun(?:e|s)?\b[^|]*\.\s*$/;
const CELL_SPLIT = /(?<!\\)\|/;

// --- Files -----------------------------------------------------------------------------------------------
export function readText(root, relative) {
  const raw = fs.readFileSync(path.join(root, relative), 'utf8');
  return { text: raw.replace(/^﻿/, '').replace(/\r\n/g, '\n'), crlf: raw.includes('\r\n'), bom: raw.startsWith('﻿') };
}

function writeText(root, relative, file, text) {
  const out = file.crlf ? text.replace(/\n/g, '\r\n') : text;
  fs.writeFileSync(path.join(root, relative), (file.bom ? '﻿' : '') + out);
}

export const exists = (root, relative) => fs.existsSync(path.join(root, relative));

// Comment and fenced-code lines never count as content: templates keep sample rows in them.
export function maskLines(lines) {
  const masked = [];
  let comment = false;
  let fence = false;
  for (const line of lines) {
    if (comment) {
      masked.push(true);
      if (line.includes('-->')) comment = false;
    } else if (fence) {
      masked.push(true);
      if (/^```/.test(line)) fence = false;
    } else if (/^```/.test(line)) {
      masked.push(true);
      fence = true;
    } else if (line.includes('<!--')) {
      masked.push(line.trim().startsWith('<!--'));
      if (!line.slice(line.indexOf('<!--')).includes('-->')) comment = true;
    } else masked.push(false);
  }
  return masked;
}

export function readProfile(root) {
  if (!exists(root, 'AGENTS.md')) throw new Error('AGENTS.md introuvable : lancer depuis la racine du projet');
  const { text } = readText(root, 'AGENTS.md');
  const declaration = text.match(/^<!--\s*commun\s*:(.*?)-->/m);
  const declared = declaration?.[1].match(/profil\s+(complet|l[ée]ger)/i)?.[1];
  if (declared) return { profile: /^c/i.test(declared) ? 'complet' : 'leger', inferred: false };
  return { profile: exists(root, 'docs/03-ROADMAP.md') ? 'complet' : 'leger', inferred: true };
}

export function normalizeStatus(value) {
  return value.replace(/\[([^\]]+)\]\([^)]*\)/g, '$1').replace(/[*`]/g, '').replace(/\s+/g, ' ').trim().replace(/\.$/, '');
}

// Also reads inherited inline metadata (« **Date** : 2026-08-07 · **Statut** : Acceptée »), since fiches are never
// rewritten (D-20): the field may follow « · » on its line and stops before the next « · **Champ** ».
export function field(text, name) {
  return text.match(new RegExp('^(?:[-*]\\s*)?(?:[^\\n]*?\\s·\\s+)?\\*{0,2}' + name + '(?:\\*{2})?\\s*:\\*{0,2}\\s*(.+?)(?:\\s+·\\s+\\*{2}[^\\n]*)?\\s*$', 'm'))?.[1];
}

function listFiles(root, directory, pattern) {
  const found = [];
  const walk = relative => {
    if (!exists(root, relative)) return;
    for (const entry of fs.readdirSync(path.join(root, relative), { withFileTypes: true })) {
      const child = path.posix.join(relative, entry.name);
      if (entry.isDirectory()) walk(child);
      else if (pattern.test(entry.name)) found.push(child);
    }
  };
  walk(directory);
  return found.sort();
}

const isAllowedStatus = (layout, status) => layout.statuses.some(allowed => allowed instanceof RegExp ? allowed.test(status) : allowed === status);

// A status may carry a précision after « ; » (« Acceptée ; règle 2 amendée par ADR-104 ») and a replaced ADR its
// date or scope (« Remplacée par ADR-030 le 2026-09-12 ») : fiches are never rewritten (D-20), so the status is
// the allowed part and the rest stays a précision of the fiche.
export function canonicalStatus(layout, value) {
  const head = value.split(';')[0].trim();
  if (isAllowedStatus(layout, head)) return head;
  const replaced = layout.prefix === 'ADR' ? head.match(/^Remplacée par ADR-\d{3}(?=\s|$)/) : null;
  return replaced ? replaced[0] : value;
}

// Fiches live in the folder or in any archives/ subfolder; XXX templates never match \d{3}.
export function loadFiches(root, layout) {
  const pattern = new RegExp('^' + layout.prefix + '-(\\d{3})(?:-.*)?\\.md$');
  const fiches = [];
  const problems = [];
  for (const file of listFiles(root, layout.dir, pattern)) {
    const { text } = readText(root, file);
    const nameId = path.posix.basename(file).match(pattern)[1];
    const heading = text.match(new RegExp('^# ' + layout.prefix + '-(\\d{3})\\s*[—–-]\\s*(.+?)\\s*$', 'm'));
    const rawStatus = field(text, 'Statut');
    const fiche = { id: nameId, file, title: heading?.[2], status: rawStatus === undefined ? undefined : canonicalStatus(layout, normalizeStatus(rawStatus)), text };
    if (!heading) problems.push(file + ' : titre « # ' + layout.prefix + '-' + nameId + ' — … » absent');
    else if (heading[1] !== nameId) problems.push(file + ' : identifiant du titre ' + layout.prefix + '-' + heading[1] + ' différent du nom du fichier');
    if (fiche.status === undefined) problems.push(file + ' : statut illisible');
    else if (!isAllowedStatus(layout, fiche.status)) problems.push(file + ' : statut « ' + fiche.status + ' » non autorisé');
    fiches.push(fiche);
  }
  const seen = new Map();
  for (const fiche of fiches) {
    if (seen.has(fiche.id)) problems.push(layout.prefix + '-' + fiche.id + ' : deux fiches (' + seen.get(fiche.id) + ', ' + fiche.file + ')');
    else seen.set(fiche.id, fiche.file);
  }
  return { fiches, problems };
}

// --- Index parsing ---------------------------------------------------------------------------------------
const rowPattern = prefix => new RegExp('^\\|\\s*\\[' + prefix + '-(\\d{3})\\]\\(([^)]*)\\)');

const splitCells = line => line.split(CELL_SPLIT);

function parseRows(lines, masked, prefix) {
  const pattern = rowPattern(prefix);
  let section = null;
  const rows = [];
  lines.forEach((line, index) => {
    if (masked[index]) return;
    if (/^## /.test(line)) section = line.slice(3).trim();
    const match = line.match(pattern);
    if (match) rows.push({ index, id: match[1], target: match[2], section, cells: splitCells(line) });
  });
  return rows;
}

function headerOf(lines, masked) {
  const index = lines.findIndex((line, i) => !masked[i] && /^\|\s*ID\s*\|/.test(line));
  if (index < 0) return null;
  return { line: lines[index], separator: lines[index + 1], cells: splitCells(lines[index]) };
}

function statusColumn(lines, masked) {
  const column = headerOf(lines, masked)?.cells.findIndex(cell => cell.trim() === 'Statut');
  return column > 0 ? column : 3;
}

function locateSections(lines, masked, layout) {
  if (layout.sections === null) {
    const h1 = lines.findIndex((line, i) => !masked[i] && /^# /.test(line));
    return [{ def: { title: null, statuses: null, insert: 'auto' }, start: h1 + 1, end: lines.length }];
  }
  const headings = [];
  lines.forEach((line, i) => { if (!masked[i] && /^## /.test(line)) headings.push({ title: line.slice(3).trim(), i }); });
  return layout.sections.map(def => {
    const at = headings.findIndex(heading => heading.title === def.title);
    if (at < 0) return { def, missing: true };
    return { def, start: headings[at].i + 1, end: headings[at + 1]?.i ?? lines.length };
  });
}

const targetOf = (layout, status) => layout.sections?.find(section => section.statuses.includes(status));

function detachedBugRows(lines, masked, rows, sections, layout) {
  if (layout.prefix !== 'BUG') return [];
  const rowIndexes = new Set(rows.map(row => row.index));
  return rows.flatMap(row => {
    const section = sections.find(candidate => candidate.def.title === row.section && !candidate.missing);
    if (!section) return [];
    let previous = row.index - 1;
    while (previous >= section.start && !masked[previous] && lines[previous].trim() === '') previous--;
    if (previous === row.index - 1 || previous < section.start || masked[previous]) return [];
    const afterHeader = previous > section.start && !masked[previous - 1]
      && /^\|\s*ID\s*\|/.test(lines[previous - 1])
      && /^\|(?:\s*:?-+:?\s*\|)+\s*$/.test(lines[previous]);
    return rowIndexes.has(previous) || afterHeader ? [{ row, start: previous + 1 }] : [];
  });
}

// --- Analysis --------------------------------------------------------------------------------------------
const maxId = ids => ids.reduce((maximum, id) => Math.max(maximum, Number(id)), 0);

// A next id below the largest used one risks reuse; a larger one is a deliberate gap and is left alone.
function checkNextId(lines, masked, prefix, ids, file) {
  const index = lines.findIndex((line, i) => !masked[i] && /Prochain identifiant libre\s*:/.test(line));
  if (index < 0) return [file + ' : mention « Prochain identifiant libre : ' + prefix + '-XXX » absente'];
  const match = lines[index].match(new RegExp('Prochain identifiant libre\\s*:\\s*' + prefix + '-(\\d{3,})'));
  if (!match) return [file + ' : « Prochain identifiant libre » illisible'];
  const maximum = maxId(ids);
  if (Number(match[1]) <= maximum) return [file + ' : prochain identifiant ' + prefix + '-' + match[1] + ' déjà attribué (le plus grand est ' + prefix + '-' + String(maximum).padStart(3, '0') + ')'];
  return [];
}

function analyzeRegister(root, layout) {
  const errors = [];
  const { fiches, problems } = loadFiches(root, layout);
  errors.push(...problems);
  const indexFile = layout.dir + '/INDEX.md';
  if (!exists(root, indexFile)) return { errors: [...errors, indexFile + ' : index absent'], fiches };
  const { text } = readText(root, indexFile);
  const lines = text.split('\n');
  const masked = maskLines(lines);
  const rows = parseRows(lines, masked, layout.prefix);
  const sections = locateSections(lines, masked, layout);
  for (const { row } of detachedBugRows(lines, masked, rows, sections, layout)) {
    errors.push(indexFile + ':' + (row.index + 1) + ' : BUG-' + row.id + ' séparé du tableau par une ligne vide');
  }
  const column = statusColumn(lines, masked);
  const byId = new Map(fiches.map(fiche => [fiche.id, fiche]));
  const indexDir = path.posix.dirname(indexFile);
  const seen = new Set();
  const name = id => layout.prefix + '-' + id;

  for (const section of sections) if (section.missing) errors.push(indexFile + ' : section « ' + section.def.title + ' » absente');
  for (const row of rows) {
    const where = indexFile + ' : ' + name(row.id);
    if (seen.has(row.id)) { errors.push(where + ' présent deux fois'); continue; }
    seen.add(row.id);
    const fiche = byId.get(row.id);
    if (!fiche) { errors.push(where + ' sans fiche'); continue; }
    const target = path.posix.normalize(path.posix.join(indexDir, decodeURI(row.target.split('#')[0])));
    if (target !== fiche.file) errors.push(where + ' : lien vers ' + target + ' au lieu de ' + fiche.file);
    if (fiche.status === undefined) continue;
    const shown = canonicalStatus(layout, normalizeStatus(row.cells[column] ?? ''));
    if (shown !== fiche.status) errors.push(where + ' : statut « ' + shown + ' » dans l’index, « ' + fiche.status + ' » dans la fiche');
    if (layout.sections) {
      const expected = targetOf(layout, fiche.status);
      if (!layout.sections.some(def => def.title === row.section)) errors.push(where + ' hors des sections de l’index (« ' + (row.section ?? 'sans titre') + ' »)');
      else if (expected && expected.title !== row.section) errors.push(where + ' : statut « ' + fiche.status + ' » dans « ' + row.section + ' » au lieu de « ' + expected.title + ' »');
    }
  }
  for (const fiche of fiches) if (!seen.has(fiche.id)) errors.push(indexFile + ' : ' + name(fiche.id) + ' absente de l’index (' + fiche.file + ')');
  for (const section of sections) {
    if (section.missing) continue;
    const body = lines.slice(section.start, section.end);
    const hasRows = body.some((line, i) => !masked[section.start + i] && rowPattern(layout.prefix).test(line));
    if (hasRows && body.some((line, i) => !masked[section.start + i] && PLACEHOLDER.test(line))) {
      errors.push(indexFile + ' : section « ' + (section.def.title ?? 'index') + ' » : mention « Aucun… » avec des lignes');
    }
  }
  errors.push(...checkNextId(lines, masked, layout.prefix, [...fiches.map(f => f.id), ...rows.map(r => r.id)], indexFile));
  return { errors, fiches };
}

function questionIds(lines, masked) {
  const table = [];
  const sections = [];
  lines.forEach((line, i) => {
    if (masked[i]) return;
    const row = line.match(/^\|\s*\[?Q-(\d{3})\b/);
    if (row) table.push(row[1]);
    const heading = line.match(/^#{2,3}\s+Q-(\d{3})\b/);
    if (heading) sections.push(heading[1]);
  });
  return { table, sections };
}

function analyzeQuestions(root) {
  const file = 'docs/QUESTIONS.md';
  if (!exists(root, file)) return [file + ' : fichier absent'];
  const errors = [];
  const lines = readText(root, file).text.split('\n');
  const masked = maskLines(lines);
  const { table, sections } = questionIds(lines, masked);
  for (const id of new Set(table)) if (!sections.includes(id)) errors.push(file + ' : Q-' + id + ' dans le tableau sans section');
  for (const id of new Set(sections)) if (!table.includes(id)) errors.push(file + ' : Q-' + id + ' avec une section sans ligne de tableau');
  for (const id of new Set(table)) if (table.indexOf(id) !== table.lastIndexOf(id)) errors.push(file + ' : Q-' + id + ' présent deux fois dans le tableau');
  const resolved = [];
  if (exists(root, 'docs/archive/QUESTIONS-RESOLUES.md')) {
    const archive = readText(root, 'docs/archive/QUESTIONS-RESOLUES.md').text;
    for (const match of archive.matchAll(/^(?:#{2,3} |\| ?\[?)Q-(\d{3})\b/gm)) resolved.push(match[1]);
  }
  errors.push(...checkNextId(lines, masked, 'Q', [...table, ...sections, ...resolved], file));
  return errors;
}

// --- Next task -------------------------------------------------------------------------------------------
export const nextTaskLocation = profile => profile === 'complet' ? 'docs/03-ROADMAP.md' : 'docs/tasks/INDEX.md';

export function checkNextTask(root, profile, tasks) {
  const file = nextTaskLocation(profile);
  if (!exists(root, file)) return [file + ' : fichier absent (section « Prochaine tâche » attendue)'];
  const lines = readText(root, file).text.split('\n');
  const masked = maskLines(lines);
  const headings = [];
  lines.forEach((line, i) => { if (!masked[i] && /^## /.test(line)) headings.push({ title: line.slice(3).trim(), i }); });
  const sections = headings.filter(heading => heading.title === 'Prochaine tâche');
  if (sections.length !== 1) return [file + ' : section « Prochaine tâche » ' + (sections.length === 0 ? 'manquante' : 'en double')];
  const errors = [];
  if (profile === 'leger' && headings[0] !== sections[0]) errors.push(file + ' : « Prochaine tâche » doit être la première section (profil léger)');
  const at = headings.indexOf(sections[0]);
  const end = headings[at + 1]?.i ?? lines.length;
  const content = [];
  for (let i = sections[0].i + 1; i < end; i++) {
    const line = lines[i].replace(/<!--.*?-->/g, '').trim();
    if (!masked[i] && line) content.push(line);
  }
  if (content.length !== 1) return [...errors, file + ' : « Prochaine tâche » : une seule ligne attendue, ' + content.length + ' trouvée(s)'];
  if (content[0] === NO_NEXT_TASK) return errors;
  const link = content[0].match(/^(?:[-*]\s+)?\[[^\]]+\]\(([^)\s]+)\)\.?$/);
  if (!link) return [...errors, file + ' : « Prochaine tâche » illisible : ' + content[0] + ' (un seul lien, ou « ' + NO_NEXT_TASK + ' »)'];
  const target = path.posix.normalize(path.posix.join(path.posix.dirname(file), decodeURI(link[1].split('#')[0])));
  const task = tasks.find(candidate => candidate.file === target);
  if (!task) return [...errors, file + ' : « Prochaine tâche » pointe vers ' + target + ', qui n’est pas une fiche TASK'];
  const label = 'TASK-' + task.id;
  if (task.status !== undefined && !['À faire', 'En cours'].includes(task.status)) errors.push(file + ' : « Prochaine tâche » ' + label + ' au statut « ' + task.status + ' »');
  const dependencies = [...new Set([...(field(task.text, 'Dépendances') ?? '').matchAll(/TASK-(\d{3})/g)].map(match => match[1]))];
  for (const dependency of dependencies) {
    const found = tasks.find(candidate => candidate.id === dependency);
    if (found?.status !== 'Terminée') errors.push(file + ' : « Prochaine tâche » ' + label + ' : dépendance TASK-' + dependency + ' non terminée' + (found ? '' : ' (fiche absente)'));
  }
  return errors;
}

// --- Fix -------------------------------------------------------------------------------------------------
const sameList = (a, b) => a.length === b.length && a.every((value, i) => value === b[i]);
const trimBlank = list => {
  let from = 0;
  let to = list.length;
  while (from < to && list[from].trim() === '') from++;
  while (to > from && list[to - 1].trim() === '') to--;
  return list.slice(from, to);
};

function buildRow(layout, header, fiche, indexDir) {
  const names = header.cells.slice(1, -1).map(cell => cell.trim());
  const values = names.map(name => {
    if (name === 'ID') return '[' + layout.prefix + '-' + fiche.id + '](' + path.posix.relative(indexDir, fiche.file) + ')';
    if (name === 'Titre') return (fiche.title ?? '').replace(/\|/g, '\\|');
    if (name === 'Statut') return fiche.status;
    if (name === 'Décision courte') return 'À compléter.';
    return normalizeStatus(field(fiche.text, name) ?? '');
  });
  return '| ' + values.join(' | ') + ' |';
}

function defaultHeader(layout, profile) {
  const columns = layout.prefix === 'TASK' ? (profile === 'complet' ? ['ID', 'Titre', 'Statut', 'Phase', 'Priorité'] : ['ID', 'Titre', 'Statut', 'Priorité'])
    : layout.prefix === 'BUG' ? ['ID', 'Titre', 'Statut', 'Gravité', 'Composant'] : ['ID', 'Titre', 'Statut', 'Décision courte'];
  const line = '| ' + columns.join(' | ') + ' |';
  return { line, separator: '|' + columns.map(() => '---').join('|') + '|', cells: splitCells(line) };
}

export function fixRegister(text, layout, fiches, profile) {
  const indexDir = path.posix.dirname(layout.dir + '/INDEX.md');
  const lines = text.split('\n');
  const masked = maskLines(lines);
  const rows = parseRows(lines, masked, layout.prefix);
  const column = statusColumn(lines, masked);
  const header = headerOf(lines, masked) ?? defaultHeader(layout, profile);
  const sections = locateSections(lines, masked, layout);
  const byId = new Map(fiches.map(fiche => [fiche.id, fiche]));
  const sectionOf = row => sections.find(section => !section.missing && (layout.sections === null || section.def.title === row.section));
  const placed = new Map(sections.map(section => [section, []]));
  const moved = [];
  const seen = new Set();
  const dropped = new Set();

  for (const row of rows) {
    const section = sectionOf(row);
    if (!section) continue;
    if (seen.has(row.id)) { dropped.add(row.index); continue; }
    seen.add(row.id);
    let line = lines[row.index];
    const fiche = byId.get(row.id);
    if (fiche) {
      const cells = splitCells(line);
      // A shown précision (« Acceptée ; … ») is kept while its status matches the fiche.
      if (fiche.status !== undefined && canonicalStatus(layout, normalizeStatus(cells[column] ?? '')) !== fiche.status) cells[column] = ' ' + fiche.status + ' ';
      line = cells.join('|').replace(/^(\|\s*\[[^\]]+\]\()[^)]*(\))/, '$1' + path.posix.relative(indexDir, fiche.file) + '$2');
      const target = fiche.status === undefined ? undefined : targetOf(layout, fiche.status);
      const reachable = target && sections.some(candidate => !candidate.missing && candidate.def.title === target.title);
      if (reachable && target.title !== section.def.title) { moved.push({ target, id: row.id, line }); continue; }
    }
    placed.get(section).push({ id: row.id, line });
  }
  for (const fiche of fiches) {
    if (seen.has(fiche.id) || fiche.status === undefined || !isAllowedStatus(layout, fiche.status)) continue;
    const target = layout.sections === null ? sections[0].def : targetOf(layout, fiche.status);
    moved.push({ target, id: fiche.id, line: buildRow(layout, header, fiche, indexDir) });
  }
  for (const entry of moved) {
    const section = sections.find(candidate => !candidate.missing && candidate.def.title === entry.target.title);
    if (!section) continue;
    const list = placed.get(section);
    const insert = section.def.insert === 'auto' ? (list.length >= 2 && list[0].id > list[list.length - 1].id ? 'desc' : 'asc') : section.def.insert;
    if (insert === 'first') list.unshift(entry);
    else {
      const after = list.findIndex(candidate => insert === 'asc' ? candidate.id > entry.id : candidate.id < entry.id);
      list.splice(after < 0 ? list.length : after, 0, entry);
    }
  }

  const detachedBlanks = new Set();
  for (const { row, start } of detachedBugRows(lines, masked, rows, sections, layout)) {
    for (let i = start; i < row.index; i++) detachedBlanks.add(i);
  }
  const result = lines.slice();
  for (const section of sections.filter(candidate => !candidate.missing).reverse()) {
    const newRows = placed.get(section).map(entry => entry.line);
    const body = lines.slice(section.start, section.end);
    const bodyMask = masked.slice(section.start, section.end);
    const isRow = (line, i) => !bodyMask[i] && line.startsWith('|');
    const originalRows = body.filter((line, i) => !bodyMask[i] && rowPattern(layout.prefix).test(line));
    const hasPlaceholder = body.some((line, i) => !bodyMask[i] && PLACEHOLDER.test(line));
    const hasDropped = body.some((line, i) => dropped.has(section.start + i));
    if (sameList(newRows, originalRows) && !(newRows.length > 0 && hasPlaceholder) && !hasDropped) {
      for (let i = section.end - 1; i >= section.start; i--) {
        if (detachedBlanks.has(i)) result.splice(i, 1);
      }
      continue;
    }
    const first = body.findIndex(isRow);
    const last = body.map(isRow).lastIndexOf(true);
    const keep = (line, i) => bodyMask[i] || !PLACEHOLDER.test(line);
    const placeholderAt = body.findIndex((line, i) => !bodyMask[i] && PLACEHOLDER.test(line));
    const split = first >= 0 ? first : Math.max(placeholderAt, 0);
    const resume = first >= 0 ? last + 1 : (placeholderAt >= 0 ? placeholderAt + 1 : 0);
    const pre = trimBlank(body.slice(0, split).filter(keep));
    const post = trimBlank(body.slice(resume).filter(keep));
    const block = newRows.length > 0 ? [header.line, header.separator, ...newRows]
      : [...(layout.keepEmptyTable ? [header.line, header.separator, ''] : []), layout.empty];
    const parts = [];
    if (pre.length > 0) parts.push(...pre, '');
    parts.push(...block);
    if (post.length > 0) parts.push('', ...post);
    result.splice(section.start, section.end - section.start, '', ...parts, '');
  }
  let output = result.join('\n').replace(/\n{3,}/g, '\n\n');
  const maximum = maxId([...fiches.map(fiche => fiche.id), ...rows.map(row => row.id)]);
  output = output.replace(new RegExp('(Prochain identifiant libre\\s*:\\s*' + layout.prefix + '-)(\\d{3,})'), (match, head, value) =>
    Number(value) <= maximum ? head + String(maximum + 1).padStart(3, '0') : match);
  return output;
}

function fixQuestionsNextId(text) {
  const lines = text.split('\n');
  const { table, sections } = questionIds(lines, maskLines(lines));
  const maximum = maxId([...table, ...sections]);
  return text.replace(/(Prochain identifiant libre\s*:\s*Q-)(\d{3,})/, (match, head, value) => Number(value) <= maximum ? head + String(maximum + 1).padStart(3, '0') : match);
}

// --- Entry point -----------------------------------------------------------------------------------------
export function checkProject(root, profile) {
  const errors = [];
  const tasks = [];
  for (const layout of Object.values(LAYOUTS)) {
    const result = analyzeRegister(root, layout);
    errors.push(...result.errors);
    if (layout.prefix === 'TASK') tasks.push(...result.fiches);
  }
  errors.push(...analyzeQuestions(root));
  errors.push(...checkNextTask(root, profile, tasks));
  return errors;
}

function applyFixes(root, profile) {
  const fixed = [];
  for (const layout of Object.values(LAYOUTS)) {
    const indexFile = layout.dir + '/INDEX.md';
    if (!exists(root, indexFile)) continue;
    const file = readText(root, indexFile);
    const updated = fixRegister(file.text, layout, loadFiches(root, layout).fiches, profile);
    if (updated !== file.text) { writeText(root, indexFile, file, updated); fixed.push(indexFile); }
  }
  if (exists(root, 'docs/QUESTIONS.md')) {
    const file = readText(root, 'docs/QUESTIONS.md');
    const updated = fixQuestionsNextId(file.text);
    if (updated !== file.text) { writeText(root, 'docs/QUESTIONS.md', file, updated); fixed.push('docs/QUESTIONS.md'); }
  }
  return fixed;
}

function main(argv) {
  const unknown = argv.filter(arg => arg !== '--fix');
  if (unknown.length > 0) {
    console.error('Argument inconnu : ' + unknown.join(' ') + '\nUsage : node check_docs.mjs [--fix]  (depuis la racine du projet)');
    return 2;
  }
  const root = process.cwd();
  if (!exists(root, 'docs')) { console.error('Dossier docs/ introuvable : lancer depuis la racine du projet'); return 2; }
  const { profile, inferred } = readProfile(root);
  const fix = argv.includes('--fix');
  if (fix) for (const file of applyFixes(root, profile)) console.log('Corrigé : ' + file);
  const errors = checkProject(root, profile);
  for (const error of errors) console.error(error);
  if (errors.length > 0) {
    console.error(errors.length + ' écart(s) de documentation' + (fix ? ' après correction' : ' (check_docs.mjs --fix corrige les index)'));
    return 1;
  }
  console.log('Documentation cohérente : index des tâches, anomalies, décisions et questions, prochaine tâche (profil ' + (profile === 'complet' ? 'complet' : 'léger') + (inferred ? ', déduit' : '') + ')');
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error('Erreur : ' + error.message); process.exitCode = 2; }
}
