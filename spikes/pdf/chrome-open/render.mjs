// E1 (navigateur maintenu ouvert) : un Chrome sans interface lancé une fois, une page par rendu.
// Commandes : render-all (40 PDF dans ../build/e1-open/) · bench (100 rendus A4, durées en JSON) · net (journal réseau)
import puppeteer from 'puppeteer-core';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const HERE = path.resolve(import.meta.dirname, '..');
const CHROME = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe';
const cases = fs.readdirSync(path.join(HERE, 'cases')).filter(f => f.endsWith('.json')).map(f => f.replace('.json', '')).sort();

const browser = await puppeteer.launch({
  executablePath: CHROME,
  headless: true,
  args: ['--disable-gpu', '--no-first-run', '--host-resolver-rules=MAP * ~NOTFOUND', `--user-data-dir=${path.join(HERE, 'build', 'chrome-profile-open')}`],
});

const requests = [];
async function render(id, fmt, out) {
  const page = await browser.newPage();
  await page.setRequestInterception(true);
  page.on('request', req => {
    requests.push(req.url());
    // toute requête qui n'est pas un fichier local est refusée et comptée
    if (!req.url().startsWith('file:')) req.abort(); else req.continue();
  });
  await page.goto(pathToFileURL(path.join(HERE, 'build', 'html', `${id}-${fmt}.html`)).href, { waitUntil: 'load' });
  await page.evaluate(() => document.fonts.ready);
  const pdf = await page.pdf({ preferCSSPageSize: true, printBackground: true, displayHeaderFooter: false });
  await page.close();
  if (out) fs.writeFileSync(out, pdf);
}

const cmd = process.argv[2];
if (cmd === 'render-all') {
  fs.mkdirSync(path.join(HERE, 'build', 'e1-open'), { recursive: true });
  const t0 = performance.now();
  for (const id of cases) for (const fmt of ['a4', 'a5']) await render(id, fmt, path.join(HERE, 'build', 'e1-open', `${id}-${fmt}.pdf`));
  console.log(`e1-open : 40 documents rendus en ${((performance.now() - t0) / 1000).toFixed(1)} s`);
} else if (cmd === 'bench') {
  for (const id of cases.slice(0, 3)) await render(id, 'a4', null); // échauffement non compté
  const times = [];
  for (let k = 0; k < 5; k++) for (const id of cases) {
    const t0 = performance.now();
    await render(id, 'a4', null);
    times.push(performance.now() - t0);
  }
  console.log(JSON.stringify(times));
}
const outside = requests.filter(u => !u.startsWith('file:'));
console.error(`requêtes : ${requests.length} au total, ${outside.length} hors fichiers locaux`);
await browser.close();
