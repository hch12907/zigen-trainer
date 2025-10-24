use std::collections::VecDeque;
use std::marker::PhantomData;

use chrono::{DateTime, Duration, Utc};
use dioxus_logger::tracing;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FlashCard<C> {
    pub content: C,
    pub card: Card,
}

impl<C> FlashCard<C> {
    pub fn is_new_card(&self) -> bool {
        self.card == Card::New
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

impl Rating {
    fn difficulty(&self) -> f64 {
        match self {
            Rating::Again => 3.0,
            Rating::Hard => 2.0,
            Rating::Good => 1.0,
            Rating::Easy => 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Card {
    /// 全新的，未曾学习过的卡片。
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Scheduler<P: ScheduleParam, C> {
    new_cards: Vec<FlashCard<C>>,
    learning_cards: VecDeque<FlashCard<C>>,
    reviewing_cards: VecDeque<FlashCard<C>>,

    /// 已成功连续学习 learning_cards 卡片的次数
    done_learning: usize,

    /// 调度器使用的参数
    params: PhantomData<P>,
}

pub trait ScheduleParam {
    /// 学习：复习的比例，N张：1张
    /// 至于学习、复习具体的定义是什么，可以阅读 ReviewStatus 的文档。
    const LEARN_REVIEW_RATIO: usize;
    /// 连续正确回答多少次后，认定用户已经学会一张卡片（从学习阶段转入复习阶段）
    const MAX_LEARNING_ATTEMPTS: usize;
    /// 在学习阶段，在用户正确回答卡片后，卡片将在什么时候（复习多少张其他卡片后）再度出现
    const LEARNING_INTERVALS_S: &'static [usize] /* [usize; Param::MAX_LEARNING_ATTEMPTS] */ ;
    /// 在学习阶段，在用户错误回答卡片后，卡片将在什么时候（复习多少张其他卡片后）再度出现
    const LEARNING_INTERVALS_F: &'static [usize] /* [usize; Param::MAX_LEARNING_ATTEMPTS] */ ;
    /// 学习阶段的卡片数量，必须是 LEARNING_INTERVALS_S[-1] + 1
    const LEARNING_CARDS: usize = Self::LEARNING_INTERVALS_S[Self::MAX_LEARNING_ATTEMPTS - 1] + 1;
}

/// 适合初学者的调度参数。
#[derive(Clone, Debug)]
pub struct ScheduleParamsNovice;

impl ScheduleParam for ScheduleParamsNovice {
    const LEARN_REVIEW_RATIO: usize = 8;
    const MAX_LEARNING_ATTEMPTS: usize = 3;
    const LEARNING_INTERVALS_S: &'static [usize] = &[3, 6, 9];
    const LEARNING_INTERVALS_F: &'static [usize] = &[2, 4, 6];
}

/// 适合养老的调度参数。
#[derive(Clone, Debug)]
pub struct ScheduleParamsAdept;

impl ScheduleParam for ScheduleParamsAdept {
    const LEARN_REVIEW_RATIO: usize = 20;
    const MAX_LEARNING_ATTEMPTS: usize = 2;
    const LEARNING_INTERVALS_S: &'static [usize] = &[3, 6];
    const LEARNING_INTERVALS_F: &'static [usize] = &[2, 4];
}

/// 练习器将练习分成了三个阶段：
/// - 学习：认识新卡片的阶段。
/// - 复习：已经学习完毕后，巩固知识的阶段。
/// - 穿插类复习：在学习阶段，为了不让已学习知识自然衰退，间隔性展出旧卡片。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReviewStatus {
    /// 学习
    Learn,
    /// 穿插性复习
    ReviewIntersperse,
    /// 复习
    Review,
}

impl<Param: ScheduleParam, C> Scheduler<Param, C> {
    pub fn new(mut pending_cards: Vec<FlashCard<C>>) -> Self {
        pending_cards.reverse();

        let mut this = Self {
            new_cards: pending_cards,
            learning_cards: VecDeque::with_capacity(Param::LEARNING_CARDS),
            reviewing_cards: VecDeque::new(),
            done_learning: 0,
            params: PhantomData,
        };

        this.populate_learning_cards();
        this
    }

    fn populate_learning_cards(&mut self) {
        if self.learning_cards.len() < Param::LEARNING_CARDS && !self.new_cards.is_empty() {
            let diff = Param::LEARNING_CARDS - self.learning_cards.len();
            let split_off = diff.min(self.new_cards.len());

            if self.new_cards.len() > 0 {
                let start = self.new_cards.len() - split_off;
                let end = self.new_cards.len();

                self.new_cards.drain(start..end)
                    .for_each(|card| {
                        self.learning_cards.push_front(card);
                    });
            }
        }
    }

    /// 用户当前处于什么练习阶段。
    fn review_status(&self) -> ReviewStatus {
        let learn_review_ratio = if self.new_cards.is_empty() {
            self.learning_cards
                .iter()
                .filter(|card| match card.card {
                    Card::Learning { attempts, .. } => {
                        attempts < Param::MAX_LEARNING_ATTEMPTS as i32
                    }
                    Card::New => true,
                    _ => unreachable!(),
                })
                .count()
        } else {
            Param::LEARN_REVIEW_RATIO
        };

        if self.done_learning >= learn_review_ratio && self.reviewing_cards.len() > 0 {
            if !self.learning_cards.is_empty() {
                ReviewStatus::ReviewIntersperse
            } else {
                ReviewStatus::Review
            }
        } else {
            ReviewStatus::Learn
        }
    }

    pub fn get_card(&self) -> &FlashCard<C> {
        match self.review_status() {
            ReviewStatus::Review => self.reviewing_cards.front().unwrap(),
            ReviewStatus::ReviewIntersperse => self.reviewing_cards.back().unwrap(),
            ReviewStatus::Learn => self.learning_cards.front().unwrap()
        }
    }

    pub fn rate_card(&mut self, rating: Rating) {
        if self.review_status() != ReviewStatus::Learn {
            let mut card = if self.review_status() == ReviewStatus::Review {
                self.reviewing_cards.pop_front().unwrap()
            } else {
                self.reviewing_cards.pop_back().unwrap()
            };
            assert!(matches!(card.card, Card::Review { .. }));

            let (back_to_learn, interval) = match card.card {
                Card::Review {
                    ref mut last_interval,
                    ref mut repetition,
                    ref mut easiness_factor,
                    ref mut last_reviewed,
                    ref mut due,
                } if rating != Rating::Again => {
                    *last_interval = match *repetition {
                        0 => 3.0,
                        1 => 6.0,
                        _ => (*last_interval * *easiness_factor).round(),
                    };

                    *repetition += 1;

                    let difficulty = rating.difficulty();
                    *easiness_factor += 0.1 - difficulty * (0.08 + difficulty * 0.2);
                    *easiness_factor = easiness_factor.max(1.3);

                    *last_reviewed = Utc::now();
                    *due = Utc::now() + Duration::days(1);

                    (false, *last_interval as usize)
                }

                Card::Review {
                    ref mut last_interval,
                    ref mut repetition,
                    ref mut easiness_factor,
                    ref mut last_reviewed,
                    ref mut due,
                } => {
                    *repetition = 0;
                    *last_interval = 3.0;

                    let difficulty = rating.difficulty();
                    *easiness_factor += 0.1 - difficulty * (0.08 + difficulty * 0.2);
                    *easiness_factor = easiness_factor.max(1.3);

                    *last_reviewed = Utc::now();
                    *due = Utc::now() + Duration::minutes(1);

                    (true, *last_interval as usize)
                }

                _ => unreachable!(),
            };

            tracing::debug!(
                "did a review card: back_to_learn={back_to_learn}, interval={interval}"
            );

            if back_to_learn && !self.learning_cards.is_empty() {
                card.card = Card::Learning {
                    attempts: 1,
                    last_reviewed: Utc::now(),
                };
                self.learning_cards
                    .insert(Param::LEARNING_INTERVALS_F[0], card);
            } else {
                let at = interval.min(self.reviewing_cards.len());
                self.reviewing_cards.insert(at, card);
            }

            self.done_learning = 0;
        } else {
            let mut card = self.learning_cards.pop_front().unwrap();
            assert!(matches!(card.card, Card::New | Card::Learning { .. }));

            let (to_review, mut interval) = match &mut card.card {
                Card::Learning {
                    attempts,
                    last_reviewed,
                } => {
                    if rating == Rating::Again {
                        *attempts = (*attempts - 1).min(0);
                    } else {
                        *attempts = (*attempts + 1).max(0);
                    }

                    *last_reviewed = Utc::now();

                    tracing::debug!("did a learning card: attempts={}", *attempts);

                    if *attempts >= Param::MAX_LEARNING_ATTEMPTS as i32 {
                        (true, usize::MAX)
                    } else {
                        let interval = if *attempts < 0 {
                            let idx =
                                ((-*attempts) as usize).min(Param::LEARNING_INTERVALS_F.len() - 1);
                            Param::LEARNING_INTERVALS_F[idx]
                        } else {
                            let idx =
                                (*attempts as usize).min(Param::LEARNING_INTERVALS_S.len() - 1);
                            Param::LEARNING_INTERVALS_S[idx]
                        };
                        (false, interval)
                    }
                }

                c @ Card::New => {
                    *c = Card::Learning {
                        attempts: 0,
                        last_reviewed: Utc::now(),
                    };

                    tracing::debug!("did a new card in learning queue:");

                    (false, Param::LEARNING_INTERVALS_S[0])
                }

                _ => unreachable!(),
            };

            if !to_review {
                let at = interval.min(self.learning_cards.len());
                self.learning_cards.insert(at, card);
            } else if self.new_cards.is_empty()
                && self.learning_cards.len() <= Param::LEARNING_CARDS
            {
                // 如果新卡已经枯竭，我们先hold住当前的所有位于学习队列的卡片，不让它们进入复习阶段。
                // 当所有学习卡片都已经学习成功，我们一股脑把这些卡片送入复习队列内。

                self.learning_cards.push_back(card);

                let can_flush = self.learning_cards.iter().all(|card| match card.card {
                    Card::Learning { attempts, .. } => {
                        attempts >= Param::MAX_LEARNING_ATTEMPTS as i32
                    }
                    _ => unreachable!(),
                });

                if can_flush {
                    for mut card in std::mem::take(&mut self.learning_cards).into_iter() {
                        let interval = match &card.card {
                            Card::Learning { attempts, .. } => (*attempts as usize) * 13 / 10,
                            _ => unreachable!(),
                        };

                        card.card = Card::Review {
                            last_interval: 1.0,
                            repetition: 1,
                            easiness_factor: 2.5,
                            last_reviewed: Utc::now(),
                            due: Utc::now() + Duration::days(1),
                        };

                        let at = interval.min(self.reviewing_cards.len());
                        self.reviewing_cards.insert(at, card);
                    }
                }
            } else {
                if interval == usize::MAX {
                    card.card = Card::Review {
                        last_interval: 1.0,
                        repetition: 1,
                        easiness_factor: 2.5,
                        last_reviewed: Utc::now(),
                        due: Utc::now() + Duration::days(1),
                    };
                    interval = 1;
                }

                self.reviewing_cards
                    .insert(interval.min(self.reviewing_cards.len()), card);
            }

            self.done_learning += 1;
        }

        self.populate_learning_cards();
    }

    pub fn reviewed_cards(&self) -> usize {
        let not_yet_learned = self
            .learning_cards
            .iter()
            .filter(|card| card.card == Card::New)
            .count();

        tracing::debug!("not_yet_learned={not_yet_learned}");
        self.learning_cards.len() - not_yet_learned + self.reviewing_cards.len()
    }

    pub fn total_cards(&self) -> usize {
        self.new_cards.len() + self.learning_cards.len() + self.reviewing_cards.len()
    }
}
