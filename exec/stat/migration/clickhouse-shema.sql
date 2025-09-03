CREATE TABLE IF NOT EXISTS default.stat_events (
    -- Common fields derived during processing
    event_time DateTime64(3) DEFAULT now64(3),
    event_name LowCardinality(String),

    -- Common fields from Proto
    object_id Int64,

    -- Fields from ObjectEvent struct
    platform_id Int32,
    obj_type_id Int32,
    category_id Int32,
    artifact_id String,
    artifact_protocol Int32,

    version_code Nullable(Int32),
    version_name Nullable(String),

    to_version_code Nullable(Int32),
    to_version_name Nullable(String)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_time)
ORDER BY (platform_id, obj_type_id, category_id, event_time);