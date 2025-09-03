#!/bin/bash

protoc \
    --prost_out=codegen/stat/src \
    --proto_path=codegen/stat/protos \
    event.proto

protoc \
    --prost_out=codegen/block/src \
    --proto_path=codegen/block/protos \
    block.proto
