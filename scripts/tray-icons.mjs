import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';

// Each size has its own pixel grid, calibrated to the user's Default/Small Wi-Fi captures.
const drawings = {
  12: { stroke: 1, band: 'M1 7V5.5a5 5 0 0 1 10 0V7', cups: '<rect x="0" y="6.5" width="3" height="4" rx=".75"/><rect x="9" y="6.5" width="3" height="4" rx=".75"/>', cut: [9, 9, 3.5], shield: 'M9 6l3 1v2l-1 1.5-2 1-2-1L6 9V7z', pause: '<rect x="7" y="7" width="2" height="4"/><rect x="10" y="7" width="2" height="4"/>' },
  15: { stroke: 1.25, band: 'M1.5 8.5v-2a6 6 0 0 1 12 0v2', cups: '<rect x="0" y="8" width="4" height="5" rx="1"/><rect x="11" y="8" width="4" height="5" rx="1"/>', cut: [11, 11, 4.25], shield: 'M11 7.5l4 1.25V11l-1 2-3 1.5L8 13l-1-2V8.75z', pause: '<rect x="9" y="8" width="2" height="6"/><rect x="13" y="8" width="2" height="6"/>' },
  18: { stroke: 1.5, band: 'M1.75 10.25v-2a7.25 7.25 0 0 1 14.5 0v2', cups: '<rect x="0" y="9" width="4" height="7" rx="1.125"/><rect x="14" y="9" width="4" height="7" rx="1.125"/>', cut: [13.5, 13, 5], shield: 'M13.5 9l4.5 1.5V13l-1.25 2.5-3.25 2L10.25 15.5 9 13v-2.5z', pause: '<rect x="11" y="10" width="2" height="7" rx=".25"/><rect x="15" y="10" width="3" height="7" rx=".25"/>' },
  36: { stroke: 3, band: 'M3.5 20.5v-4a14.5 14.5 0 0 1 29 0v4', cups: '<rect x="0" y="18" width="9" height="14" rx="2.25"/><rect x="27" y="18" width="9" height="14" rx="2.25"/>', cut: [27, 27, 10], shield: 'M27 18l9 3v5l-2.5 5-6.5 4-6.5-4-2.5-5v-5z', pause: '<rect x="22" y="20" width="5" height="14" rx=".75"/><rect x="31" y="20" width="5" height="14" rx=".75"/>' },
  16: { stroke: 1.25, band: 'M1.5 9v-2a6.5 6.5 0 0 1 13 0v2', cups: '<rect x="0" y="8" width="4" height="6" rx="1"/><rect x="12" y="8" width="4" height="6" rx="1"/>', cut: [12, 12, 4.5], shield: 'M12 8l4 1.5V12l-1 2-3 1.5L9 14l-1-2V9.5z', pause: '<rect x="10" y="9" width="2" height="6" rx=".25"/><rect x="14" y="9" width="2" height="6" rx=".25"/>' },
  20: { stroke: 1.5, band: 'M1.75 10.75v-2a8.25 8.25 0 0 1 16.5 0v2', cups: '<rect x="0" y="10" width="5" height="8" rx="1.25"/><rect x="15" y="10" width="5" height="8" rx="1.25"/>', cut: [15, 15, 5.5], shield: 'M15 10l5 1.75V14c0 2-1.75 4-5 5-3.25-1-5-3-5-5v-2.25z', pause: '<rect x="12" y="11" width="3" height="7" rx=".5"/><rect x="17" y="11" width="3" height="7" rx=".5"/>' },
  24: { stroke: 1.75, band: 'M2.25 13.75v-3a9.75 9.75 0 0 1 19.5 0v3', cups: '<rect x="0" y="12" width="6" height="9" rx="1.5"/><rect x="18" y="12" width="6" height="9" rx="1.5"/>', cut: [18, 18, 6.5], shield: 'M18 12l6 2v3c0 2.5-2.5 5-6 6-3.5-1-6-3.5-6-6v-3z', pause: '<rect x="15" y="13" width="3" height="9" rx=".5"/><rect x="21" y="13" width="3" height="9" rx=".5"/>' },
  32: { stroke: 2.5, band: 'M3 18v-4a13 13 0 0 1 26 0v4', cups: '<rect x="0" y="16" width="8" height="12" rx="2"/><rect x="24" y="16" width="8" height="12" rx="2"/>', cut: [24, 24, 9], shield: 'M24 16l8 3v4c0 3-3 6-8 8-5-2-8-5-8-8v-4z', pause: '<rect x="20" y="18" width="4" height="12" rx=".75"/><rect x="28" y="18" width="4" height="12" rx=".75"/>' },
  48: { stroke: 3.5, band: 'M4.5 27v-6a19.5 19.5 0 0 1 39 0v6', cups: '<rect x="0" y="24" width="12" height="18" rx="3"/><rect x="36" y="24" width="12" height="18" rx="3"/>', cut: [36, 36, 13.5], shield: 'M36 24l12 4v7c0 4.5-5 9-12 12-7-3-12-7.5-12-12v-7z', pause: '<rect x="30" y="26" width="6" height="18" rx="1"/><rect x="42" y="26" width="6" height="18" rx="1"/>' },
};
for (const small of [false, true]) for (const baseVariant of ['light', 'dark', 'unprotected-light', 'unprotected-dark', 'paused-light', 'paused-dark']) {
  const variant = small ? "small-" + baseVariant : baseVariant;
  const entries = [];
  for (const size of [16, 20, 24, 32, 48]) {
    const extent = small ? size * .75 : size;
    const drawing = drawings[extent];
    const ink = variant.endsWith('dark') ? '#ffffff' : '#1a1a1a';
    const body = `<path d="${drawing.band}" fill="none" stroke="${ink}" stroke-width="${drawing.stroke}" stroke-linecap="round"/>${drawing.cups}`;
    const [cx, cy, r] = drawing.cut;
    const content = baseVariant.startsWith('unprotected') ? `<g opacity=".5">${body}</g>` : `<defs><mask id="cut" maskUnits="userSpaceOnUse" x="0" y="0" width="${size}" height="${size}"><rect width="${size}" height="${size}" fill="white"/><circle cx="${cx}" cy="${cy}" r="${r}" fill="black"/></mask></defs><g mask="url(#cut)">${body}</g>${baseVariant.startsWith('paused') ? drawing.pause : `<path d="${drawing.shield}"/>`}`;
    const inset = Math.floor((size - extent) / 2);
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" fill="${ink}"><g transform="translate(${inset} ${inset})">${content}</g></svg>`;
    mkdirSync(`src-tauri/icons/${variant}`, { recursive: true });
    const source = `assets/tray-${variant}-${size}.svg`;
    writeFileSync(source, svg);
    execFileSync(process.execPath, ['node_modules/@tauri-apps/cli/tauri.js', 'icon', source, '--output', `src-tauri/icons/${variant}`, '--png', String(size)], { stdio: 'inherit' });
    if (small) continue;
    const png = readFileSync(`src-tauri/icons/${variant}/${size}x${size}.png`);
    const entry = Buffer.alloc(16);
    entry[0] = size; entry[1] = size; entry.writeUInt16LE(1, 4); entry.writeUInt16LE(32, 6); entry.writeUInt32LE(png.length, 8);
    entries.push({ entry, png });
  }
  if (small) continue;
  const header = Buffer.alloc(6); header.writeUInt16LE(1, 2); header.writeUInt16LE(entries.length, 4);
  let offset = 6 + entries.length * 16;
  for (const { entry, png } of entries) { entry.writeUInt32LE(offset, 12); offset += png.length; }
  writeFileSync(`src-tauri/icons/${variant}.ico`, Buffer.concat([header, ...entries.map(e => e.entry), ...entries.map(e => e.png)]));
}
