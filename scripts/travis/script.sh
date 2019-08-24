#!/bin/bash

source ./scripts/travis/helper.sh

#------------------------------------------------------------------------------#
# check version

# if tag is provided
# -> deploy later
# -> check versions before everything to save runtime
if [[ -n "${TRAVIS_TAG}" ]]; then
    OSMGRAPHING_VERSION="v$(cat ./Cargo.toml | grep 'version' | sed 's_.*version.*"\(.*\)".*_\1_')"
    echo "OSMGRAPHING_VERSION=${OSMGRAPHING_VERSION}"
    echo "TRAVIS_TAG=${TRAVIS_TAG}"
    if [[ "${TRAVIS_TAG}" != "${OSMGRAPHING_VERSION}" ]]; then
        echo -e "${RED}The version in 'Cargo.toml' doesn't match the provided tag '${TRAVIS_TAG}'.${NC}"
        exit 1
    fi
fi

#------------------------------------------------------------------------------#
# cargo build and test

cargo build --verbose --all
cargo test --verbose --all

#------------------------------------------------------------------------------#
# check deployment

if [[ -n "${TRAVIS_TAG}" ]]; then
    cargo publish --dry-run
fi
