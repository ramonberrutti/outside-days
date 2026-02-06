use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_meta::*;
use leptos::*;
use leptos::view;
use serde::{Deserialize, Serialize};
use leptos::logging::log;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Trip {
    id: u32,
    depart: NaiveDate,
    ret: NaiveDate,
}

impl Trip {
    // Difference in days between depart and return, exclusive.
    fn interval(&self) -> usize {
        self.ret.signed_duration_since(self.depart).num_days() as usize - 1
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let next_id = RwSignal::new(1u32);

    let trips = RwSignal::new(vec![Trip {
        id: 0,
        depart: NaiveDate::from_ymd_opt(2026, 2, 6).unwrap(),
        ret: NaiveDate::from_ymd_opt(2026, 2, 8).unwrap(),
    }]);

    // TODO: move to separate component
    let new_depart = RwSignal::new("2026-03-01".to_string());
    let new_return = RwSignal::new("2026-03-10".to_string());

    let add_trip = move |_| {
        let d = parse_date(&new_depart.get());
        let r = parse_date(&new_return.get());
        if let (Some(depart), Some(ret)) = (d, r) {
            // TODO: Validate that the trip is valid (depart < return) and that it doesn't overlap with existing trips
            let id = next_id.get();
            next_id.set(id + 1);
            trips.update(|v| v.push(Trip { id, depart, ret }));

            log!("New trip added!!");
        }
    };

    view! {
        <div class="max-w-6xl mx-auto p-6">
            <Header />
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <section class="bg-white rounded-xl shadow-sm border border-slate-200 p-5">
                    <h2 class="text-lg font-semibold mb-3">"Trips"</h2>

                    <TripForm
                        new_depart=new_depart
                        new_return=new_return
                        on_add=add_trip
                    />

                    <div class="space-y-3">
                        <TripList
                            trips=trips.read_only()
                            on_remove=move |id| trips.update(|v| v.retain(|x| x.id != id))
                        />
                    </div>
                </section>
            </div>
            <Footer />
        </div>
    }
}


#[component]
fn TripForm(
    new_depart: RwSignal<String>,
    new_return: RwSignal<String>,
    on_add: impl Fn(ev::MouseEvent) + 'static + Clone,
) -> impl IntoView {
    view! {
        <div class="flex flex-col sm:flex-row gap-3 items-end mb-4">
            <div class="flex-1">
                <label class="block text-sm text-slate-600 mb-1">"Depart"</label>
                <input
                    class="w-full rounded-lg border border-slate-300 px-3 py-2"
                    prop:value=move || new_depart.get()
                    on:input=move |ev| new_depart.set(event_target_value(&ev))
                    type="date"
                />
            </div>

            <div class="flex-1">
                <label class="block text-sm text-slate-600 mb-1">"Return"</label>
                <input
                    class="w-full rounded-lg border border-slate-300 px-3 py-2"
                    prop:value=move || new_return.get()
                    on:input=move |ev| new_return.set(event_target_value(&ev))
                    type="date"
                />
            </div>

            <button
                class="rounded-lg bg-slate-900 text-white px-4 py-2 hover:bg-slate-800"
                on:click=on_add
            >
                "Add trip"
            </button>
        </div> 
    }
}


#[component]
fn TripList(
    trips: ReadSignal<Vec<Trip>>,
    on_remove: impl Fn(u32) + 'static + Clone + Send,
) -> impl IntoView {
    view! {
        <For
            each=move || trips.get()
            key=|t| t.id
            children=move |trip: Trip| {
                let on_remove = on_remove.clone();
                view! {
                    <TripRow trip=trip on_remove=on_remove />
                }
            }
        />
    }
}

#[component]
fn TripRow(
    trip: Trip,
    on_remove: impl Fn(u32) + 'static + Clone,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-200 p-3">
            <div>
                <div class="font-medium">
                    {format!("{} → {}", trip.depart, trip.ret)}
                </div>
                <div class="text-sm text-slate-600">
                    {format!("Counted days: {}", trip.interval())}
                </div>
            </div>

            <button
                class="text-sm rounded-lg border border-slate-300 px-3 py-1.5 hover:bg-slate-50"
                on:click=move |_| on_remove(trip.id)
            >
                "Remove"
            </button>
        </div>
    }
}


#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="mb-6">
            <h1 class="text-2xl font-semibold">"Days Outside Calculator"</h1>
            <p class="text-slate-600 mt-1">
                "Counts full days between depart and return (exclusive)."
            </p>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="mt-6 text-sm text-slate-500">
            "Rule reminder: depart/return days are not counted; only full days in between are."
        </footer>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount::mount_to_body(App);
}
