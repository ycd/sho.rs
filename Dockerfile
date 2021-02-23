FROM rustlang/rust:nightly-alpine

COPY . .
RUN apk add --update alpine-sdk
RUN cargo build 


# COPY TO BIN
RUN cp target/debug/shors /usr/bin/shors

CMD [ "shors" ]