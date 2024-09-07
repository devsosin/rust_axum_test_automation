# BUILDER ----------------------------------------------------------------
FROM        rust:latest as builder

WORKDIR     /usr/src/app

RUN         USER=root cargo new backend

WORKDIR     /usr/src/app/backend

COPY        ./Cargo.toml ./Cargo.toml

# for caching
RUN         cargo build --release

RUN         rm src/*.rs

ADD         . .

RUN         cargo build --release

# RUNNER ----------------------------------------------------------------
FROM        debian:bookworm-slim

RUN         apt-get update && \
                apt-get install -y --no-install-recommends \
                    libssl3 \
                    ca-certificates

ARG         APP=/usr/src/app

EXPOSE      3000

ENV         APP_USER=appuser

RUN         groupadd $APP_USER \
                && useradd -g $APP_USER $APP_USER \
                && mkdir -p ${APP}

COPY        --from=builder /usr/src/app/backend/target/release/backend ${APP}/backend

RUN         chown -R $APP_USER:$APP_USER ${APP}
USER        $APP_USER
WORKDIR     ${APP}

CMD         ["./backend"]