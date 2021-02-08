use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use serde::de::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(transparent)]
pub struct PlayerID(pub usize);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Trump {
    Standard { suit: Suit, number: Number },
    NoTrump { number: Number },
}

impl Trump {
    pub fn number(self) -> Number {
        match self {
            Trump::Standard { number, .. } => number,
            Trump::NoTrump { number } => number,
        }
    }

    pub fn suit(self) -> Option<Suit> {
        match self {
            Trump::Standard { suit, .. } => Some(suit),
            Trump::NoTrump { .. } => None,
        }
    }

    pub fn effective_suit(self, card: Card) -> EffectiveSuit {
        match (self, card) {
            (_, Card::Unknown) => EffectiveSuit::Unknown,
            (_, Card::SmallJoker) | (_, Card::BigJoker) => EffectiveSuit::Trump,

            (
                Trump::Standard { number, .. },
                Card::Suited {
                    number: card_number,
                    ..
                },
            )
            | (
                Trump::NoTrump { number },
                Card::Suited {
                    number: card_number,
                    ..
                },
            ) if number == card_number => EffectiveSuit::Trump,

            (
                Trump::Standard {
                    suit: trump_suit, ..
                },
                Card::Suited { suit, .. },
            ) => match suit {
                _ if suit == trump_suit => EffectiveSuit::Trump,
                Suit::Clubs => EffectiveSuit::Clubs,
                Suit::Diamonds => EffectiveSuit::Diamonds,
                Suit::Spades => EffectiveSuit::Spades,
                Suit::Hearts => EffectiveSuit::Hearts,
            },
            (Trump::NoTrump { .. }, Card::Suited { suit, .. }) => match suit {
                Suit::Clubs => EffectiveSuit::Clubs,
                Suit::Diamonds => EffectiveSuit::Diamonds,
                Suit::Spades => EffectiveSuit::Spades,
                Suit::Hearts => EffectiveSuit::Hearts,
            },
        }
    }

    pub fn suit_ordinal(self, card: Card) -> impl Ord {
        let effective_suit = self.effective_suit(card);
        match self {
            Trump::Standard {
                suit: Suit::Clubs, ..
            } => match effective_suit {
                EffectiveSuit::Unknown => 0,
                EffectiveSuit::Diamonds => 1,
                EffectiveSuit::Spades => 2,
                EffectiveSuit::Hearts => 3,
                EffectiveSuit::Clubs => 4,
                EffectiveSuit::Trump => 5,
            },
            Trump::Standard {
                suit: Suit::Diamonds,
                ..
            } => match effective_suit {
                EffectiveSuit::Unknown => 0,
                EffectiveSuit::Spades => 1,
                EffectiveSuit::Hearts => 2,
                EffectiveSuit::Clubs => 3,
                EffectiveSuit::Diamonds => 4,
                EffectiveSuit::Trump => 5,
            },
            Trump::Standard {
                suit: Suit::Spades, ..
            } => match effective_suit {
                EffectiveSuit::Unknown => 0,
                EffectiveSuit::Hearts => 1,
                EffectiveSuit::Clubs => 2,
                EffectiveSuit::Diamonds => 3,
                EffectiveSuit::Spades => 4,
                EffectiveSuit::Trump => 5,
            },
            Trump::Standard {
                suit: Suit::Hearts, ..
            } => match effective_suit {
                EffectiveSuit::Unknown => 0,
                EffectiveSuit::Clubs => 1,
                EffectiveSuit::Diamonds => 2,
                EffectiveSuit::Spades => 3,
                EffectiveSuit::Hearts => 4,
                EffectiveSuit::Trump => 5,
            },
            Trump::NoTrump { .. } => match effective_suit {
                EffectiveSuit::Unknown => 0,
                EffectiveSuit::Clubs => 1,
                EffectiveSuit::Diamonds => 2,
                EffectiveSuit::Spades => 3,
                EffectiveSuit::Hearts => 4,
                EffectiveSuit::Trump => 5,
            },
        }
    }

    pub fn successor(self, card: Card) -> Vec<Card> {
        match card {
            Card::Unknown => vec![],
            Card::BigJoker => vec![],
            Card::SmallJoker => vec![Card::BigJoker],
            Card::Suited { suit, number } if number == self.number() => match self {
                Trump::Standard {
                    suit: trump_suit,
                    number: trump_number,
                } => {
                    if suit == trump_suit {
                        vec![Card::SmallJoker]
                    } else {
                        vec![Card::Suited {
                            suit: trump_suit,
                            number: trump_number,
                        }]
                    }
                }
                Trump::NoTrump { .. } => vec![Card::SmallJoker],
            },
            Card::Suited { suit, number } if number.successor() == Some(self.number()) => {
                match number.successor().and_then(|n| n.successor()) {
                    Some(n) => vec![Card::Suited { suit, number: n }],
                    None if self.effective_suit(card) == EffectiveSuit::Trump => ALL_SUITS
                        .iter()
                        .flat_map(|s| {
                            if Some(*s) != self.suit() {
                                Some(Card::Suited {
                                    suit: *s,
                                    number: self.number(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                    None => vec![],
                }
            }
            Card::Suited { suit, number } => match number.successor() {
                Some(n) => vec![Card::Suited { suit, number: n }],
                None if self.effective_suit(card) == EffectiveSuit::Trump => ALL_SUITS
                    .iter()
                    .flat_map(|s| {
                        if Some(*s) != self.suit() {
                            Some(Card::Suited {
                                suit: *s,
                                number: self.number(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
                None => vec![],
            },
        }
    }

    pub fn compare(self, card1: Card, card2: Card) -> Ordering {
        self.compare_effective(card1, card2)
            .then(card1.as_char().cmp(&card2.as_char()))
    }

    pub fn compare_effective(self, card1: Card, card2: Card) -> Ordering {
        if card1 == card2 {
            return Ordering::Equal;
        }

        self.suit_ordinal(card1)
            .cmp(&self.suit_ordinal(card2))
            .then(match (card1, card2) {
                (Card::Unknown, _) => Ordering::Less,
                (_, Card::Unknown) => Ordering::Greater,
                (Card::BigJoker, _) => Ordering::Greater,
                (_, Card::BigJoker) => Ordering::Less,
                (Card::SmallJoker, _) => Ordering::Greater,
                (_, Card::SmallJoker) => Ordering::Less,
                (
                    Card::Suited {
                        number: number_1,
                        suit: suit_1,
                    },
                    Card::Suited {
                        number: number_2,
                        suit: suit_2,
                    },
                ) => {
                    let trump_number = self.number();
                    if number_1 == trump_number && number_2 == trump_number {
                        if let Trump::Standard {
                            suit: trump_suit, ..
                        } = self
                        {
                            if suit_1 == trump_suit && suit_2 == trump_suit {
                                Ordering::Equal
                            } else if suit_1 == trump_suit {
                                Ordering::Greater
                            } else if suit_2 == trump_suit {
                                Ordering::Less
                            } else {
                                Ordering::Equal
                            }
                        } else {
                            Ordering::Equal
                        }
                    } else if number_1 == trump_number {
                        Ordering::Greater
                    } else if number_2 == trump_number {
                        Ordering::Less
                    } else {
                        number_1.cmp(&number_2)
                    }
                }
            })
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum EffectiveSuit {
    Unknown,
    Clubs,
    Diamonds,
    Spades,
    Hearts,
    Trump,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CardInfo {
    value: char,
    display_value: char,
    typ: char,
    number: Option<&'static str>,
    points: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Card {
    Unknown,
    Suited { suit: Suit, number: Number },
    SmallJoker,
    BigJoker,
}

impl Serialize for Card {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(self.as_char())
    }
}

impl<'d> Deserialize<'d> for Card {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        Card::from_char(c).ok_or_else(|| D::Error::custom(format!("Unexpected char '{:?}'", c)))
    }
}

impl Card {
    pub fn count(iter: impl IntoIterator<Item = Card>) -> HashMap<Card, usize> {
        let mut counts = HashMap::new();
        for card in iter {
            *counts.entry(card).or_insert(0) += 1
        }
        counts
    }

    pub fn cards<'a, 'b: 'a>(
        iter: impl Iterator<Item = (&'b Card, &'b usize)> + 'a,
    ) -> impl Iterator<Item = &'b Card> + 'a {
        iter.flat_map(|(card, count)| (0..*count).map(move |_| card))
    }

    pub fn as_info(self) -> CardInfo {
        let value = self.as_char();
        CardInfo {
            value,
            display_value: if self == Card::BigJoker {
                Card::SmallJoker.as_char()
            } else {
                value
            },
            number: self.number().map(|n| n.as_str()),
            typ: self.suit().map(|s| s.as_char()).unwrap_or(value),
            points: self.points().unwrap_or(0),
        }
    }

    pub fn as_char(self) -> char {
        match self {
            cards::C_A => '🃑',
            cards::C_K => '🃞',
            cards::C_Q => '🃝',
            cards::C_J => '🃛',
            cards::C_10 => '🃚',
            cards::C_9 => '🃙',
            cards::C_8 => '🃘',
            cards::C_7 => '🃗',
            cards::C_6 => '🃖',
            cards::C_5 => '🃕',
            cards::C_4 => '🃔',
            cards::C_3 => '🃓',
            cards::C_2 => '🃒',
            cards::D_A => '🃁',
            cards::D_K => '🃎',
            cards::D_Q => '🃍',
            cards::D_J => '🃋',
            cards::D_10 => '🃊',
            cards::D_9 => '🃉',
            cards::D_8 => '🃈',
            cards::D_7 => '🃇',
            cards::D_6 => '🃆',
            cards::D_5 => '🃅',
            cards::D_4 => '🃄',
            cards::D_3 => '🃃',
            cards::D_2 => '🃂',
            cards::H_A => '🂱',
            cards::H_K => '🂾',
            cards::H_Q => '🂽',
            cards::H_J => '🂻',
            cards::H_10 => '🂺',
            cards::H_9 => '🂹',
            cards::H_8 => '🂸',
            cards::H_7 => '🂷',
            cards::H_6 => '🂶',
            cards::H_5 => '🂵',
            cards::H_4 => '🂴',
            cards::H_3 => '🂳',
            cards::H_2 => '🂲',
            cards::S_A => '🂡',
            cards::S_K => '🂮',
            cards::S_Q => '🂭',
            cards::S_J => '🂫',
            cards::S_10 => '🂪',
            cards::S_9 => '🂩',
            cards::S_8 => '🂨',
            cards::S_7 => '🂧',
            cards::S_6 => '🂦',
            cards::S_5 => '🂥',
            cards::S_4 => '🂤',
            cards::S_3 => '🂣',
            cards::S_2 => '🂢',
            Card::SmallJoker => '🃟',
            Card::BigJoker => '🃏',
            Card::Unknown => '🂠',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '🃑' => Some(cards::C_A),
            '🃞' => Some(cards::C_K),
            '🃝' => Some(cards::C_Q),
            '🃛' => Some(cards::C_J),
            '🃚' => Some(cards::C_10),
            '🃙' => Some(cards::C_9),
            '🃘' => Some(cards::C_8),
            '🃗' => Some(cards::C_7),
            '🃖' => Some(cards::C_6),
            '🃕' => Some(cards::C_5),
            '🃔' => Some(cards::C_4),
            '🃓' => Some(cards::C_3),
            '🃒' => Some(cards::C_2),
            '🃁' => Some(cards::D_A),
            '🃎' => Some(cards::D_K),
            '🃍' => Some(cards::D_Q),
            '🃋' => Some(cards::D_J),
            '🃊' => Some(cards::D_10),
            '🃉' => Some(cards::D_9),
            '🃈' => Some(cards::D_8),
            '🃇' => Some(cards::D_7),
            '🃆' => Some(cards::D_6),
            '🃅' => Some(cards::D_5),
            '🃄' => Some(cards::D_4),
            '🃃' => Some(cards::D_3),
            '🃂' => Some(cards::D_2),
            '🂱' => Some(cards::H_A),
            '🂾' => Some(cards::H_K),
            '🂽' => Some(cards::H_Q),
            '🂻' => Some(cards::H_J),
            '🂺' => Some(cards::H_10),
            '🂹' => Some(cards::H_9),
            '🂸' => Some(cards::H_8),
            '🂷' => Some(cards::H_7),
            '🂶' => Some(cards::H_6),
            '🂵' => Some(cards::H_5),
            '🂴' => Some(cards::H_4),
            '🂳' => Some(cards::H_3),
            '🂲' => Some(cards::H_2),
            '🂡' => Some(cards::S_A),
            '🂮' => Some(cards::S_K),
            '🂭' => Some(cards::S_Q),
            '🂫' => Some(cards::S_J),
            '🂪' => Some(cards::S_10),
            '🂩' => Some(cards::S_9),
            '🂨' => Some(cards::S_8),
            '🂧' => Some(cards::S_7),
            '🂦' => Some(cards::S_6),
            '🂥' => Some(cards::S_5),
            '🂤' => Some(cards::S_4),
            '🂣' => Some(cards::S_3),
            '🂢' => Some(cards::S_2),
            '🃟' => Some(Card::SmallJoker),
            '🃏' => Some(Card::BigJoker),
            '🂠' => Some(Card::Unknown),
            _ => None,
        }
    }

    pub fn is_joker(self) -> bool {
        match self {
            Card::SmallJoker | Card::BigJoker => true,
            Card::Unknown | Card::Suited { .. } => false,
        }
    }

    pub fn number(self) -> Option<Number> {
        match self {
            Card::Unknown | Card::SmallJoker | Card::BigJoker => None,
            Card::Suited { number, .. } => Some(number),
        }
    }

    pub fn points(self) -> Option<usize> {
        self.number().and_then(|n| n.points())
    }

    pub fn suit(self) -> Option<Suit> {
        match self {
            Card::Unknown | Card::SmallJoker | Card::BigJoker => None,
            Card::Suited { suit, .. } => Some(suit),
        }
    }
}
impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Card::Suited { suit, number } => write!(f, "{}{}", number.as_str(), suit.as_char()),
            Card::SmallJoker => write!(f, "LJ"),
            Card::BigJoker => write!(f, "HJ"),
            Card::Unknown => write!(f, "[]"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Number {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl slog::Value for Number {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.as_str())
    }
}
impl Serialize for Number {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
impl<'d> Deserialize<'d> for Number {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Number::from_str(&s).ok_or_else(|| D::Error::custom(format!("Unexpected string '{}'", s)))
    }
}

impl Number {
    pub fn as_u32(self) -> u32 {
        match self {
            Number::Two => 2,
            Number::Three => 3,
            Number::Four => 4,
            Number::Five => 5,
            Number::Six => 6,
            Number::Seven => 7,
            Number::Eight => 8,
            Number::Nine => 9,
            Number::Ten => 10,
            Number::Jack => 11,
            Number::Queen => 12,
            Number::King => 13,
            Number::Ace => 1,
        }
    }

    pub fn from_u32(n: u32) -> Option<Self> {
        match n {
            2 => Some(Number::Two),
            3 => Some(Number::Three),
            4 => Some(Number::Four),
            5 => Some(Number::Five),
            6 => Some(Number::Six),
            7 => Some(Number::Seven),
            8 => Some(Number::Eight),
            9 => Some(Number::Nine),
            10 => Some(Number::Ten),
            11 => Some(Number::Jack),
            12 => Some(Number::Queen),
            13 => Some(Number::King),
            1 => Some(Number::Ace),
            _ => None,
        }
    }

    pub fn successor(self) -> Option<Self> {
        match self {
            Number::Two => Some(Number::Three),
            Number::Three => Some(Number::Four),
            Number::Four => Some(Number::Five),
            Number::Five => Some(Number::Six),
            Number::Six => Some(Number::Seven),
            Number::Seven => Some(Number::Eight),
            Number::Eight => Some(Number::Nine),
            Number::Nine => Some(Number::Ten),
            Number::Ten => Some(Number::Jack),
            Number::Jack => Some(Number::Queen),
            Number::Queen => Some(Number::King),
            Number::King => Some(Number::Ace),
            Number::Ace => None,
        }
    }

    pub fn predecessor(self) -> Option<Self> {
        match self {
            Number::Two => None,
            Number::Three => Some(Number::Two),
            Number::Four => Some(Number::Three),
            Number::Five => Some(Number::Four),
            Number::Six => Some(Number::Five),
            Number::Seven => Some(Number::Six),
            Number::Eight => Some(Number::Seven),
            Number::Nine => Some(Number::Eight),
            Number::Ten => Some(Number::Nine),
            Number::Jack => Some(Number::Ten),
            Number::Queen => Some(Number::Jack),
            Number::King => Some(Number::Queen),
            Number::Ace => Some(Number::King),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Number::Two => "2",
            Number::Three => "3",
            Number::Four => "4",
            Number::Five => "5",
            Number::Six => "6",
            Number::Seven => "7",
            Number::Eight => "8",
            Number::Nine => "9",
            Number::Ten => "10",
            Number::Jack => "J",
            Number::Queen => "Q",
            Number::King => "K",
            Number::Ace => "A",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "2" => Some(Number::Two),
            "3" => Some(Number::Three),
            "4" => Some(Number::Four),
            "5" => Some(Number::Five),
            "6" => Some(Number::Six),
            "7" => Some(Number::Seven),
            "8" => Some(Number::Eight),
            "9" => Some(Number::Nine),
            "10" => Some(Number::Ten),
            "J" => Some(Number::Jack),
            "Q" => Some(Number::Queen),
            "K" => Some(Number::King),
            "A" => Some(Number::Ace),
            _ => None,
        }
    }

    pub fn points(self) -> Option<usize> {
        match self {
            Number::Five => Some(5),
            Number::Ten | Number::King => Some(10),
            _ => None,
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Spades,
    Hearts,
}
pub const ALL_SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

impl Serialize for Suit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(self.as_char())
    }
}

impl<'d> Deserialize<'d> for Suit {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        let c = char::deserialize(deserializer)?;
        Suit::from_char(c).ok_or_else(|| D::Error::custom(format!("Unexpected char '{:?}'", c)))
    }
}

impl Suit {
    pub fn unicode_offset(self) -> char {
        match self {
            Suit::Spades => '\u{1f0a0}',
            Suit::Hearts => '\u{1f0b0}',
            Suit::Diamonds => '\u{1f0c0}',
            Suit::Clubs => '\u{1f0d0}',
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Suit::Hearts => '♡',
            Suit::Diamonds => '♢',
            Suit::Spades => '♤',
            Suit::Clubs => '♧',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '♡' => Some(Suit::Hearts),
            '♢' => Some(Suit::Diamonds),
            '♤' => Some(Suit::Spades),
            '♧' => Some(Suit::Clubs),
            _ => None,
        }
    }
}
impl fmt::Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

pub const FULL_DECK: [Card; 54] = [
    cards::D_A,
    cards::D_K,
    cards::D_Q,
    cards::D_J,
    cards::D_10,
    cards::D_9,
    cards::D_8,
    cards::D_7,
    cards::D_6,
    cards::D_5,
    cards::D_4,
    cards::D_3,
    cards::D_2,
    cards::C_A,
    cards::C_K,
    cards::C_Q,
    cards::C_J,
    cards::C_10,
    cards::C_9,
    cards::C_8,
    cards::C_7,
    cards::C_6,
    cards::C_5,
    cards::C_4,
    cards::C_3,
    cards::C_2,
    cards::H_A,
    cards::H_K,
    cards::H_Q,
    cards::H_J,
    cards::H_10,
    cards::H_9,
    cards::H_8,
    cards::H_7,
    cards::H_6,
    cards::H_5,
    cards::H_4,
    cards::H_3,
    cards::H_2,
    cards::S_A,
    cards::S_K,
    cards::S_Q,
    cards::S_J,
    cards::S_10,
    cards::S_9,
    cards::S_8,
    cards::S_7,
    cards::S_6,
    cards::S_5,
    cards::S_4,
    cards::S_3,
    cards::S_2,
    Card::SmallJoker,
    Card::BigJoker,
];

pub mod cards {
    use super::{Card, Number, Suit};

    pub const C_A: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Ace,
    };
    pub const C_K: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::King,
    };
    pub const C_Q: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Queen,
    };
    pub const C_J: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Jack,
    };
    pub const C_10: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Ten,
    };
    pub const C_9: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Nine,
    };
    pub const C_8: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Eight,
    };
    pub const C_7: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Seven,
    };
    pub const C_6: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Six,
    };
    pub const C_5: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Five,
    };
    pub const C_4: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Four,
    };
    pub const C_3: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Three,
    };
    pub const C_2: Card = Card::Suited {
        suit: Suit::Clubs,
        number: Number::Two,
    };
    pub const D_A: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Ace,
    };
    pub const D_K: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::King,
    };
    pub const D_Q: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Queen,
    };
    pub const D_J: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Jack,
    };
    pub const D_10: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Ten,
    };
    pub const D_9: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Nine,
    };
    pub const D_8: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Eight,
    };
    pub const D_7: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Seven,
    };
    pub const D_6: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Six,
    };
    pub const D_5: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Five,
    };
    pub const D_4: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Four,
    };
    pub const D_3: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Three,
    };
    pub const D_2: Card = Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Two,
    };
    pub const H_A: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Ace,
    };
    pub const H_K: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::King,
    };
    pub const H_Q: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Queen,
    };
    pub const H_J: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Jack,
    };
    pub const H_10: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Ten,
    };
    pub const H_9: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Nine,
    };
    pub const H_8: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Eight,
    };
    pub const H_7: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Seven,
    };
    pub const H_6: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Six,
    };
    pub const H_5: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Five,
    };
    pub const H_4: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Four,
    };
    pub const H_3: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Three,
    };
    pub const H_2: Card = Card::Suited {
        suit: Suit::Hearts,
        number: Number::Two,
    };
    pub const S_A: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Ace,
    };
    pub const S_K: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::King,
    };
    pub const S_Q: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Queen,
    };
    pub const S_J: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Jack,
    };
    pub const S_10: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Ten,
    };
    pub const S_9: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Nine,
    };
    pub const S_8: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Eight,
    };
    pub const S_7: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Seven,
    };
    pub const S_6: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Six,
    };
    pub const S_5: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Five,
    };
    pub const S_4: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Four,
    };
    pub const S_3: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Three,
    };
    pub const S_2: Card = Card::Suited {
        suit: Suit::Spades,
        number: Number::Two,
    };
}

#[cfg(test)]
mod tests {
    use super::{cards, Card, Number, Suit, Trump, FULL_DECK};

    #[test]
    fn test_char_roundtrip() {
        for card in FULL_DECK.iter() {
            assert_eq!(*card, Card::from_char(card.as_char()).unwrap());
        }
    }

    #[test]
    fn test_deck_completeness() {
        assert_eq!(
            "🃁🃎🃍🃋🃊🃉🃈🃇🃆🃅🃄🃃🃂🃑🃞🃝🃛🃚🃙🃘🃗🃖🃕🃔🃓🃒🂱🂾🂽🂻🂺🂹🂸🂷🂶🂵🂴🂳🂲🂡🂮🂭🂫🂪🂩🂨🂧🂦🂥🂤🂣🂢🃟🃏",
            FULL_DECK
                .iter()
                .map(|card| card.as_char())
                .collect::<String>()
        );
    }

    #[test]
    fn test_ordering() {
        let mut hand = vec![
            cards::H_6,
            cards::H_5,
            cards::H_4,
            cards::S_3,
            cards::S_2,
            cards::C_4,
            cards::H_2,
            cards::D_3,
            Card::SmallJoker,
            Card::BigJoker,
        ];
        let trump = Trump::Standard {
            number: Number::Two,
            suit: Suit::Spades,
        };
        hand.sort_by(|a, b| trump.compare(*a, *b));
        assert_eq!(
            hand.iter().map(|c| format!("{:?}", c)).collect::<String>(),
            "4♡5♡6♡4♧3♢3♤2♡2♤LJHJ"
        );
    }

    #[test]
    fn test_adjacent() {
        let trump = Trump::Standard {
            number: Number::Four,
            suit: Suit::Spades,
        };

        let s = |c| trump.successor(c).into_iter().collect::<Vec<_>>();
        assert_eq!(s(cards::S_3), vec![cards::S_5]);
        assert_eq!(s(cards::S_4), vec![Card::SmallJoker]);
        assert!(s(cards::H_4).contains(&cards::S_4));
        assert!(s(cards::S_A).contains(&cards::H_4));
        assert!(s(cards::H_A).is_empty());

        let no_trump = Trump::NoTrump {
            number: Number::Four,
        };
        let s = |c| no_trump.successor(c).into_iter().collect::<Vec<_>>();
        assert_eq!(s(cards::S_3), vec![cards::S_5]);
        assert_eq!(s(cards::S_4), vec![Card::SmallJoker]);
        assert_eq!(s(cards::H_4), vec![Card::SmallJoker]);
        assert!(s(cards::S_A).is_empty());
        assert!(s(cards::H_A).is_empty());

        let trump_ace = Trump::Standard {
            number: Number::Ace,
            suit: Suit::Spades,
        };
        let s = |c| trump_ace.successor(c).into_iter().collect::<Vec<_>>();
        assert_eq!(s(cards::S_3), vec![cards::S_4]);
        assert_eq!(s(cards::S_A), vec![Card::SmallJoker]);
        assert_eq!(s(cards::H_A), vec![cards::S_A]);
        assert!(s(cards::S_K).contains(&cards::H_A));
        assert!(s(cards::H_K).is_empty());

        let no_trump_ace = Trump::NoTrump {
            number: Number::Ace,
        };
        let s = |c| no_trump_ace.successor(c).into_iter().collect::<Vec<_>>();
        assert_eq!(s(cards::S_3), vec![cards::S_4]);
        assert_eq!(s(cards::S_A), vec![Card::SmallJoker]);
        assert_eq!(s(cards::H_A), vec![Card::SmallJoker]);
        assert!(s(cards::S_K).is_empty());
        assert!(s(cards::H_K).is_empty());
    }
}
