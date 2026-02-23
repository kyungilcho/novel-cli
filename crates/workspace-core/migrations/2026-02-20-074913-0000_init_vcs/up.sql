CREATE TABLE nodes (
  id TEXT PRIMARY KEY NOT NULL,
  message TEXT NOT NULL,
  created_at_unix_ms BIGINT NOT NULL
);

CREATE TABLE node_parents (
  node_id TEXT NOT NULL,
  parent_id TEXT NOT NULL,
  ord INTEGER NOT NULL,
  PRIMARY KEY (node_id, ord),
  FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE RESTRICT
);

CREATE INDEX idx_node_parents_parent_id ON node_parents(parent_id);

CREATE TABLE head (
  singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
  node_id TEXT NULL,
  FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE SET NULL
);

INSERT INTO head (singleton, node_id) VALUES (1, NULL);