# cota-aggregator

The aggregator of [CoTA](https://talk.nervos.org/t/rfc-cota-a-compact-token-aggregator-standard-for-extremely-low-cost-nfts-and-fts/6338) service

### Quick Start

Update `database_url` in `aggregator.toml` with your mysql url or set `DATABASE_URL` as environment variable

```shell
make build

make run

make test
```

If you set `DATABASE_URL` as environment variable, you can run as below:

```shell
DATABASE_URL=mysql://root:password@localhost:3306/db_name make run
```

### Usage

```shell
make build-release
RUST_LOG=info ./target/release/cota-aggregator

# or
make install
RUST_LOG=info cota-aggregator
```

If you set `DATABASE_URL` as environment variable, you can run as below:

```shell
make build-release
RUST_LOG=info DATABASE_URL=mysql://root:password@localhost:3306/db_name ./target/release/cota-aggregator

# or
make install
RUST_LOG=info DATABASE_URL=mysql://root:password@localhost:3306/db_name cota-aggregator
```

```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "generate_define_cota_smt",
    "params": {"lock_hash": "0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1", "cota_id" : "0xea28c98f38b4a57aa81756b167bb37fa42daf67e", "total": "0x00000050", "issued": "0x00000000", "configure": "0x00"}
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

### Deploy with docker

```shell
# Build cota-aggregator images from the Dockerfile and run cota-aggregator via docker
docker build -t cota-aggregator .
docker run -d -p 3030:3030 cota-aggregator:latest

# or
docker-compose up -d --build
```