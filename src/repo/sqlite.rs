#![allow(unused)]
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

use crate::repo::ProjectRepository;
use crate::repo::Repositories;
use crate::repo::SessionRepository;
use crate::repo::TagRepository;
use crate::repo::TaskRepository;
use crate::repo::error::RepoError;

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub struct SqliteRepositories {
    db: SqliteDatabase,
    project: SqliteProjectRepository,
    tag: SqliteTagRepository,
    task: SqliteTaskRepository,
    session: SqliteSessionRepository,
}

impl SqliteRepositories {
    pub fn new(db: SqliteDatabase) -> Self {
        let pool = db.pool();
        Self {
            db,
            project: SqliteProjectRepository::new(pool.clone()),
            tag: SqliteTagRepository::new(pool.clone()),
            task: SqliteTaskRepository::new(pool.clone()),
            session: SqliteSessionRepository::new(pool.clone()),
        }
    }
}

impl Repositories for SqliteRepositories {
    fn project(&self) -> impl ProjectRepository {
        self.project.clone()
    }

    fn tag(&self) -> impl TagRepository {
        self.tag.clone()
    }

    fn task(&self) -> impl TaskRepository {
        self.task.clone()
    }

    fn session(&self) -> impl SessionRepository {
        self.session.clone()
    }
}

#[derive(Clone)]
pub struct SqliteProjectRepository {
    pool: SqlitePool,
}

impl SqliteProjectRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ProjectRepository for SqliteProjectRepository {}

#[derive(Clone)]
pub struct SqliteTagRepository {
    pool: SqlitePool,
}

impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TagRepository for SqliteTagRepository {}

#[derive(Clone)]
pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TaskRepository for SqliteTaskRepository {}

#[derive(Clone)]
pub struct SqliteSessionRepository {
    pool: SqlitePool,
}

impl SessionRepository for SqliteSessionRepository {}

impl SqliteSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub fn new(url: impl ToString) -> Result<Self, RepoError> {
        let manager = ConnectionManager::new(url.to_string());
        let pool = Pool::builder()
            .build(manager)
            .map_err(|e| RepoError::Connection(e.to_string()))?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}
