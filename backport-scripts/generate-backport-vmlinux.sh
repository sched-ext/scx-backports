#!/bin/bash

set -euxo pipefail

KERNEL_VERSION="$1"

START_PWD="$(pwd)"

REPO_ROOT="$(git rev-parse --show-toplevel)"

docker build -f backport-scripts/Dockerfile -t local-build-container .

BUILD_CONTAINER="$(docker run -it --rm --detach local-build-container)"

docker exec -it "${BUILD_CONTAINER}" "/exec-entrypoint.sh" "${KERNEL_VERSION}"

clean_up_container () {
    ARG=$?
    docker rm -f "${BUILD_CONTAINER}"
    exit $ARG
} 

trap clean_up_container EXIT

# copy artifact to host
docker cp "${BUILD_CONTAINER}:/vmlinux-${KERNEL_VERSION}.h" "${REPO_ROOT}/scheds/include/vmlinux/vmlinux-${KERNEL_VERSION}.h"

# update vmlinux.h symlink
ln -sf "${REPO_ROOT}/scheds/include/vmlinux/vmlinux-${KERNEL_VERSION}.h" "${REPO_ROOT}/scheds/include/vmlinux/vmlinux.h"

# return to where called from
cd "${START_PWD}"

echo "updated vmlinux.h for kernel version ${KERNEL_VERSION}"

exit 0
