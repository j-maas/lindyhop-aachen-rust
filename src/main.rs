#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod store;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use maud::{html, Markup, DOCTYPE};
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use store::{EventWithOccurrences, Overview, Store};

#[get("/")]
fn index(store: Store) -> Markup {
    html! {
        ( DOCTYPE )
        html lang="de" {
            head {
                link href="static/main.css" rel="stylesheet";
            }
            body {
                header {
                    h1 { "Lindy Hop Aachen" }
                }
                main {
                    ol.schedule {
                        @for entry in store.occurrences_by_date() {
                            li { ( render_entry(&entry, &store.locations) ) }
                        }
                    }
                }
            }
        }
    }
}

fn render_entry(
    (date, entries): &(NaiveDate, Vec<(&Occurrence, &Event)>),
    locations: &Locations,
) -> Markup {
    html! {
        div.date { ( format_date(date) ) }
        ol.events {
            @for occurrence_entry in entries {
                li.event { ( render_occurrence(occurrence_entry, locations) ) }
            }
        }
    }
}

fn format_date(date: &NaiveDate) -> String {
    use chrono::Weekday::*;

    let day = match date.weekday() {
        Mon => "Mo",
        Tue => "Di",
        Wed => "Mi",
        Thu => "Do",
        Fri => "Fr",
        Sat => "Sa",
        Sun => "So",
    };
    let format = format!("{}, %d.%m.", day);

    date.format(&format).to_string()
}

fn render_occurrence((occurrence, event): &(&Occurrence, &Event), locations: &Locations) -> Markup {
    html! {
        @let entry =  html_from_occurrence(occurrence, event, locations);
        h2.title { ( entry.title )}
        div.content {
            ul.quick-info {
                li.time { ( entry.time ) }
                li.location { ( entry.location ) }
            }
            div.description {
                div.teaser { ( entry.teaser ) }
            }
        }
    }
}

struct OccurrenceHtml {
    time: Markup,
    location: Markup,
    title: Markup,
    teaser: Markup,
}

fn html_from_occurrence(
    occurrence: &Occurrence,
    event: &Event,
    locations: &Locations,
) -> OccurrenceHtml {
    let maybe_location = locations
        .validate(occurrence.location_id)
        .map(|id| locations.get(&id));

    OccurrenceHtml {
        time: html! {(occurrence.start.format("%H:%M")) small { " bis " (occurrence.end().format("%H:%M"))} },
        location: html! { @match maybe_location {
                Some(location) => (location.name),
                None => "Steht noch nicht fest."
                }
        },
        title: html! { (event.title) },
        teaser: html! { (event.teaser) },
    }
}

#[get("/admin")]
fn admin_route() -> Option<NamedFile> {
    admin()
}

// We also want to serve the file when subroutes are called, e. g. `/admin/event/42`.
// Removing this would break reloading the admin on subroutes.
#[get("/admin/<path..>")]
#[allow(unused_variables)]
fn admin_subroute(path: PathBuf) -> Option<NamedFile> {
    admin()
}

fn admin() -> Option<NamedFile> {
    NamedFile::open(Path::new("admin/dist/index.html")).ok()
}

#[get("/")]
fn api_overview(store: Store) -> Json<Overview> {
    Json(store.read_all())
}

fn main() {
    use store::routes::*;

    rocket::ignite()
        .attach(Store::fairing())
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/", routes![index, admin_route, admin_subroute])
        .mount("/api", routes![api_overview])
        .mount("/api/events/", event_with_occurrences::routes())
        .mount("/api/locations/", location::routes())
        .launch();
}
