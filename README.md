## Description

The **Burgers API** is a RESTful API service built with **Axum** and **SQLx** that provides functionality similiar to [Cocktail DB API](https://www.thecocktaildb.com/api.php?ref=public_apis) but of smaller scale and for burgers.

## Resource management functionality

- Burgers
  - Creation
  - Filtering
  - Get by internal ID
  - Upload images
- Ingredient
  - Creation
  - Filtering
  - Get by internal ID
  - Upload images
- Tags
  - Creation
  - Filtering

## Other features

- Cargo feature **fixture** for downloading limited burger data from [Edamam API](https://www.edamam.com/) for testing purposes.
- NixOS flake for setting up a remote VM for deployment of the app as an OCI container and the required infrastructure: PostgreSQL and MinIO as a self-hosted alternative for S3-like services.
- Self-describing Open API spec set up with the help of **aide** crate.
- Basic e2e testing with **hurl**.

## Running locally

`just dev` command should have you covered for deploying the local infrastructure required for local development.
The command, besides **just** requires **sqlx cli** and **docker** installation.

Then, granted, **cargo-watch** is installed,
the app can be run in the watch mode without setting up the environment any further using `just watch` command.
