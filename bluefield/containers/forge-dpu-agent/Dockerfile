# Runtime image for forge-dpu-agent on arm64 DPUs.
#
# The binary is cross-compiled externally using the
# build-artifacts-container-cross-aarch64 image and copied into this
# minimal runtime container.

FROM --platform=linux/arm64 debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY target/aarch64-unknown-linux-gnu/release/forge-dpu-agent /usr/bin/forge-dpu-agent

ENTRYPOINT ["/usr/bin/forge-dpu-agent"]
CMD ["run", "--enable-metadata-service"]
