use chrono::Utc;
use dioxus_logger::tracing;
use rs_fsrs::FSRS;
use serde_derive::{Serialize, Deserialize};

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
        match &self.card {
            Card::New => true,
            Card::Review(card) => card.get_retrievability(Utc::now()) < 0.001,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
enum Card {
    #[default]
    New,

    Review(rs_fsrs::Card),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct FsrsScheduler {
    new_cards: Vec<FsrsSchedulerCard>,
    learning_cards: Vec<FsrsSchedulerCard>,
}

const MINIMUM_LEARNING_CARDS: usize = 6;
const GOOD_RETENTION_RATE: f64 = 0.95;

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
            let Some(mut new_card) = self.new_cards.pop() else {
                break
            };

            new_card.card = Card::Review(rs_fsrs::Card::new());

            self.learning_cards.push(new_card);
        }

        let now = Utc::now();

        if self.new_cards.len() > 0
            && self.learning_cards.iter().all(|card| match &card.card {
                Card::New => unreachable!(),
                Card::Review(card) => card.get_retrievability(now) > GOOD_RETENTION_RATE,
            })
        {
            let mut new_card = self.new_cards.pop().unwrap();

            new_card.card = Card::Review(rs_fsrs::Card::new());

            self.learning_cards.push(new_card);
        }

        self.learning_cards.sort_by(|a, b| {
            let Card::Review(a) = &a.card else {
                unreachable!()
            };

            let Card::Review(b) = &b.card else {
                unreachable!()
            };

            a.get_retrievability(now).partial_cmp(&b.get_retrievability(now))
                .expect("retrivability不应是NaN")
        });

        for card in self.learning_cards.iter() {
            let Card::Review(card) = &card.card else {
                unreachable!()
            };
            tracing::debug!("retrievability={}", card.get_retrievability(now));
        }
    }

    pub fn get_card(&mut self) -> &FsrsSchedulerCard {
        self.populate_learning_cards();
        self.learning_cards.first().unwrap()
    }

    pub fn rate_card(&mut self, rating: Rating) {
        let fsrs = FSRS::new(rs_fsrs::Parameters {
            request_retention: GOOD_RETENTION_RATE,
            decay: 0.5,
            enable_fuzz: true,
            enable_short_term: true,
            ..Default::default()
        });

        let current_card = self.learning_cards.first_mut().unwrap();
        let Card::Review(card) = &mut current_card.card else {
            unreachable!()
        };

        let new_card = fsrs.next(card.clone(), Utc::now(), match rating {
            Rating::Again => rs_fsrs::Rating::Again,
            Rating::Easy => rs_fsrs::Rating::Easy,
            Rating::Good => rs_fsrs::Rating::Good,
            Rating::Hard => rs_fsrs::Rating::Hard,
        }).card;

        *card = new_card;
    }

    pub fn reviewed_cards(&self) -> usize {
        let now = Utc::now();

        let not_yet_learned = self
            .learning_cards
            .iter()
            .filter(|card| match &card.card {
                Card::New => unreachable!(),
                Card::Review(card) => card.get_retrievability(now) < GOOD_RETENTION_RATE,
            })
            .count();

        tracing::debug!("not_yet_learned={not_yet_learned}");
        self.learning_cards.len() - not_yet_learned
    }

    pub fn total_cards(&self) -> usize {
        self.new_cards.len() + self.learning_cards.len()
    }
}
