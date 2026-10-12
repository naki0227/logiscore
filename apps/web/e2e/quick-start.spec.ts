import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";

test.describe("Quick start guide", () => {
  test("Given the demo, when encoded and re-imported, then every step completes", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("準備完了");
    const steps = page.locator(".quickstart-step");

    await page.getByRole("button", { name: "空気伝送のデモを試す" }).click();
    await expect(page.getByLabel(/^送るもの/)).toHaveValue("text");
    await expect(page.getByLabel(/^モード/)).toHaveValue("reliable");
    await expect(steps.nth(0)).toHaveClass(/done/);

    await page.getByRole("button", { name: "変換する" }).click();
    await expect(steps.nth(1)).toHaveClass(/done/);

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "WAVをダウンロード" }).click();
    const download = await downloadPromise;
    await expect(steps.nth(2)).toHaveClass(/done/);

    const downloadPath = await download.path();
    if (!downloadPath) throw new Error("Playwright did not provide a path");
    await page.getByLabel("ファイルを読み込む").setInputFiles({
      name: download.suggestedFilename(),
      mimeType: "audio/wav",
      buffer: await readFile(downloadPath),
    });
    await expect(page.getByLabel("復元したデータ")).toHaveValue(
      /Logiscoreからこんにちは！/,
      { timeout: 15_000 },
    );
    await expect(steps.nth(3)).toHaveClass(/done/);
  });

  test("Given a dismissed guide, when reloaded, then it stays hidden", async ({
    page,
  }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "ガイドを閉じる" }).click();
    await page.reload();
    await expect(page.locator(".quickstart")).toHaveCount(0);
    await page.getByRole("button", { name: "使い方を見る" }).click();
    await expect(page.locator(".quickstart")).toBeVisible();
  });
});

test.describe("Output freshness", () => {
  test("Given an encoded result, when the payload or input changes, then the old output is cleared", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("準備完了");
    const download = page.getByRole("button", { name: "MIDIをダウンロード" });

    await page.getByRole("button", { name: "変換する" }).click();
    await expect(download).toBeVisible();
    await page.getByLabel("ソースコード").fill("fn main() {}");
    await expect(download).toHaveCount(0);

    await page.getByRole("button", { name: "変換する" }).click();
    await expect(download).toBeVisible();
    await page.getByLabel(/^送るもの/).selectOption("text");
    await expect(download).toHaveCount(0);
  });
});

test.describe("Mobile layout", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test("Given a phone viewport, then controls are reachable without horizontal scroll", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("準備完了");
    for (const name of ["変換する", "再生", "停止"]) {
      const button = page.getByRole("button", { name });
      await button.scrollIntoViewIfNeeded();
      await expect(button).toBeInViewport();
    }
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });
});
