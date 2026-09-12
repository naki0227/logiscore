import { expect, test } from "@playwright/test";
import path from "node:path";

import {
  corpusRoot,
  fixtureFileName,
  fixtureSha256,
  loadRecordingCorpus,
  parseRecordingCorpus,
} from "./recording-corpus";
import { measureRecordingFixture } from "./measure-recording";

const corpus = loadRecordingCorpus();

test("smartphone corpus manifest has a verified 1m Text entry", () => {
  expect(corpus.entries).not.toHaveLength(0);
  expect(corpus.entries[0]).toMatchObject({
    status: "verified",
    speaker: "MacBook Speaker",
    distanceM: 1,
    recorderApp: "iPhone Voice Memos",
    environment: "quiet",
    transmissionFile: "capture-text-fixed-fallback.wav",
  });
});

test("smartphone corpus rejects a captured entry without physical metadata", () => {
  expect(() =>
    parseRecordingCorpus({
      schema_version: 1,
      entries: [
        {
          id: "invalid-capture",
          status: "captured",
          payload: {
            type: "text",
            text: "Hello from Logiscore.",
            sha256:
              "bd6d791c5fd7623349f54a5988953a5df6f3b457b1523d20d89a5413a2639a1c",
          },
          capture: {
            speaker: "MacBook Speaker",
            distance_m: 1,
            recorder_app: "iPhone Voice Memos",
            environment: "quiet",
            transmission_file: "capture-text-fixed-fallback.wav",
            device_model: null,
            recorded_at: null,
          },
          recording_file: null,
          recording_sha256: null,
          measurement: null,
        },
      ],
    }),
  ).toThrow("Captured corpus metadata is incomplete");
});

test("smartphone corpus rejects fixture paths outside the corpus", () => {
  expect(() => fixtureFileName("../private.m4a", "recording_file")).toThrow(
    "local M4A or WAV fixture filename",
  );
  expect(() => fixtureFileName("capture.mp3", "recording_file")).toThrow(
    "local M4A or WAV fixture filename",
  );
});

for (const entry of corpus.entries) {
  test(`transmission fixture restores ${entry.id} before physical capture`, async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    await page
      .getByLabel("Import file")
      .setInputFiles(path.join(corpusRoot, entry.transmissionFile));
    await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED", {
      timeout: 15_000,
    });
    await expect(page.getByLabel("Decoded source")).toHaveValue(entry.text);
    const metrics = await measureRecordingFixture(
      page,
      path.join(corpusRoot, entry.transmissionFile),
      entry.text,
    );
    expect(metrics.finalRecoveryRatePercent).toBe(100);
    expect(metrics.rawSymbolAccuracyPercent).toBe(100);
  });
}

for (const entry of corpus.entries.filter(
  ({ status }) => status !== "planned",
)) {
  const recordingFile = entry.recordingFile;
  if (!recordingFile) {
    throw new Error(`Captured recording file is missing for ${entry.id}`);
  }
  if (!entry.recordingSha256) {
    throw new Error(`Captured recording digest is missing for ${entry.id}`);
  }
  test(`smartphone recording measures ${entry.id} through Web Audio`, async ({
    page,
  }, testInfo) => {
    expect(fixtureSha256(recordingFile)).toBe(entry.recordingSha256);
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    await page
      .getByLabel("Import file")
      .setInputFiles(path.join(corpusRoot, recordingFile));
    await expect(page.getByRole("status")).not.toHaveText(
      "IMPORTING RECORDING...",
      { timeout: 15_000 },
    );
    const metrics = await measureRecordingFixture(
      page,
      path.join(corpusRoot, recordingFile),
      entry.text,
    );
    await testInfo.attach("recording-measurement.json", {
      body: Buffer.from(JSON.stringify(metrics, null, 2)),
      contentType: "application/json",
    });
    console.info(
      `recording measurement ${entry.id}: ${JSON.stringify(metrics)}`,
    );
    if (entry.status === "verified" && entry.measurement) {
      await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
      await expect(page.getByLabel("Decoded source")).toHaveValue(entry.text);
      expect(metrics.finalRecoveryRatePercent).toBe(
        entry.measurement.finalRecoveryRatePercent,
      );
      expect(metrics.rawSymbolAccuracyPercent).toBe(
        entry.measurement.rawSymbolAccuracyPercent,
      );
      expect(metrics.correctedErrors).toBe(entry.measurement.correctedErrors);
      expect(
        Math.abs(
          metrics.payloadBitrateBps - entry.measurement.payloadBitrateBps,
        ),
      ).toBeLessThan(0.001);
      // Web Audio may retain or discard a few AAC priming samples.
      expect(
        Math.abs(
          metrics.playbackDurationMs - entry.measurement.playbackDurationMs,
        ),
      ).toBeLessThan(1);
    }
    expect(metrics.decodeTimeMicros).toBeGreaterThan(0);
  });
}
