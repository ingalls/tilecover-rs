FROM ubuntu:17.10

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y \
    && apt-get install -y \
        curl \
        git \
    && locale-gen en_US.UTF-8 \
    && bash -c "echo \"America/New_York\" > /etc/timezone"

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

WORKDIR /usr/local/src/tilecover
ADD . /usr/local/src/tilecover

CMD cargo test
