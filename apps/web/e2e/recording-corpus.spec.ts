import { expect, test } from "@playwright/test";
import path from "node:path";

import {
  corpusRoot,
  loadRecordingCorpus,
  parseRecordingCorpus,
} from "./recording-corpus";

const corpus = loadRecordingCorpus();

test("smartphone corpus manifest is valid and has a staged 1m Text capture", () => {
  expect(corpus.entries).not.toHaveLength(0);
  expect(corpus.entries[0]).toMatchObject({
    status: "planned",
    speaker: "MacBook Speaker",
    distanceM: 1,
    recorderApp: "iPhone Voice Memos",
    environment: "quiet",
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
            device_model: null,
            recorded_at: null,
          },
          recording_file: null,
          measurement: null,
        },
      ],
    }),
  ).toThrow("Captured corpus metadata is incomplete");
});

for (const entry of corpus.entries.filter(
  ({ status }) => status !== "planned",
)) {
  const recordingFile = entry.recordingFile;
  if (!recordingFile) {
    throw new Error(`Captured recording file is missing for ${entry.id}`);
  }
  test(`smartphone recording restores ${entry.id} through Web Audio`, async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    await page
      .getByLabel("Import file")
      .setInputFiles(path.join(corpusRoot, recordingFile));
    await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
    await expect(page.getByLabel("Decoded source")).toHaveValue(entry.text);
  });
}
