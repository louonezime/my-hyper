SRC =	server	\
		client 	\
		reverse

all: server client reverse

server:
	make -C server
	mv server/server .

client:
	make -C client
	mv client/client .

reverse:
	make -C reverse
	mv reverse/reverse .

clean:
	make clean -C server
	make clean -C client
	make clean -C reverse

fclean:
	$(RM) server client reverse

re: fclean all

.PHONY: all clean re server client reverse
