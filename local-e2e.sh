#!/bin/bash

set -euxo pipefail

SCHED="$1"
REPO_ROOT="$(git rev-parse --show-toplevel)"

cd ./local-linux-checkout || exit 1
vng -v --memory 10G --cpu 8 --build --config "${REPO_ROOT}/.github/workflows/sched-ext.config"
cd $OLDPWD || exit 1
meson setup build -Dkernel=./local-linux-checkout -Dkernel_headers=./local-linux-checkout/usr/include -Denable_stress=true
ln -sf "${REPO_ROOT}/local-linux-checkout" "${REPO_ROOT}/build/local-linux-checkout"
meson compile -C build "$SCHED"
meson compile -C build test_sched_"$SCHED"
meson compile -C build stress_tests_"$SCHED"

