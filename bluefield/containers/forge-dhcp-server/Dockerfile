# Runtime image for forge-dhcp-server on arm64 DPUs.
#
# The binary is cross-compiled externally using the
# build-artifacts-container-cross-aarch64 image and copied into this
# minimal runtime container.

FROM --platform=linux/arm64 debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /var/lib/hbn/var/support/forge-dhcp/conf \
             /var/lib/hbn/var/support/forge-dhcp/logs

COPY target/aarch64-unknown-linux-gnu/release/forge-dhcp-server /usr/bin/forge-dhcp-server

ENTRYPOINT ["/usr/bin/forge-dhcp-server"]
