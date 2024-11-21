## Requirements
- [Docker](https://docs.docker.com/get-docker/)
- [Rust](https://www.rust-lang.org/tools/install)
- [protoc](https://grpc.io/docs/protoc-installation/)
- 


### Docker Builds
```bash
docker build -f docker/api.Dockerfile \
--build-arg GIT_HTTPS_USERNAME=dallinwright-tradecrit \
--build-arg GIT_HTTPS_PASSWORD=$GITHUB_TOKEN \
-t ghcr.io/tradecrit/asset-details:api-latest --push .

docker build -f docker/ingestor.Dockerfile \
--build-arg GIT_HTTPS_USERNAME=dallinwright-tradecrit \
--build-arg GIT_HTTPS_PASSWORD=$GITHUB_TOKEN \
-t ghcr.io/tradecrit/asset-details:ingestor-latest --push .
```

### Development
```bash
kubectl create namespace asset-details || true; \
kubectl label namespace asset-details istio-injection=enabled --overwrite; \
helm dep update ./deployment/service; \
helm upgrade --install \
-n asset-details \
-f ./deployment/development.yaml \
asset-details \
./deployment/service
```

### Production
```bash
kubectl create namespace asset-details || true; \
kubectl label namespace asset-details istio-injection=enabled --overwrite; \
helm dep update ./deployment/service; \
helm upgrade --install \
-n asset-details \
-f ./deployment/production.yaml \
asset-details \
./deployment/service
```


hyperfine --min-runs 10 \
--parameter-scan num_threads 8 8 \
'/home/dallin/.cargo/bin/cargo test \
--color=always \
--profile test \
--test load_test tests \
--no-fail-fast \
--config env.RUSTC_BOOTSTRAP=\"1\" \
--manifest-path /home/dallin/projects/asset-details/crates/grpc/Cargo.toml \
-- \
--format=json -Z unstable-options --show-output'

CREATE DATABASE "asset-details";

CREATE USER "aduser";

ALTER USER "aduser" WITH PASSWORD 'xxx';

GRANT ALL PRIVILEGES ON DATABASE "asset-events" TO "aduser";

\c asset-events

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO "aduser";
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO "aduser";
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO "aduser";
GRANT CREATE ON DATABASE "asset-events" TO "aduser";
GRANT CREATE, USAGE ON SCHEMA public TO aduser;
