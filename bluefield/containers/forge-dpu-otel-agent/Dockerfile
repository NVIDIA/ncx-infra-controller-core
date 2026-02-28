# Runtime image for forge-dpu-otel-agent on arm64 DPUs.
#
# The binary is cross-compiled externally using the
# build-artifacts-container-cross-aarch64 image and copied into this
# minimal runtime container.

FROM --platform=linux/arm64 debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY target/aarch64-unknown-linux-gnu/release/forge-dpu-otel-agent /usr/bin/forge-dpu-otel-agent

ENTRYPOINT ["/usr/bin/forge-dpu-otel-agent"]
CMD ["--config-path=/etc/forge-dpu-otel-agent/config.toml", "run", "--source-tls-cert-path=/opt/forge/machine_cert.pem", "--source-tls-key-path=/opt/forge/machine_cert.key"]
