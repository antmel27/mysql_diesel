FROM rust:1 as builder
WORKDIR /mysql_diesel
COPY . .
RUN cargo install --path .


FROM debian:buster-slim as runner
RUN apt-get -y update && apt-get install -y default-libmysqlclient-dev && apt-get install -y libssl1.1
COPY --from=builder /usr/local/cargo/bin/mysql_diesel /usr/local/bin/mysql_diesel
ENV ROCKET_ADDRESS=0.0.0.0
ENV DATABASE_URL=mysql://admin:KGvm8i3he28Zv7X@database-test.cco6ewvqipkm.eu-north-1.rds.amazonaws.com:3306/courselend-db
ENV TOTP_SECRET=1234
EXPOSE 8000
CMD ["mysql_diesel"]





