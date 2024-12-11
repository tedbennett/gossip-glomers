serve:
    maelstrom/maelstrom serve

echo:
    cargo build --bin echo
    maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10

guids:
    cargo build --bin guids
    maelstrom/maelstrom test -w unique-ids --bin target/debug/guids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-a:
    cargo build --bin broadcast
    maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 1 --time-limit 20 --rate 10

broadcast-b:
    cargo build --bin broadcast
    maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10

broadcast-c:
    cargo build --bin broadcast
    maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition

broadcast-d:
    cargo build --bin broadcast
    maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100

broadcast-e:
    cargo build --bin broadcast
    maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100

g-counter:
    cargo build --bin g-counter
    maelstrom/maelstrom test -w g-counter --bin target/debug/g-counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition

kafka-a:
    cargo build --bin kafka
    maelstrom/maelstrom test -w kafka --bin target/debug/kafka --node-count 1 --concurrency 2n --time-limit 20 --rate 1000
