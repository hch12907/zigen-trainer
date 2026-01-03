use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use dioxus_logger::tracing;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::scheduler::{
    Rating, ScheduleParamsAdept, ScheduleParamsNovice, Scheduler, SchedulerCard, ZigenCard
};
use crate::scheduler_v2::{SchedulerV2, SchedulerV2Card};
use crate::scheme::{LoadedScheme, SchemeOptions, SchemeZigen, ZigenConfusableUnpopulated};

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

            if scheme.0.is_empty() {
                return Err(String::from("无练习卡片！"));
            }

            self.progresses.insert(
                scheme_id.to_owned(),
                TrainProgress::new(scheme.0, options.adept, options.v2_sched),
            );
        }

        Ok(())
    }

    pub fn current_scheme(&self) -> &str {
        &self.current_scheme
    }

    pub fn has_progress(&self, scheme_name: &str) -> bool {
        self.progresses.contains_key(scheme_name)
    }

    pub fn reset_progress(&mut self, scheme_name: &str) {
        if self.progresses.contains_key(scheme_name) {
            self.progresses.remove(scheme_name);
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
    scheduler: UsedScheduler,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum UsedScheduler {
    Novice(Scheduler<ScheduleParamsNovice>),
    Adept(Scheduler<ScheduleParamsAdept>),
    V2(SchedulerV2),
}

impl TrainProgress {
    pub fn new(zigens: Vec<SchemeZigen>, adept: bool, v2: bool) -> Self {
        if v2 {
            let pending_cards = zigens
                .into_iter()
                .map(|zigen| {
                    let mut card = SchedulerV2Card::default();
                    *card.zigen_mut() = zigen.clone();
                    card
                })
                .collect::<Vec<SchedulerV2Card>>();

            if pending_cards.is_empty() {
                tracing::error!("pending_cards 不能为空！");
                panic!("pending_cards is empty");
            }

            Self {
                start_time: Utc::now(),
                scheduler: UsedScheduler::V2(SchedulerV2::new(pending_cards, adept)),
            }
        } else {
            let pending_cards = zigens
                .into_iter()
                .map(|zigen| {
                    let mut card = SchedulerCard::default();
                    *card.zigen_mut() = zigen.clone();
                    card
                })
                .collect::<Vec<SchedulerCard>>();

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
    }

    pub fn is_adept(&self) -> bool {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.is_adept(),
            UsedScheduler::Adept(scheduler) => scheduler.is_adept(),
            UsedScheduler::V2(scheduler) => scheduler.is_adept(),
        }
    }

    pub fn rate_card(&mut self, rating: Rating) {
        match &mut self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.rate_card(rating),
            UsedScheduler::Adept(scheduler) => scheduler.rate_card(rating),
            UsedScheduler::V2(scheduler) => scheduler.rate_card(rating),
        }
    }

    pub fn get_card(&mut self) -> Box<dyn ZigenCard> {
        match &mut self.scheduler {
            UsedScheduler::Novice(scheduler) => Box::new(scheduler.get_card().clone()),
            UsedScheduler::Adept(scheduler) => Box::new(scheduler.get_card().clone()),
            UsedScheduler::V2(scheduler) => Box::new(scheduler.get_card().clone()),
        }
    }

    pub fn reviewed_cards(&self) -> usize {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.reviewed_cards(),
            UsedScheduler::Adept(scheduler) => scheduler.reviewed_cards(),
            UsedScheduler::V2(scheduler) => scheduler.reviewed_cards(),
        }
    }

    pub fn total_cards(&self) -> usize {
        match &self.scheduler {
            UsedScheduler::Novice(scheduler) => scheduler.total_cards(),
            UsedScheduler::Adept(scheduler) => scheduler.total_cards(),
            UsedScheduler::V2(scheduler) => scheduler.total_cards(),
        }
    }
}
