#!/bin/bash

set -euxo pipefail

cd /

if [ -f /.dockerenv ]; then
    echo "Running artifact build in docker container";
else
    echo "This script is not being ran inside a docker container, please run this script inside a docker container.";
fi

git clone --recurse-submodules https://github.com/libbpf/bpftool.git /bpftool

cd /bpftool/src

make -j "$(nproc)"

make -j "$(nproc)" install

cd /

KERNEL_VERSION="$1"

git clone --depth 1 --branch "v${KERNEL_VERSION}" https://git.kernel.org/pub/scm/linux/kernel/git/tj/sched_ext.git /sched-ext-linux

cd /sched-ext-linux

vng -v --kconfig --config /sched-ext.config

make -j "$(nproc)"

bpftool btf dump file "/sched-ext-linux/vmlinux" format c > "/vmlinux-${KERNEL_VERSION}.h"

echo "generated vmlinux-${KERNEL_VERSION}.h"
