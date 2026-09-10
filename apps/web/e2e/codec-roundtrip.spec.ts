import { expect, test, type Download, type Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import path from "node:path";

async function openReadyApp(page: Page) {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");
}

async function downloadAndImport(
  page: Page,
  format: "MIDI" | "WAV" = "MIDI",
): Promise<Download> {
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: `DOWNLOAD ${format}` }).click();
  const download = await downloadPromise;
  const path = await download.path();
  if (!path) throw new Error("Playwright did not provide a download path");
  const midiInput = page.getByLabel(/Import MIDI/);
  const importInput = (await midiInput.count())
    ? midiInput
    : page.getByLabel("Import file");
  await importInput.setInputFiles({
    name: download.suggestedFilename(),
    mimeType: format === "WAV" ? "audio/wav" : "audio/midi",
    buffer: await readFile(path),
  });
  return download;
}

test.describe("Logiscore v2 codec scenarios", () => {
  test("Given Text, when MIDI is downloaded and imported, then UTF-8 is restored", async ({
    page,
  }) => {
    await test.step("Given the initialized Text payload editor", async () => {
      await openReadyApp(page);
      await page.getByLabel("Payload").selectOption("text");
      await expect(page.locator(".codec-protocol")).toHaveText("V2 / DENSE");
      await page.getByLabel("Text payload").fill("E2Eからこんにちは 🎼");
    });

    await test.step("When the user encodes, downloads, and imports the MIDI", async () => {
      await page.getByRole("button", { name: "ENCODE" }).click();
      await expect(page.getByRole("status")).toContainText("V2 TEXT ENCODED");
      await downloadAndImport(page);
    });

    await test.step("Then the original UTF-8 text is restored", async () => {
      await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
      await expect(page.getByLabel("Decoded source")).toHaveValue(
        "E2Eからこんにちは 🎼",
      );
    });
  });

  test("Given Music mode Text, when Musical MIDI is re-imported, then UTF-8 is restored", async ({
    page,
  }) => {
    const text = "Musical E2Eからこんにちは 🎹";

    await test.step("Given the Musical Text editor", async () => {
      await openReadyApp(page);
      await page.getByLabel("Payload").selectOption("text");
      await page.getByLabel("Mode").selectOption("music");
      await expect(page.locator(".codec-protocol")).toHaveText("V2 / RHYTHMIC");
      await page.getByLabel("Text payload").fill(text);
    });

    await test.step("When Musical MIDI is encoded, downloaded, and imported", async () => {
      await page.getByRole("button", { name: "ENCODE" }).click();
      await expect(page.getByRole("status")).toContainText(
        "V2 MUSICAL TEXT ENCODED",
      );
      await downloadAndImport(page);
    });

    await test.step("Then the decoder identifies the profile and restores the text", async () => {
      await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
      await expect(page.getByLabel("Decoded source")).toHaveValue(text);
    });
  });

  test("Given a Source File, when MIDI is re-imported, then source metadata and content survive", async ({
    page,
  }) => {
    const source = 'fn main() { println!("E2E 🎵"); }';

    await test.step("Given the initialized Source File editor", async () => {
      await openReadyApp(page);
      await expect(page.getByLabel("Payload")).toHaveValue("source-file");
      await expect(page.locator(".codec-protocol")).toHaveText("V2 / DENSE");
      await page.getByLabel("Source code").fill(source);
      await page.locator(".ext-select").selectOption(".rs");
    });

    await test.step("When the user encodes, downloads, and imports the MIDI", async () => {
      await page.getByRole("button", { name: "ENCODE" }).click();
      await expect(page.getByRole("status")).toContainText(
        "V2 SOURCE FILE ENCODED",
      );
      await downloadAndImport(page);
    });

    await test.step("Then the v2 Source File is identified and restored", async () => {
      await expect(page.getByRole("status")).toHaveText(
        "✅ V2 SOURCE FILE IMPORTED",
      );
      await expect(page.getByLabel("Decoded source")).toHaveValue(source);
      await expect(page.locator(".filename-badge")).toContainText(
        "logiscore_output",
      );
    });
  });

  test("Given Music mode Source File, when Musical MIDI is re-imported, then metadata survives", async ({
    page,
  }) => {
    const source = 'fn musical() { println!("四声"); }';
    await openReadyApp(page);
    await page.getByLabel("Mode").selectOption("music");
    await expect(page.locator(".codec-protocol")).toHaveText("V2 / RHYTHMIC");
    await page.getByLabel("Source code").fill(source);
    await page.locator(".ext-select").selectOption(".rs");

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText(
      "V2 MUSICAL SOURCE FILE ENCODED",
    );
    await downloadAndImport(page);

    await expect(page.getByRole("status")).toHaveText(
      "✅ V2 SOURCE FILE IMPORTED",
    );
    await expect(page.getByLabel("Decoded source")).toHaveValue(source);
  });

  test("Given a Project, when MIDI is re-imported, then every file and path survives", async ({
    page,
  }) => {
    await test.step("Given two supported project files", async () => {
      await openReadyApp(page);
      await page.getByLabel("Payload").selectOption("project");
      await expect(page.locator(".codec-protocol")).toHaveText("V2 / DENSE");
      await page
        .getByLabel("Import Project folder")
        .setInputFiles(path.join(import.meta.dirname, "fixtures/project"));
      await expect(page.getByRole("status")).toHaveText(
        "PROJECT LOADED: 2 FILES",
      );
      await expect(
        page.locator(".file-item").filter({ hasText: "main.rs" }),
      ).toContainText("src/main.rs");
      await expect(
        page.locator(".file-item").filter({ hasText: "README.md" }),
      ).toBeVisible();
    });

    await test.step("When the Project is encoded, downloaded, and imported", async () => {
      await page.getByRole("button", { name: "ENCODE" }).click();
      await expect(page.getByRole("status")).toContainText("SYMPHONY VERIFIED");
      await downloadAndImport(page);
    });

    await test.step("Then the v2 archive restores both source files", async () => {
      await expect(page.getByRole("status")).toHaveText(
        "✅ V2 PROJECT IMPORTED: 2 FILES",
      );
      await expect(
        page.locator(".file-item").filter({ hasText: "main.rs" }),
      ).toContainText("src/main.rs");
      await expect(
        page.locator(".file-item").filter({ hasText: "README.md" }),
      ).toBeVisible();
    });
  });

  test("Given Music mode Project, when Musical MIDI is re-imported, then every file survives", async ({
    page,
  }) => {
    await openReadyApp(page);
    await page.getByLabel("Payload").selectOption("project");
    await page.getByLabel("Mode").selectOption("music");
    await expect(page.locator(".codec-protocol")).toHaveText("V2 / RHYTHMIC");
    await page
      .getByLabel("Import Project folder")
      .setInputFiles(path.join(import.meta.dirname, "fixtures/project"));
    await expect(page.getByRole("status")).toHaveText(
      "PROJECT LOADED: 2 FILES",
    );

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText("SYMPHONY VERIFIED");
    await downloadAndImport(page);

    await expect(page.getByRole("status")).toHaveText(
      "✅ V2 PROJECT IMPORTED: 2 FILES",
      { timeout: 15_000 },
    );
    await expect(page.locator(".file-item")).toHaveCount(2);
  });

  test("Music and Reliable modes are available and identify their profiles", async ({
    page,
  }) => {
    await openReadyApp(page);
    const mode = page.getByLabel("Mode");
    await expect(mode.locator('option[value="music"]')).toBeEnabled();
    await expect(mode.locator('option[value="reliable"]')).toBeEnabled();
    await mode.selectOption("reliable");
    await expect(page.locator(".codec-protocol")).toHaveText("V2 / PCM + FEC");
    await page.getByLabel("Payload").selectOption("project");
    await expect(page.locator(".codec-protocol")).toHaveText("V2 / PCM + FEC");
    await expect(
      page.getByText("Drop a folder here to start Symphony"),
    ).toBeVisible();
  });

  test("Reliable Environment and Priority select an adaptive profile", async ({
    page,
  }) => {
    await openReadyApp(page);
    await page.getByLabel("Mode").selectOption("reliable");

    await expect(page.getByLabel("Environment")).toHaveValue("auto");
    const profileValue = page
      .locator(".profile-summary > div", { hasText: "Profile" })
      .locator("strong");
    const confidenceValue = page
      .locator(".profile-summary > div", { hasText: "Confidence" })
      .locator("strong");
    await expect(profileValue).toHaveText("Balanced");
    await expect(confidenceValue).toHaveText("50%");
    await expect(
      page.locator(".profile-summary div", { hasText: "Estimated Duration" }),
    ).toContainText(/sec|min/);

    await page.getByLabel("Environment").selectOption("quiet");
    await page
      .getByRole("slider", { name: "Reliability priority", exact: true })
      .fill("50");
    await expect(profileValue).toHaveText("Quiet");
    await expect(confidenceValue).toHaveText("100%");

    await page.getByLabel("Environment").selectOption("long-distance");
    await expect(profileValue).toHaveText("Long Distance");
  });

  test("Adaptive WAV roundtrips every selectable environment in browser WASM", async ({
    page,
  }) => {
    await openReadyApp(page);
    const result = await page.evaluate(async () => {
      const moduleUrl = `${location.origin}/src/lib/acoustic-codec.ts`;
      const codec = (await import(
        /* @vite-ignore */ moduleUrl
      )) as typeof import("../src/lib/acoustic-codec");
      const environments = [
        "auto",
        "quiet",
        "conversation",
        "noisy",
        "online",
        "long-distance",
      ] as const;
      return environments.map((environment) => {
        const text = `adaptive-${environment}`;
        const settings = { environment, reliabilityPriority: 50 };
        const profile = codec.describeAcousticProfile(settings, text.length);
        const wav = codec.encodeTextV2WavReliable(text, settings);
        return {
          environment,
          profile: profile.name,
          decoded: codec.decodeTextV2WavReliable(wav),
        };
      });
    });

    expect(result.map((entry) => entry.decoded)).toEqual([
      "adaptive-auto",
      "adaptive-quiet",
      "adaptive-conversation",
      "adaptive-noisy",
      "adaptive-online",
      "adaptive-long-distance",
    ]);
    expect(result.map((entry) => entry.profile)).toEqual([
      "Balanced",
      "Quiet",
      "Conversation",
      "Noisy",
      "Online",
      "Long Distance",
    ]);
  });

  test("PCM WASM APIs roundtrip every v2 payload in the browser", async ({
    page,
  }) => {
    await openReadyApp(page);

    const result = await page.evaluate(async () => {
      const moduleUrl = `${location.origin}/src/lib/wasm-loader.ts`;
      const wasm = (await import(
        /* @vite-ignore */ moduleUrl
      )) as typeof import("../src/lib/wasm-loader");
      await wasm.initWasm();

      const textSamples = wasm.encodeTextV2Pcm("PCM E2E 音声 🎧");
      const sourceSamples = wasm.encodeSourceFileV2Pcm(
        "main.rs",
        ".rs",
        "fn main() {}",
      );
      const projectSamples = wasm.encodeProjectV2Pcm([
        { name: "README.md", extension: ".md", source: "# PCM" },
      ]);

      return {
        sampleRate: wasm.getPcmSampleRate(),
        sampleCount: textSamples.length,
        text: wasm.decodeTextV2Pcm(textSamples),
        source: wasm.decodeSourceFileV2Pcm(sourceSamples),
        project: wasm.decodeProjectV2Pcm(projectSamples),
      };
    });

    expect(result.sampleRate).toBe(8_000);
    expect(result.sampleCount).toBeGreaterThan(result.sampleRate);
    expect(result.text).toBe("PCM E2E 音声 🎧");
    expect(result.source).toEqual({
      filename: "main.rs",
      extension: ".rs",
      source: "fn main() {}",
    });
    expect(result.project).toEqual([
      { name: "README.md", extension: ".md", source: "# PCM" },
    ]);
  });

  test("Reliable PCM applies FEC profile 1 to every payload in the browser", async ({
    page,
  }) => {
    await openReadyApp(page);

    const result = await page.evaluate(async () => {
      const moduleUrl = `${location.origin}/src/lib/wasm-loader.ts`;
      const wasm = (await import(
        /* @vite-ignore */ moduleUrl
      )) as typeof import("../src/lib/wasm-loader");
      await wasm.initWasm();

      const textSamples = wasm.encodeTextV2PcmReliable("FEC 🎼");
      const sourceSamples = wasm.encodeSourceFileV2PcmReliable(
        "main.rs",
        ".rs",
        "fn main() {}",
      );
      const projectSamples = wasm.encodeProjectV2PcmReliable([
        { name: "README.md", extension: ".md", source: "# FEC" },
      ]);

      return {
        text: wasm.decodeTextV2PcmReliable(textSamples),
        source: wasm.decodeSourceFileV2PcmReliable(sourceSamples),
        project: wasm.decodeProjectV2PcmReliable(projectSamples),
      };
    });

    expect(result.text).toBe("FEC 🎼");
    expect(result.source).toEqual({
      filename: "main.rs",
      extension: ".rs",
      source: "fn main() {}",
    });
    expect(result.project).toEqual([
      { name: "README.md", extension: ".md", source: "# FEC" },
    ]);
  });

  test("Reliable Text downloads a WAV and restores it through file import", async ({
    page,
  }) => {
    const text = "録音ファイルから復元 🎙️";
    await openReadyApp(page);
    await page.getByLabel("Payload").selectOption("text");
    await page.getByLabel("Mode").selectOption("reliable");
    await page.getByLabel("Text payload").fill(text);

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText(
      "V2 RELIABLE TEXT VERIFIED",
    );
    const download = await downloadAndImport(page, "WAV");

    expect(download.suggestedFilename()).toBe("logiscore_output.wav");
    await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
    await expect(page.getByLabel("Decoded source")).toHaveValue(text);
  });

  test("Reliable Source File preserves WAV metadata through file import", async ({
    page,
  }) => {
    const source = "fn acoustic() {}";
    await openReadyApp(page);
    await page.getByLabel("Mode").selectOption("reliable");
    await page.getByLabel("Source code").fill(source);
    await page.locator(".ext-select").selectOption(".rs");

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText(
      "V2 RELIABLE SOURCE FILE VERIFIED",
    );
    await downloadAndImport(page, "WAV");

    await expect(page.getByRole("status")).toHaveText(
      "✅ V2 SOURCE FILE IMPORTED",
    );
    await expect(page.getByLabel("Decoded source")).toHaveValue(source);
    await expect(page.locator(".filename-badge")).toContainText(
      "logiscore_output",
    );
  });

  test("Reliable Project restores every file from its WAV", async ({
    page,
  }) => {
    await openReadyApp(page);
    await page.getByLabel("Payload").selectOption("project");
    await page.getByLabel("Mode").selectOption("reliable");
    await page
      .getByLabel("Import Project folder")
      .setInputFiles(path.join(import.meta.dirname, "fixtures/project"));
    await expect(page.getByRole("status")).toHaveText(
      "PROJECT LOADED: 2 FILES",
    );

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toHaveText(
      "✅ RELIABLE SYMPHONY VERIFIED: 100% BIT-PERFECT",
    );
    await downloadAndImport(page, "WAV");

    await expect(page.getByRole("status")).toHaveText(
      "✅ V2 PROJECT IMPORTED: 2 FILES",
    );
    await expect(page.locator(".file-item")).toHaveCount(2);
  });

  test("Browser-decoded recording PCM restores Reliable Text", async ({
    page,
  }) => {
    const text = "Web Audio recording path";
    await openReadyApp(page);
    await page.getByLabel("Payload").selectOption("text");
    await page.getByLabel("Mode").selectOption("reliable");
    await page.getByLabel("Text payload").fill(text);
    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText("VERIFIED");

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "DOWNLOAD WAV" }).click();
    const download = await downloadPromise;
    const downloadedPath = await download.path();
    if (!downloadedPath) throw new Error("WAV download path is unavailable");
    await page.getByLabel("Import file").setInputFiles({
      name: "voice-memo.m4a",
      mimeType: "audio/mp4",
      buffer: await readFile(downloadedPath),
    });

    await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
    await expect(page.getByLabel("Decoded source")).toHaveValue(text);
  });

  test("Damaged recordings are rejected without exposing decoder details", async ({
    page,
  }) => {
    await openReadyApp(page);
    await page.getByLabel("Mode").selectOption("reliable");
    await page.getByLabel("Import file").setInputFiles({
      name: "damaged.wav",
      mimeType: "audio/wav",
      buffer: Buffer.from("not a wav"),
    });
    await expect(page.getByRole("status")).toHaveText(
      "⚠️ Unsupported or damaged Logiscore recording",
    );
  });
});
