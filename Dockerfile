FROM --platform=$TARGETOS/$TARGETARCH rust:latest AS build-image
LABEL org.opencontainers.image.description="Mock service"
LABEL authors="Olegs Korsaks"

ARG upx_version=5.0.2
ARG TARGETARCH
ARG TARGETOS

WORKDIR /build

RUN apt update && apt install -y --no-install-recommends xz-utils musl-tools musl-dev && \
  curl -Ls https://github.com/upx/upx/releases/download/v${upx_version}/upx-${upx_version}-${TARGETARCH}_${TARGETOS}.tar.xz -o - | tar xvJf - -C /tmp && \
  cp /tmp/upx-${upx_version}-${TARGETARCH}_${TARGETOS}/upx /usr/local/bin/ && \
  chmod +x /usr/local/bin/upx && \
  apt remove -y xz-utils && \
  rm -rf /var/lib/apt/lists/*

COPY ./ /build/

# Map Docker architecture to Rust target
RUN echo "Target architecture is: ${TARGETARCH}" && \
    if [ "${TARGETARCH}" = "amd64" ]; then \
        RUST_TARGETARCH=x86_64 make release; \
    elif [ "${TARGETARCH}" = "arm64" ]; then \
        RUST_TARGETARCH=aarch64 make release; \
    else \
        echo "Unsupported architecture: ${TARGETARCH}"; exit 1; \
    fi

FROM --platform=$TARGETOS/$TARGETARCH gcr.io/distroless/static-debian12:nonroot

LABEL org.opencontainers.image.description="Mock service"
LABEL authors="Olegs Korsaks"

ARG TARGETARCH
ARG TARGETOS

WORKDIR /
COPY --from=build-image /build/target/mock-service /build/LICENSE /

USER nonroot:nonroot

ENTRYPOINT ["/mock-serivce"]
