# Asset Details Service

## Overview

This service is responsible for providing asset details to the TradeCrit platform. It is a gRPC service that provides a single endpoint for fetching asset details. The service is built using Rust and uses the [Tonic](https://github.com/hyperium/tonic) library for gRPC.

The idea is that this is a data service that ingests basic details about every company listed in US equities markets
and providers it to the platform. The 3rd party service has rate limits, etc. so this allows us to make
many requests per second, store the data we want, how we want is a much more responsive, flexible way.

The third party sdk (polygon-sdk also written by me) provides an abstracted library so that we can easily
swap out the data provider if we need to with any data provider without impacting the entire platform.

## Requirements

Baseline requirements for the project are as follows:

- [Docker](https://docs.docker.com/get-docker/)
- [Rust](https://www.rust-lang.org/tools/install)
- [protoc](https://grpc.io/docs/protoc-installation/)
- [protobuf-compiler](https://grpc.io/docs/protoc-installation/)
- [Redis](https://redis.io/download)
- [Postgres](https://www.postgresql.org/download/)

Postgres and Redis can be deployed anywhere, we just need the endpoints to be provided to the service.

## Topology

API
  - The API is a gRPC service that provides a single endpoint for fetching asset details.

Ingestor
    - The ingestor is a service that is responsible for fetching asset details from the third party service and storing them in the database.
this runs currently every 24 hours to fetch updated information such as market cap, employee count, etc. used elsewhere for analysis.

### Docker Builds

The two commands below will build and push the docker images to the GitHub Container Registry. The images are tagged with the latest version.
These are quick utils to allow building locally and pushing to the registry, as opposed to waiting 20 minutes for the CI to build and push on GitHub actions limited runner sizes.

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

## Deployment

### Development/Local Environment

Quick setup for kubernetes environments to provide a development environment for the service. You can see in the `.github/workflows` directory the CI/CD pipeline that is used to deploy the service to the kubernetes cluster.

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

Quick setup for kubernetes environments to provide a production environment for the service. You can see in the `.github/workflows` directory the CI/CD pipeline that is used to deploy the service to the kubernetes cluster.

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

### Load Testing

```bash
cargo install hyperfine
```

```bash
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
```


### Database Setup 

Quick command for shared database compute instance but still maintain microservice isolation for cost reduction.

Run the following commands to create the database and user for the service on a Postgres 16+ instance.

```sql
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
```

Ideally this is all automated, but for local development, this is a quick way to get the database setup.
