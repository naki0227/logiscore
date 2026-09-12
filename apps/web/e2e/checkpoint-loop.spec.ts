import { expect, test } from "@playwright/test";

test("two-loop Text recording recovers from an arbitrary chunk position", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");

  const result = await page.evaluate(async () => {
    const moduleUrl = "/src/lib/acoustic-codec.ts";
    const codec = await import(/* @vite-ignore */ moduleUrl);
    const expected = "Loop recording from any position 🎼";
    const samples = codec.encodeTextV2CheckpointLoopPcm(expected, 64, 2);
    const offset = Math.floor(samples.length * 0.3);
    const rotated = new Float32Array(samples.length);
    rotated.set(samples.subarray(offset));
    rotated.set(samples.subarray(0, offset), samples.length - offset);
    return {
      decoded: codec.decodeTextV2CheckpointLoopPcm(rotated, 8_000),
      sampleCount: samples.length,
    };
  });

  expect(result.decoded).toBe("Loop recording from any position 🎼");
  expect(result.sampleCount).toBeGreaterThan(0);
});

test("incomplete checkpoint recording is rejected", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");

  const outcome = await page.evaluate(async () => {
    const moduleUrl = "/src/lib/acoustic-codec.ts";
    const codec = await import(/* @vite-ignore */ moduleUrl);
    const samples = codec.encodeTextV2CheckpointLoopPcm("incomplete", 64, 1);
    try {
      codec.decodeTextV2CheckpointLoopPcm(
        samples.slice(0, Math.floor(samples.length / 2)),
        8_000,
      );
      return "unexpected success";
    } catch {
      return "rejected";
    }
  });

  expect(outcome).toBe("rejected");
});
