# Backports of scx to older kernels

***Work In Progress***

The goal of this repo is to enable sched-ext binaries to build 
on kernels with older versions of the sched-ext source tree.

## Dependencies
* Docker

## How to use this repo

### Setting up a backport

1) Create a branch named `${KERNEL_VERSION}-vmlinux` , where `${KERNEL_VERSION}` is the branch of the scx tree of the kernel source you are targeting.

2) Run `backport-scripts/generate-backport-vmlinux.sh` passing it a kernel version (such as `scx-dsq-iter-v5`).

3) Update the symlink `scheds/include/vmlinux/vmlinux.h` to point to your new vmlinux.h.

4) Add a CI badge to this branch on this page.

5) Commit and push your branch.

6) Open a new branch (i.e. `scx-dsq-iter-v5-${SCX_RELEASE}-fixes`) off of your initial branch (i.e. `scx-dsq-iter-v5-vmlinux`)
and edit files other than `vmlinux.h` until first `cargo test` (ran in this repo's root) passes and then CI passes.

7) Push this branch as you get things working, push it as `scx-dsq-iter-v5-${SCX_RELEASE}-backport` once CI passes.

### Updating a backport

1) Create a new `-fixes` branch from either a `-vmlinux`, `-fixes` branch or a `-backport` branch (whichever is closest to what you want).
2) Continue the instructions above from (4).

### Updating this doc and the setup scripts.

Please make any updates to this document and the setup scripts on the branch `setup-repo`. This will be rebased atop `main` when merging in updates.

*`main` will be updated periodically (roughly coinciding with releases of scx), backports will be updated/created on an as-needed basis.*