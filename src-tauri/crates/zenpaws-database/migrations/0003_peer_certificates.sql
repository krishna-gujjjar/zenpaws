CREATE TABLE peer_certificates (
    peer_uuid TEXT PRIMARY KEY REFERENCES peers(uuid),
    certificate_der BLOB NOT NULL
);