#!/bin/bash

cargo build --release \
  --package client --bin daemon-client \
  --package client --bin api-client \
  --package oracle --bin oracle
