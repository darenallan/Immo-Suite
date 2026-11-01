// F02 : « pour chaque rôle, les entrées visibles sont exactement celles du tableau de UI_UX_DESIGN.md » (17 entrées × 7 rôles).
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { fr, type RoleKey } from '../texts/fr';
import { NAV_ENTRIES, bottomBar, visibleEntries } from './nav';

const doc = readFileSync(resolve(process.cwd(), '..', 'UI_UX_DESIGN.md'), 'utf8');

const ROLE_ORDER: RoleKey[] = [
  'manager',
  'property_manager',
  'cashier',
  'field_agent',
  'accountant',
  'owner',
  'collaborator',
];

type Cell = 'yes' | 'setting' | 'no';

function parseCell(raw: string): Cell {
  const c = raw.trim();
  if (c === '—') return 'no';
  if (c.startsWith('Proposer')) return 'no'; // refusé au pilote (DP-66)
  if (c.includes('réglable') || c.includes('si autorisé')) return 'setting';
  if (c.startsWith('É') || c.startsWith('L')) return 'yes';
  throw new Error(`cellule illisible : « ${raw} »`);
}

function expectedTable(): Map<string, Cell[]> {
  const start = doc.indexOf('| Entrée | Gérant |');
  expect(start).toBeGreaterThan(0);
  const rows = new Map<string, Cell[]>();
  for (const line of doc.slice(start).split('\n').slice(2)) {
    if (!line.startsWith('|')) break;
    const cells = line.split('|').map((c) => c.trim());
    // cells : ['', entrée, G, Ge, C, A, Co, P, Col, '']
    rows.set(cells[1] ?? '', cells.slice(2, 9).map(parseCell));
  }
  return rows;
}

describe('entrées de menu', () => {
  it('le tableau du document compte 17 entrées', () => {
    expect(expectedTable().size).toBe(17);
    expect(NAV_ENTRIES).toHaveLength(17);
  });

  it('chaque entrée et chaque rôle correspondent au tableau de UI_UX_DESIGN.md (119 cellules)', () => {
    const expected = expectedTable();
    let checked = 0;
    for (const entry of NAV_ENTRIES) {
      const label = fr.nav.entries[entry.id];
      // le document écrit l'apostrophe droite, l'interface l'apostrophe typographique
      const cells = expected.get(label.replaceAll('’', "'"));
      expect(cells, `entrée absente du document : ${label}`).toBeDefined();
      ROLE_ORDER.forEach((role, i) => {
        expect(entry.roles[role], `${label} × ${role}`).toBe(cells?.[i]);
        checked += 1;
      });
    }
    expect(checked).toBe(119);
  });

  it('les entrées réglables ne sont visibles que si le droit est accordé', () => {
    const labels = (role: RoleKey, perms: string[]) => visibleEntries(role, perms).map((e) => e.id);
    expect(labels('property_manager', [])).not.toContain('collect');
    expect(labels('property_manager', ['payment.record'])).toContain('collect');
    expect(labels('property_manager', [])).not.toContain('import');
    expect(labels('property_manager', ['import.run'])).toContain('import');
    expect(labels('collaborator', ['payment.record'])).toContain('collect');
    expect(labels('manager', [])).toContain('collect');
  });

  it('la barre du bas compte toujours 3 entrées avant « Plus », selon le rôle', () => {
    expect(bottomBar('manager', [])).toEqual(['home', 'collect', 'arrears']);
    expect(bottomBar('cashier', [])).toEqual(['home', 'collect', 'arrears']);
    expect(bottomBar('owner', [])).toEqual(['home', 'collect', 'arrears']);
    expect(bottomBar('field_agent', [])).toEqual(['home', 'arrears', 'tenants']);
    expect(bottomBar('accountant', [])).toEqual(['home', 'arrears', 'payments']);
    // gestionnaire sans droit d'encaisser : « Encaisser » est remplacé par « Locataires »
    expect(bottomBar('property_manager', [])).toEqual(['home', 'tenants', 'arrears']);
    expect(bottomBar('property_manager', ['payment.record'])).toEqual(['home', 'collect', 'arrears']);
  });
});
