
-- migrations/{timestamp}_create_messages/up.sql
CREATE TABLE messages (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          chat_id INTEGER NOT NULL,
                          user_id INTEGER NOT NULL,
                          content TEXT,
                          timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                          FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
                          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
