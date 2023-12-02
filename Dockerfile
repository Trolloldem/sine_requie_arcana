FROM rust:1 as builder
WORKDIR /app
COPY . .
RUN sed -i "s;<YOUR_SECRET_GOES_HERE>;$(openssl rand -base64 32);" Rocket.toml
RUN cargo install --path .

FROM debian:bookworm-slim as runner
COPY --from=builder /usr/local/cargo/bin/sine_requie_arcana /usr/local/bin/sine_requie_arcana
WORKDIR /app/arcana-frontend
COPY --from=builder /app/arcana-frontend .
COPY --from=builder /app/Rocket.toml ..
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8080
CMD ["sine_requie_arcana"]
