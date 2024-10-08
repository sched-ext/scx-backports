#!/bin/bash

set -euxo pipefail

SHORT_SHA="$1"
BRANCH="$2"
REPO="$3"

START_PWD="$(pwd)"

REPO_ROOT="$(git rev-parse --show-toplevel)"

docker build -f backport-scripts/Dockerfile -t local-build-container .

BUILD_CONTAINER="$(docker run -it --rm --detach local-build-container)"

docker exec -it "${BUILD_CONTAINER}" "/exec-entrypoint.sh" "${SHORT_SHA}" "${BRANCH}" "${REPO:=https://git.kernel.org/pub/scm/linux/kernel/git/tj/sched_ext.git}"

clean_up_container () {
    ARG=$?
    docker rm -f "${BUILD_CONTAINER}"
    exit $ARG
} 
trap clean_up_container EXIT

# copy artifact to host
docker cp "${BUILD_CONTAINER}:/vmlinux-${SHORT_SHA}.h" "${REPO_ROOT}/scheds/include/vmlinux/vmlinux-${SHORT_SHA}.h"

# update vmlinux.h symlink
ln -sf "${REPO_ROOT}/scheds/include/vmlinux/vmlinux-${SHORT_SHA}.h" "${REPO_ROOT}/scheds/include/vmlinux/vmlinux.h"

# return to where called from
cd "${START_PWD}"

echo "updated vmlinux.h for kernel version ${SHORT_SHA}"

exit 0
