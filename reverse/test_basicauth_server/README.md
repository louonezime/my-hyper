# Lighttpd

A Lighttpd server using Docker, used to test basic authentication to a server with resources.

## Installation

To build the Docker image, simply run the following command:

```
$ docker build -t lighttpd_image .
```

(lighttpd_image can be replaced with any way you want to name your image)

## Quickstart

Now, to run the Docker container, run the following command:

```
$ docker run -d -p 8080:80 lighttpd_image
```
