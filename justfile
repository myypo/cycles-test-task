default: 
    just --list --unsorted

@dev:
    #!/usr/bin/env bash

    docker compose --file ./docker/dev/dev.docker-compose.yaml up --detach
    until docker exec burger-database pg_isready -p 5432 ; do sleep 0.25 ; done
    cargo sqlx migrate run -D "postgresql://postgres:postgres@127.0.0.1:5432/burger"
    cargo sqlx prepare -D "postgresql://postgres:postgres@127.0.0.1:5432/burger" -- --all-features

@e2e:
    #!/usr/bin/env bash

    cargo sqlx prepare -- --all-features
    git add -A

    set -e
    nix build .#e2eDocker
    docker load < result
    set +e

    docker compose --file ./docker/e2e/e2e.docker-compose.yaml up --detach
    until docker exec burger-e2e-database pg_isready -p 6969 ; do sleep 0.25 ; done
    
    URL="http://localhost:16161"
    STATUS_CODE=0
    while [[ $STATUS_CODE -ne 200 ]]; do
      STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$URL/api/v1/health")
      if [[ $STATUS_CODE -ne 200 ]]; then
        echo "server not up yet, retrying"
        sleep 0.25
      fi
    done

    hurl --test --error-format long --continue-on-error \
        --variable url="$URL" \
        ./tests/default.hurl || true

    docker stop burger-e2e-database burger-e2e-server burger-e2e-minio
    docker rm burger-e2e-database burger-e2e-server burger-e2e-minio
    docker volume rm e2e_burger-e2e-data e2e_minio-burger-e2e-data e2e_minio-burger-e2e-root
    docker image rm burger-e2e-server

@watch:
	cargo-watch -- cargo run --features=dev

@lint:
	cargo clippy -- -D warnings
