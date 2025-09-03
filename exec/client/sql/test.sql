BEGIN;

INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (0, 'Open Store - Be, Love, Work', 'org.openstore.example.android', '0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF', 'openstore.com', 'https://thumbs.dreamstime.com/b/vector-logo-colorful-design-41236752.jpg', 'Introducing ChatGPT for Android: OpenAI’s latest advancements at your fingertips. This official app is free, syncs your history across devices, and brings you the latest from OpenAI, including the new image generator. With ChatGPT in your pocket', 1, 1, 1, TRUE, TRUE, FALSE, FALSE, 4.21, 0, 100000);

-- Object ID 1 (For VList)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (1, 'App Example One', 'org.openstore.example.android.app1', '0xAAAAAAAAAAAAAAAaaaaaaaaaaaaaaaaaaaaa', 'app1.example.com', 'https://play-lh.googleusercontent.com/lmG9HlI0awHie0cyBieWXeNjpyXvHPwDBb8MNOVIyp0P8VEh95AiBHtUZSDVR3HLe3A=w480-h960-rw', 'First app in the vertical list.', 1, 1, 1, TRUE, TRUE, FALSE, FALSE, 4.10, 0, 50000);

-- Object ID 2 (For VList)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (2, 'App Example Two (Music)', 'org.openstore.example.android.app2', '0xBBBBBBBBBBBBBBBbbbbbbbbbbbbbbbbbbbb', 'app2.example.com', 'https://play-lh.googleusercontent.com/Ui_-OW6UJI147ySDX9guWWDiCPSq1vtxoC-xG17BU2FpU0Fi6qkWwuLdpddmT9fqrA=w480-h960-rw', 'Second app, focused on music.', 2, 1, 1, TRUE, FALSE, FALSE, FALSE, 4.55, 199, 25000); -- price in cents? assuming 1.99

-- Object ID 3 (For VList)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (3, 'Business Tool App', 'org.openstore.example.android.app3', '0xCCCCCCCCCCCCCCCCcccccccccccccccccccc', 'app3.example.com', 'https://play-lh.googleusercontent.com/Nz5sdWyh7jn4eTy_GSaRBDgaKhLC1pvYywC6fklDOlPGbopmeFN9NkqgKGjsvJMbKVEI=w480-h960-rw', 'Third app, a business utility.', 3, 1, 1, FALSE, TRUE, FALSE, FALSE, 3.90, 0, 10000);

-- Object ID 5 (For Highlight 1)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (5, 'Featured News App', 'org.openstore.example.android.newsfeat', '0xDDDDDDDDDDDDDDdddddddddddddddddddd', 'newsfeature.com', 'https://play-lh.googleusercontent.com/cpBOIqHqOJNwoPL9aRvlDVjKmNdEzHMMbu4tLXKZgoyQgWO4nwEbPBkM2i_Cy-9_S9g=w480-h960-rw', 'A highlighted application for news readers.', 1, 1, 1, TRUE, TRUE, TRUE, FALSE, 4.80, 0, 500000);

-- Object ID 7 (For Highlight 2)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (7, 'Indie Music Streamer', 'org.openstore.example.android.musicindie', '0xEEEEEEEEEEEEEEEEeeeeeeeeeeeeeeeeeeee', 'indiemusic.io', 'https://play-lh.googleusercontent.com/q0Sk9ckxrsLuBUQtKrP-Qp8GbUIbsXaZ2OOvRHBqV5y7ySNMAmq_Z5L7-F75GjPJquiD=s512-rw', 'Discover new indie artists.', 2, 1, 1, FALSE, FALSE, FALSE, FALSE, 4.30, 0, 75000);

-- Object ID 8 (For Highlight 3)
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (8, 'Startup Business Suite', 'org.openstore.example.android.bizsuite', '0x111111111111111111111111111111111111', 'bizsuite.app', 'https://play-lh.googleusercontent.com/B9YOy-OE7lpzz7qaUQV3LBJ1Ss-DOBsySrjFeNSl0kWYCQHGFO4uc_xrgyiGDYFZ4SM=s512-rw', 'Tools for the modern startup.', 3, 1, 1, TRUE, FALSE, FALSE, FALSE, 4.00, 4999, 5000); -- price in cents? assuming 49.99

-- Object ID 6 (For HList - BestInCategory(News))
INSERT INTO obj
(id, name, package_name, address, website, logo, description, category_id, platform_id, type_id, is_oracle_verified, is_build_verified, is_os_verified, is_hidden, rating, price, downloads)
VALUES
    (6, 'Another Great News App', 'org.openstore.example.android.newsplus', '0x222222222222222222222222222222222222', 'newsplus.org', 'https://play-lh.googleusercontent.com/ynzziGfmYSMe4TjtQ98tn0LSLnb3xdQJKdXM-foaH2Iv0-eBXJWkQEXejQjbIvXG0g=s512-rw', 'Yet another highly rated news application.', 1, 1, 1, TRUE, TRUE, FALSE, FALSE, 4.75, 0, 250000);


INSERT INTO artifact
(id, ref_id, protocol_id, size, version_name, version_code)
VALUES
    (0, '0x0000000000000000000000000000000000000000000000000000000000174018', 0, 57400, '1.0', 1);

INSERT INTO publishing
(id, object_id, track_id, artifact_id, status, is_active)
VALUES
    (0, 1, 1, 0, 1, true);

INSERT INTO build_request
(id, request_type_id, object_id, track_id, owner_version)
VALUES
    (0, 1, 1, 1, 1);

COMMIT;