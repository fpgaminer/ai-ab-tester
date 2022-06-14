FROM rust:latest as builder

WORKDIR /usr/src/ai-ab-tester
COPY src ./src
COPY Cargo.* ./
COPY migrations ./migrations

RUN cargo install --path .


# Build the final image
FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/ai-ab-tester /usr/local/bin/
COPY webapp/dist/index.html ./webapp/dist/
COPY webapp/dist/main.bundle.js ./webapp/dist/

EXPOSE 8080

CMD ["ai-ab-tester"]