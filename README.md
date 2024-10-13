# Tsunami

A UCI chess engine written in Rust. At the moment, it uses a NNUE implementation from [akimbo](https://github.com/jw1912/akimbo). A self-made trained from zero network is planned.

## Using

This engine can be used with any GUI that supports UCI, such as [Cute Chess](https://cutechess.com/), [Arena](https://www.playwitharena.de/), or [Lucas Chess](https://lucaschess.pythonanywhere.com/). There are releases for macOS (ARM and Intel), and Windows.

## Features

- Negamax search
- Alpha-beta pruning
- MVV-LVA for move ordering
- Iterative deepening for time control
