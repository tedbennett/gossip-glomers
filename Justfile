echo:
	maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10	
guids:
	maelstrom/maelstrom test -w unique-ids --bin target/debug/guids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
