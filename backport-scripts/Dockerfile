FROM ubuntu:24.04
# keep codename and ver in sync
ENV UBUNTU_CODENAME=noble

ENV DEBIAN_FRONTEND=noninteractive
RUN echo "deb-src http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME} main" >> /etc/apt/sources.list
RUN echo "deb-src http://archive.ubuntu.com/ubuntu ${UBUNTU_CODENAME}-updates main" >> /etc/apt/sources.list
RUN cat /etc/apt/sources.list
RUN apt update && apt install -y build-essential \
    libelf-dev \
    libz-dev \
    libcap-dev \
    clang llvm llvm-dev lld \
    binutils-dev \
    pkg-config \
    git \
    python3-pip \    
    --no-install-recommends && \
    apt -y build-dep linux --no-install-recommends && \
    apt clean && rm -rf /var/lib/apt/lists/*
RUN pip3 install virtme-ng --break-system-packages
COPY .github/workflows/sched-ext.config /sched-ext.config
COPY backport-scripts/exec-entrypoint.sh /exec-entrypoint.sh
