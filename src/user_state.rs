use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use dioxus_logger::tracing;
use gloo_storage::{LocalStorage, Storage};
use rs_fsrs::{Card, FSRS, Rating};
use serde::{Deserialize, Serialize};

use crate::scheme::{LoadedScheme, SchemeOptions, SchemeZigen};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserState {
    current_scheme: String,
    progresses: BTreeMap<String, TrainProgress>,
}

impl UserState {
    pub fn read_from_local_storage() -> Self {
        let current_scheme = LocalStorage::get::<String>("currentScheme").unwrap_or_default();

        let progresses =
            LocalStorage::get::<BTreeMap<String, TrainProgress>>("progresses").unwrap_or_default();

        Self {
            current_scheme,
            progresses,
        }
    }

    pub fn write_to_local_storage(&self) {
        let _ = LocalStorage::set("currentScheme", &self.current_scheme)
            .inspect_err(|e| tracing::error!("unable to write to localStorage due to {e}"));

        let _ = LocalStorage::set("progresses", &self.progresses)
            .inspect_err(|e| tracing::error!("unable to write to localStorage due to {e}"));
    }

    pub fn try_initialize_scheme(&mut self, scheme_id: &str, scheme: &LoadedScheme, options: SchemeOptions) {
        self.current_scheme = scheme_id.to_owned();

        if !self.progresses.contains_key(scheme_id) {
            let mut scheme = scheme.clone();
            scheme.sort_to_options(options);
            tracing::debug!("{:?}", &scheme);

            let cards = scheme
                .0
                .into_iter()
                .map(|zigen| ZigenCard {
                    fsrs_card: Card::new(),
                    zigen: zigen.clone(),
                })
                .collect::<Vec<ZigenCard>>();

            self.progresses
                .insert(scheme_id.to_owned(), TrainProgress::new(cards));
        }
    }

    pub fn current_progress(&self) -> &TrainProgress {
        &self.progresses[&self.current_scheme]
    }

    pub fn current_progress_mut(&mut self) -> &mut TrainProgress {
        self.progresses.get_mut(&self.current_scheme).unwrap()
    }

    pub fn reset_current_progress(&mut self) {
        if self.progresses.contains_key(&self.current_scheme) {
            self.progresses.remove(&self.current_scheme);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainProgress {
    start_time: DateTime<Utc>,
    cards: BTreeSet<ZigenCard>,
}

impl TrainProgress {
    pub fn new(pending_cards: Vec<ZigenCard>) -> Self {
        if pending_cards.is_empty() {
            tracing::error!("pending_cards 不能为空！");
            panic!("pending_cards is empty");
        }

        let mut cards = BTreeSet::new();
        let now = Utc::now();
        let start_time = now + Duration::microseconds(pending_cards.len() as i64);

        for (i, mut card) in pending_cards.into_iter().enumerate() {
            // 为了防止所有卡片都拥有相同的时间，导致BTreeSet认为它们都是相等的元素，
            // 随便加点时间
            card.fsrs_card.due = now + Duration::microseconds(i as i64);
            cards.insert(card);
        }

        Self { start_time, cards }
    }

    // pub fn add_card(&mut self, mut card: ZigenCard, rating: Rating) {
    //     card.fsrs_card = FSRS::default().next(card.fsrs_card, Utc::now(), rating).card;
    //     tracing::info!("new due date for this card: {:?}", card.fsrs_card.due);

    //     if !self.cards.insert(card) {
    //         tracing::warn!("inserted a card that already exists!")
    //     }
    // }

    pub fn rate_card(&mut self, rating: Rating) {
        let mut card = self.cards.pop_first().unwrap();
        card.fsrs_card = FSRS::default()
            .next(card.fsrs_card, Utc::now(), rating)
            .card;

        // 防止一直重复展现同一张卡
        // if let Some(next_card) = self.cards.first()
        //     && next_card.fsrs_card.due > card.fsrs_card.due
        // {
        //     card.fsrs_card.due = next_card.fsrs_card.due + Duration::microseconds(1);
        // }

        self.cards.insert(card);
    }

    pub fn get_card(&self) -> &ZigenCard {
        self.cards.first().unwrap()
    }

    pub fn pending_cards(&self) -> usize {
        self.cards.iter().filter(|card| card.fsrs_card.due <= self.start_time).count()
    }

    pub fn total_cards(&self) -> usize {
        self.cards.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZigenCard {
    pub zigen: SchemeZigen,
    pub fsrs_card: Card,
}

impl Ord for ZigenCard {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fsrs_card.due.cmp(&other.fsrs_card.due)
    }
}

impl PartialOrd for ZigenCard {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ZigenCard {
    fn eq(&self, other: &Self) -> bool {
        self.fsrs_card.due.eq(&other.fsrs_card.due)
    }
}

impl Eq for ZigenCard {}
