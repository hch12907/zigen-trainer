use chrono::{DateTime, Utc};
use dioxus_logger::tracing;
use serde_derive::{Deserialize, Serialize};

use crate::scheduler::{Rating, ZigenCard};
use crate::scheme::SchemeZigen;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct FsrsSchedulerCard {
    zigen: SchemeZigen,
    card: Card,
}

impl ZigenCard for FsrsSchedulerCard {
    fn zigen(&self) -> &SchemeZigen {
        &self.zigen
    }

    fn zigen_mut(&mut self) -> &mut SchemeZigen {
        &mut self.zigen
    }

    fn is_new_card(&self) -> bool {
        self.card == Card::New
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
enum Card {
    #[default]
    New,

    Review {
        stability: f64,
        difficulty: f64,
        last_reviewed: DateTime<Utc>,
    },
}

impl Card {
    fn get_retrievability(&self, now: DateTime<Utc>) -> f64 {
        match self {
            Card::New => 0.0,
            Card::Review {
                stability,
                difficulty: _,
                last_reviewed,
            } => {
                const W20: f64 = 11111.0;

                let time = now.signed_duration_since(last_reviewed).as_seconds_f64();
                let factor = 0.9f64.powf(-W20.recip()) - 1.0;
                (1.0 + factor * time / stability).powf(-W20)
            }
        }
    }

    fn rate_card(&mut self, rating: Rating) {
        // 算法来源：https://expertium.github.io/Algorithm.html#short-term-s

        const W0: f64 = 0.2120;
        const W1: f64 = 1.2921;
        const W2: f64 = 2.3065;
        const W3: f64 = 8.2956;
        const W4: f64 = 6.4133;
        const W5: f64 = 0.8334;
        const W6: f64 = 3.0194;
        const W7: f64 = 0.0010;
        const W8: f64 = 1.8722;
        const W9: f64 = 0.1666;
        const W10: f64 = 0.7960;
        const W11: f64 = 1.4835;
        const W12: f64 = 0.0614;
        const W13: f64 = 0.2629;
        const W14: f64 = 1.6483;
        const W15: f64 = 0.6014;
        const W16: f64 = 1.8729;
        const W17: f64 = 0.5425;
        const W18: f64 = 0.0912;
        const W19: f64 = 0.1542;
        // 4.0 源自 Rating::Easy
        let diffculty_mean: f64 = W4 - f64::exp(W5 * (4.0 - 1.0)) + 1.0;

        let now = Utc::now();

        let retrievability = self.get_retrievability(Utc::now());

        let grade = match rating {
            Rating::Again => 1.0f64,
            Rating::Hard => 2.0f64,
            Rating::Good => 3.0f64,
            Rating::Easy => 4.0f64,
        };

        if let Self::New = self {
            let s0 = match rating {
                Rating::Again => W0,
                Rating::Hard => W1,
                Rating::Good => W2,
                Rating::Easy => W3,
            };
            let d0 = W4 - f64::exp(W5 * (grade - 1.0)) + 1.0;

            *self = Self::Review {
                stability: s0,
                difficulty: d0,
                last_reviewed: now,
            };
        }
        //
        else if let Self::Review {
            stability,
            difficulty,
            last_reviewed,
        } = self
        {
            if rating != Rating::Again {
                // 卡片的记忆难度越高，记忆稳定性的增长速度越慢。
                let diff_factor = 11.0f64 - *difficulty;

                // 随着记忆稳定性上升，稳定性后期增长的速度会逐渐下降。
                let scaled_s = stability.powf(-W9);

                // 回忆率越低，记忆稳定性上升得越快。
                let scaled_r = f64::exp(W10 * (1.0 - retrievability)) - 1.0;

                let stability_inc =
                    1.0 + W15 * W16 * f64::exp(W8) * diff_factor * scaled_s * scaled_r;
                let hours_since_last_review = now.signed_duration_since(*last_reviewed).num_hours();

                let new_stability = if hours_since_last_review >= 20 {
                    *stability * stability_inc
                } else {
                    let s =
                        *stability * (f64::exp(W17 * (grade - 3.0 + W18))) * stability.powf(-W19);

                    if grade >= 3.0 {
                        f64::max(*stability, s)
                    } else {
                        s
                    }
                };

                *stability = new_stability;

                let difficulty_delta = -W6 * (grade - 3.0);

                let new_difficulty = *difficulty + difficulty_delta * (10.0 - *difficulty) / 9.0;

                *difficulty = W7 * diffculty_mean + (1.0 - W7) * new_difficulty;
            } else {
                let diff_factor = difficulty.powf(-W12);

                let scaled_s = (*stability + 1.0).powf(W13) - 1.0;

                let scaled_r = f64::exp(W14 * (1.0 - retrievability));

                let new_stability = W11 * diff_factor * scaled_s * scaled_r;

                *stability = f64::min(*stability, new_stability);

                let difficulty_delta = -W6 * (grade - 3.0);
                let new_difficulty = *difficulty + difficulty_delta * (10.0 - *difficulty) / 9.0;
                *difficulty = W7 * diffculty_mean + (1.0 - W7) * new_difficulty;
            }

            *last_reviewed = now;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct FsrsScheduler {
    new_cards: Vec<FsrsSchedulerCard>,
    learning_cards: Vec<FsrsSchedulerCard>,
}

const MINIMUM_LEARNING_CARDS: usize = 8;
const GOOD_RETENTION_RATE: f64 = 0.85;

impl FsrsScheduler {
    pub fn new(mut pending_cards: Vec<FsrsSchedulerCard>) -> Self {
        let len = pending_cards.len();
        pending_cards.reverse();

        let mut this = Self {
            new_cards: pending_cards,
            learning_cards: Vec::with_capacity(len),
        };

        this.populate_learning_cards();
        this
    }

    fn populate_learning_cards(&mut self) {
        while self.learning_cards.len() < MINIMUM_LEARNING_CARDS {
            let Some(new_card) = self.new_cards.pop() else {
                break;
            };

            self.learning_cards.push(new_card);
        }

        let now = Utc::now();

        if self.new_cards.len() > 0 {
            let well_studied_cards = self
                .learning_cards
                .iter()
                .filter(|card| card.card.get_retrievability(now) > GOOD_RETENTION_RATE)
                .count();

            if well_studied_cards + MINIMUM_LEARNING_CARDS / 3 > self.learning_cards.len() {
                let new_card = self.new_cards.pop().unwrap();

                self.learning_cards.push(new_card);
            }
        }

        self.learning_cards.sort_by(|a, b| {
            a.card
                .get_retrievability(now)
                .partial_cmp(&b.card.get_retrievability(now))
                .expect("retrivability不应是NaN")
        });

        for card in self.learning_cards.iter() {
            tracing::debug!("retrievability={}", card.card.get_retrievability(now));
        }
    }

    pub fn get_card(&mut self) -> &FsrsSchedulerCard {
        self.populate_learning_cards();
        self.learning_cards.first().unwrap()
    }

    pub fn rate_card(&mut self, rating: Rating) {
        self.learning_cards
            .first_mut()
            .unwrap()
            .card
            .rate_card(rating);
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
