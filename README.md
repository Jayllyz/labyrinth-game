# labyrinth-game

[rules](https://github.com/haveneer-training/sauve_qui_peut)

## How to play

1 - Start server

```bash
cargo run -p server -- --host <ip> -p <port>
```

2 - Start client

```bash
cargo run -p client -- --host <ip> -p <port> -t <team_name>
```

In case of any doubt, you can always run the following command to get help:

```bash
cargo run -p client -- --help
# or
cargo run -p server -- --help
```
