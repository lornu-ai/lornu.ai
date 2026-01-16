#!/usr/bin/env bun
/**
 * Secret Lifecycle Cleanup Utility
 *
 * Issue #44: Secure secret handling with automatic cleanup
 *
 * This script scans for and securely deletes files matching sensitive patterns.
 * It's designed to run after CI/CD pipelines or development sessions to ensure
 * no secrets persist on disk.
 *
 * Usage:
 *   bun run infra/scripts/cleanup.ts [--dry-run] [--verbose]
 */

import { unlink, stat, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { Glob } from "bun";
import { parseArgs } from "node:util";

// Patterns for sensitive files that should never persist
const SENSITIVE_PATTERNS = [
  "**/setup.sh",
  "**/credentials.json",
  "**/.env.tmp",
  "**/.env.local",
  "**/secrets.json",
  "**/*-credentials.json",
  "**/*-service-account.json",
  "**/gcloud-*.json",
  "**/*.pem",
  "**/*.key",
  "**/token.txt",
  "**/api-key.txt",
];

// Directories to exclude from scanning
const EXCLUDED_DIRS = [
  "node_modules",
  ".git",
  "target",
  "dist",
  ".next",
];

interface CleanupOptions {
  dryRun: boolean;
  verbose: boolean;
  baseDir: string;
}

/**
 * Securely overwrite file contents before deletion (defense in depth)
 */
async function secureWipe(filePath: string): Promise<void> {
  try {
    const stats = await stat(filePath);
    const size = stats.size;

    // Overwrite with zeros
    await writeFile(filePath, Buffer.alloc(size, 0));

    // Overwrite with ones
    await writeFile(filePath, Buffer.alloc(size, 0xff));

    // Overwrite with pattern
    const pattern = Buffer.alloc(size);
    for (let i = 0; i < size; i++) {
      pattern[i] = (i * 0xab) & 0xff;
    }
    await writeFile(filePath, pattern);
  } catch (error) {
    // If secure wipe fails, we still try to delete
    console.warn(`  ⚠️  Secure wipe failed for ${filePath}: ${error}`);
  }
}

/**
 * Check if a path is in an excluded directory
 */
function isExcluded(filePath: string): boolean {
  return EXCLUDED_DIRS.some((dir) => filePath.includes(`/${dir}/`) || filePath.startsWith(`${dir}/`));
}

/**
 * Scan and clean sensitive files
 */
async function nukeSensitiveFiles(options: CleanupOptions): Promise<string[]> {
  const { dryRun, verbose, baseDir } = options;
  const nukedFiles: string[] = [];

  console.log("🔍 Scanning for sensitive files...\n");

  // Combine all patterns into a single glob scan for efficiency
  const combinedPattern = `{${SENSITIVE_PATTERNS.join(",")}}`;
  const glob = new Glob(combinedPattern);

  for await (const file of glob.scan({ cwd: baseDir, absolute: true })) {
    // Skip excluded directories
    if (isExcluded(file)) {
      if (verbose) {
        console.log(`  ⏭️  Skipping (excluded): ${file}`);
      }
      continue;
    }

    // Check if file exists (glob might return stale results)
    if (!existsSync(file)) {
      continue;
    }

    if (dryRun) {
      console.log(`  🔸 Would delete: ${file}`);
      nukedFiles.push(file);
    } else {
      try {
        // Secure wipe before deletion
        await secureWipe(file);

        // Delete the file
        await unlink(file);
        console.log(`  🧨 Nuked: ${file}`);
        nukedFiles.push(file);
      } catch (error) {
        console.error(`  ❌ Failed to delete ${file}: ${error}`);
      }
    }
  }

  return nukedFiles;
}

/**
 * Print summary report
 */
function printSummary(nukedFiles: string[], dryRun: boolean): void {
  console.log("\n" + "═".repeat(60));

  if (nukedFiles.length === 0) {
    console.log("✅ No sensitive files found. Your workspace is clean!");
  } else if (dryRun) {
    console.log(`🔍 DRY RUN: Would delete ${nukedFiles.length} sensitive file(s)`);
    console.log("   Run without --dry-run to actually delete them.");
  } else {
    console.log(`🧹 Securely wiped ${nukedFiles.length} sensitive file(s)`);
  }

  console.log("═".repeat(60));
}

// Main execution
async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      "dry-run": { type: "boolean", default: false },
      verbose: { type: "boolean", short: "v", default: false },
      help: { type: "boolean", short: "h", default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    console.log(`
Secret Lifecycle Cleanup Utility (Issue #44)

Usage:
  bun run infra/scripts/cleanup.ts [options]

Options:
  --dry-run     Show what would be deleted without actually deleting
  --verbose     Show skipped files and detailed output
  --help, -h    Show this help message

Patterns scanned:
${SENSITIVE_PATTERNS.map((p) => `  - ${p}`).join("\n")}
`);
    process.exit(0);
  }

  const options: CleanupOptions = {
    dryRun: values["dry-run"] ?? false,
    verbose: values.verbose ?? false,
    baseDir: process.cwd(),
  };

  console.log("🔐 Secret Lifecycle Cleanup Utility");
  console.log(`   Base directory: ${options.baseDir}`);
  console.log(`   Mode: ${options.dryRun ? "DRY RUN" : "LIVE"}\n`);

  const nukedFiles = await nukeSensitiveFiles(options);
  printSummary(nukedFiles, options.dryRun);
}

// Run if executed directly
main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});

export { nukeSensitiveFiles, secureWipe, SENSITIVE_PATTERNS };
