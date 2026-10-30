export const states = ["protected", "unprotected", "paused"];
export function draw(size, state, ink) {
  const stroke = size <= 16 ? 1 : Math.round((size / 12) * 2) / 2;
  const cup = Math.round(size / 4),
    top = Math.round(size / 2),
    height = Math.round((size * 3) / 8);
  const inset = stroke / 2;
  const band = `<path d="M${cup / 2} ${top + 1}V${size * 0.42}a${size * 0.375} ${size * 0.375} 0 0 1 ${size * 0.75} 0V${top + 1}" fill="none" stroke="${ink}" stroke-width="${stroke}"/>`;
  const cups = [0, size - cup]
    .map((x) =>
      state === "unprotected"
        ? `<rect x="${x + inset}" y="${top + inset}" width="${cup - stroke}" height="${height - stroke}" rx="${stroke / 2}" fill="none" stroke="${ink}" stroke-width="${stroke}"/>`
        : `<rect x="${x}" y="${top}" width="${cup}" height="${height}" rx="${stroke}"/>`,
    )
    .join("");
  const pause =
    state === "paused"
      ? [0.4, 0.57]
          .map(
            (x) =>
              `<rect x="${Math.round(size * x)}" y="${top + 1}" width="${Math.max(1, Math.round(size / 12))}" height="${height - 1}"/>`,
          )
          .join("")
      : "";
  return band + cups + pause;
}
