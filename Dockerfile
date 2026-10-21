FROM gcr.io/distroless/static@sha256:034df5c1737e8241f6ed6423a12e720ee0f6d6f088609ee172f98f6f2166ef5d
COPY target/x86_64-unknown-linux-musl/release/sh-mqtt /usr/local/bin/sh-mqtt
CMD ["sh-mqtt"]
