use ansi_term::Color::{Purple, Red, White, RGB};
use ansi_term::Style;
use chrono::{DateTime, Local, Utc};
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use kube::{
    api::{Api, ResourceExt},
    runtime::{watcher, WatchStreamExt},
    Client,
};
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<(), watcher::Error> {
    let client = Client::try_default().await.unwrap();
    let red_bold_blink: Style = Red.bold().blink();
    let white_bold: Style = White.bold();
    let purple_normal = Purple.normal();
    let focus = RGB(70, 130, 180).bold();

    let seen_events = Arc::new(Mutex::new(HashMap::<u64, bool>::new()));

    let events: Api<Event> = Api::all(client);
    let w = watcher(events, watcher::Config::default())
        .applied_objects()
        .try_for_each(move |e| {
            // Outer scope. We clone the Arc's to move into the async block
            let seen_events_clone = seen_events.clone();

            async move {
                let e_clone = e.clone();
                let event_ns = e_clone.namespace().unwrap();
                let time_now: Time = Time(Utc::now());
                let event_timestamp_utc: &DateTime<Utc> =
                    &e_clone.first_timestamp.unwrap_or(time_now).0;
                let event_timestamp_local: DateTime<Local> = DateTime::from(*event_timestamp_utc);
                let event_source_component = e_clone
                    .source
                    .unwrap_or_default()
                    .component
                    .unwrap_or_default();
                let event_msg = e_clone.message.unwrap_or_default();
                let event_type = e_clone.type_.unwrap();
                let event_hash: u64 = fnv_hash(
                    event_timestamp_local,
                    event_source_component.clone(),
                    event_ns.clone(),
                    event_type.clone(),
                    event_msg.clone(),
                );

                let mut seen_events = seen_events_clone.lock().unwrap();
                if seen_events.contains_key(&event_hash)
                    && seen_events.get(&event_hash).unwrap() == &true
                {
                    println!("dup event fnv: {:x}", event_hash);
                    return Ok(());
                }
                seen_events.insert(event_hash, true);

                #[allow(unused_assignments)]
                let mut color_style: Style = Style::new();
                if event_type != "Normal" {
                    color_style = red_bold_blink;
                } else {
                    color_style = white_bold;
                }
                let event_reason = e_clone.reason.unwrap_or_default();
                let event_count = e_clone.count.unwrap_or_default();
                println!(
                    "{} {}@{}: {} ({})({})({}) (fnv: {:x})",
                    purple_normal.paint(event_timestamp_local.to_string()),
                    event_source_component,
                    event_ns,
                    focus.paint(event_msg),
                    color_style.paint(event_type),
                    event_reason,
                    event_count,
                    event_hash,
                );
                Ok(())
            }
        });

    w.await?;

    Ok(())
}

fn fnv_hash(
    event_timestamp_local: DateTime<Local>,
    event_source_component: String,
    event_ns: String,
    event_type: String,
    event_msg: String,
) -> u64 {
    let mut fnv_hasher = fnv::FnvHasher::default();
    let mut hash_key = String::with_capacity(5);

    hash_key.push_str(event_timestamp_local.to_string().as_str());
    hash_key.push_str(&event_source_component);
    hash_key.push_str(&event_ns);
    hash_key.push_str(&event_type);
    hash_key.push_str(event_msg.as_str());

    fnv_hasher.write(hash_key.as_bytes());
    fnv_hasher.finish()
}
