// First-play time (ROADMAP step 2): fresh profile, cache disabled, DevTools
// "Fast 4G" + 4x CPU, from navigation to the Keep selected by a tap.
// usage (from village/): bun tools/first_play.mjs http://127.0.0.1:8790/ [runs]
import { chromium } from '../.exact/checkout/node_modules/playwright-core/index.mjs';
const url = process.argv[2], runs = Number(process.argv[3] ?? 3);
// Chrome DevTools' Fast 4G preset (front_end/core/sdk/NetworkManager.ts).
const FAST_4G = { offline: false, latency: 60 * 2.75, downloadThroughput: 9e6 / 8 * 0.9, uploadThroughput: 1.5e6 / 8 * 0.9 };
const results = [];
for (let r = 0; r < runs; r++) {
  const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: false, args: ['--enable-unsafe-webgpu'] });
  const context = await browser.newContext({ viewport: { width: 1054, height: 722 } }); // new context = fresh profile
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('Network.enable');
  await cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
  await cdp.send('Network.emulateNetworkConditions', FAST_4G);
  await cdp.send('Emulation.setCPUThrottlingRate', { rate: 4 });
  const t0 = Date.now();
  await page.goto(url, { waitUntil: 'commit' });
  const play = page.getByRole('button', { name: 'Play' });
  await play.waitFor({ timeout: 120000 });
  const tPlay = Date.now() - t0;
  await play.click();
  let tKeep = null;
  while (Date.now() - t0 < 120000) {
    await page.mouse.click(527, 275);
    try { await page.getByText(/Keep · level \d/).first().waitFor({ timeout: 400 }); tKeep = Date.now() - t0; break; } catch {}
  }
  const bytes = await page.evaluate(() => performance.getEntriesByType('resource').reduce((s, e) => s + (e.transferSize || 0), 0));
  await page.screenshot({ path: `artifacts/first-play-run${r}.png` });
  results.push({ run: r, play_button_ms: tPlay, keep_selected_ms: tKeep, transferred_bytes: bytes });
  console.log(JSON.stringify(results.at(-1)));
  await browser.close();
}
const ok = results.map(r => r.keep_selected_ms).filter(Boolean).sort((a, b) => a - b);
console.log('median keep_selected_ms', ok[Math.floor(ok.length / 2)] ?? 'none');
