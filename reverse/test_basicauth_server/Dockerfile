FROM debian:latest

RUN apt-get update && apt-get install -y lighttpd

RUN sed -i 's/\(server.modules.*\)mod_authn_file/\1 mod_authn_file/' /etc/lighttpd/lighttpd.conf
RUN sed -i 's/\(server.modules.*\)mod_auth/\1 mod_auth/' /etc/lighttpd/lighttpd.conf
RUN sed -i 's/\(server.modules.*\)mod_proxy/\1 mod_proxy/' /etc/lighttpd/lighttpd.conf

COPY lighttpd/lighttpd.conf /etc/lighttpd/lighttpd.conf
COPY passwords.txt /etc/lighttpd/passwords.txt
COPY ./test_basic/* /var/www/html/

EXPOSE 80

CMD ["lighttpd", "-D", "-f", "/etc/lighttpd/lighttpd.conf"]
