// Checks that docs/03-ROADMAP.md and the task fiches agree: phases, tasks per phase, dependency order,
// milestones, and that the roadmap carries no status. Full profile only; light profile has no roadmap.
// Run from the project root, no argument: node tools/commun/check_roadmap.mjs.
// Why separate from check_docs: it needs the roadmap, which only the full profile has, and Test-Docs
// reports it as its own check so a roadmap drift is not mixed with index drift.
// Exit codes: 0 consistent or not applicable, 1 inconsistencies, 2 error. Node built-ins only (D-9).

import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { LAYOUTS, exists, field, loadFiches, maskLines, readProfile, readText } from './check_docs.mjs';

export const ROADMAP = 'docs/03-ROADMAP.md';
const OFF_SEQUENCE = /^Hors séquence/i;

// Phases are « ## Phase N — Titre » sections; the first cell of each table row names the task.
export function parseRoadmap(text) {
  const lines = text.split('\n');
  const masked = maskLines(lines);
  const phases = [];
  const errors = [];
  let current = null;
  lines.forEach((line, i) => {
    if (masked[i]) return;
    const heading = line.match(/^## (.+?)\s*$/);
    if (heading) {
      const phase = heading[1].match(/^Phase (\d+)\b/);
      current = phase ? { number: Number(phase[1]), title: heading[1], tasks: [], milestone: false } : null;
      if (current) phases.push(current);
      return;
    }
    if (!current) return;
    if (/^Jalon\b/i.test(line)) current.milestone = true;
    if (!line.startsWith('|')) return;
    const cells = line.split(/(?<!\\)\|/).map(cell => cell.trim());
    if (cells.slice(1, -1).some(cell => /^Statut$/i.test(cell))) {
      errors.push(current.title + ' : colonne de statut interdite, la roadmap porte l’ordre jamais les statuts');
    }
    for (const id of new Set([...(cells[1] ?? '').matchAll(/TASK-(\d{3})/g)].map(match => match[1]))) current.tasks.push(id);
  });
  return { phases, errors };
}

export function checkRoadmap(roadmapText, tasks) {
  const { phases, errors } = parseRoadmap(roadmapText);
  const tasksById = new Map(tasks.map(task => [task.id, task]));
  const ordered = new Map();

  phases.forEach((phase, index) => {
    if (index > 0 && phase.number !== phases[index - 1].number + 1) errors.push('« ' + phase.title + ' » : numérotation non consécutive après la phase ' + phases[index - 1].number);
    if (!phase.milestone) errors.push('« ' + phase.title + ' » : ligne « Jalon : … » absente');
    for (const id of phase.tasks) {
      if (!tasksById.has(id)) errors.push('« ' + phase.title + ' » : TASK-' + id + ' sans fiche');
      if (ordered.has(id)) errors.push('TASK-' + id + ' listée deux fois dans la roadmap');
      else ordered.set(id, ordered.size);
    }
  });

  const phaseOf = new Map();
  for (const task of tasks) {
    const value = field(task.text, 'Phase');
    const label = 'TASK-' + task.id;
    if (!value) { errors.push(label + ' : champ Phase absent (' + task.file + ')'); continue; }
    if (OFF_SEQUENCE.test(value)) { phaseOf.set(task.id, 'hors'); continue; }
    const number = value.match(/^Phase (\d+)\b/)?.[1];
    if (number === undefined) { errors.push(label + ' : Phase « ' + value + ' » illisible'); continue; }
    phaseOf.set(task.id, Number(number));
    const phase = phases.find(candidate => candidate.number === Number(number));
    if (!phase) errors.push(label + ' : Phase ' + number + ' absente de la roadmap');
    else {
      if (!phase.tasks.includes(task.id)) errors.push(label + ' : fiche en Phase ' + number + ' mais absente de la roadmap');
      // « Phase N » alone names the phase without repeating its title; a longer value must match it.
      if (!/^Phase \d+$/.test(value.trim()) && phase.title !== value.replace(/\s+/g, ' ').trim()) errors.push(label + ' : Phase « ' + value + ' » différente du titre de la roadmap « ' + phase.title + ' »');
    }
  }
  for (const phase of phases) {
    for (const id of phase.tasks) {
      const declared = phaseOf.get(id);
      if (tasksById.has(id) && declared !== undefined && declared !== phase.number) errors.push('TASK-' + id + ' listée en phase ' + phase.number + ', fiche en ' + (declared === 'hors' ? 'Hors séquence' : 'Phase ' + declared));
    }
  }

  for (const task of tasks) {
    const own = phaseOf.get(task.id);
    if (own === undefined || own === 'hors') continue;
    const dependencies = new Set([...(field(task.text, 'Dépendances') ?? '').matchAll(/TASK-(\d{3})/g)].map(match => match[1]));
    for (const dependency of dependencies) {
      const label = 'TASK-' + task.id + ' → TASK-' + dependency;
      if (!tasksById.has(dependency)) { errors.push('Dépendance inconnue : ' + label); continue; }
      const theirs = phaseOf.get(dependency);
      if (theirs === 'hors') errors.push('Dépendance hors séquence : ' + label);
      else if (theirs !== undefined && theirs > own) errors.push('Dépendance dans une phase ultérieure : ' + label);
      else if (ordered.has(dependency) && ordered.has(task.id) && ordered.get(dependency) > ordered.get(task.id)) errors.push('Dépendance placée après sa tâche dans la roadmap : ' + label);
    }
  }
  return errors;
}

function main(argv) {
  if (argv.length > 0) { console.error('Usage : node check_roadmap.mjs  (depuis la racine du projet, sans argument)'); return 2; }
  const root = process.cwd();
  const { profile } = readProfile(root);
  if (profile !== 'complet') { console.log('Non applicable : profil léger, pas de roadmap.'); return 0; }
  if (!exists(root, ROADMAP)) { console.error(ROADMAP + ' : fichier absent'); return 1; }
  const tasks = loadFiches(root, LAYOUTS.TASK).fiches;
  const errors = checkRoadmap(readText(root, ROADMAP).text, tasks);
  for (const error of errors) console.error(error);
  if (errors.length > 0) { console.error(errors.length + ' écart(s) de roadmap'); return 1; }
  console.log('Roadmap cohérente : ' + parseRoadmap(readText(root, ROADMAP).text).phases.length + ' phase(s), ' + tasks.length + ' fiche(s) de tâche');
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error('Erreur : ' + error.message); process.exitCode = 2; }
}
