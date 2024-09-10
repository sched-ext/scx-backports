# Backports of scx to older kernels

***Work In Progress***

The goal of this repo is to enable sched-ext binaries to build 
on kernels with older versions of the sched-ext source tree.

## Dependencies
* Docker

## How to use this repo

### Setting up a backport

1) Create a branch named `$SHORT_SHA-vmlinux` , where `$SHORT_SHA` is the first 7 characters of the commit hash in the scx kernel tree you are targeting. In this branch, in `.github/workflows/caching-build.yml`, replace KERNEL_COMMIT_SHA7_HERE with the 7 character commit hash of the kernel commit you will be packporting to.

2) Run `backport-scripts/generate-backport-vmlinux.sh` passing it `$SHORT_SHA` (such as `af1234`) and branch name, in that order. Optionally, pass a third arg, the git repo to obtain that commit and branch from, if neccessary.

3) Update the symlink `scheds/include/vmlinux/vmlinux.h` to point to your new vmlinux.h.

4) Commit and push your branch.

5) Open a new branch (i.e. `af1234-${SCX_RELEASE}-fixes`) off of your initial branch (i.e. `af1234`)
and edit files other than `vmlinux.h` until first `cargo test` (ran in this repo's root) passes and then CI passes. Edit `.github/workflows/caching-build.yml` as is neccessary to reducing the schedulers built/tested (not everything can be reasonably backported).

6) Push this branch as you get things working, push it as `af1234-${SCX_RELEASE}-backport` once CI passes.

7) Add a CI badge on this page on the setup-repo branch for the CI job for that new branch.

### Updating a backport

1) Create a new `-fixes` branch from either a `-vmlinux`, `-fixes` branch or a `-backport` branch (whichever is closest to what you want).
2) Continue the instructions above from (4).

### Updating this doc and the setup scripts.

Please make any updates to this document and the setup scripts on the branch `setup-repo`. This will be rebased atop `main` when merging in updates.

*`main` will be updated periodically (roughly coinciding with releases of scx), backports will be updated/created on an as-needed basis.*

*`main` will be overwritten with commits from upstream during syncs. setup-repo will be rebased upon main and pushed to main before rebasing other branches, so put things to persist across this process on setup-repo.`
