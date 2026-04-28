#![allow(unused)]
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

use crate::repo::ProjectRepo;
use crate::repo::Repos;
use crate::repo::SessionRepo;
use crate::repo::TagRepo;
use crate::repo::TaskRepo;
use crate::repo::error::RepoError;

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub struct SqliteRepos {
    db: SqliteDb,
    project: SqliteProjectRepo,
    tag: SqliteTagRepo,
    task: SqliteTaskRepo,
    session: SqliteSessionRepo,
}

impl SqliteRepos {
    pub fn new(db: SqliteDb) -> Self {
        let pool = db.pool();
        Self {
            db,
            project: SqliteProjectRepo::new(pool.clone()),
            tag: SqliteTagRepo::new(pool.clone()),
            task: SqliteTaskRepo::new(pool.clone()),
            session: SqliteSessionRepo::new(pool.clone()),
        }
    }
}

impl Repos for SqliteRepos {
    fn project(&self) -> impl ProjectRepo {
        self.project.clone()
    }

    fn tag(&self) -> impl TagRepo {
        self.tag.clone()
    }

    fn task(&self) -> impl TaskRepo {
        self.task.clone()
    }

    fn session(&self) -> impl SessionRepo {
        self.session.clone()
    }
}

#[derive(Clone)]
pub struct SqliteProjectRepo {
    pool: SqlitePool,
}

impl SqliteProjectRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ProjectRepo for SqliteProjectRepo {}

#[derive(Clone)]
pub struct SqliteTagRepo {
    pool: SqlitePool,
}

impl SqliteTagRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TagRepo for SqliteTagRepo {}

#[derive(Clone)]
pub struct SqliteTaskRepo {
    pool: SqlitePool,
}

impl SqliteTaskRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TaskRepo for SqliteTaskRepo {}

#[derive(Clone)]
pub struct SqliteSessionRepo {
    pool: SqlitePool,
}

impl SessionRepo for SqliteSessionRepo {}

impl SqliteSessionRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
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
