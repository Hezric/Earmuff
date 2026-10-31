export const states = ["protected", "unprotected", "paused"];
export function draw(size, state, ink) {
  const band = '<path d="M4 15.5V12a8 8 0 0 1 16 0v3.5" fill="none" stroke-width="2.5"/>';
  const left = '<rect x="2.6" y="13.6" width="5.4" height="8" rx="2" stroke="none"/>';
  const right = '<rect x="16" y="13.6" width="5.4" height="8" rx="2" stroke="none"/>';
  const cutout = state === "unprotected" ? "" : '<circle cx="18.4" cy="18.4" r="6.2" fill="black"/>';
  const symbol = state === "protected"
    ? '<path d="M16.2 18.5l1.5 1.5 2.9-3" fill="none" stroke="black" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/>'
    : '<rect x="16.6" y="16.3" width="1.3" height="4.2" rx=".5" fill="black"/><rect x="18.9" y="16.3" width="1.3" height="4.2" rx=".5" fill="black"/>';
  const badge = state === "unprotected" ? "" : `<defs><mask id="symbol"><rect width="24" height="24" fill="white"/>${symbol}</mask></defs><circle cx="18.4" cy="18.4" r="4.8" stroke="none" mask="url(#symbol)"/>`;
  return `<g transform="scale(${size / 24})" fill="${ink}" stroke="${ink}" stroke-linecap="round" stroke-linejoin="round"><defs><mask id="headphones"><rect width="24" height="24" fill="white"/>${cutout}</mask></defs><g mask="url(#headphones)">${band}${left}${state === "unprotected" ? right : ""}</g>${badge}</g>`;
}
