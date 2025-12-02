build arch='linux/amd64,linux/arm64' v='latest':
    docker buildx build --platform {{arch}} -t ghcr.io/merklefruit/taiko-forced-inclusion-toolbox:{{v}} --push .
