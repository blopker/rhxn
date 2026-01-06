use std::sync::LazyLock;

use arc_swap::ArcSwap;
use moka::{policy::EvictionPolicy, sync::Cache};

use crate::types;

const CACHE_SIZE: u64 = 20_000;

pub static DB: LazyLock<DB> = LazyLock::new(DB::new);

pub struct DB {
    pub items: Cache<types::ItemID, types::Item>,
    pub top: ArcSwap<Vec<types::ItemID>>,
}

impl DB {
    fn new() -> Self {
        DB {
            items: Cache::builder()
                .eviction_policy(EvictionPolicy::lru())
                .max_capacity(CACHE_SIZE)
                .build(),
            top: ArcSwap::from_pointee(Vec::with_capacity(30)),
        }
    }

    pub fn get_top_stories(&self) -> Vec<types::Item> {
        self.top
            .load()
            .iter()
            .filter_map(|id| self.items.get(id))
            .collect()
    }

    pub fn set_top(&self, ids: Vec<types::ItemID>) {
        self.top.store(ids.into());
    }
}
