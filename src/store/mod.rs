mod db;

use diesel::prelude::*;
use rocket::fairing::{self, Fairing};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::Rocket;

use db::{Id, Location, NewLocation, UpdateLocation};

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn new_schema_instance() -> Schema {
    Schema::new(Query, Mutation)
}

pub struct Query;

#[juniper::object(Context=Store)]
impl Query {
    fn all_locations(context: &Store) -> Vec<Location> {
        use db::schema::locations::dsl::*;
        locations
            .load(&*context.0)
            .expect("Error loading from database.")
    }
}

pub struct Mutation;

#[juniper::object(Context=Store)]
impl Mutation {
    fn add_location(context: &Store, new_location: NewLocation) -> Id {
        use db::schema::locations::dsl::*;
        context
            .0
            .transaction(|| {
                diesel::insert_into(locations)
                    .values(&new_location)
                    .execute(&*context.0)?;
                locations.select(id).order(id.desc()).first(&*context.0)
            })
            .expect("Error inserting into database.")
    }

    fn update_location(
        context: &Store,
        id_to_update: Id,
        new_location: UpdateLocation,
    ) -> Location {
        use db::schema::locations::dsl::*;
        let item = locations
            .find(id_to_update)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::update(&item)
            .set(new_location)
            .execute(&*context.0)
            .expect("Error updating in database.");
        item
    }

    fn remove_location(context: &Store, id_to_remove: Id) -> Location {
        use db::schema::locations::dsl::*;
        let item = locations
            .find(id_to_remove)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::delete(&item)
            .execute(&*context.0)
            .expect("Error deleting from database.");
        item
    }
}

pub struct Store(db::Connection);

impl juniper::Context for Store {}

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }
}

pub struct StoreFairing;

impl Fairing for StoreFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Events Store Fairing",
            kind: fairing::Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        db::Connection::fairing()
            .on_attach(rocket)
            .and_then(db::initialize)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}
