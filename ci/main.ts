#!/usr/bin/env bun
/**
 * Lornu AI - Dagger CI Pipeline
 *
 * Orchestrates the full build and deployment pipeline:
 * 1. Build Rust services (engine, gateway, agent-worker)
 * 2. Build web frontend (Bun)
 * 3. Install Crossplane (Helm)
 * 4. Synthesize and apply CDK8s manifests
 *
 * Usage:
 *   dagger run bun ci/main.ts
 *   dagger run bun ci/main.ts --skip-infra  # Skip Crossplane setup
 */

import { connect } from '@dagger.io/dagger';

const SKIP_INFRA = process.argv.includes('--skip-infra');
const ENV = process.env.LORNU_ENV || 'dev';

async function pipeline() {
  await connect(async (client) => {
    const src = client.host().directory('.', {
      exclude: ['node_modules', 'target', '.git', 'cdk8s.out'],
    });

    console.log('Starting Lornu AI CI Pipeline');
    console.log(`   Environment: ${ENV}`);
    console.log(`   Skip Infra: ${SKIP_INFRA}`);
    console.log('');

    // =========================================
    // Stage 1: Build Rust Services
    // =========================================
    console.log('Building Rust services...');

    const rustBuilder = client
      .container()
      .from('rust:1.83-slim')
      .withExec(['apt-get', 'update'])
      .withExec(['apt-get', 'install', '-y', 'pkg-config', 'libssl-dev'])
      .withDirectory('/src', src)
      .withWorkdir('/src');

    const engineBuild = rustBuilder
      .withWorkdir('/src/services/engine')
      .withExec(['cargo', 'build', '--release', '--locked']);

    const gatewayBuild = rustBuilder
      .withWorkdir('/src/services/gateway')
      .withExec(['cargo', 'build', '--release', '--locked']);

    const workerBuild = rustBuilder
      .withWorkdir('/src/services/agent-worker')
      .withExec(['cargo', 'build', '--release', '--locked']);

    // =========================================
    // Stage 2: Infrastructure (Crossplane)
    // =========================================
    if (!SKIP_INFRA) {
      console.log('Setting up Crossplane infrastructure...');

      const infraContainer = client
        .container()
        .from('oven/bun:latest')
        .withDirectory('/src', src)
        .withWorkdir('/src')
        .withExec(['sh', '-c',
          'curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && chmod +x kubectl && mv kubectl /usr/local/bin/'
        ])
        .withExec(['sh', '-c',
          'curl -fsSL https://get.helm.sh/helm-v3.14.0-linux-amd64.tar.gz | tar xz && mv linux-amd64/helm /usr/local/bin/'
        ]);

      const crossplaneInstall = infraContainer
        .withExec(['helm', 'repo', 'add', 'crossplane-stable', 'https://charts.crossplane.io/stable'])
        .withExec(['helm', 'repo', 'update'])
        .withExec(['helm', 'upgrade', '--install', 'crossplane',
          '--namespace', 'crossplane-system',
          '--create-namespace',
          '--wait',
          'crossplane-stable/crossplane'
        ]);

      const synthManifests = client
        .container()
        .from('oven/bun:latest')
        .withDirectory('/src', src)
        .withWorkdir('/src/infra')
        .withExec(['bun', 'install'])
        .withEnvVariable('LORNU_ENV', ENV)
        .withExec(['bun', 'run', 'synth']);

      await crossplaneInstall.sync();
      await synthManifests.sync();
    }

    // =========================================
    // Stage 3: Run all builds in parallel
    // =========================================
    console.log('Running builds in parallel...');

    await Promise.all([
      engineBuild.sync(),
      gatewayBuild.sync(),
      workerBuild.sync(),
    ]);

    console.log('');
    console.log('='.repeat(50));
    console.log('Pipeline completed successfully!');

  }, { LogOutput: process.stdout });
}

pipeline().catch((e) => {
  console.error('Pipeline failed:', e);
  process.exit(1);
});
