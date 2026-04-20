mod exporter;
mod firebase;
mod trip;
mod utils;

use chrono::NaiveDate;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::view;
use leptos::*;
use leptos_meta::*;
use std::collections::BTreeMap;
use trip::Trip;
use wasm_bindgen_futures::spawn_local;

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let trips = RwSignal::new(vec![Trip::new(
        NaiveDate::from_ymd_opt(2026, 2, 6).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 8).unwrap(),
        String::from("example trip!"),
    )]);

    let synced_with_google = RwSignal::new(false);

    let add_trip = move |trip: Trip| -> Result<usize, String> {
        // Check if the trip overlaps with existing trips
        if let Some(t) = trips.get().iter().find(|t| trip.overlap(t)) {
            return Err(format!(
                "Trip overlaps with the trip: {} → {}",
                t.get_depart(),
                t.get_return()
            ));
        }

        // Get the index of the first trip that is after the new trip
        let index = trips
            .get()
            .iter()
            .position(|t| t.get_depart() > trip.get_depart())
            .unwrap_or(trips.get().len());

        // Add in order of departure
        trips.update(|v| v.insert(index, trip));
        // firebase::save_trips(&trips.get());
        if synced_with_google.get() {
            log!("Syncing with Google");
        }

        Ok(index)
    };

    view! {
        <PageShell>
            <Header />
            <Content>
                <Panel title="Trips">
                    <TripForm on_add=add_trip />

                    <div class="space-y-3">
                        <TripList
                            trips=trips.read_only()
                            on_remove=move |index| {
                                trips
                                    .update(|v| {
                                        v.remove(index);
                                    })
                            }
                        />
                    </div>

                    <div class="flex flex-col sm:flex-row gap-3 mt-4">
                        <button
                            disabled=synced_with_google.get()
                            type="button"
                            class="flex-1 rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50"
                            on:click=move |_| exporter::import_file_picker(move |result| {
                                match result {
                                    Ok(imported) => trips.set(imported),
                                    Err(e) => log!("Import error: {}", e),
                                }
                            })
                        >
                            "Sync with Google"
                        </button>

                        <button
                            type="button"
                            class="flex-1 rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50"
                            on:click=move |_| exporter::import_file_picker(move |result| {
                                match result {
                                    Ok(imported) => trips.set(imported),
                                    Err(e) => log!("Import error: {}", e),
                                }
                            })
                        >
                            "Import CSV"
                        </button>

                        <button
                            type="button"
                            class="flex-1 rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50"
                            on:click=move |_| exporter::export_download(&trips.get())
                        >
                            "Export CSV"
                        </button>

                    </div>
                </Panel>

                <Panel title="Results">
                    <Results trips=trips.read_only() limit=180 />
                </Panel>
            </Content>
            <Footer />
            <button on:click=move |_| {
                log!("Signing in with Google");
                firebase::start_google_redirect();
            }>
                "Sign in with Google"
            </button>
        </PageShell>
    }
}

#[component]
fn PageShell(children: Children) -> impl IntoView {
    view! { <div class="max-w-6xl mx-auto p-6">{children()}</div> }
}

#[component]
fn Content(children: Children) -> impl IntoView {
    view! { <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">{children()}</div> }
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
fn TripForm(on_add: impl Fn(Trip) -> Result<usize, String> + 'static) -> impl IntoView {
    let (error, set_error) = signal::<Option<String>>(None);
    let depart_date = RwSignal::new("2026-03-01".to_string());
    let return_date = RwSignal::new("2026-03-10".to_string());
    let description = RwSignal::new(String::new());

    // Validate that the trip is valid (depart < return) and that it doesn't overlap with existing trips
    let create_trip = move || {
        let d = parse_date(&depart_date.get());
        let r = parse_date(&return_date.get());
        let description = description.get();
        if let (Some(depart), Some(ret)) = (d, r) {
            if depart >= ret {
                return Err("Depart date must be before return date");
            }
            return Ok(Trip::new(depart, ret, description));
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
        <div class="flex flex-col gap-3 mb-4">
            <div class="flex flex-col gap-3 sm:flex-row sm:gap-3 sm:items-end">
                <div class="flex-1 min-w-0">
                    <label class="block text-sm text-slate-600 mb-1">"Depart"</label>
                    <input
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
                        prop:value=move || depart_date.get()
                        on:input=move |ev| depart_date.set(event_target_value(&ev))
                        type="date"
                    />
                </div>

                <div class="flex-1 min-w-0">
                    <label class="block text-sm text-slate-600 mb-1">"Return"</label>
                    <input
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
                        prop:value=move || return_date.get()
                        on:input=move |ev| return_date.set(event_target_value(&ev))
                        type="date"
                    />
                </div>
            </div>
            <div class="w-full min-w-0">
                <label class="block text-sm text-slate-600 mb-1">Description (optional)</label>
                <textarea
                    class="w-full min-h-[4.5rem] rounded-lg border border-slate-300 px-3 py-2 text-base resize-y"
                    prop:value=move || description.get()
                    on:input=move |ev| description.set(event_target_value(&ev))
                    placeholder="e.g. Trip to Argentina"
                    rows=1
                />
            </div>
            <button
                class="w-full sm:w-auto rounded-lg bg-slate-900 text-white px-4 py-2.5 hover:bg-slate-800 text-base touch-manipulation"
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
            key=|t| t.get_depart().to_string()
            children=move |index, trip: Trip| {
                let on_remove = on_remove.clone();
                let on_remove = move || {
                    on_remove(index.get());
                };
                view! { <TripRow trip=trip on_remove=on_remove /> }
            }
        />
    }
}

#[component]
fn TripRow(trip: Trip, on_remove: impl Fn() + 'static + Clone) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-200 p-3">
            <div>
                <div class="font-medium">{format!("{} → {}", trip.get_depart(), trip.get_return())}</div>
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
        <div
            class:hidden=move || error.get().is_none()
            class="mb-3 rounded-lg border border-rose-200 bg-rose-50 p-3 text-sm text-rose-800"
        >
            {move || error.get().map(|e| view! { <p class="text-red-500">{e}</p> })}
        </div>
    }
}

#[component]
fn Results(trips: ReadSignal<Vec<Trip>>, limit: usize) -> impl IntoView {
    let days_outside: Memo<(BTreeMap<NaiveDate, usize>, usize, usize)> = Memo::new(move |_| {
        let trips = move || trips.get();
        let days = Trip::calculate_outside_days(&trips());
        let outside_days = days.len();
        let rolling_max = *days.values().max().unwrap_or(&0) as usize;
        (days, outside_days, rolling_max)
    });

    let days_outside_only = Memo::new(move |_| {
        let (days, _, _) = days_outside.get();
        days
    });

    view! {
        { move || {
            let (_, total_outside, rolling_max) = days_outside.get();
            view! {
                <StatsRow
                    total_outside=total_outside
                    rolling_max=rolling_max
                    limit=limit
                />
            }
        }}
        <DaysOutsideTable days_outside=days_outside_only.into() limit=limit />
    }
}

#[component]
fn StatsRow(total_outside: usize, rolling_max: usize, limit: usize) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-4">
            <StatCard label="Total outside days" value=total_outside.to_string() />
            <StatCard label="12-month max" value=rolling_max.to_string() />
            <StatCard label="Limit" value=limit.to_string()/>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-200 p-3">
            <div class="text-sm text-slate-600">{label}</div>
            <div class="text-2xl font-semibold">{value}</div>
        </div>
    }
}

#[component]
fn DaysOutsideTable(
    days_outside: Signal<BTreeMap<NaiveDate, usize>>,
    limit: usize,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-lg border border-slate-200">
            <table class="w-full text-left text-sm">
                <thead class="border-b border-slate-200 bg-slate-50 text-slate-600">
                    <tr>
                        <th class="px-4 py-3 font-medium">"Date"</th>
                        <th class="px-4 py-3 font-medium text-right">"Count"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-200">
                    <For
                        each=move || days_outside.get()
                        key=|d| format!("{}-{}", d.0, d.1)
                        children=move |(day, count)| {
                            let over_limit = count > limit;
                            view! {
                                <tr
                                    class:bg-red-50=move || over_limit
                                    class:border-l-4=move || over_limit
                                    class:border-red-400=move || over_limit
                                    class="hover:bg-slate-50"
                                >
                                    <td class="px-4 py-2.5">{day.to_string()}</td>
                                    <td class="px-4 py-2.5 text-right tabular-nums">{count.to_string()}</td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="mb-6 flex items-start justify-between">
            <div>
                <h1 class="text-2xl font-semibold">"Days Outside Calculator"</h1>
                <p class="text-slate-600 mt-1">
                    "Counts full days between depart and return (exclusive)."
                </p>
            </div>

            <a
                href="https://github.com/ramonberrutti/outside-days"
                target="_blank"
                rel="noopener noreferrer"
                class="flex items-center gap-1.5 text-sm text-slate-400 hover:text-slate-700 transition-colors"
                aria-label="GitHub repository"
            >
                "Source Code"
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
            </a>
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

    spawn_local(async {
        firebase::handle_google_redirect().await;
    });

    mount::mount_to_body(App);
}
