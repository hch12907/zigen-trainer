use chrono::{DateTime, Duration, Utc};
use dioxus_logger::tracing;
use rand::seq::SliceRandom;
use serde_derive::{Deserialize, Serialize};

use crate::scheduler::{Rating, ZigenCard};
use crate::scheme::{LearnMode, SchemeZigen};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct SchedulerV2Card {
    zigen: SchemeZigen,
    card: Card,
}

impl ZigenCard for SchedulerV2Card {
    fn zigen(&self) -> &SchemeZigen {
        &self.zigen
    }

    fn zigen_mut(&mut self) -> &mut SchemeZigen {
        &mut self.zigen
    }

    fn shuffle(&mut self) {
        self.zigen_mut()
            .as_raw_parts_mut()
            .0
            .shuffle(&mut rand::rng());
    }

    fn is_new_card(&self) -> bool {
        self.card == Card::New
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
enum Card {
    /// 全新的，未曾学习过的卡片。
    #[default]
    New,

    /// 正在学习的卡片。
    Learning {
        /// 卡片的回答次数。
        ///
        /// 如果回答失败，且attempts = 0，则attempts -= 1；
        /// 如果回答失败，且attempts > 0，则attempts = 0；
        /// 如果回答成功，且attempts = 0，则attempts += 1；
        /// 如果回答成功，且attempts < 0，则attempts = 0。
        attempts: i32,
        /// 卡片的间隔数，用户每回答一次卡片，队列里的所有学习卡片的间隔数都会减一，
        /// 间隔数达到零时，卡片再次出现。
        interval: usize,
        /// 上一次作答的时间。
        last_reviewed: DateTime<Utc>,
    },

    /// 已经能够回答数次，正在复习阶段的卡片。
    /// 回答失败后可能会回归到学习阶段。
    Review {
        /// 最后一次作答时使用的间隔数（卡片在复习多少张其他卡片后再度出现）
        last_interval: f64,
        /// 已复习次数
        repetition: usize,
        /// 这张卡片的容易系数
        easiness_factor: f64,
        /// 上一次作答的时间。
        last_reviewed: DateTime<Utc>,
        /// 最迟下一次复习的时间。
        due: DateTime<Utc>,
    }, // todo: 在这个阶段的卡片使用due排序，「每天」复习已due的卡片
}

impl Card {
    fn rate_card(&self, param: &ScheduleParam, rating: Rating) -> Self {
        match self {
            Card::New => {
                let attempts = if rating == Rating::Again {
                    0
                } else {
                    param.attempts_boost()
                };

                if (attempts.max(0) as usize) < param.learning_intervals_s().len() {
                    let idx = if attempts < 0 { (-attempts - 1) as usize } else { 0 };
                    Self::Learning {
                        attempts: attempts,
                        interval: param.learning_intervals_s()[idx],
                        last_reviewed: Utc::now(),
                    }
                } else {
                    // 引入一定的随机性，避免同一批卡片扎堆出现
                    let noise = rand::random_range(0.9..1.2);
                    let interval =
                        (param.initial_review_interval_sec() as f64 * noise).ceil() as i64;

                    Self::Review {
                        last_interval: 1.0,
                        repetition: 1,
                        easiness_factor: 2.5,
                        last_reviewed: Utc::now(),
                        due: Utc::now() + Duration::seconds(interval),
                    }
                }
            }

            Card::Learning { attempts, .. } => {
                let new_attempts = if rating != Rating::Again {
                    *attempts + 1
                } else {
                    *attempts - 1
                };

                if new_attempts >= param.max_learning_attempts() as i32 {
                    // 引入一定的随机性，避免同一批卡片扎堆出现
                    let noise = rand::random_range(0.9..1.2);
                    let interval =
                        (param.initial_review_interval_sec() as f64 * noise).ceil() as i64;

                    Self::Review {
                        last_interval: 1.0,
                        repetition: 1,
                        easiness_factor: 2.5,
                        last_reviewed: Utc::now(),
                        due: Utc::now() + Duration::seconds(interval),
                    }
                } else {
                    // 引入一定的随机性，避免同一批卡片扎堆出现
                    let noise = rand::random_range(0.9..1.5);

                    Self::Learning {
                        attempts: new_attempts,
                        interval: if new_attempts >= 0 {
                            let idx = new_attempts as usize;
                            let idx = idx.min(param.learning_intervals_s().len() - 1);
                            let interval = param.learning_intervals_s()[idx];
                            (interval as f64 * noise).ceil() as usize
                        } else {
                            let idx = -new_attempts as usize - 1;
                            let idx = idx.min(param.learning_intervals_f().len() - 1);
                            let interval = param.learning_intervals_f()[idx];
                            (interval as f64 * noise).ceil() as usize
                        },
                        last_reviewed: Utc::now(),
                    }
                }
            }

            Card::Review {
                last_interval,
                repetition,
                easiness_factor,
                ..
            } => {
                let last_interval = match *repetition {
                    0 => 0.5,
                    1 => 1.5,
                    2 => 3.0,
                    _ => (*last_interval * *easiness_factor).round(),
                };

                let repetition = if rating != Rating::Again {
                    *repetition + 1
                } else {
                    0
                };

                let difficulty = rating.difficulty();
                let easiness_factor =
                    easiness_factor + 0.1 - difficulty * (0.08 + difficulty * 0.2);
                let easiness_factor = easiness_factor.max(1.3);

                let due = Utc::now()
                    + Duration::seconds(30 + (param.review_interval_factor() * last_interval) as i64);

                Self::Review {
                    last_interval,
                    repetition,
                    easiness_factor,
                    last_reviewed: Utc::now(),
                    due,
                }
            }
        }
    }

    fn needs_learning(&self, now: DateTime<Utc>) -> bool {
        match self {
            Card::New => true,
            Card::Learning { interval, .. } => *interval == 0,
            Card::Review { due, .. } => *due < now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
enum ScheduleParam {
    /// 适合初学者的调度参数。
    #[default]
    Novice,
    /// 适合复习者的调度参数。
    Adept,
    /// 极速模式。
    Rapid,
}

impl ScheduleParam {
    /// 连续正确回答多少次后，认定用户已经学会一张卡片（从学习阶段转入复习阶段）
    fn max_learning_attempts(&self) -> usize {
        match self {
            ScheduleParam::Novice => 3,
            ScheduleParam::Adept => 2,
            ScheduleParam::Rapid => 2,
        }
    }

    /// 在学习阶段，在用户正确回答卡片后，卡片将在什么时候（复习多少张其他卡片后）再度出现。
    fn learning_intervals_s(&self) -> &'static [usize] {
        match self {
            ScheduleParam::Novice => &[2, 3, 9],
            ScheduleParam::Adept => &[2, 6],
            ScheduleParam::Rapid => &[2, 3],
        }
    }

    /// 在学习阶段，在用户错误回答卡片后，卡片将在什么时候（复习多少张其他卡片后）再度出现。
    fn learning_intervals_f(&self) -> &'static [usize] {
        match self {
            ScheduleParam::Novice => &[2, 4, 6],
            ScheduleParam::Adept => &[2, 4],
            ScheduleParam::Rapid => &[3, 3],
        }
    }

    /// 卡片在第一次作答后，会得到多大的“已回答次数”奖励。
    fn attempts_boost(&self) -> i32 {
        match self {
            ScheduleParam::Adept => 1,
            ScheduleParam::Rapid => self.max_learning_attempts() as i32,
            _ => 0,
        }
    }

    /// 卡片从学习阶段转入复习阶段后，多久才会被复习。
    fn initial_review_interval_sec(&self) -> i64 {
        match self {
            ScheduleParam::Novice => 300,
            ScheduleParam::Adept => 450,
            ScheduleParam::Rapid => 600,
        }
    }

    /// 卡片复习间隔的因子。
    fn review_interval_factor(&self) -> f64 {
        match self {
            ScheduleParam::Novice => 120.0,
            ScheduleParam::Adept => 240.0,
            ScheduleParam::Rapid => 300.0,
        }
    }

    /// 学习阶段需要维持的卡片数量。
    fn learning_cards(&self) -> usize {
        match self {
            ScheduleParam::Novice => 6,
            ScheduleParam::Adept => 3,
            ScheduleParam::Rapid => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct SchedulerV2 {
    new_cards: Vec<SchedulerV2Card>,
    learning_cards: Vec<SchedulerV2Card>,
    sched_param: ScheduleParam,
}

impl SchedulerV2 {
    pub fn new(mut pending_cards: Vec<SchedulerV2Card>, learn_mode: LearnMode) -> Self {
        let len = pending_cards.len();
        pending_cards.reverse();

        let mut this = Self {
            new_cards: pending_cards,
            learning_cards: Vec::with_capacity(len),
            sched_param: match learn_mode {
                LearnMode::Novice => ScheduleParam::Novice,
                LearnMode::Adept => ScheduleParam::Adept,
                LearnMode::Rapid => ScheduleParam::Rapid,
            },
        };

        this.populate_learning_cards();
        this
    }

    pub fn show_hint(&self) -> bool {
        self.sched_param == ScheduleParam::Novice
    }

    fn populate_learning_cards(&mut self) {
        let now = Utc::now();

        if !self.new_cards.is_empty() {
            let currently_learning = self
                .learning_cards
                .iter()
                .filter(|card| card.card.needs_learning(now))
                .count();

            for _ in 0..self
                .sched_param
                .learning_cards()
                .saturating_sub(currently_learning)
            {
                let Some(new_card) = self.new_cards.pop() else {
                    break;
                };

                self.learning_cards.push(new_card);
            }
        }

        // 排序优先度：
        // - 最先：已过期复习阶段卡片，越早过期的越优先
        // - 其次：刚加入队列（New阶段）的卡片。
        // - 再次：学习阶段的卡片；此处依赖稳定排序。
        // - 最后：未过期复习阶段卡片。
        self.learning_cards.sort_by(|a, b| {
            use std::cmp::Ordering::*;

            match (&a.card, &b.card) {
                (Card::New, Card::New) => Equal,
                (Card::New, Card::Learning { .. }) => Less,
                (Card::New, Card::Review { due, .. }) if *due < now => Greater,
                (Card::New, Card::Review { .. }) => Less,
                (
                    Card::Learning {
                        attempts: _attempts1,
                        interval: interval1,
                        last_reviewed: last_reviewed1,
                    },
                    Card::Learning {
                        attempts: _attempts2,
                        interval: interval2,
                        last_reviewed: last_reviewed2,
                    },
                ) => (*interval1)
                    .cmp(interval2)
                    .then(last_reviewed1.cmp(last_reviewed2)),
                (Card::Learning { .. }, Card::New) => Greater,
                (Card::Learning { .. }, Card::Review { due, .. }) if *due < now => Greater,
                (Card::Learning { .. }, Card::Review { .. }) => Less,
                (Card::Review { due, .. }, Card::New) if *due < now => Less,
                (Card::Review { .. }, Card::New) => Greater,
                (Card::Review { due, .. }, Card::Learning { .. }) if *due < now => Less,
                (Card::Review { .. }, Card::Learning { .. }) => Greater,
                (Card::Review { due: due1, .. }, Card::Review { due: due2, .. }) => due1.cmp(due2),
            }
        });
    }

    pub fn get_card(&mut self) -> &mut SchedulerV2Card {
        self.populate_learning_cards();
        self.learning_cards.first_mut().unwrap()
    }

    pub fn rate_card(&mut self, rating: Rating) {
        let card = self
            .learning_cards
            .first()
            .unwrap()
            .card
            .rate_card(&self.sched_param, rating);

        for card in self.learning_cards.iter_mut() {
            match &mut card.card {
                Card::Learning { interval, .. } => *interval = interval.saturating_sub(1),
                _ => (),
            };
        }

        self.learning_cards.first_mut().unwrap().card = card;
    }

    pub fn reviewed_cards(&self) -> usize {
        let not_yet_learned = self
            .learning_cards
            .iter()
            .filter(|card| card.card == Card::New)
            .count();

        tracing::debug!("not_yet_learned={not_yet_learned}");
        self.learning_cards.len() - not_yet_learned
    }

    pub fn total_cards(&self) -> usize {
        self.new_cards.len() + self.learning_cards.len()
    }
}
