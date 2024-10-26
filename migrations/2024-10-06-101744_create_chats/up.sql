-- migrations/{timestamp}_create_chats/up.sql
CREATE TABLE chats (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       user1_id INTEGER NOT NULL,
                       user2_id INTEGER NOT NULL,
                       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                       FOREIGN KEY (user1_id) REFERENCES users(id) ON DELETE CASCADE,
                       FOREIGN KEY (user2_id) REFERENCES users(id) ON DELETE CASCADE
);

