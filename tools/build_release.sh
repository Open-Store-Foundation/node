#!/bin/bash

cargo build --release \
  --package client --bin daemon-client \
  --package client --bin api-client \
  --package validator --bin validator \
  --package oracle --bin oracle
