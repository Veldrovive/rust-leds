FROM rust:1.31

WORKDIR /usr/src/leds
COPY . .

RUN cargo install --path .

CMD ["leds"]