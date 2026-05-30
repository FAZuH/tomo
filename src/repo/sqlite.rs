#![allow(unused)]
use std::error::Error;

use chrono::Local;
use chrono::NaiveDateTime;
use diesel::ExpressionMethods as _;
use diesel::NullableExpressionMethods as _;
use diesel::QueryDsl as _;
use diesel::RunQueryDsl as _;
use diesel::SelectableHelper as _;
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::MigrationHarness as _;

use crate::model::Mode;
use crate::model::Session;
use crate::model::Task;
use crate::repo::MIGRATIONS;
use crate::repo::error::RepoError;
use crate::repo::model::*;
use crate::repo::traits::*;

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
type RepoResult<T> = Result<T, RepoError>;

pub struct SqliteRepos {
    db: SqliteDb,
    project: Box<SqliteProjectRepo>,
    tag: Box<SqliteTagRepo>,
    task: Box<SqliteTaskRepo>,
    session: Box<SqliteSessionRepo>,
}

impl SqliteRepos {
    pub fn new(db: SqliteDb) -> Self {
        let pool = db.pool();
        Self {
            db,
            project: Box::new(SqliteProjectRepo::new(pool.clone())),
            tag: Box::new(SqliteTagRepo::new(pool.clone())),
            task: Box::new(SqliteTaskRepo::new(pool.clone())),
            session: Box::new(SqliteSessionRepo::new(pool.clone())),
        }
    }
}

impl Repos for SqliteRepos {
    fn project(&self) -> Box<dyn ProjectRepo> {
        self.project.clone()
    }

    fn tag(&self) -> Box<dyn TagRepo> {
        self.tag.clone()
    }

    fn task(&self) -> Box<dyn TaskRepo> {
        self.task.clone()
    }

    fn session(&self) -> Box<dyn SessionRepo> {
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

impl TaskRepo for SqliteTaskRepo {
    fn add(&self, task_name: String, desc: Option<String>) -> RepoResult<Task> {
        use crate::repo::schema::tasks::*;
        let row = diesel::insert_into(table)
            .values((name.eq(task_name), description.eq(desc)))
            .returning(TaskRow::as_returning())
            .get_result(&mut self.pool.get()?)?;

        Ok(row.into())
    }

    fn find_by_name(&self, task_name: String) -> RepoResult<Task> {
        use crate::repo::schema::tasks::*;
        let row = table
            .filter(name.eq(task_name))
            .limit(1)
            .select(TaskRow::as_select())
            .get_result(&mut self.pool.get()?)?;

        Ok(row.into())
    }

    fn all_tasks(&self) -> RepoResult<Vec<Task>> {
        use crate::repo::schema::tasks::*;
        let rows = table
            .order(name.asc())
            .select(TaskRow::as_select())
            .load(&mut self.pool.get()?)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Clone)]
pub struct SqliteSessionRepo {
    pool: SqlitePool,
}

impl SessionRepo for SqliteSessionRepo {
    fn new_session(&self, task_id: Option<i32>, mode: Mode) -> RepoResult<Session> {
        use crate::repo::schema::sessions as s;
        let now = now();
        let row = diesel::insert_into(s::table)
            .values((
                s::task_id.eq(task_id),
                s::start_at.eq(now),
                s::updated_at.eq(now),
                s::pomodoro_state.eq(PomodoroState::from(mode)),
            ))
            .returning(SessionRow::as_returning())
            .get_result(&mut self.pool.get()?)?;

        Ok(row.into())
    }

    fn update(&self, id: i32) -> RepoResult<usize> {
        use crate::repo::schema::sessions as s;
        let now = now();
        let ret = diesel::update(s::table.filter(s::id.eq(id)))
            .set(s::updated_at.eq(now))
            .execute(&mut self.pool.get()?)?;

        Ok(ret)
    }

    fn end_session(&self, id: i32) -> RepoResult<usize> {
        use crate::repo::schema::sessions as s;
        let now = now();
        let ret = diesel::update(s::table.filter(s::id.eq(id)))
            .set((s::updated_at.eq(now), s::end_at.eq(now)))
            .execute(&mut self.pool.get()?)?;

        Ok(ret)
    }

    fn close_all_sessions(&self) -> RepoResult<()> {
        use crate::repo::schema::sessions as s;
        let now = now();
        let _ = diesel::update(s::table.filter(s::end_at.is_null()))
            .set(s::end_at.eq(s::updated_at.nullable()))
            .execute(&mut self.pool.get()?)?;

        Ok(())
    }
}

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
        let pool: SqlitePool = Pool::builder()
            .build(manager)
            .map_err(|e| RepoError::Connection(e.to_string()))?;

        let mut conn = pool
            .get()
            .map_err(|e| RepoError::Connection(e.to_string()))?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| RepoError::Migration(e.to_string()))?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}

fn now() -> NaiveDateTime {
    Local::now().naive_local()
}
