use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use std::time::Duration;

static TIMEAGO_FORMATTER: LazyLock<timeago::Formatter> = LazyLock::new(timeago::Formatter::new);
// Field | Description
// ------|------------
// **id** | The item's unique id.
// deleted | `true` if the item is deleted.
// type | The type of item. One of "job", "story", "comment", "poll", or "pollopt".
// by | The username of the item's author.
// time | Creation date of the item, in [Unix Time](http://en.wikipedia.org/wiki/Unix_time).
// text | The comment, story or poll text. HTML.
// dead | `true` if the item is dead.
// parent | The comment's parent: either another comment or the relevant story.
// poll | The pollopt's associated poll.
// kids | The ids of the item's comments, in ranked display order.
// url | The URL of the story.
// score | The story's score, or the votes for a pollopt.
// title | The title of the story, poll or job. HTML.
// parts | A list of related pollopts, in display order.
// descendants | In the case of stories or polls, the total comment count.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Job,
    Story,
    Comment,
    Poll,
    #[serde(rename = "pollopt")]
    PollOpt,
}

pub type ItemID = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemID,
    #[serde(rename = "type")]
    pub item_type: Option<ItemType>,
    #[serde(default)]
    pub deleted: bool,
    pub by: Option<String>,
    pub time: Option<u64>,
    pub text: Option<String>,
    #[serde(default)]
    pub dead: bool,
    pub parent: Option<u32>,
    pub poll: Option<u32>,
    #[serde(default)]
    pub kids: Vec<u64>,
    pub url: Option<String>,
    #[serde(default)]
    pub score: i32,
    pub title: Option<String>,
    #[serde(default)]
    pub parts: Vec<u32>,
    #[serde(default)]
    pub descendants: u32,
}

impl Item {
    pub fn display_url(&self) -> String {
        // println!("Rendering {:?}", self);
        match self.item_type {
            Some(ItemType::Story) => {
                // Only show hostname unless it's on the 'allow list' of domains
                let allowed_domains = ["github.com", "news.ycombinator.com"];
                let url = self
                    .url
                    .clone()
                    .unwrap_or(format!("news.ycombinator.com/item?id={}", self.id));
                // remove protocol
                let url = url.split_once("://").unwrap_or(("", &url)).1;
                let domain = url.split('/').nth(0).unwrap_or_default();
                if allowed_domains.contains(&domain) {
                    url.to_string()
                } else {
                    domain.to_string()
                }
            }
            _ => format!("news.ycombinator.com/item?id={}", self.id),
        }
    }
    pub fn display_url_long(&self) -> String {
        self.url
            .clone()
            .unwrap_or(format!("https://news.ycombinator.com/item?id={}", self.id))
    }
    pub fn humantime(&self) -> String {
        let current_epoch = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let t = match self.time {
            Some(time) => Duration::from_secs(time),
            None => return "Unknown".to_string(),
        };

        TIMEAGO_FORMATTER.convert(current_epoch - t)
    }
    pub fn href_url(&self) -> String {
        match &self.url {
            Some(url) => url.clone(),
            None => self.rhxn_url(),
        }
    }
    pub fn rhxn_url(&self) -> String {
        format!("/item/{}", self.id)
    }
    pub fn hn_url(&self) -> String {
        format!("https://news.ycombinator.com/item?id={}", self.id)
    }
}
