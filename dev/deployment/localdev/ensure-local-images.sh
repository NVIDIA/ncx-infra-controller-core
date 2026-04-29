#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

ensure_image() {
  local image_name="$1"
  local dockerfile="$2"

  if docker image inspect "${image_name}" >/dev/null 2>&1; then
    return
  fi

  echo "Local image ${image_name} is missing; rebuilding it..."
  DOCKER_BUILDKIT=1 docker build \
    --pull=false \
    -t "${image_name}" \
    -f "${REPO_ROOT}/${dockerfile}" \
    "${REPO_ROOT}"
}

ensure_image "runtime-container-localdev:latest" \
  "dev/deployment/localdev/Dockerfile.runtime-container.localdev"

