CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR NOT NULL,
    teaser VARCHAR NOT NULL,
    description VARCHAR NOT NULL
);
CREATE TABLE locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    address VARCHAR NOT NULL
);
CREATE TABLE occurrences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start TIMESTAMP NOT NULL,
    duration INTEGER NOT NULL,
    event_id INTEGER NOT NULL,
    location_id INTEGER NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (location_id) REFERENCES locations(id)
);
