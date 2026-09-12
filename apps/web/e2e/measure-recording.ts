import type { Page } from "@playwright/test";

export interface BrowserRecordingMeasurement {
  profileId: number | null;
  finalRecoveryRatePercent: number;
  rawSymbolAccuracyPercent: number;
  correctedErrors: number;
  payloadBitrateBps: number;
  decodeTimeMicros: number;
  playbackDurationMs: number;
}

export async function measureRecordingFixture(
  page: Page,
  fixturePath: string,
  expectedText: string,
): Promise<BrowserRecordingMeasurement> {
  await page.evaluate(() => {
    document.querySelector("#recording-metrics-fixture")?.remove();
    const input = document.createElement("input");
    input.type = "file";
    input.id = "recording-metrics-fixture";
    input.hidden = true;
    document.body.append(input);
  });
  const metricsInput = page.locator("#recording-metrics-fixture");
  await metricsInput.setInputFiles(fixturePath);
  const result: unknown = await metricsInput.evaluate(async (element, text) => {
    if (!(element instanceof HTMLInputElement) || !element.files?.[0]) {
      throw new Error("Recording fixture was not attached");
    }
    const moduleUrl = "/src/features/audio/recordingMetrics.ts";
    const { measureTextRecordingFile } = await import(
      /* @vite-ignore */ moduleUrl
    );
    return measureTextRecordingFile(element.files[0], text);
  }, expectedText);
  return parseMeasurement(result);
}

function parseMeasurement(input: unknown): BrowserRecordingMeasurement {
  if (typeof input !== "object" || input === null || Array.isArray(input)) {
    throw new Error("Browser recording measurement is invalid");
  }
  const value = input as Record<string, unknown>;
  for (const field of [
    "finalRecoveryRatePercent",
    "rawSymbolAccuracyPercent",
    "correctedErrors",
    "payloadBitrateBps",
    "decodeTimeMicros",
    "playbackDurationMs",
  ]) {
    if (typeof value[field] !== "number" || !Number.isFinite(value[field])) {
      throw new Error(`Browser recording measurement ${field} is invalid`);
    }
  }
  if (
    value.profileId !== null &&
    (typeof value.profileId !== "number" || !Number.isInteger(value.profileId))
  ) {
    throw new Error("Browser recording measurement profileId is invalid");
  }
  return value as unknown as BrowserRecordingMeasurement;
}
