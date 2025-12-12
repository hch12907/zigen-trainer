use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use dioxus_logger::tracing;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::scheduler::{
    Card, Rating, ScheduleParamsAdept, ScheduleParamsNovice, Scheduler, ZigenCard,
};
use crate::scheme::{LoadedScheme, SchemeOptions, ZigenConfusableUnpopulated};

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

    pub fn try_initialize_scheme(
        &mut self,
        scheme_id: &str,
        scheme: &LoadedScheme<ZigenConfusableUnpopulated>,
        options: SchemeOptions,
    ) -> Result<(), String> {
        self.current_scheme = scheme_id.to_owned();

        if !self.progresses.contains_key(scheme_id) {
            let mut scheme = scheme.clone().populate_confusables();
            scheme.sort_to_options(&options);
            tracing::debug!("{:?}", &scheme);

            let cards = scheme
                .0
                .into_iter()
                .map(|zigen| ZigenCard {
                    card: Card::New,
                    zigen: zigen.clone(),
                })
                .collect::<Vec<ZigenCard>>();

            if cards.is_empty() {
                return Err(String::from("无练习卡片！"));
            }

            self.progresses.insert(
                scheme_id.to_owned(),
                TrainProgress::new(cards, options.adept),
            );
        }

        Ok(())
    }

    pub fn current_scheme(&self) -> &str {
        &self.current_scheme
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
    scheduler: UsedScheduler,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum UsedScheduler {
    Novice(Scheduler<ScheduleParamsNovice>),
    Adept(Scheduler<ScheduleParamsAdept>),
}

impl TrainProgress {
    pub fn new(pending_cards: Vec<ZigenCard>, adept: bool) -> Self {
        if pending_cards.is_empty() {
            tracing::error!("pending_cards 不能为空！");
            panic!("pending_cards is empty");
        }

        Self {
            start_time: Utc::now(),
            scheduler: if adept {
                UsedScheduler::Adept(Scheduler::new(pending_cards))
            } else {
                UsedScheduler::Novice(Scheduler::new(pending_cards))
            },
        }
    }

    pub fn rate_card(&mut self, rating: Rating) {
        match &mut self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.rate_card(rating),
            UsedScheduler::Adept(scheduler) => scheduler.rate_card(rating),
        }
    }

    pub fn get_card(&self) -> &ZigenCard {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.get_card(),
            UsedScheduler::Adept(scheduler) => scheduler.get_card(),
        }
    }

    pub fn reviewed_cards(&self) -> usize {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.reviewed_cards(),
            UsedScheduler::Adept(scheduler) => scheduler.reviewed_cards(),
        }
    }

    pub fn total_cards(&self) -> usize {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.total_cards(),
            UsedScheduler::Adept(scheduler) => scheduler.total_cards(),
        }
    }
}
