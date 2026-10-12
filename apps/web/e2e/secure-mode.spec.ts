import { expect, test, type Page } from "@playwright/test";
import { readFile } from "node:fs/promises";

const PASSWORD = "correct horse 電池 staple";

async function openSecureApp(page: Page, payload = "text") {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("準備完了");
  await page.getByLabel(/^送るもの/).selectOption(payload);
  await page.getByLabel(/^モード/).selectOption("secure");
  await page.getByLabel("パスワード", { exact: true }).fill(PASSWORD);
  await page.getByLabel("パスワード（確認）").fill(PASSWORD);
}

test.describe("Logiscore v2 Secure Mode scenarios", () => {
  test.setTimeout(120_000);

  test("password confirmation gates encoding without persisting the secret", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("準備完了");
    await page.getByLabel(/^モード/).selectOption("secure");
    await expect(page.locator(".codec-protocol")).toHaveText(
      "V2 / PCM + AEAD + FEC",
    );
    const encode = page.getByRole("button", { name: "変換する" });
    await expect(encode).toBeDisabled();
    await page.getByLabel("パスワード", { exact: true }).fill(PASSWORD);
    await page.getByLabel("パスワード（確認）").fill("wrong");
    await expect(page.locator(".secure-invalid")).toContainText(
      "パスワードが一致しません",
    );
    await expect(encode).toBeDisabled();
    await page.getByLabel("パスワード（確認）").fill(PASSWORD);
    await expect(encode).toBeEnabled();
    expect(await page.evaluate(() => localStorage.length)).toBe(0);
    expect(await page.evaluate(() => sessionStorage.length)).toBe(0);
  });

  test("Secure Text WAV restores with the right password and reveals nothing with a wrong one", async ({
    page,
  }) => {
    const secret = "Playwrightだけが復号する秘密 🔐";
    await openSecureApp(page);
    await page.getByLabel("送るテキスト").fill(secret);
    await page.getByRole("button", { name: "変換する" }).click();
    await expect(page.getByRole("status")).toContainText(
      "テキストを暗号化 WAVに変換しました",
    );

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "WAVをダウンロード" }).click();
    const download = await downloadPromise;
    const downloadedPath = await download.path();
    if (!downloadedPath) throw new Error("WAV download path is unavailable");
    const wav = await readFile(downloadedPath);

    await page.getByLabel("送るテキスト").fill("redacted before import");
    await page.getByLabel("パスワード", { exact: true }).fill("wrong password");
    await page.getByLabel("パスワード（確認）").fill("wrong password");
    await page.getByLabel("ファイルを読み込む").setInputFiles({
      name: "secure.wav",
      mimeType: "audio/wav",
      buffer: wav,
    });
    await expect(page.getByRole("status")).toHaveText(
      "Logiscoreの録音として認識できませんでした",
    );
    await expect(page.locator("body")).not.toContainText(secret);
    await expect(page.locator("body")).not.toContainText("Argon2");
    await expect(page.locator("body")).not.toContainText("ChaCha");

    await page.getByLabel("パスワード", { exact: true }).fill(PASSWORD);
    await page.getByLabel("パスワード（確認）").fill(PASSWORD);
    await page.getByLabel("ファイルを読み込む").setInputFiles({
      name: "secure-again.wav",
      mimeType: "audio/wav",
      buffer: wav,
    });
    await expect(page.getByRole("status")).toHaveText("テキストを復元しました");
    await expect(page.getByLabel("復元したデータ")).toHaveValue(secret);
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
