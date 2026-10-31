// Compare deux fichiers octet par octet ; supprime le second. Sert à vérifier que le client d'API est à jour (C3).
import { readFileSync, rmSync } from 'node:fs';

const [expected, generated] = process.argv.slice(2);
if (!expected || !generated) {
  console.error('usage : same-file.mjs <versionné> <généré>');
  process.exit(2);
}
const same = readFileSync(expected).equals(readFileSync(generated));
rmSync(generated);
if (!same) {
  console.error(`${expected} est périmé : relancer « npm run api:types » et valider le résultat`);
  process.exit(1);
}
