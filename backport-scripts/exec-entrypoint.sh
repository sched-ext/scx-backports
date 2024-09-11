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

SHORT_SHA="$1"
BRANCH="$2"

git clone -b "$BRANCH" https://git.kernel.org/pub/scm/linux/kernel/git/tj/sched_ext.git /sched-ext-linux

cd /sched-ext-linux

git checkout "$SHORT_SHA"

# this is for backports and mixing new bpf with old kernel
find . -type f -exec sed -i 's/-Werror/-Wno-error/g' {} \;

vng -v --kconfig --config /sched-ext.config

make -j "$(nproc)"

bpftool btf dump file "/sched-ext-linux/vmlinux" format c > "/vmlinux-${SHORT_SHA}.h"

echo "generated vmlinux-${SHORT_SHA}.h"
