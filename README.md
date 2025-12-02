# taiko forced-inclusion toolbox 🧰

Simple CLI tool to interact with the Taiko `ForcedInclusionStore` contract.

Supports the Shasta fork specification.

## Usage (with docker)

You can use this tool without cloning the repo, using the pre-built image directly:

```shell
# copy the .env.example file to your working directory and edit it
curl https://raw.githubusercontent.com/merklefruit/taiko-forced-inclusion-toolbox/refs/heads/main/.env.example > .env
vim .env

# to send a transaction through a forced-inclusion batch:
docker run -v .env:/app/.env ghcr.io/merklefruit/taiko-forced-inclusion-toolbox:latest send

# to read the current queue from the contract:
docker run -v .env:/app/.env ghcr.io/merklefruit/taiko-forced-inclusion-toolbox:latest read-queue

# to monitor the queue for new events as they are emitted:
docker run -v .env:/app/.env ghcr.io/merklefruit/taiko-forced-inclusion-toolbox:latest monitor-queue

# to periodically send a forced-inclusion batch in a loop:
docker run -v .env:/app/.env ghcr.io/merklefruit/taiko-forced-inclusion-toolbox:latest spam
```

## Usage (from source)

Requires Rust and Cargo to be installed. You can install them from [rustup.rs](https://rustup.rs/).

```shell
# clone the repo locally
git clone git@github.com:merklefruit/taiko-forced-inclusion-toolbox.git && cd taiko-forced-inclusion-toolbox

# fill out the .env file with the required variables
cp .env.example .env
vim .env
```

Then you can run the binary:

```shell
# to send a transaction through a forced-inclusion batch:
cargo run send

# to read the current queue from the contract:
cargo run read-queue

# to monitor the queue for new events as they are emitted:
cargo run monitor-queue

# to periodically send a forced-inclusion batch in a loop:
cargo run spam
```

## License

[MIT](./LICENSE).
