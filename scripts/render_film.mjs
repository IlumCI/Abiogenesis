// Render a film.bin to MP4, frame by frame, through the viewer in headless Chromium.
//
//   node scripts/render_film.mjs FILM.bin OUT.mp4 [--fps 30] [--hold 3]
//
// Needs ffmpeg and playwright-core (`npm i playwright-core`, or set PLAYWRIGHT_CORE to its
// index.mjs). CHROME picks the browser binary; otherwise Playwright's own is used.
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";

const args = process.argv.slice(2);
const opt = (k, d) => { const i = args.indexOf(k); return i >= 0 ? args.splice(i, 2)[1] : d; };
const fps = Number(opt("--fps", 30)), hold = Number(opt("--hold", 3));
const [film, out] = args;
if (!film || !out) { console.error("usage: render_film.mjs FILM.bin OUT.mp4 [--fps 30] [--hold 3]"); process.exit(2); }

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const types = { ".html": "text/html; charset=utf-8", ".bin": "application/octet-stream" };
const filmAbs = path.resolve(film);
const server = http.createServer((req, res) => {
  const url = decodeURIComponent(req.url.split("?")[0]);
  const file = url === "/film.bin" ? filmAbs : path.join(root, url);
  if (!file.startsWith(root) && file !== filmAbs) { res.writeHead(403).end(); return; }
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404).end(); return; }
    res.writeHead(200, { "content-type": types[path.extname(file)] ?? "application/octet-stream" }).end(data);
  });
});
await new Promise(r => server.listen(0, "127.0.0.1", r));
const port = server.address().port;

const { chromium } = await import(process.env.PLAYWRIGHT_CORE ?? "playwright-core");
const browser = await chromium.launch({ executablePath: process.env.CHROME, args: ["--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1920, height: 1200 } });
await page.goto(`http://127.0.0.1:${port}/viewer/index.html#film=/film.bin&at=0`);
await page.waitForFunction(() => window.abioFrames && window.abioFrames() > 0, null, { timeout: 60000 });
await page.evaluate(() => document.fonts.ready);
const n = await page.evaluate(() => window.abioFrames());

const ff = spawn("ffmpeg", ["-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(fps), "-c:v", "png", "-i", "-",
  "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", "-preset", "slow", "-movflags", "+faststart", out], { stdio: ["pipe", "inherit", "inherit"] });
const write = buf => new Promise(r => ff.stdin.write(buf) ? r() : ff.stdin.once("drain", r));
let last = null;
for (let i = 0; i < n; i++) {
  const url = await page.evaluate(i => window.abioShot(i), i);
  last = Buffer.from(url.slice(url.indexOf(",") + 1), "base64");
  await write(last);
  if (i % 50 === 0) process.stderr.write(`frame ${i + 1}/${n}\r`);
}
for (let i = 0; i < fps * hold; i++) await write(last);
ff.stdin.end();
await new Promise(r => ff.on("close", r));
await browser.close();
server.close();
console.error(`\n${out}: ${n} frames at ${fps} fps, plus ${hold}s hold`);
