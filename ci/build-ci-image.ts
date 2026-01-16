#!/usr/bin/env bun
/**
 * Builds and publishes the custom CI runner image.
 *
 * This image comes pre-installed with:
 * - Bun
 * - Dagger CLI
 * - Just
 *
 * Usage:
 *   bun run image:build
 *   bun run image:publish
 */

import { dag } from "@dagger.io/dagger";
import { parseArgs } from "util";
import { versions } from "./versions";

const args = parseArgs({
  options: {
    tag: {
      type: "string",
      description: "Image tag",
      default: "latest",
    },
    push: {
      type: "boolean",
      description: "Push the image to the registry",
      default: false,
    },
    help: {
      type: "boolean",
      description: "Show help message",
      short: "h",
    },
  },
});

if (args.values.help) {
  console.log(`
Builds and publishes the custom CI runner image.

Usage:
  bun ci/build-ci-image.ts [options]

Options:
  --tag <tag>   Image tag (default: latest)
  --push        Push the image to the registry
  --help, -h    Show this help message
`);
  process.exit(0);
}

async function main() {
  const { values } = args;
  const client = await dag.connect({ logOutput: process.stdout });
  const imageRef = `${versions.images.ciRunner}:${values.tag}`;

  console.log(`Building CI image: ${imageRef}`);

  // Define the CI image
  const ciImage = dag
    .container()
    .from(versions.images.bun)
    .withExec(["apk", "add", "--no-cache", "curl", "bash", "just"])
    .withExec([
      "curl",
      "-L",
      `https://dl.dagger.io/dagger/install.sh`,
      "|",
      "DAGGER_VERSION=" + versions.dagger,
      "sh",
    ]);

  if (values.push) {
    console.log(`Publishing image to ${imageRef}...`);
    await ciImage.publish(imageRef);
    console.log(`✅ Image published successfully!`);
  } else {
    console.log("Image built. Run with --push to publish to registry.");
    // You can use .export() here to save as a local tarball if needed
  }

  await client.close();
}

main().catch((e) => console.error(e));