use std::error::Error;

use chrono::Duration;
use chrono::Local;
use chrono::NaiveDateTime;
use diesel::BelongingToDsl;
use diesel::Connection;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use diesel::SqliteConnection;
use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::methods::SelectDsl;
use diesel_migrations::MigrationHarness;
use tomo::repo::MIGRATIONS;
use tomo::repo::model::*;
use tomo::repo::schema::*;

type Conn<'a> = &'a mut SqliteConnection;
type DbResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[test]
fn test_insert_project() {
    use projects::*;
    let conn = &mut conn();
    let (r1, r2) = insert_projects(conn).unwrap();

    let res = table.load::<Project>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
}

#[test]
fn test_insert_tags() {
    use tags::*;
    let conn = &mut conn();
    let (r1, r2) = insert_tags(conn).unwrap();

    let res = table.load::<Tag>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
}

#[test]
fn test_insert_tasks() {
    use tasks::*;
    let conn = &mut conn();
    let (r1, r2, r3, r4) = insert_tasks(conn, 1, 2).unwrap();

    let res = table.load::<Task>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
    assert_eq!(r3, res[2]);
    assert_eq!(r4, res[3]);
}

#[test]
fn test_insert_sessions() {
    use sessions::*;
    let conn = &mut conn();
    let (r1, r2) = insert_sessions(conn, 1, 2).unwrap();

    let res = table.load::<Session>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
}

#[test]
fn test_insert_task_tags() {
    use task_tags::*;
    let conn = &mut conn();
    let (r1, r2, r3) = insert_task_tags(conn, 1, 2, 3, 4).unwrap();

    let res = table.load::<TaskTag>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
    assert_eq!(r3, res[2]);
}

#[test]
fn test_insert_project_default_tags() {
    use project_default_tags::*;
    let conn = &mut conn();
    let (r1, r2) = insert_project_tags(conn, 1, 2, 3, 4).unwrap();

    let res = table.load::<ProjectDefaultTag>(conn).unwrap();

    assert_eq!(r1, res[0]);
    assert_eq!(r2, res[1]);
}

#[test]
fn test_task_many_sessions() {
    let conn = &mut conn();
    let (task, _, _, _) = insert_tasks(conn, 1, 2).unwrap();
    let (ses1, ses2) = insert_sessions(conn, task.id, task.id).unwrap();

    let res = Session::belonging_to(&task)
        .select(Session::as_select())
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(ses1, res[0]);
    assert_eq!(ses2, res[1]);
}

#[test]
fn test_task_many_tasks() {
    let conn = &mut conn();
    let parent = new_task(conn, "Parent", None, None).unwrap();
    let subtask1 = new_task(conn, "Subtask1", None, Some(parent.id)).unwrap();
    let subtask2 = new_task(conn, "Subtask2", None, Some(parent.id)).unwrap();
    let subtask3 = new_task(conn, "Subtask3", None, Some(parent.id)).unwrap();

    let res = Task::belonging_to(&parent)
        .select(Task::as_select())
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 3);
    assert_eq!(subtask1, res[0]);
    assert_eq!(subtask2, res[1]);
    assert_eq!(subtask3, res[2]);
}

#[test]
fn test_project_many_tasks() {
    let conn = &mut conn();
    let proj = new_project(conn, "Project").unwrap();
    let task1 = new_task(conn, "Task1", Some(proj.id), None).unwrap();
    let task2 = new_task(conn, "Task2", Some(proj.id), None).unwrap();
    let task3 = new_task(conn, "Task3", Some(proj.id), None).unwrap();

    let res = Task::belonging_to(&proj)
        .select(Task::as_select())
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 3);
    assert_eq!(task1, res[0]);
    assert_eq!(task2, res[1]);
    assert_eq!(task3, res[2]);
}

#[test]
fn test_task_many_tags() {
    let conn = &mut conn();
    let software = new_tag(conn, "software").unwrap();
    let academic = new_tag(conn, "academic").unwrap();
    let task = new_task(conn, "CS101 task", None, None).unwrap();
    let _ = new_task_tag(conn, task.id, software.id).unwrap();
    let _ = new_task_tag(conn, task.id, academic.id).unwrap();

    let joins: Vec<TaskTag> = TaskTag::belonging_to(&task)
        .select(TaskTag::as_select())
        .load(conn)
        .unwrap();
    let tag_ids: Vec<i32> = joins.iter().map(|j| j.tag_id).collect();
    let res: Vec<Tag> = tags::table
        .filter(tags::id.eq_any(tag_ids))
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(software, res[0]);
    assert_eq!(academic, res[1]);
}

#[test]
fn test_tag_many_tasks() {
    let conn = &mut conn();
    let software = new_tag(conn, "software").unwrap();
    let task1 = new_task(conn, "Add db feature", None, None).unwrap();
    let task2 = new_task(conn, "Fix bug", None, None).unwrap();
    let _ = new_task_tag(conn, task1.id, software.id).unwrap();
    let _ = new_task_tag(conn, task2.id, software.id).unwrap();

    let joins: Vec<TaskTag> = TaskTag::belonging_to(&software)
        .select(TaskTag::as_select())
        .load(conn)
        .unwrap();
    let task_ids: Vec<i32> = joins.iter().map(|j| j.task_id).collect();
    let res: Vec<Task> = tasks::table
        .filter(tasks::id.eq_any(task_ids))
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(task1, res[0]);
    assert_eq!(task2, res[1]);
}

#[test]
fn test_project_many_default_tags() {
    let conn = &mut conn();
    let tomo = new_project(conn, "tomo").unwrap();
    let software = new_tag(conn, "software").unwrap();
    let rust = new_tag(conn, "rust").unwrap();
    let _ = set_project_tag(conn, tomo.id, software.id).unwrap();
    let _ = set_project_tag(conn, tomo.id, rust.id).unwrap();

    let joins: Vec<ProjectDefaultTag> = ProjectDefaultTag::belonging_to(&tomo)
        .select(ProjectDefaultTag::as_select())
        .load(conn)
        .unwrap();
    let tag_ids: Vec<i32> = joins.iter().map(|j| j.tag_id).collect();
    let res: Vec<Tag> = tags::table
        .filter(tags::id.eq_any(tag_ids))
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(software, res[0]);
    assert_eq!(rust, res[1]);
}

#[test]
fn test_tag_many_default_projects() {
    let conn = &mut conn();
    let tomo = new_project(conn, "tomo").unwrap();
    let college = new_project(conn, "college").unwrap();
    let academic = new_tag(conn, "academic").unwrap();
    let _ = set_project_tag(conn, tomo.id, academic.id).unwrap();
    let _ = set_project_tag(conn, college.id, academic.id).unwrap();

    let joins: Vec<ProjectDefaultTag> = ProjectDefaultTag::belonging_to(&academic)
        .select(ProjectDefaultTag::as_select())
        .load(conn)
        .unwrap();
    let project_ids: Vec<i32> = joins.iter().map(|j| j.project_id).collect();
    let res: Vec<Project> = projects::table
        .filter(projects::id.eq_any(project_ids))
        .load(conn)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(tomo, res[0]);
    assert_eq!(college, res[1]);
}

fn conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:")
        .unwrap_or_else(|e| panic!("Failed to connect, error: {e}"));
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    conn
}

fn insert_projects(conn: Conn) -> DbResult<(Project, Project)> {
    let tomo = new_project(conn, "tomo")?;
    let college = new_project(conn, "college")?;
    Ok((tomo, college))
}

fn insert_tags(conn: Conn) -> DbResult<(Tag, Tag)> {
    let software = new_tag(conn, "software")?;
    let academic = new_tag(conn, "academic")?;
    Ok((software, academic))
}

fn insert_tasks(conn: Conn, college_id: i32, tomo_id: i32) -> DbResult<(Task, Task, Task, Task)> {
    let ass = new_task(conn, "CS101 task", Some(college_id), None)?;
    let ass_disc = new_task(conn, "Discuss with peer", Some(college_id), Some(ass.id))?;
    let laundry = new_task(conn, "Laundry", None, None)?;
    let featdb = new_task(conn, "Add db feature", Some(tomo_id), None)?;
    Ok((ass, ass_disc, laundry, featdb))
}

fn insert_sessions(conn: Conn, ass_id: i32, laundry_id: i32) -> DbResult<(Session, Session)> {
    let now = Local::now().naive_local();
    let ass_ses = new_session(
        conn,
        ass_id,
        now - Duration::hours(1),
        now - Duration::minutes(30),
        Some(now - Duration::minutes(15)),
        PomodoroState::Focus,
        false,
    )?;
    let laundry_ses = new_session(
        conn,
        laundry_id,
        now - Duration::minutes(10),
        now,
        None,
        PomodoroState::Focus,
        false,
    )?;

    Ok((ass_ses, laundry_ses))
}

fn insert_task_tags(
    conn: Conn,
    ass_id: i32,
    featdb_id: i32,
    academic_id: i32,
    software_id: i32,
) -> DbResult<(TaskTag, TaskTag, TaskTag)> {
    let ass_academic = new_task_tag(conn, ass_id, academic_id)?;
    let ass_software = new_task_tag(conn, ass_id, software_id)?;
    let featdb_software = new_task_tag(conn, featdb_id, software_id)?;
    Ok((ass_academic, ass_software, featdb_software))
}

fn insert_project_tags(
    conn: Conn,
    tomo_id: i32,
    college_id: i32,
    software_id: i32,
    academic_id: i32,
) -> DbResult<(ProjectDefaultTag, ProjectDefaultTag)> {
    let tomo_software = set_project_tag(conn, tomo_id, software_id)?;
    let college_academic = set_project_tag(conn, college_id, academic_id)?;
    Ok((tomo_software, college_academic))
}

fn new_project(conn: Conn, name: &str) -> DbResult<Project> {
    let project = diesel::insert_into(projects::table)
        .values(projects::name.eq(name))
        .returning(Project::as_returning())
        .get_result(conn)?;
    Ok(project)
}

fn new_tag(conn: Conn, name: &str) -> DbResult<Tag> {
    let tag = diesel::insert_into(tags::table)
        .values(tags::name.eq(name))
        .returning(Tag::as_returning())
        .get_result(conn)?;
    Ok(tag)
}

fn new_task(
    conn: Conn,
    name: &str,
    project_id: Option<i32>,
    parent_id: Option<i32>,
) -> DbResult<Task> {
    let task = diesel::insert_into(tasks::table)
        .values((
            tasks::name.eq(name),
            tasks::project_id.eq(project_id),
            tasks::parent_id.eq(parent_id),
        ))
        .returning(Task::as_returning())
        .get_result(conn)?;

    Ok(task)
}

fn new_session(
    conn: Conn,
    task_id: i32,
    start: NaiveDateTime,
    update: NaiveDateTime,
    end: Option<NaiveDateTime>,
    pomo_state: PomodoroState,
    paused: bool,
) -> DbResult<Session> {
    let session = diesel::insert_into(sessions::table)
        .values((
            sessions::task_id.eq(task_id),
            sessions::start_at.eq(start),
            sessions::updated_at.eq(update),
            sessions::end_at.eq(end),
            sessions::pomodoro_state.eq(pomo_state),
            sessions::paused.eq(paused),
        ))
        .returning(Session::as_returning())
        .get_result(conn)?;
    Ok(session)
}

fn set_project_tag(conn: Conn, project_id: i32, tag_id: i32) -> DbResult<ProjectDefaultTag> {
    let ret = diesel::insert_into(project_default_tags::table)
        .values((
            project_default_tags::project_id.eq(project_id),
            project_default_tags::tag_id.eq(tag_id),
        ))
        .returning(ProjectDefaultTag::as_returning())
        .get_result(conn)?;
    Ok(ret)
}

fn new_task_tag(conn: Conn, task_id: i32, tag_id: i32) -> DbResult<TaskTag> {
    let ret = diesel::insert_into(task_tags::table)
        .values((task_tags::task_id.eq(task_id), task_tags::tag_id.eq(tag_id)))
        .returning(TaskTag::as_returning())
        .get_result(conn)?;
    Ok(ret)
}
