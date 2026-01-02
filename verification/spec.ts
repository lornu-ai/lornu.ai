import { test, expect } from '@playwright/test';

test('HeroSection component screenshot', async ({ page }) => {
  await page.goto('/');
  const heroSection = page.getByTestId('hero-section');
  await expect(heroSection).toBeVisible();
  await heroSection.screenshot({ path: 'HeroSection.png' });
});
