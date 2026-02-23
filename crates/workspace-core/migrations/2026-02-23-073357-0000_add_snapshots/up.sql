CREATE TABLE blobs (
  id TEXT PRIMARY KEY NOT NULL,
  content BLOB NOT NULL
);

CREATE TABLE node_files (
  node_id TEXT NOT NULL,
  path TEXT NOT NULL,
  blob_id TEXT NOT NULL,
  PRIMARY KEY (node_id, path),
  FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (blob_id) REFERENCES blobs(id) ON DELETE RESTRICT
);

CREATE INDEX idx_node_files_node_id ON node_files(node_id);
CREATE INDEX idx_node_files_blob_id ON node_files(blob_id);