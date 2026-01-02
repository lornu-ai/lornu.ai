
import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto('http://localhost:5175');
  const heroSection = page.getByTestId('hero-section');
  await heroSection.screenshot({ path: 'HeroSection.png' });
  await browser.close();
})();
