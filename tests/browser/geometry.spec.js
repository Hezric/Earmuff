import { test, expect } from '@playwright/test';
import { geometryChecks, materialChecks } from 'linen-ui/tooling/flyout-geometry.mjs';
import { mockState } from '../../src/state.js';

geometryChecks(test, expect, { ...mockState(), language: 'en' });

materialChecks(test, expect, { ...mockState(), language: 'en' });

test('high contrast keeps protection switch states visible', async ({ page }) => {
  await page.emulateMedia({ forcedColors: 'active' });
  await page.goto('/?theme=light');
  for (const checked of ['true', 'false']) {
    const toggle = page.locator(`.switch[aria-checked="${checked}"]`).first();
    await expect(toggle).toBeVisible();
    const colors = await toggle.evaluate(element => ({
      track: getComputedStyle(element).backgroundColor,
      thumb: getComputedStyle(element, '::after').backgroundColor,
    }));
    expect(colors.thumb).not.toBe(colors.track);
  }
  await page.screenshot({ path: test.info().outputPath('visible-switches.png') });
});
