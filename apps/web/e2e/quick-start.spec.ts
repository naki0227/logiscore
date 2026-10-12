import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";

test.describe("Quick start guide", () => {
  test("Given the demo, when encoded and re-imported, then every step completes", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    const steps = page.locator(".quickstart-step");

    await page
      .getByRole("button", { name: "Try the over-the-air demo" })
      .click();
    await expect(page.getByLabel(/^Payload/)).toHaveValue("text");
    await expect(page.getByLabel(/^Mode/)).toHaveValue("reliable");
    await expect(steps.nth(0)).toHaveClass(/done/);

    await page.getByRole("button", { name: "ENCODE" }).click();
    await expect(steps.nth(1)).toHaveClass(/done/);

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "DOWNLOAD WAV" }).click();
    const download = await downloadPromise;
    await expect(steps.nth(2)).toHaveClass(/done/);

    const downloadPath = await download.path();
    if (!downloadPath) throw new Error("Playwright did not provide a path");
    await page.getByLabel("Import file").setInputFiles({
      name: download.suggestedFilename(),
      mimeType: "audio/wav",
      buffer: await readFile(downloadPath),
    });
    await expect(page.getByLabel("Decoded source")).toHaveValue(
      /Hello from Logiscore!/,
    );
    await expect(steps.nth(3)).toHaveClass(/done/);
  });

  test("Given a dismissed guide, when reloaded, then it stays hidden", async ({
    page,
  }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "Hide guide" }).click();
    await page.reload();
    await expect(page.locator(".quickstart")).toHaveCount(0);
    await page.getByRole("button", { name: "How does it work?" }).click();
    await expect(page.locator(".quickstart")).toBeVisible();
  });
});

test.describe("Mobile layout", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test("Given a phone viewport, then controls are reachable without horizontal scroll", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("READY");
    for (const name of ["ENCODE", "PLAY", "STOP"]) {
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
