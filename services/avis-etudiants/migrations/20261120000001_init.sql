-- Schéma initial — compatible MySQL 5.7+ / MariaDB 10.3+
-- Les colonnes TIMESTAMP sont stockées en UTC (la connexion force time_zone = '+00:00').

CREATE TABLE users (
  email        VARCHAR(255) NOT NULL,
  display_name VARCHAR(255) NOT NULL,
  first_login  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_login   TIMESTAMP NULL DEFAULT NULL,
  PRIMARY KEY (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE admins (
  email     VARCHAR(255) NOT NULL,
  added_by  VARCHAR(255) NOT NULL,
  added_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE tickets (
  id            INT UNSIGNED NOT NULL AUTO_INCREMENT,
  author_email  VARCHAR(255) NOT NULL,
  ticket_date   DATE NOT NULL,
  category      VARCHAR(100) NOT NULL,
  teacher       VARCHAR(255) NOT NULL DEFAULT '',
  rating        TINYINT UNSIGNED NOT NULL,          -- 0..5 (contrôlé par l'application)
  comment       TEXT NOT NULL,
  status        VARCHAR(16) NOT NULL DEFAULT 'ouvert', -- ouvert | en_cours | ferme | annule | supprime
  reply         TEXT NULL,
  replied_by    VARCHAR(255) NULL DEFAULT NULL,
  replied_at    TIMESTAMP NULL DEFAULT NULL,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_tickets_author (author_email),
  KEY idx_tickets_status (status),
  KEY idx_tickets_date (ticket_date),
  KEY idx_tickets_category (category)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE ticket_events (
  id           INT UNSIGNED NOT NULL AUTO_INCREMENT,
  ticket_id    INT UNSIGNED NOT NULL,
  actor_email  VARCHAR(255) NOT NULL,
  kind         VARCHAR(32) NOT NULL,                -- created | status | reply | cancelled | deleted
  old_value    TEXT NULL,
  new_value    TEXT NULL,
  created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_events_ticket (ticket_id),
  CONSTRAINT fk_events_ticket FOREIGN KEY (ticket_id) REFERENCES tickets (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
