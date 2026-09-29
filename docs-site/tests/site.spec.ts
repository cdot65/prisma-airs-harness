import {test, expect} from '@playwright/test';
import {readFileSync} from 'node:fs';

const manifest = JSON.parse(readFileSync('sources.json', 'utf8')) as {slug: string}[];
const routes = ['', ...manifest.map(row => row.slug.slice(1) + '/'),
  ...['environments', 'mcp', 'terminal', 'judge', 'releases', 'architecture', 'development', 'deployment']
    .map(name => `guides/${name}/`),
  ...['airs', 'env', 'env-create', 'login', 'mcp-login', 'doctor', 'typesafe']
    .map(name => `reference/${name}/`)];

for (const route of routes) {
  test(`renders ${route || 'home'}`, async ({page}) => {
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    const response = await page.goto(route || './');
    expect(response?.status()).toBe(200);
    await expect(page.locator('article h1')).toBeVisible();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await expect(page.locator('.navbar__title')).toHaveText('Prisma AIRS Harness');
    await expect(page.locator('article')).not.toContainText('Page Not Found');
    expect(errors).toEqual([]);
  });
}

test('desktop navigation, collapsible TOC, code and diagram', async ({page}) => {
  await page.setViewportSize({width: 1440, height: 1000});
  await page.goto('guides/terminal/');
  const toggle = page.getByRole('button', {name: 'Collapse on-page navigation'});
  await toggle.click();
  await expect(page.getByRole('button', {name: 'Expand on-page navigation'})).toHaveAttribute('aria-expanded', 'false');
  await page.getByRole('button', {name: 'Expand on-page navigation'}).click();
  await page.getByRole('link', {name: 'CLI Reference', exact: true}).first().click();
  await expect(page.locator('article pre')).toContainText('Usage: airs');
  await page.goto('guides/architecture/');
  await expect(page.locator('.docusaurus-mermaid-container svg')).toBeVisible();
});

test('mobile navigation and readable layout', async ({page}) => {
  await page.setViewportSize({width: 390, height: 844});
  await page.goto('./');
  await page.getByRole('button', {name: 'Toggle navigation bar'}).click();
  await expect(page.locator('.navbar-sidebar')).toBeVisible();
  await page.keyboard.press('Escape');
  await page.goto('getting-started/');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});

test('published provenance matches the build inputs', async ({request}) => {
  const response = await request.get('source.json');
  expect(response.ok()).toBe(true);
  const source = await response.json();
  const expected = JSON.parse(readFileSync('static/source.json', 'utf8'));
  expect(source).toEqual(expected);
  expect(source.inputs.length).toBe(14);
});
