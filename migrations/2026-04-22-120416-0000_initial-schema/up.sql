-- Your SQL goes here

PRAGMA foreign_keys = ON;

CREATE TABLE projects (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- Name for this project
    name TEXT NOT NULL,
    -- Optional description for this project
    description TEXT
);

CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- Name for this tag
    name TEXT NOT NULL,
    -- Optional description for this tag
    description TEXT
);

CREATE TABLE tasks (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- Name for this task
    name TEXT NOT NULL,
    -- Optional description for this task
    description TEXT,
    -- Optional deadline for this task
    deadline TIMESTAMP,
    -- Optional parent of this subtask
    parent_id INTEGER,
    -- Optional id of the project this task belongs to. A task can only belong to 1 project
    project_id INTEGER,
    FOREIGN KEY(parent_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE sessions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER,
    -- Time this session was started
    start_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Time this session was updated. Used for crash recovery
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Time this session was ended
    end_at TIMESTAMP,
    pomodoro_state TEXT NOT NULL,
    paused BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE project_default_tags (
    project_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE task_tags (
    task_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
