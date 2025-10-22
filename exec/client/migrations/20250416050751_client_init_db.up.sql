CREATE TABLE obj (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    logo VARCHAR(255),
    package_name VARCHAR(255) NOT NULL,
    address VARCHAR(100) NOT NULL,

    description VARCHAR(255),

    category_id INT NOT NULL,
    platform_id INT NOT NULL,
    type_id INT NOT NULL,

    is_os_verified BOOLEAN NOT NULL DEFAULT false,
    is_hidden BOOLEAN NOT NULL DEFAULT true,

    price BIGINT NOT NULL DEFAULT 0,
    downloads BIGINT NOT NULL DEFAULT 0,
    rating REAL NOT NULL DEFAULT 0 CHECK (rating >= 0 AND rating <= 5),

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_object_address ON obj(address);
CREATE INDEX idx_object_name ON obj(name);
CREATE INDEX idx_object_platform_type_id ON obj(platform_id, type_id);
CREATE INDEX idx_object_platform_category_id ON obj(platform_id, category_id);

CREATE TABLE assetlink_sync (
    id BIGSERIAL PRIMARY KEY,
    object_address VARCHAR(100) NOT NULL,
    domain VARCHAR(255) NOT NULL,
    owner_version BIGINT NOT NULL,
    status INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_assetlink_sync ON assetlink_sync(object_address, owner_version);

CREATE TABLE validation_proof (
    id BIGSERIAL PRIMARY KEY,
    object_address VARCHAR(100) NOT NULL,
    owner_version BIGINT NOT NULL,
    status INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_proof_validation ON assetlink_sync(object_address, owner_version);

CREATE TABLE build_request (
    id BIGSERIAL PRIMARY KEY,

    request_type_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL,
    status INT,

    object_address VARCHAR(100) NOT NULL,
    version_code BIGINT NOT NULL,
    owner_version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_build_request ON build_request(object_address);

CREATE TABLE publishing (
    id BIGSERIAL PRIMARY KEY,
    object_address VARCHAR(100) NOT NULL,
    track_id INT NOT NULL,
    version_code BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_publishing_address_id ON publishing(object_address, track_id);

CREATE TABLE artifact (
    id BIGSERIAL PRIMARY KEY,
    ref_id VARCHAR(100) NOT NULL,
    object_address VARCHAR(100) NOT NULL,
    protocol_id INT NOT NULL,
    size BIGINT NOT NULL,
    version_name VARCHAR(50),
    checksum VARCHAR(66) NOT NULL,
    version_code BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_artifact_address_version ON artifact(object_address, version_code);

CREATE TABLE report (
    id BIGSERIAL PRIMARY KEY,
    object_address VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL,
    category_id INT NOT NULL,
    subcategory_id INT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_reports_object_address ON report(object_address);
CREATE INDEX idx_reports_email ON report(email);

CREATE TABLE transactions_batch (
    id BIGSERIAL PRIMARY KEY,
    from_block_number BIGINT NOT NULL,
    to_block_number BIGINT NOT NULL,
    status INT NOT NULL
);





CREATE TABLE category (
    id INT PRIMARY KEY,
    type_id INT NOT NULL,
    name VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE achievement (
    id INT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    value VARCHAR(20),
    object_id BIGINT NOT NULL REFERENCES obj(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE review (
    id BIGSERIAL PRIMARY KEY,
    object_id BIGINT NOT NULL REFERENCES obj(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,
    rating INT NOT NULL CHECK (rating >= 0 AND rating <= 5),
    text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_reviews_object_user_id ON review(object_id, user_id);
CREATE INDEX idx_reviews_user_id ON review(user_id);
