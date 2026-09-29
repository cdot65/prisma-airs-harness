import {readFile, writeFile, mkdir, rm, access} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.resolve(site, '..');
const commit = execFileSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8'}).trim();
const manifest = JSON.parse(await readFile(path.join(site, 'sources.json'), 'utf8'));
const routes = new Map(manifest.map(row => [row.source, `/prisma-airs-harness${row.slug}/`]));
const output = path.join(site, 'docs/generated');
await rm(output, {recursive: true, force: true});
await mkdir(path.join(output, 'reference'), {recursive: true});
const inputs = [];
async function readSource(source) {
  const text = await readFile(path.join(root, source), 'utf8');
  inputs.push({path: source, sha256: createHash('sha256').update(text).digest('hex')});
  return text;
}
function sourceUrl(source) {
  return `https://git.cdot.io/cdot/prisma-airs-harness/src/commit/${commit}/${source}`;
}
const staticPrefix = 'docs-site/static/';
const missingStatic = [];
async function checkStatic(text) {
  for (const [, href] of text.matchAll(/\]\(([^\s)]+)\)/g)) {
    if (!href.startsWith(staticPrefix)) continue;
    try { await access(path.join(root, href)); } catch { missingStatic.push(href); }
  }
}
function rewriteLinks(text, source) {
  return text.replace(/\]\(([^\s)]+)\)/g, (match, href) => {
    const ownPrefix = 'https://github.com/cdot65/prisma-airs-harness/blob/main/';
    const fromRoot = href.startsWith(ownPrefix);
    if (fromRoot) href = href.slice(ownPrefix.length);
    if (/^(?:[a-z]+:|\/|#)/i.test(href)) return match;
    const [file, anchor] = href.split('#');
    const target = path.posix.normalize(path.posix.join(fromRoot ? '' : path.posix.dirname(source), file));
    if (target.startsWith(staticPrefix)) return `](/prisma-airs-harness/${target.slice(staticPrefix.length)})`;
    const url = routes.get(target) ?? sourceUrl(target);
    return `](${url}${anchor ? `#${anchor}` : ''})`;
  });
}
for (const {source, id, slug, title} of manifest) {
  const original = await readSource(source);
  await checkStatic(original);
  const content = rewriteLinks(original, source).replace(/^# .+\n/, '');
  await writeFile(path.join(output, `${id}.md`),
    `---\ntitle: ${JSON.stringify(title)}\nslug: ${slug}\n---\n\n` +
    `Maintained in [${source}](${sourceUrl(source)}). Generated from the same repository revision as this site.\n\n${content}`);
}
if (missingStatic.length) throw new Error(`Missing images referenced by guides: ${[...new Set(missingStatic)].join(', ')}`);
const commands = [
  ['airs', 'airs_harness_help'], ['env', 'airs_environment_help'],
  ['env-create', 'airs_environment_create_help'], ['login', 'airs_login_help'],
  ['mcp-login', 'airs_mcp_login_help'], ['doctor', 'airs_doctor_help'],
  ['typesafe', 'airs_environment_typesafe_help'],
];
for (const [name, fixture] of commands) {
  const source = `codex-rs/cli/src/snapshots/airs_harness__airs_help__tests__${fixture}.snap`;
  const snapshot = await readSource(source);
  const help = snapshot.replace(/^---\n[\s\S]*?\n---\n/, '');
  if (!help.includes('Usage:')) throw new Error(`Missing help in ${source}`);
  await writeFile(path.join(output, 'reference', `${name}.md`),
    `---\ntitle: ${JSON.stringify(name === 'airs' ? 'airs' : `airs ${name === 'typesafe' ? 'env typesafe' : name.replaceAll('-', ' ')}`)}\nslug: /reference/${name}\n---\n\n` +
    `Generated from the [Rust CLI help snapshot](${sourceUrl(source)}). This reference follows the source tree; an installed stable release can expose fewer options. Check \`airs --help\` on your installation.\n\n\`\`\`text\n${help}\`\`\`\n`);
}
const pkg = JSON.parse(await readSource('npm/airs-harness/package.json'));
await writeFile(path.join(site, 'static/source.json'), JSON.stringify({commit,
  sourceVersion: pkg.version, bundledCli: pkg.dependencies['@cdot65/prisma-airs-cli'], inputs}, null, 2) + '\n');
console.log(`Generated ${manifest.length} guides and ${commands.length} references from ${commit}`);
