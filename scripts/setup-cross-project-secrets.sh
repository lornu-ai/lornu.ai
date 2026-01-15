#!/bin/bash
# Setup cross-project secret access for lornu.ai
# 
# This script grants Service Accounts in the workload project access to
# secrets stored in the secret project (cross-project access).
#
# Usage:
#   ./scripts/setup-cross-project-secrets.sh
#   SECRET_PROJECT=lornu-legacy WORKLOAD_PROJECT=lornu-v2 ./scripts/setup-cross-project-secrets.sh

set -euo pipefail

# Configuration (can be overridden via environment variables)
SECRET_PROJECT="${SECRET_PROJECT:-lornu-legacy}"
WORKLOAD_PROJECT="${WORKLOAD_PROJECT:-lornu-v2}"
SERVICE_ACCOUNT="${SERVICE_ACCOUNT:-engine-sa@${WORKLOAD_PROJECT}.iam.gserviceaccount.com}"

# List of secrets to grant access to
# Add or remove secrets as needed
SECRETS=(
  "OPENAI_KEY"
  "CLOUDFLARE_TOKEN"
  "DATABASE_PASSWORD"
)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Setting up cross-project secret access"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Secret Project:    ${SECRET_PROJECT}"
echo "Workload Project:  ${WORKLOAD_PROJECT}"
echo "Service Account:   ${SERVICE_ACCOUNT}"
echo ""
echo "Secrets to configure:"
for secret in "${SECRETS[@]}"; do
  echo "  - ${secret}"
done
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Aborted."
  exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Verifying prerequisites..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Verify service account exists
echo "Checking service account: ${SERVICE_ACCOUNT}"
if ! gcloud iam service-accounts describe "${SERVICE_ACCOUNT}" >/dev/null 2>&1; then
  echo "❌ Error: Service account not found: ${SERVICE_ACCOUNT}"
  echo "   Create it with:"
  echo "   gcloud iam service-accounts create engine-sa --project=${WORKLOAD_PROJECT}"
  exit 1
fi
echo "✅ Service account exists"

# Verify secrets exist
echo ""
echo "Checking secrets in ${SECRET_PROJECT}..."
for secret in "${SECRETS[@]}"; do
  if ! gcloud secrets describe "${secret}" --project="${SECRET_PROJECT}" >/dev/null 2>&1; then
    echo "⚠️  Warning: Secret not found: ${secret}"
    echo "   Create it with:"
    echo "   gcloud secrets create ${secret} --project=${SECRET_PROJECT} --replication-policy=automatic"
  else
    echo "✅ Secret exists: ${secret}"
  fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Granting cross-project access..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SUCCESS_COUNT=0
FAILED_SECRETS=()

for secret in "${SECRETS[@]}"; do
  echo ""
  echo "Granting access to ${secret}..."
  
  if gcloud secrets add-iam-policy-binding "${secret}" \
    --project="${SECRET_PROJECT}" \
    --role="roles/secretmanager.secretAccessor" \
    --member="serviceAccount:${SERVICE_ACCOUNT}" 2>&1; then
    echo "✅ Access granted to ${secret}"
    ((SUCCESS_COUNT++))
  else
    # Check if binding already exists
    if gcloud secrets get-iam-policy "${secret}" --project="${SECRET_PROJECT}" \
      --format="value(bindings[].members)" | grep -q "${SERVICE_ACCOUNT}"; then
      echo "ℹ️  Access already granted to ${secret}"
      ((SUCCESS_COUNT++))
    else
      echo "❌ Failed to grant access to ${secret}"
      FAILED_SECRETS+=("${secret}")
    fi
  fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Successfully configured: ${SUCCESS_COUNT}/${#SECRETS[@]} secrets"

if [ ${#FAILED_SECRETS[@]} -gt 0 ]; then
  echo ""
  echo "⚠️  Failed secrets:"
  for secret in "${FAILED_SECRETS[@]}"; do
    echo "  - ${secret}"
  done
  exit 1
fi

echo ""
echo "✅ Cross-project access configured successfully!"
echo ""
echo "💡 To access these secrets from ${WORKLOAD_PROJECT}, use full resource names:"
echo ""
for secret in "${SECRETS[@]}"; do
  echo "   projects/${SECRET_PROJECT}/secrets/${secret}/versions/latest"
done
echo ""
