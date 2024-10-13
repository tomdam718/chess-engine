# Use debian-bookwork-slim rust image as base
FROM rust:1-slim-bookworm as builder

# Set the working directory and copy the source code
WORKDIR /usr/src/tsunami
COPY . .

# Build the application with cargo
RUN cargo install --path .

# Lichess bot
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y git python3 python3-pip python3-virtualenv python3-venv && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/tsunami

# Clone lichess-bot
RUN git clone https://github.com/lichess-bot-devs/lichess-bot

# Setup the lichess-bot
COPY config.yml lichess-bot/config.yml
COPY --from=builder /usr/local/cargo/bin/tsunami lichess-bot/engines/tsunami
RUN chmod +x lichess-bot/engines/tsunami

# Set the working directory to lichess-bot
WORKDIR /usr/src/tsunami/lichess-bot

# Install the dependencies
RUN python3 -m venv venv

RUN virtualenv venv -p python3
RUN ./venv/bin/python3 -m pip install -r requirements.txt

# Run the application
CMD ["./venv/bin/python3", "lichess-bot.py"]
