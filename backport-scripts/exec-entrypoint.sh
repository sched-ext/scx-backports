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
REPO="$3"

git clone -b "$BRANCH" "$REPO" /sched-ext-linux

cd /sched-ext-linux

git checkout "$SHORT_SHA"

# this is for backports and mixing new bpf with old kernel
# we only care about generated vmlinux.h, so make build work.

echo '' >> /sched-ext.config

echo 'CONFIG_DEBUG_INFO_DWARF4=y' >> /sched-ext.config
echo 'CONFIG_DEBUG_INFO_BTF=y' >> /sched-ext.config
echo 'CONFIG_DEBUG_INFO=y' >> /sched-ext.config

vng -v --kconfig --config /sched-ext.config

make ARCH=x86 KCFLAGS="-fno-pic -fno-stack-protector" -j "$(nproc)" all 

pahole -J /sched-ext-linux/vmlinux

bpftool btf dump file "/sched-ext-linux/vmlinux" format c > "/vmlinux-${SHORT_SHA}.h"

echo "generated vmlinux-${SHORT_SHA}.h"
