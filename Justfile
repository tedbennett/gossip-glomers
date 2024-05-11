echo:
	maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10	

guids:
	maelstrom/maelstrom test -w unique-ids --bin target/debug/guids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-a:
	maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 1 --time-limit 20 --rate 10

broadcast-b:
	maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
