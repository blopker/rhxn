use futures::{StreamExt, stream::FuturesUnordered};
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::{db, types};

const BASE_URL: &str = "https://hacker-news.firebaseio.com";
const CONCURRENT_REQUESTS: usize = 100;
static TOP_STORIES: LazyLock<String> = LazyLock::new(|| format!("{}/v0/topstories.json", BASE_URL));
static RCLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn run() -> Result<(), reqwest::Error> {
    tracing::info!("Getting items");
    let resp = RCLIENT.get(TOP_STORIES.as_str()).send().await?;
    let mut ids: Vec<types::ItemID> = resp.json().await?;
    ids.truncate(30);
    db::DB.set_top(ids.clone());
    fetch_items(ids).await;
    tracing::info!("Done");
    Ok(())
}

async fn fetch_items(initial_ids: Vec<types::ItemID>) {
    let mut pending: Vec<types::ItemID> = initial_ids;
    let mut seen: HashSet<types::ItemID> = HashSet::new();
    let mut in_flight = FuturesUnordered::new();

    loop {
        // Fill up to CONCURRENT_REQUESTS, skipping already-seen IDs
        while in_flight.len() < CONCURRENT_REQUESTS && !pending.is_empty() {
            let id = pending.pop().unwrap();
            if seen.insert(id) {
                in_flight.push(get_item(id));
            }
        }

        if in_flight.is_empty() {
            break;
        }

        // Wait for next result
        if let Some(result) = in_flight.next().await {
            match result {
                Ok(item) => {
                    pending.extend(item.kids.iter().copied());
                    db::DB.items.insert(item.id, item);
                }
                Err(e) => tracing::error!("Failed to fetch item: {}", e),
            }
        }
    }
}

async fn get_item(id: types::ItemID) -> Result<types::Item, reqwest::Error> {
    let url = format!("{}/v0/item/{}.json", BASE_URL, id);
    let response = RCLIENT.get(&url).send().await?;
    let item: types::Item = response.json().await?;
    Ok(item)
}

/// Fetch a single item and all its children, storing in DB
pub async fn fetch_item_tree(id: types::ItemID) -> Option<types::Item> {
    // Check cache first
    if let Some(item) = db::DB.items.get(&id) {
        return Some(item);
    }

    // Fetch the item and its children
    fetch_items(vec![id]).await;

    // Return from cache
    db::DB.items.get(&id)
}
