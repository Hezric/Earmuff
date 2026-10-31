import { readFileSync } from "node:fs";
import { strict as assert } from "node:assert";
import { test } from "node:test";
import { localeFor, mockState, mockUpdate } from "./state.js";
test("language follows display language with English fallback", () => {
  for (const [language, expected] of [
    ["zh-CN", "zh"],
    ["zh-Hans", "zh"],
    ["en-GB", "en"],
    ["ja-JP", "en"],
    ["zh-TW", "zh"],
  ])
    assert.equal(localeFor(language), expected);
});
test("browser mock protects new devices, folds others, and updates each control", () => {
  const state = mockState();
  assert.equal(state.devices.filter((d) => d.folded).length, 4);
  assert.equal(state.devices.filter((d) => d.protected).length, 5);
  assert.equal(
    mockUpdate(state, { type: "setProtected", id: "usb", protected: true }).devices.find(
      (d) => d.id === "usb",
    ).protected,
    true,
  );
  assert.equal(mockUpdate(state, { type: "setLevels", cap: 36, drop: 8 }).cap, 36);
  assert.equal(mockUpdate(state, { type: "togglePause" }).pauseRemaining, 3600);
  assert.equal(mockUpdate(state, { type: "togglePause" }).pauseRemaining, 0);
});
test("app opens without a resident webview or startup registration", () => {
  const config = JSON.parse(
    readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  );
  assert.deepEqual(config.app.windows, []);
  assert.equal(config.version, "0.1.0");
});
test("folded devices keep their individual protection control", () => {
  const state = mockState();
  const other = state.devices.filter((d) => d.id !== "virtual").map((d) => d.protected);
  for (const protectedValue of [false, true]) {
    mockUpdate(state, { type: "setProtected", id: "virtual", protected: protectedValue });
    assert.equal(state.devices.find((d) => d.id === "virtual").protected, protectedValue);
    assert.equal(state.devices.find((d) => d.id === "virtual").folded, true);
    assert.deepEqual(
      state.devices.filter((d) => d.id !== "virtual").map((d) => d.protected),
      other,
    );
  }
});

import { parsePauseMinutes, pauseLabel } from "./state.js";
test("custom pause accepts only 1–720 whole minutes and formats the saved value", () => {
  for (const [input, expected] of [
    ["1", 1],
    ["720", 720],
    ["45", 45],
    [" 60 ", 60],
    ["0", null],
    ["721", null],
    ["1.5", null],
    ["1e2", null],
    ["", null],
    ["abc", null],
  ])
    assert.equal(parsePauseMinutes(input), expected, input);
  for (const [minutes, en, zh] of [
    [60, "Pause 1 hr", "暂停 1 小时"],
    [120, "Pause 2 hr", "暂停 2 小时"],
    [45, "Pause 45 min", "暂停 45 分钟"],
  ]) {
    assert.equal(pauseLabel(minutes, "en"), en);
    assert.equal(pauseLabel(minutes, "zh"), zh);
  }
  const state = mockState();
  mockUpdate(state, { type: "setPauseMinutes", minutes: 30 });
  mockUpdate(state, { type: "togglePause" });
  assert.equal(state.pauseRemaining, 1800);
});
