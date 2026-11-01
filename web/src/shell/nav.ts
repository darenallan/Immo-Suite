// Entrées de menu par rôle : copie du tableau « Entrées de menu par rôle » de UI_UX_DESIGN.md (test : nav.test.ts le relit dans le
// document). Une entrée que la matrice ne définit pas est masquée (DP-66). Le serveur applique le droit quoi qu'il arrive.
import { fr, type RoleKey } from '../texts/fr';

export type EntryId = keyof typeof fr.nav.entries;

/** `yes` : visible. `setting` : visible seulement si le droit réglable correspondant est accordé (`permissions[]`). */
type Visibility = 'yes' | 'setting' | 'no';

export interface NavEntry {
  id: EntryId;
  path: string;
  /** Droit exigé en plus pour les entrées réglables. */
  permission?: string;
  /** Visibilité par rôle. */
  roles: Record<RoleKey, Visibility>;
}

const row = (
  manager: Visibility,
  property_manager: Visibility,
  cashier: Visibility,
  field_agent: Visibility,
  accountant: Visibility,
  owner: Visibility,
  collaborator: Visibility,
): Record<RoleKey, Visibility> => ({
  manager,
  property_manager,
  cashier,
  field_agent,
  accountant,
  owner,
  collaborator,
});

export const NAV_ENTRIES: readonly NavEntry[] = [
  { id: 'home', path: '/', roles: row('yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes') },
  {
    id: 'collect',
    path: '/encaisser',
    permission: 'payment.record',
    roles: row('yes', 'setting', 'yes', 'no', 'no', 'yes', 'setting'),
  },
  { id: 'arrears', path: '/impayes', roles: row('yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes') },
  { id: 'payments', path: '/paiements', roles: row('yes', 'yes', 'yes', 'no', 'yes', 'yes', 'yes') },
  { id: 'properties', path: '/biens', roles: row('yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes') },
  { id: 'tenants', path: '/locataires', roles: row('yes', 'yes', 'yes', 'yes', 'yes', 'yes', 'yes') },
  { id: 'leases', path: '/baux', roles: row('yes', 'yes', 'no', 'no', 'no', 'yes', 'no') },
  { id: 'statements', path: '/comptes-rendus', roles: row('yes', 'yes', 'no', 'no', 'yes', 'no', 'no') },
  { id: 'mandates', path: '/mandants', roles: row('yes', 'no', 'no', 'no', 'no', 'no', 'no') },
  { id: 'works', path: '/travaux', roles: row('yes', 'yes', 'no', 'no', 'no', 'yes', 'no') },
  { id: 'agenda', path: '/agenda', roles: row('yes', 'yes', 'no', 'no', 'no', 'yes', 'no') },
  {
    id: 'import',
    path: '/import',
    permission: 'import.run',
    roles: row('yes', 'setting', 'no', 'no', 'no', 'yes', 'no'),
  },
  { id: 'cashClose', path: '/cloture-caisse', roles: row('yes', 'no', 'yes', 'no', 'no', 'yes', 'no') },
  { id: 'notifications', path: '/notifications', roles: row('yes', 'yes', 'no', 'no', 'no', 'yes', 'no') },
  { id: 'settings', path: '/parametres', roles: row('yes', 'no', 'no', 'no', 'no', 'yes', 'no') },
  { id: 'users', path: '/utilisateurs', roles: row('yes', 'no', 'no', 'no', 'no', 'yes', 'no') },
  { id: 'audit', path: '/journal-audit', roles: row('yes', 'no', 'no', 'no', 'yes', 'yes', 'no') },
];

export function isRole(value: string): value is RoleKey {
  return value in fr.roles;
}

/** L'entrée est-elle visible pour ce rôle, compte tenu des droits accordés par le serveur ? */
export function isVisible(entry: NavEntry, role: RoleKey, permissions: readonly string[]): boolean {
  const v = entry.roles[role];
  if (v === 'yes') return true;
  if (v === 'setting') return entry.permission !== undefined && permissions.includes(entry.permission);
  return false;
}

export function visibleEntries(role: RoleKey, permissions: readonly string[]): NavEntry[] {
  return NAV_ENTRIES.filter((e) => isVisible(e, role, permissions));
}

/** Barre du bas (< 1024 px) : toujours 4 entrées, la dernière est « Plus ». */
export function bottomBar(role: RoleKey, permissions: readonly string[]): EntryId[] {
  const can = (id: EntryId) => NAV_ENTRIES.some((e) => e.id === id && isVisible(e, role, permissions));
  const first: EntryId[] = ['home'];
  if (role === 'field_agent') return [...first, 'arrears', 'tenants'];
  if (role === 'accountant') return [...first, 'arrears', 'payments'];
  // gérant, caissier, propriétaire, gestionnaire, collaborateur : Encaisser, ou Locataires si le droit d'encaisser est retiré
  return [...first, can('collect') ? 'collect' : 'tenants', 'arrears'];
}
