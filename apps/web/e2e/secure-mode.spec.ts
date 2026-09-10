import { expect, test, type Page } from "@playwright/test";
import { readFile } from "node:fs/promises";

const PASSWORD = "correct horse 電池 staple";

async function openSecureApp(page: Page, payload = "text") {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");
  await page.getByLabel("Payload").selectOption(payload);
  await page.getByLabel("Mode").selectOption("secure");
  await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
  await page.getByLabel("Confirm Password").fill(PASSWORD);
}

test.describe("Logiscore v2 Secure Mode scenarios", () => {
  test.setTimeout(120_000);

  test("password confirmation gates encoding without persisting the secret", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    await page.getByLabel("Mode").selectOption("secure");
    await expect(page.locator(".codec-protocol")).toHaveText(
      "V2 / PCM + AEAD + FEC",
    );
    const encode = page.getByRole("button", { name: "ENCODE" });
    await expect(encode).toBeDisabled();
    await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
    await page.getByLabel("Confirm Password").fill("wrong");
    await expect(page.locator(".secure-invalid")).toContainText(
      "Passwords do not match",
    );
    await expect(encode).toBeDisabled();
    await page.getByLabel("Confirm Password").fill(PASSWORD);
    await expect(encode).toBeEnabled();
    expect(await page.evaluate(() => localStorage.length)).toBe(0);
    expect(await page.evaluate(() => sessionStorage.length)).toBe(0);
  });

  test("Secure Text WAV restores with the right password and reveals nothing with a wrong one", async ({
    page,
  }) => {
    const secret = "Playwrightだけが復号する秘密 🔐";
    await openSecureApp(page);
    await page.getByLabel("Text payload").fill(secret);
    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(page.getByRole("status")).toContainText(
      "V2 SECURE TEXT VERIFIED",
    );

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "DOWNLOAD WAV" }).click();
    const download = await downloadPromise;
    const downloadedPath = await download.path();
    if (!downloadedPath) throw new Error("WAV download path is unavailable");
    const wav = await readFile(downloadedPath);

    await page.getByLabel("Text payload").fill("redacted before import");
    await page.getByLabel("Password", { exact: true }).fill("wrong password");
    await page.getByLabel("Confirm Password").fill("wrong password");
    await page.getByLabel("Import file").setInputFiles({
      name: "secure.wav",
      mimeType: "audio/wav",
      buffer: wav,
    });
    await expect(page.getByRole("status")).toHaveText(
      "⚠️ Unsupported or damaged Logiscore recording",
    );
    await expect(page.locator("body")).not.toContainText(secret);
    await expect(page.locator("body")).not.toContainText("Argon2");
    await expect(page.locator("body")).not.toContainText("ChaCha");

    await page.getByLabel("Password", { exact: true }).fill(PASSWORD);
    await page.getByLabel("Confirm Password").fill(PASSWORD);
    await page.getByLabel("Import file").setInputFiles({
      name: "secure-again.wav",
      mimeType: "audio/wav",
      buffer: wav,
    });
    await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
    await expect(page.getByLabel("Decoded source")).toHaveValue(secret);
  });

  test("browser WASM randomizes ciphertext and roundtrips every Secure payload", async ({
    page,
  }) => {
    await openSecureApp(page);
    const result = await page.evaluate(
      async ({ password }) => {
        const moduleUrl = `${location.origin}/src/lib/secure-acoustic-codec.ts`;
        const codec = (await import(
          /* @vite-ignore */ moduleUrl
        )) as typeof import("../src/lib/secure-acoustic-codec");
        const settings = {
          environment: "quiet" as const,
          reliabilityPriority: 50,
        };
        const textA = Uint8Array.from(
          codec.encodeTextV2WavSecure("same text", password, settings),
        );
        const textB = Uint8Array.from(
          codec.encodeTextV2WavSecure("same text", password, settings),
        );
        const source = codec.encodeSourceFileV2WavSecure(
          "main",
          ".rs",
          "fn secure() {}",
          password,
          settings,
        );
        const project = codec.encodeProjectV2WavSecure(
          [{ name: "README.md", extension: ".md", source: "# Secret" }],
          password,
          settings,
        );
        const firstDifference = textA.findIndex(
          (value, index) => value !== textB[index],
        );
        return {
          randomized: firstDifference >= 0,
          textLengths: [textA.length, textB.length],
          firstDifference,
          text: codec.decodeV2WavSecure(textA, password),
          source: codec.decodeV2WavSecure(source, password),
          project: codec.decodeV2WavSecure(project, password),
        };
      },
      { password: PASSWORD },
    );

    expect(
      result.randomized,
      `lengths=${result.textLengths.join(",")} firstDifference=${result.firstDifference}`,
    ).toBe(true);
    expect(result.text).toEqual({ type: "text", text: "same text" });
    expect(result.source).toMatchObject({
      type: "source-file",
      filename: "main",
      source: "fn secure() {}",
    });
    expect(result.project).toMatchObject({
      type: "project",
      files: [{ name: "README.md", source: "# Secret" }],
    });
  });
});
