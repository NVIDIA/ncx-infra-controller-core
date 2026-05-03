# Runtime image for forge-dpu-otel-agent on arm64 DPUs.
#
# The binary is cross-compiled externally using the
# build-artifacts-container-cross-aarch64 image and copied into this
# minimal runtime container.

ARG CC_VERSION=v3.2.1 # NVIDIA Distroless Container version
ARG TAG=${CC_VERSION}
ARG IMG=nvcr.io/nvidia/distroless/cc
ARG DEBUG_DISTROLESS=true
ARG DISTROLESS_DEBUG_TAG=${DEBUG_DISTROLESS:+"dev"} # 'dev' provides a busybox shell

FROM --platform=linux/arm64 debian:bookworm-slim AS runtime-deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    coreutils \
    libudev1 \
 && rm -rf /var/lib/apt/lists/*

RUN bins="/bin/bash /bin/mv /bin/date /bin/rm /usr/bin/env /usr/bin/stat /usr/bin/install" \
 && mkdir -p /rootfs \
 && for bin in $bins; do \
      install -D "$bin" "/rootfs$bin"; \
      ldd "$bin" | grep "=> /" | awk '{print $3}' | while read -r lib; do \
        install -D "$lib" "/rootfs$lib"; \
      done; \
    done \
 && install -D /lib/aarch64-linux-gnu/libudev.so.1 /rootfs/lib/aarch64-linux-gnu/libudev.so.1

# NVIDIA Distroless Container
FROM --platform=linux/arm64 ${IMG}:${TAG}-${DISTROLESS_DEBUG_TAG}

COPY --from=runtime-deps /rootfs/ /

COPY --chmod=755 target/aarch64-unknown-linux-gnu/release/forge-dpu-otel-agent /usr/bin/forge-dpu-otel-agent
COPY --chmod=755 bluefield/otel/scripts/otel_copy_certs.sh /usr/local/sbin/otel_copy_certs.sh
COPY --chmod=755 bluefield/otel/otel-agent /otel/otel-agent

ENTRYPOINT ["/otel/otel-agent"]
