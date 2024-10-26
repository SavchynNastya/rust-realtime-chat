-- migrations/{timestamp}_create_files/up.sql
CREATE TABLE files (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       message_id INTEGER NOT NULL,
                       file_path TEXT NOT NULL,
                       file_type TEXT NOT NULL,
                       file_size INTEGER NOT NULL,
                       timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                       FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);
