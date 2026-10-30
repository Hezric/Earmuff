import { version } from "../package.json";
import {
  createMotion,
  fitPanel,
  flyout,
  statusCard,
  pauseSettings,
  languageSettings,
} from "linen-ui";
import { sharedGlyph } from "linen-ui/glyphs";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { localeFor, mockState, mockUpdate, text, parsePauseMinutes, pauseLabel } from "./state.js";
import "./style.css";
const glyphs = import.meta.glob("../assets/fluent-v1/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});
const icon = (name, size = 20, filled = false) =>
  (
    glyphs[`../assets/fluent-v1/${name}_${size}_${filled ? "filled" : "regular"}.svg`] ||
    sharedGlyph(`${name}_${size}_${filled ? "filled" : "regular"}`)
  ).replace("<svg ", '<svg aria-hidden="true" focusable="false" ');
const native = isTauri();
const params = new URLSearchParams(location.search);
const systemLanguage = localeFor(
  native ? navigator.language : params.get("lang") || navigator.language,
);
let language = systemLanguage;
let state = native
  ? { ...mockState(), devices: [], current: null, currentVolume: null }
  : mockState();
let deviceSignature = "";
let localError = "";
let settingsOpen = false;
let pauseInvalid = false;
let motionReady = false;
document.body.classList.toggle("native", native);
if (!native && ["light", "dark"].includes(params.get("theme")))
  document.documentElement.dataset.theme = params.get("theme");
const root = document.querySelector("#app");
root.innerHTML = flyout({
  name: "Earmuff",
  icon: icon("headphones", 20),
  card: statusCard('<div class="volume"><span id="volume">—</span><small>%</small></div>'),
  main: `<section id="device-view" class="device-view"><div class="device-heading"><h1 id="devices-label"></h1><span id="protected-label"></span></div>
  <section class="device-list" aria-labelledby="devices-label"><div id="main-devices"></div><button id="fold" class="device-row fold" aria-expanded="false" aria-controls="folded-wrap"><span class="chevron">${icon("chevron_down")}</span><span id="other-label"></span></button><div id="folded-wrap" class="folded"><div id="folded-devices" class="folded-devices"></div></div><p id="empty" class="empty" hidden></p></section>
  </section>`,
  settings: `<h2 id="protection-label"></h2><div class="limits"><div class="slider-label"><label for="cap" id="cap-label"></label><output for="cap" id="cap-value"></output></div><input id="cap" type="range" min="10" max="100" step="1"><div class="slider-label drop-label"><label for="drop" id="drop-label"></label><output for="drop" id="drop-value"></output></div><input id="drop" type="range" min="0" max="100" step="1"></div>${pauseSettings()}${languageSettings("Earmuff", version, icon("open", 24))}`,
  gear: icon("settings", 24),
});
const cap = document.querySelector("#cap");
const drop = document.querySelector("#drop");
const { setContent, setText, animateView, reducedMotion, motion } = createMotion(
  root,
  () => motionReady,
);
function updateSlider(input) {
  input.style.setProperty(
    "--progress",
    `${(100 * (input.value - input.min)) / (Number(input.max) - Number(input.min) || 1)}%`,
  );
  setText(`${input.id}-value`, `${input.value}%`);
  input.setAttribute("aria-valuetext", `${input.value}%`);
}
const fit = native ? fitPanel(root, invoke) : async () => {};
function render() {
  const views = document.querySelector("#views");
  const before = views.getBoundingClientRect().height;
  const previousView = views.querySelector(":scope > section:not([hidden])");
  let outgoing =
    previousView && previousView.id !== (settingsOpen ? "settings" : "device-view")
      ? previousView.cloneNode(true)
      : null;
  if (!native) language = state.languageMode === "system" ? systemLanguage : state.languageMode;
  const t = text[language];
  document.documentElement.lang = language === "zh" ? "zh-Hans" : "en";
  root.querySelector(".flyout").setAttribute("aria-label", t.app);
  if (state.accent) {
    document.documentElement.style.setProperty("--accent", state.accent);
    document.documentElement.style.setProperty("--accent-ink", state.accentInk);
    document.documentElement.dataset.theme = state.dark ? "dark" : "light";
  }
  const current = state.devices.find((d) => d.id === state.current);
  const paused = state.pauseRemaining > 0;
  const inactive = paused || !current?.protected || !!state.error;
  root.querySelector(".header").classList.toggle("inactive", inactive);
  root.querySelector(".badge").classList.toggle("inactive", inactive);
  setContent(
    "current-icon",
    paused
      ? icon("pause", 24)
      : icon(
          current?.kind === "speaker" ? "speaker_2" : "headphones",
          current?.kind === "speaker" ? 20 : 24,
        ),
    true,
  );
  setText("current-name", current?.name || t.none);
  document.querySelector("#current-name").title = current?.name || t.none;
  const status = document.querySelector("#current-status");
  const statusText = state.error
    ? t.error
    : paused
      ? `${t.paused} · ${t.remaining} ${Math.ceil(state.pauseRemaining / 60)} ${t.minutes}`
      : current
        ? current.protected
          ? `${t.protected}${state.reduction !== null ? ` · ${t.reduced} ${state.reduction}%` : ` · ${t.limit} ${state.cap}%`}`
          : t.unprotected
        : t.noDevices;
  const shield =
    current?.protected && !inactive
      ? `<span class="status-shield">${icon("shield_checkmark", 16, true)}</span>`
      : "";
  setContent("current-status", shield + statusText, true);
  status.title = statusText;
  setText("volume", state.currentVolume ?? "—");
  setText("cap-label", t.cap);
  setText("drop-label", t.drop);
  if (document.activeElement !== cap) cap.value = state.cap;
  if (document.activeElement !== drop) drop.value = state.drop;
  drop.max = cap.value;
  updateSlider(cap);
  updateSlider(drop);
  setText("devices-label", t.devices);
  setText("protected-label", t.protected);
  setText("other-label", t.others);
  setText("empty", t.noDevices);
  const signature = JSON.stringify([
    state.devices.map(({ protected: _protected, ...device }) => device),
    state.current,
    language,
  ]);
  if (signature !== deviceSignature) {
    if (motionReady && !settingsOpen && !outgoing) outgoing = previousView.cloneNode(true);
    const focused = document.activeElement?.dataset.device;
    for (const group of ["main-devices", "folded-devices"])
      document.getElementById(group).replaceChildren();
    for (const device of state.devices) {
      const row = document.createElement("div");
      row.className = `device-row${device.folded ? " dim" : ""}`;
      const glyph = document.createElement("span");
      glyph.className = "device-icon";
      const kind =
        device.kind ||
        (device.folded
          ? device.id === "display"
            ? "display"
            : "software"
          : device.id === state.current
            ? "headphones"
            : "speaker");
      glyph.innerHTML =
        kind === "display"
          ? icon("desktop", 24)
          : kind === "software"
            ? icon("plug_connected", 24)
            : icon(kind === "headphones" ? "headphones" : "speaker_2");
      const label = document.createElement("span");
      label.className = "device-name";
      label.textContent = device.name;
      label.title = device.name;
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.className = "switch";
      toggle.role = "switch";
      toggle.dataset.device = device.id;
      toggle.setAttribute("aria-checked", String(device.protected));
      toggle.setAttribute("aria-label", `${t.protect} ${device.name}`);
      toggle.addEventListener("click", () =>
        change({
          type: "setProtected",
          id: device.id,
          protected: toggle.getAttribute("aria-checked") !== "true",
        }),
      );
      row.append(glyph, label);
      if (device.id === state.current) {
        const badge = document.createElement("span");
        badge.className = "tag";
        badge.textContent = t.current;
        row.append(badge);
      }
      row.append(toggle);
      document.getElementById(device.folded ? "folded-devices" : "main-devices").append(row);
    }
    if (focused)
      [...root.querySelectorAll(".switch")].find((b) => b.dataset.device === focused)?.focus();
    deviceSignature = signature;
  }
  for (const toggle of root.querySelectorAll(
    "#main-devices [data-device], #folded-devices [data-device]",
  ))
    toggle.setAttribute(
      "aria-checked",
      String(state.devices.find((device) => device.id === toggle.dataset.device).protected),
    );
  const hasFolded = state.devices.some((d) => d.folded);
  document.querySelector("#fold").hidden = !hasFolded;
  document.querySelector("#folded-wrap").hidden = !hasFolded;
  document.querySelector("#empty").hidden = state.devices.length > 0;
  setText("error", localError || (state.error ? `${t.error}. ${state.error}` : ""));
  const errorWrap = document.querySelector("#error-wrap");
  errorWrap.classList.toggle("open", !!(state.error || localError));
  errorWrap.setAttribute("aria-hidden", String(!(state.error || localError)));
  const pause = document.querySelector("#pause");
  setContent(
    "pause",
    `${paused ? icon("play", 24) : icon("pause", 24)}<span>${paused ? t.resume : pauseLabel(state.pauseMinutes, language)}</span>`,
    true,
  );
  pause.classList.toggle("primary", paused);
  pause.setAttribute("aria-pressed", String(paused));
  setText("protection-label", t.protection);
  setText("pause-for-label", t.pauseFor);
  setText("minutes-unit", t.minutes);
  setText("language-label", t.language);
  document.querySelector("#device-view").hidden = settingsOpen;
  const minutesInput = document.querySelector("#pause-minutes");
  if (!pauseInvalid && document.activeElement !== minutesInput)
    minutesInput.value = state.pauseMinutes;
  minutesInput.setAttribute("aria-invalid", String(pauseInvalid));
  if (pauseInvalid) minutesInput.setAttribute("aria-describedby", "pause-error");
  else minutesInput.removeAttribute("aria-describedby");
  setContent("pause-error", icon("error_circle", 16) + t.pauseError, true);
  const pauseErrorWrap = document.querySelector("#pause-error-wrap");
  pauseErrorWrap.classList.toggle("open", pauseInvalid);
  pauseErrorWrap.setAttribute("aria-hidden", String(!pauseInvalid));
  for (const preset of root.querySelectorAll("[data-minutes]"))
    preset.setAttribute(
      "aria-pressed",
      String(Number(preset.dataset.minutes) === state.pauseMinutes),
    );
  const gear = document.querySelector("#gear");
  gear.setAttribute("aria-label", t.settings);
  gear.title = t.settings;
  gear.setAttribute("aria-expanded", String(settingsOpen));
  document.querySelector("#settings").hidden = !settingsOpen;
  for (const button of root.querySelectorAll("[data-language]")) {
    if (button.dataset.language === "system") setContent(button, t.system);
    button.setAttribute("aria-pressed", String(button.dataset.language === state.languageMode));
  }
  document.querySelector("#github").setAttribute("aria-label", t.github);
  animateView(before, outgoing);
  requestAnimationFrame(fit);
}
let pending = Promise.resolve();
function change(action) {
  pending = pending.then(async () => {
    localError = "";
    try {
      state = native ? await invoke("update", { action }) : mockUpdate(state, action);
      if (native) language = state.language;
    } catch {
      localError = text[language].failed;
    }
    render();
  });
}
for (const input of [cap, drop]) {
  input.addEventListener("input", () => {
    if (input === cap) {
      drop.max = cap.value;
      updateSlider(drop);
    }
    updateSlider(input);
  });
  input.addEventListener("change", () =>
    change({ type: "setLevels", cap: Number(cap.value), drop: Number(drop.value) }),
  );
}
document.querySelector("#pause").addEventListener("click", () => change({ type: "togglePause" }));
document.querySelector("#gear").addEventListener("click", () => {
  settingsOpen = !settingsOpen;
  render();
});
document.querySelector("#folded-wrap").inert = true;
document.querySelector("#fold").addEventListener("click", (event) => {
  const button = event.currentTarget;
  const open = button.getAttribute("aria-expanded") !== "true";
  button.setAttribute("aria-expanded", String(open));
  const folded = document.querySelector("#folded-wrap");
  folded.classList.toggle("open", open);
  folded.inert = !open;
  requestAnimationFrame(fit);
});
const minutesInput = document.querySelector("#pause-minutes");
function savePause() {
  const minutes = parsePauseMinutes(minutesInput.value);
  pauseInvalid = minutes === null;
  if (!pauseInvalid && minutes !== state.pauseMinutes) change({ type: "setPauseMinutes", minutes });
  render();
}
minutesInput.addEventListener("blur", savePause);
minutesInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    savePause();
  }
});
for (const button of root.querySelectorAll("[data-minutes]"))
  button.addEventListener("click", () => {
    pauseInvalid = false;
    minutesInput.value = button.dataset.minutes;
    change({ type: "setPauseMinutes", minutes: Number(button.dataset.minutes) });
  });
for (const button of root.querySelectorAll("[data-language]"))
  button.addEventListener("click", () =>
    change({ type: "setLanguage", language: button.dataset.language }),
  );
document.querySelector("#github").addEventListener("click", (event) => {
  if (native) {
    event.preventDefault();
    invoke("open_project").catch(console.error);
  }
});
document.addEventListener("contextmenu", (event) => event.preventDefault());
document.addEventListener("dragstart", (event) => event.preventDefault());
document.addEventListener(
  "wheel",
  (event) => {
    if (event.ctrlKey) event.preventDefault();
  },
  { passive: false },
);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    if (native) invoke("dismiss_flyout").catch(() => {});
    else {
      const panel = root.querySelector(".flyout");
      if (reducedMotion.matches) root.hidden = true;
      else
        panel
          .animate(
            [
              { opacity: 1, transform: "none" },
              { opacity: 0, transform: "translateY(12px)" },
            ],
            motion,
          )
          .finished.then(() => {
            root.hidden = true;
          })
          .catch(() => {});
    }
  }
  if ((event.ctrlKey || event.metaKey) && ["+", "-", "=", "0"].includes(event.key))
    event.preventDefault();
});
if (native) {
  await listen("state-changed", (event) => {
    state = event.payload;
    language = state.language;
    render();
  });
  try {
    state = await invoke("snapshot");
    language = state.language;
  } catch {
    state.error = text[language].error;
  }
}
render();
await document.fonts.ready;
motionReady = true;
reducedMotion.addEventListener("change", () => {
  if (reducedMotion.matches) document.getAnimations().forEach((animation) => animation.finish());
  requestAnimationFrame(fit);
});
if (native) {
  await fit();
  await invoke("prepare_flyout", { height: root.querySelector(".flyout").scrollHeight });
  await new Promise(requestAnimationFrame);
  await new Promise(requestAnimationFrame);
  await invoke("present_flyout");
  new ResizeObserver(fit).observe(root.querySelector("#folded-wrap"));
  new ResizeObserver(fit).observe(root.querySelector(".flyout"));
} else if (!reducedMotion.matches)
  root.querySelector(".flyout").animate(
    [
      { opacity: 0, transform: "translateY(12px)" },
      { opacity: 1, transform: "none" },
    ],
    motion,
  );
// The backend owns expiration; this updates only the visible countdown.
setInterval(() => {
  if (state.pauseRemaining > 0) {
    state.pauseRemaining--;
    render();
  }
}, 1000);
