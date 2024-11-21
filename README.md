# Asset Details Service

## Overview

This service is responsible for providing asset details to the TradeCrit platform. It is a gRPC service that provides a single endpoint for fetching asset details. The service is built using Rust and uses the [Tonic](https://github.com/hyperium/tonic) library for gRPC.

The idea is that this is a data service that ingests basic details about every company listed in US equities markets
and providers it to the platform. The 3rd party service has rate limits, etc. so this allows us to make
many requests per second, store the data we want, how we want is a much more responsive, flexible way.

There are a few third party sdks (polygon-sdk, kinde-sdk, cloudflare-sdk also written by me) providing an abstracted set of libraries so that we can easily swap out the data provider if we need to with any data provider without impacting the entire platform.

### Binaries
The project is split into two distinct binaries, the long-running API and the ingestor job.

#### API
The API is a gRPC service that provides a single endpoint for fetching asset details. The protobuf files come from the
`crates/grpc/proto` git submodule and directory.

#### Ingestor
The ingestor is a service that is responsible for fetching asset details from the third party service and storing them in the database.
As part of this we need to also transfer the various branding images from the source to our CDN for later use
in the frontend portion of the platform.

## Requirements

Baseline requirements for the project are as follows:

- [Docker](https://docs.docker.com/get-docker/)
- [Rust](https://www.rust-lang.org/tools/install)
- [protoc](https://grpc.io/docs/protoc-installation/)
- [protobuf-compiler](https://grpc.io/docs/protoc-installation/)
- [Redis](https://redis.io/download)
- [Postgres](https://www.postgresql.org/download/)

Postgres and Redis can be deployed anywhere, we just need the endpoints to be provided to the service.

### Docker Builds

The two commands below will build and push the docker images to the GitHub Container Registry. The images are tagged with the latest version.
These are quick utils to allow building locally and pushing to the registry, as opposed to waiting 20 minutes for the CI to build and push on GitHub actions limited runner sizes.

In these two examples, in the shell I have set the environment variables `GIT_HTTPS_USERNAME` and `GIT_HTTPS_PASSWORD` to my GitHub username and access token to then be consumed by the build process.

```bash
docker build -f docker/api.Dockerfile \
--build-arg GIT_HTTPS_USERNAME=$GIT_HTTPS_USERNAME \
--build-arg GIT_HTTPS_PASSWORD=$GIT_HTTPS_PASSWORD \
-t ghcr.io/tradecrit/asset-details:api-latest --push .

docker build -f docker/ingestor.Dockerfile \
--build-arg GIT_HTTPS_USERNAME=$GIT_HTTPS_USERNAME \
--build-arg GIT_HTTPS_PASSWORD=$GIT_HTTPS_PASSWORD \
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

### Example Ingestor Output

The ingestor logs are structured in a way that allows for easy parsing and analysis. When running the ingestor, the logs will look similar to the following:

```json lines
{"timestamp":"2024-11-21T17:22:48.056971Z","level":"INFO","fields":{"message":"Successfully inserted company details for WY"},"target":"ingestor","filename":"bins/ingestor/src/main.rs","line_number":84}
{"timestamp":"2024-11-21T17:22:48.056990Z","level":"INFO","fields":{"message":"Stock: CLNE - Clean Energy Fuels Corp. (8.41%)"},"target":"ingestor","filename":"bins/ingestor/src/main.rs","line_number":35}
{"timestamp":"2024-11-21T17:22:48.295869Z","level":"INFO","fields":{"message":"Processing branding images for CLNE"},"target":"ingestor::images","filename":"bins/ingestor/src/images.rs","line_number":122}
{"timestamp":"2024-11-21T17:22:49.059876Z","level":"INFO","fields":{"message":"Successfully processed branding images for CLNE"},"target":"ingestor","filename":"bins/ingestor/src/main.rs","line_number":67}
{"timestamp":"2024-11-21T17:22:49.063153Z","level":"INFO","fields":{"message":"Successfully inserted company details for CLNE"},"target":"ingestor","filename":"bins/ingestor/src/main.rs","line_number":84}
{"timestamp":"2024-11-21T17:22:49.063180Z","level":"INFO","fields":{"message":"Stock: CCEC - Capital Clean Energy Carriers Corp. Common Share (8.43%)"},"target":"ingestor","filename":"bins/ingestor/src/main.rs","line_number":35}

```

### Example API Usage

```json lines
{
    "id": "01934fdd-d6c7-77a3-bfc4-2f45eed64e55",
    "symbol": "CBT",
    "name": "Cabot Corporation",
    "address": {
        "value": "TWO SEAPORT LANE SUITE 1400"
    },
    "city": {
        "value": "BOSTON"
    },
    "state": {
        "value": "MA"
    },
    "zip": {
        "value": "02210"
    },
    "icon_url": null,
    "logo_url": null,
    "cik": {
        "value": "0000016040"
    },
    "description": {
        "value": "Cabot Corp manufactures and sells a variety of chemicals, materials, and chemical-based products. The company organizes itself into two segments based on the product type. The reinforcement materials segment, which generates more revenue than any other segment, sells rubber-grade carbon black products used in hoses and belts in automobiles. The performance chemicals segment sells ink-jet colorants and metal oxides used in the automotive and construction industries."
    },
    "homepage_url": {
        "value": "https://www.cabotcorp.com"
    },
    "list_date": {
        "value": "1968-08-23"
    },
    "market_cap": {
        "value": 5918943331.51
    },
    "phone_number": {
        "value": "(617) 345-0100"
    },
    "primary_exchange_id": {
        "value": "XNYS"
    },
    "primary_exchange_name": {
        "value": "New York Stock Exchange"
    },
    "sic_code": {
        "value": "2890"
    },
    "sic_description": {
        "value": "MISCELLANEOUS CHEMICAL PRODUCTS"
    },
    "total_employees": {
        "value": "4300"
    },
    "weighted_shares_outstanding": {
        "value": "54297251"
    }
}
```
