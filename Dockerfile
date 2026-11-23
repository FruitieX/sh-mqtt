FROM gcr.io/distroless/static@sha256:4c2cd1dee442c9a99924a378c7c126fac8fef14771579ef567c3f2bdd7a5122a
COPY target/x86_64-unknown-linux-musl/release/sh-mqtt /usr/local/bin/sh-mqtt
CMD ["sh-mqtt"]
