use chrono::NaiveDate;
use leptos::prelude::*;
use leptos::view;
use leptos::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Trip {
    depart: NaiveDate,
    ret: NaiveDate,
    description: Option<String>,
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

    let trips = RwSignal::new(vec![Trip {
        depart: NaiveDate::from_ymd_opt(2026, 2, 6).unwrap(),
        ret: NaiveDate::from_ymd_opt(2026, 2, 8).unwrap(),
        description: None,
    }]);

    let add_trip = move |trip: Trip| -> Result<usize, &str> {
        // Check if the trip overlaps with existing trips
        if trips
            .get()
            .iter()
            .any(|t| t.depart <= trip.depart && t.ret >= trip.ret)
        {
            return Err("Trip overlaps with existing trip");
        }

        // Get the index of the first trip that is after the new trip
        let index = trips
            .get()
            .iter()
            .position(|t| t.depart > trip.depart)
            .unwrap_or(trips.get().len());

        // Add in order of departure
        trips.update(|v| v.insert(index, Trip { ..trip }));

        Ok(index)
    };

    view! {
        <PageShell>
            <Header />
            <Content>
                <Panel title="Trips">
                    <TripForm
                        on_add=add_trip
                    />

                    <div class="space-y-3">
                        <TripList
                            trips=trips.read_only()
                            on_remove=move |index| trips.update(|v| { v.remove(index); })
                        />
                    </div>
                </Panel>

                <Panel title="Results" />
            </Content>
            <Footer />
        </PageShell>
    }
}

#[component]
fn PageShell(children: Children) -> impl IntoView {
    view! {
        <div class="max-w-6xl mx-auto p-6">
            {children()}
        </div>
    }
}

#[component]
fn Content(children: Children) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {children()}
        </div>
    }
}

#[component]
fn Panel(title: &'static str, #[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <section class="bg-white rounded-xl shadow-sm border border-slate-200 p-5">
            <h2 class="text-lg font-semibold mb-3">{title}</h2>
            {children.map(|c| c())}
        </section>
    }
}

#[component]
fn TripForm(on_add: impl Fn(Trip) -> Result<usize, &'static str> + 'static) -> impl IntoView {
    let (error, set_error) = signal::<Option<String>>(None);
    let new_depart = RwSignal::new("2026-03-01".to_string());
    let new_return = RwSignal::new("2026-03-10".to_string());

    // Validate that the trip is valid (depart < return) and that it doesn't overlap with existing trips
    let create_trip = move || {
        let d = parse_date(&new_depart.get());
        let r = parse_date(&new_return.get());
        if let (Some(depart), Some(ret)) = (d, r) {
            if depart >= ret {
                return Err("Depart date must be before return date");
            }
            return Ok(Trip {
                depart,
                ret,
                description: None,
            });
        }
        Err("Invalid trip")
    };

    let on_submit = move |_ev: ev::MouseEvent| {
        let result = create_trip();
        if let Err(e) = result {
            set_error.set(Some(e.to_string()));
        } else if let Err(e) = on_add(result.unwrap()) {
            set_error.set(Some(e.to_string()));
        } else {
            set_error.set(None);
        }
    };

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
                on:click=on_submit
            >
                "Add trip"
            </button>

        </div>
        <ErrorBox error=error />
    }
}

#[component]
fn TripList(
    trips: ReadSignal<Vec<Trip>>,
    on_remove: impl Fn(usize) + 'static + Clone + Send,
) -> impl IntoView {
    view! {
        <ForEnumerate
            each=move || trips.get()
            key=|t| t.depart.to_string()
            children=move |index, trip: Trip| {
                let on_remove = on_remove.clone();
                let on_remove = move||{
                   on_remove(index.get());
                };
                view! {
                    <TripRow trip=trip on_remove=on_remove />
                }
            }
        />
    }
}

#[component]
fn TripRow(trip: Trip, on_remove: impl Fn() + 'static + Clone) -> impl IntoView {
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
                on:click=move |_| on_remove()
            >
                "Remove"
            </button>
        </div>
    }
}

#[component]
fn ErrorBox(error: ReadSignal<Option<String>>) -> impl IntoView {
    view! {
        <div class:hidden=move || error.get().is_none() class="mb-3 rounded-lg border border-rose-200 bg-rose-50 p-3 text-sm text-rose-800">
            {move || error.get().map(|e| view! { <p class="text-red-500">{e}</p> })}
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
