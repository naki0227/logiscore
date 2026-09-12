import { expect, test } from "@playwright/test";
import path from "node:path";
import { measureRecordingFixture } from "./measure-recording";

test("Opus recording restores the original v2 Text payload through Web Audio", async ({
  page,
}) => {
  const fixturePath = path.join(
    import.meta.dirname,
    "fixtures/logiscore-opus.ogg",
  );
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");
  await page.getByLabel("Import file").setInputFiles(fixturePath);
  await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
  await expect(page.getByLabel("Decoded source")).toHaveValue("Opus");

  const metrics = await measureRecordingFixture(page, fixturePath, "Opus");
  expect(metrics.profileId).toBe(7);
  expect(metrics.finalRecoveryRatePercent).toBe(100);
  expect(metrics.rawSymbolAccuracyPercent).toBe(100);
  expect(metrics.correctedErrors).toBe(0);
  expect(metrics.payloadBitrateBps).toBeGreaterThan(0);
  expect(metrics.decodeTimeMicros).toBeGreaterThan(0);
  expect(metrics.playbackDurationMs).toBeGreaterThan(69_000);
});
