# chess-platform

An online chess platform based on existing platforms such as
[Chess.com](https://www.chess.com/) and [lichess.org](https://lichess.org/).

## Planned features

- [] A Rust library that models a chessboard, allowing the consumer to make
  legal moves until the game ends
- [] A Rust implementation of a chess engine, which the user can play against
  in the frontend
- [] A frontend which allows a user to make moves on a chessboard
- [] Multiplayer chess

## Architecture

```mermaid
graph TD
    DB[(PostgreSQL)]
    Engine[Engine (Rust)]
    ChessLibrary[Chessboard Library (Rust)]
    Backend[Backend API (Rust)]
    Frontend[Frontend (SvelteKit)]

    Frontend --> Backend
    Engine --> ChessLibrary
    Backend --> ChessLibrary
    Backend --> Engine
    Backend --> DB
```
