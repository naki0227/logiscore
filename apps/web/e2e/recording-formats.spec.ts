import { expect, test } from "@playwright/test";
import path from "node:path";

test("Opus recording restores the original v2 Text payload through Web Audio", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("READY");
  await page
    .getByLabel("Import file")
    .setInputFiles(
      path.join(import.meta.dirname, "fixtures/logiscore-opus.ogg"),
    );
  await expect(page.getByRole("status")).toHaveText("✅ V2 TEXT IMPORTED");
  await expect(page.getByLabel("Decoded source")).toHaveValue("Opus");
});
