use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::embed_migrations;

pub mod error;
pub mod model;
pub mod schema;
pub mod sqlite;
pub mod traits;

pub use error::RepoError;
pub use traits::ProjectRepository;
pub use traits::Repositories;
pub use traits::SessionRepository;
pub use traits::TagRepository;
pub use traits::TaskRepository;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
