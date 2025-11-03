#!/bin/bash
docker build --platform linux/riscv64 --rm -t exporter .
if [ $? -ne 0 ]; then
    echo "Docker build failed"
    exit 1
fi

CONTAINER=$(docker create exporter)
docker cp $CONTAINER:/dora ./dora
docker rm $CONTAINER
