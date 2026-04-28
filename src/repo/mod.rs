use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::embed_migrations;

pub mod error;
pub mod model;
pub mod schema;
pub mod sqlite;
pub mod traits;

pub use error::RepoError;
pub use traits::ProjectRepo;
pub use traits::Repos;
pub use traits::SessionRepo;
pub use traits::TagRepo;
pub use traits::TaskRepo;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
