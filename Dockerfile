FROM debian:trixie-backports

RUN apt update

RUN apt install make -y
RUN apt install sudo -y
RUN apt install curl -y
RUN apt-get install grub-pc-bin xorriso -y
RUN apt install nasm -y
RUN apt install build-essential -y

WORKDIR /kfs

COPY ./code .

RUN make

ENTRYPOINT ["/bin/sh", "-c"]