NAME	=	client

all:	$(NAME)

$(NAME):
	cargo build
	mv target/debug/$(NAME) .

clean:
	cargo clean

fclean: clean
	$(RM) $(NAME)

re: fclean all

.PHONY:	all clean fclean re
