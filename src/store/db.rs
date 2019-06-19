use std::hash::{Hash, Hasher};

use diesel::{self, prelude::*};
use rocket::Rocket;
use uuid::Uuid;

#[database("sqlite_database")]
pub struct Connection(SqliteConnection);

embed_migrations!();

pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = Connection::get_one(&rocket).expect("Database connection failed.");
    let result = match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            println!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    };

    result
}

pub mod schema {
    table! {
        events {
            id -> Binary,
            title -> Text,
            teaser -> Text,
            description -> Text,
        }
    }
    table! {
        occurrences {
            id -> Binary,
            event_id -> Binary,
            start -> Timestamp,
            duration -> Integer,
            location_id -> Binary,
        }
    }
    table! {
        locations {
            id -> Binary,
            name -> Text,
            address -> Text,
        }
    }
    table! {
        newsletter {
            id -> Binary,
            title -> Text,
            date -> Timestamp,
            content -> Text
        }
    }
}

use std::io::Write;

use super::*;
use diesel::backend::Backend;
use diesel::deserialize;
use diesel::expression::{bound::Bound, AsExpression};
use diesel::serialize::{self, Output};
use diesel::sql_types::{Binary, HasSqlType};
use diesel::sqlite::Sqlite;
use diesel::types::{FromSql, ToSql};
use schema::*;

// SqlId implementation taken from https://github.com/forte-music/core/blob/fc9cd6217708b0dd6ae684df3a53276804479c59/src/models/id.rs#L67
#[derive(Debug, Deserialize, FromSqlRow, Clone)]
pub struct SqlId(Uuid);

impl From<SqlId> for super::Id {
    fn from(id: SqlId) -> super::Id {
        id.0
    }
}

impl From<super::Id> for SqlId {
    fn from(id: super::Id) -> SqlId {
        SqlId(id)
    }
}

impl<DB: Backend + HasSqlType<Binary>> ToSql<Binary, DB> for SqlId {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        let bytes = self.0.as_bytes();
        <[u8] as ToSql<Binary, DB>>::to_sql(bytes, out)
    }
}

impl FromSql<Binary, Sqlite> for SqlId {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let bytes_vec = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        Ok(SqlId(Uuid::from_slice(&bytes_vec)?))
    }
}

impl AsExpression<Binary> for SqlId {
    type Expression = Bound<Binary, SqlId>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a> AsExpression<Binary> for &'a SqlId {
    type Expression = Bound<Binary, &'a SqlId>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl Hash for SqlId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        let inner: Vec<Uuid> = data.iter().map(|s| s.0).collect();
        Uuid::hash_slice(inner.as_ref(), state);
    }
}

impl PartialEq for SqlId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        self.0.ne(&other.0)
    }
}

impl Eq for SqlId {}

#[derive(Queryable, Insertable, Debug, Identifiable, Clone, PartialEq, AsChangeset)]
#[table_name = "events"]
pub struct SqlEvent {
    pub id: SqlId,
    pub title: String,
    pub teaser: String,
    pub description: String,
}

impl From<SqlEvent> for (super::Id, Event) {
    fn from(event: SqlEvent) -> (super::Id, Event) {
        (
            event.id.0,
            Event {
                title: event.title,
                teaser: event.teaser,
                description: event.description,
            },
        )
    }
}

impl From<Event> for SqlEvent {
    fn from(event: Event) -> SqlEvent {
        let id = Uuid::new_v4();

        SqlEvent {
            id: SqlId(id),
            title: event.title,
            teaser: event.teaser,
            description: event.description,
        }
    }
}

#[derive(
    Queryable, Insertable, Clone, Debug, Identifiable, PartialEq, AsChangeset, Associations,
)]
#[belongs_to(SqlEvent, foreign_key = "event_id")]
#[table_name = "occurrences"]
pub struct SqlOccurrence {
    pub id: SqlId,
    pub event_id: SqlId,
    pub start: NaiveDateTime,
    pub duration: i32,
    pub location_id: SqlId,
}

impl From<SqlOccurrence> for (Id, Occurrence) {
    fn from(occurrence: SqlOccurrence) -> (Id, Occurrence) {
        (
            occurrence.id.0,
            Occurrence {
                start: occurrence.start,
                duration: occurrence.duration as u32,
                location_id: occurrence.location_id.into(),
            },
        )
    }
}

impl From<(Occurrence, SqlId)> for SqlOccurrence {
    fn from((occurrence, event_id): (Occurrence, SqlId)) -> SqlOccurrence {
        let id = Uuid::new_v4();

        SqlOccurrence {
            id: SqlId(id),
            start: occurrence.start,
            duration: occurrence.duration as i32,
            location_id: occurrence.location_id.into(),
            event_id: event_id,
        }
    }
}

#[derive(Queryable, Clone, Insertable, Debug, AsChangeset)]
#[table_name = "locations"]
pub struct SqlLocation {
    pub id: SqlId,
    pub name: String,
    pub address: String,
}
impl From<Location> for SqlLocation {
    fn from(location: Location) -> SqlLocation {
        let id = Uuid::new_v4();

        SqlLocation {
            id: SqlId(id),
            name: location.name,
            address: location.address,
        }
    }
}
impl From<SqlLocation> for (Id, Location) {
    fn from(location: SqlLocation) -> (Id, Location) {
        (
            location.id.0,
            Location {
                name: location.name,
                address: location.address,
            },
        )
    }
}
