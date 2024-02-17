#!/usr/bin/env bash

set -euf -o pipefail
#set -x

SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
CLUSTER_NAME="localdev"

# Setup local registry
setup_registry() {
  local CLUSTER_NAME="kind-registry"
  local reg_name="${CLUSTER_NAME}"
  local reg_port='5000'
  local is_running;
  is_running="$(docker inspect -f '{{.State.Running}}' "${reg_name}" 2>/dev/null || true)"
  if [ "${is_running}" != 'true' ]; then
    docker run \
      -d --restart=always -p "${reg_port}:5000" --name "${reg_name}" \
      registry:latest
  fi
  kind delete cluster -n "${CLUSTER_NAME}"
  kind create cluster -n "${CLUSTER_NAME}" --config "${SCRIPTS_DIR}/../manifests/registry-kind-config.yaml"
  # Connect registry to cluster
  docker network connect "kind" "${CLUSTER_NAME}" || { echo "Already connected"; }
  sleep 5
}

# Setup local kind cluster
setup_kind() {
  kind delete cluster -n "${CLUSTER_NAME}"
  kind create cluster -n "${CLUSTER_NAME}" \
    --config "${SCRIPTS_DIR}/../manifests/localdev-kind-config.yaml"
  sleep 5
  setup_registry
}

# Deploy kubernetes-events to local kind cluster
deploy() {
  setup_kind
  local IMAGE_NAME="kubernetes-events"

  docker build -t "localhost:5000/${IMAGE_NAME}:${IMAGE_TAG}" -f deployment/build/Dockerfile .
  docker push "localhost:5000/${IMAGE_NAME}:${IMAGE_TAG}"
  kubectl delete -f deployment/manifests/test-service.yaml --ignore-not-found=true
  kubectl apply -f deployment/manifests/test-service.yaml
} && deploy
