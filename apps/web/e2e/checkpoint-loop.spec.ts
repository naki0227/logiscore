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

test("three differently damaged loops recover by symbol confidence", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");

  const result = await page.evaluate(async () => {
    const moduleUrl = "/src/lib/acoustic-codec.ts";
    const codec = await import(/* @vite-ignore */ moduleUrl);
    const expected = "Soft combine survives three damaged loops";
    const samples = codec.encodeTextV2CheckpointLoopPcm(expected, 512, 3);
    const sampleRate = 8_000;
    const frameLength = samples.length / 3;
    const framingSamples = sampleRate * 0.8;
    const symbolStride = sampleRate * 0.24;
    const toneSamples = sampleRate * 0.16;

    const frequency = (midi: number) => 440 * Math.pow(2, (midi - 69) / 12);
    const energyAt = (start: number, midi: number) => {
      const coefficient =
        2 * Math.cos((2 * Math.PI * frequency(midi)) / sampleRate);
      let previous = 0;
      let beforePrevious = 0;
      for (let index = 0; index < toneSamples; index += 1) {
        const current =
          samples[start + index] + coefficient * previous - beforePrevious;
        beforePrevious = previous;
        previous = current;
      }
      return (
        previous * previous +
        beforePrevious * beforePrevious -
        coefficient * previous * beforePrevious
      );
    };

    [19, 20, 21].forEach((byteIndex, loopIndex) => {
      const toneStart =
        loopIndex * frameLength + framingSamples + byteIndex * 2 * symbolStride;
      let originalNibble = 0;
      let bestEnergy = -1;
      for (let nibble = 0; nibble < 16; nibble += 1) {
        const energy = energyAt(toneStart, 60 + nibble);
        if (energy > bestEnergy) {
          bestEnergy = energy;
          originalNibble = nibble;
        }
      }
      const replacement = (originalNibble + 1) % 16;
      for (let index = 0; index < toneSamples; index += 1) {
        const time = index / sampleRate;
        samples[toneStart + index] =
          Math.sin(2 * Math.PI * frequency(60 + replacement) * time) * 0.6;
      }
    });

    let rejectedLoops = 0;
    for (let loopIndex = 0; loopIndex < 3; loopIndex += 1) {
      try {
        codec.decodeTextV2CheckpointLoopPcm(
          samples.slice(loopIndex * frameLength, (loopIndex + 1) * frameLength),
          sampleRate,
        );
      } catch {
        rejectedLoops += 1;
      }
    }
    return {
      decoded: codec.decodeTextV2CheckpointLoopPcm(samples, sampleRate),
      rejectedLoops,
    };
  });

  expect(result.rejectedLoops).toBe(3);
  expect(result.decoded).toBe("Soft combine survives three damaged loops");
});
