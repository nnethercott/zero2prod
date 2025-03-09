#!/bin/bash
set -x # lists all commands run
set -eo pipefail # something about nonzero exit

docker run \
    -p 6379:6379 \
    -d \
    --name "redis_$(date '+%s')" \
    redis:7

>&2 echo "Redis is good to go!"
