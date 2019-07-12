use core::cmp::max;
use std::collections::HashMap;

use chrono::prelude::*;
use maud::{html, Markup, DOCTYPE};
use rocket::Rocket;

use crate::store::{
    Actions, Event, EventWithOccurrences, Id, Location, Occurrence, OccurrenceFilter,
    OccurrenceWithEvent, OccurrenceWithLocation, Store,
};

pub fn mount(rocket: Rocket, prefix: &'static str) -> Rocket {
    rocket.mount(prefix, routes![occurrence_overview, event_overview])
}

#[get("/")]
fn occurrence_overview(store: Store) -> Markup {
    base_html(html! {
        ol.schedule {
            @let locations: HashMap<Id<Location>, Location> = store.all();
            @for occurrences_for_date in store.occurrences_by_date(&OccurrenceFilter::upcoming()) {
                li { ( render_entry(&occurrences_for_date, &locations) ) }
            }
        }
    })
}

#[get("/veranstaltungen")]
fn event_overview(store: Store) -> Markup {
    base_html(html! {
        ol.events {
            @let locations: HashMap<Id<Location>, Location> = store.all();
            @let events = store.all_events_with_occurrences(&OccurrenceFilter::upcoming());
            @for event in events.values() {
                li { ( render_event(event, &locations) ) }
            }
        }
    })
}

fn base_html(main: Markup) -> Markup {
    html! {
        ( DOCTYPE )
        html lang="de" {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";

                link href="static/main.css" rel="stylesheet";
            }
            body {
                header {
                    h1 { "Lindy Hop Aachen" }
                }
                main {
                    ( main )
                }
            }
        }
    }
}

fn render_entry(
    (date, entries): &(NaiveDate, Vec<OccurrenceWithEvent>),
    locations: &HashMap<Id<Location>, Location>,
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
    let day = format_weekday(&date.weekday());
    let format = format!("{}, %d.%m.", day);

    date.format(&format).to_string()
}

fn format_weekday(day: &Weekday) -> &'static str {
    use chrono::Weekday::*;

    match day {
        Mon => "Mo",
        Tue => "Di",
        Wed => "Mi",
        Thu => "Do",
        Fri => "Fr",
        Sat => "Sa",
        Sun => "So",
    }
}

fn render_occurrence(
    entry: &OccurrenceWithEvent,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        @let entry_html =  html_from_occurrence(&entry.occurrence, &entry.event, locations);
        div.quick-info { ( entry_html.quick_info ) }
        h2.title { ( entry_html.title ) }
        div.content {
            div.description {
                div.teaser { ( entry_html.teaser ) }
            }
        }
    }
}

struct OccurrenceHtml {
    title: Markup,
    quick_info: Markup,
    teaser: Markup,
}

fn html_from_occurrence(
    occurrence: &OccurrenceWithLocation,
    event: &Event,
    locations: &HashMap<Id<Location>, Location>,
) -> OccurrenceHtml {
    let maybe_location = locations.get(&occurrence.location_id);
    let location_name = match maybe_location {
        Some(location) => &location.name,
        None => "Steht noch nicht fest.",
    };

    OccurrenceHtml {
        title: html! { ( event.title ) },
        quick_info: html! { ( format!("{} - {}", occurrence.occurrence.start.format("%H:%M"), location_name) ) },
        teaser: html! { ( event.teaser ) },
    }
}

fn render_event(
    event_with_occurrences: &EventWithOccurrences,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        div.event {
            div.overview {
                h2 { ( event_with_occurrences.event.title ) }
                p { (event_with_occurrences.event.teaser ) }
            }
            div.occurrences {
                h3 { "Termine" }
                ol {
                    @let preview_length = 5;
                    @let occurrences = event_with_occurrences.occurrences.iter().take(preview_length);
                    @let remaining = max(0, event_with_occurrences.occurrences.len() - preview_length);
                    @for occurrence in occurrences {
                        li {
                            ( quickinfo_occurrence(occurrence, locations) )
                        }
                    }
                    @if remaining > 0 {
                        span.overflow { ( format!("(+ {} weitere)", remaining) ) }
                    }
                }
            }
        }
    }
}

fn quickinfo_occurrence(
    occurrence_with_location: &OccurrenceWithLocation,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    let date = format_date(&occurrence_with_location.occurrence.start.date());
    let time = occurrence_with_location.occurrence.start.format("%H:%M");
    let maybe_location = locations.get(&occurrence_with_location.location_id);
    let location_name = match maybe_location {
        Some(location) => &location.name,
        None => "Steht noch nicht fest.",
    };
    html! {
        span.quick-info { ( format!("{} {} - {}", date, time, location_name)) }
    }
}
