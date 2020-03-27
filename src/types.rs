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
                Trump::Standard { suit, .. },
                Card::Suited {
                    suit: card_suit, ..
                },
            ) if suit == card_suit => EffectiveSuit::Trump,

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

    pub fn successor(self, card: Card) -> Vec<Card> {
        match card {
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
        if card1 == card2 {
            return Ordering::Equal;
        }

        let card1_suit = self.effective_suit(card1);
        let card2_suit = self.effective_suit(card2);
        match self.suit() {
            Some(Suit::Hearts) | Some(Suit::Diamonds) => EffectiveSuitRedTrump::from(card1_suit)
                .cmp(&EffectiveSuitRedTrump::from(card2_suit)),
            _ => card1_suit.cmp(&card2_suit),
        }
        .then(match (card1, card2) {
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
                    number_1.as_u32().cmp(&number_2.as_u32())
                }
            }
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum EffectiveSuit {
    Clubs,
    Diamonds,
    Spades,
    Hearts,
    Trump,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum EffectiveSuitRedTrump {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
    Trump,
}
impl From<EffectiveSuit> for EffectiveSuitRedTrump {
    fn from(other: EffectiveSuit) -> EffectiveSuitRedTrump {
        match other {
            EffectiveSuit::Clubs => EffectiveSuitRedTrump::Clubs,
            EffectiveSuit::Diamonds => EffectiveSuitRedTrump::Diamonds,
            EffectiveSuit::Spades => EffectiveSuitRedTrump::Spades,
            EffectiveSuit::Hearts => EffectiveSuitRedTrump::Hearts,
            EffectiveSuit::Trump => EffectiveSuitRedTrump::Trump,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Card {
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

    pub fn as_char(self) -> char {
        match self {
            Card::Suited { suit, number } => {
                std::char::from_u32(suit.unicode_offset() as u32 + number.as_u32()).unwrap()
            }
            Card::SmallJoker => '🃟',
            Card::BigJoker => '🃏',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        if c == '🃟' {
            Some(Card::SmallJoker)
        } else if c == '🃏' {
            Some(Card::BigJoker)
        } else if c > '\u{1f0a0}' && c < '\u{1f0e0}' {
            for suit in ALL_SUITS.iter() {
                let offset = c as u32 - (suit.unicode_offset() as u32);
                if let Some(number) = Number::from_u32(offset) {
                    return Some(Card::Suited {
                        suit: *suit,
                        number,
                    });
                }
            }
            None
        } else {
            None
        }
    }

    pub const fn is_joker(self) -> bool {
        match self {
            Card::SmallJoker | Card::BigJoker => true,
            Card::Suited { .. } => false,
        }
    }

    pub const fn number(self) -> Option<Number> {
        match self {
            Card::SmallJoker | Card::BigJoker => None,
            Card::Suited { number, .. } => Some(number),
        }
    }

    pub const fn points(self) -> Option<usize> {
        match self.number() {
            Some(Number::Five) => Some(5),
            Some(Number::Ten) | Some(Number::King) => Some(10),
            _ => None,
        }
    }

    pub const fn suit(self) -> Option<Suit> {
        match self {
            Card::SmallJoker | Card::BigJoker => None,
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
    pub const fn as_u32(self) -> u32 {
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

    pub const fn from_u32(n: u32) -> Option<Self> {
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

    pub const fn successor(self) -> Option<Self> {
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

    pub const fn predecessor(self) -> Option<Self> {
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

    pub const fn as_str(self) -> &'static str {
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
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}
const ALL_SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

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
    pub const fn unicode_offset(self) -> char {
        match self {
            Suit::Spades => '\u{1f0a0}',
            Suit::Hearts => '\u{1f0b0}',
            Suit::Diamonds => '\u{1f0c0}',
            Suit::Clubs => '\u{1f0d0}',
        }
    }

    pub const fn as_char(self) -> char {
        match self {
            Suit::Hearts => '♡',
            Suit::Diamonds => '♢',
            Suit::Spades => '♤',
            Suit::Clubs => '♧',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
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
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Ace,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::King,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Queen,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Jack,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Ten,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Nine,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Eight,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Seven,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Six,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Five,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Four,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Three,
    },
    Card::Suited {
        suit: Suit::Clubs,
        number: Number::Two,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Ace,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::King,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Queen,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Jack,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Ten,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Nine,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Eight,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Seven,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Six,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Five,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Four,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Three,
    },
    Card::Suited {
        suit: Suit::Diamonds,
        number: Number::Two,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Ace,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::King,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Queen,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Jack,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Ten,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Nine,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Eight,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Seven,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Six,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Five,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Four,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Three,
    },
    Card::Suited {
        suit: Suit::Hearts,
        number: Number::Two,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Ace,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::King,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Queen,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Jack,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Ten,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Nine,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Eight,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Seven,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Six,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Five,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Four,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Three,
    },
    Card::Suited {
        suit: Suit::Spades,
        number: Number::Two,
    },
    Card::SmallJoker,
    Card::BigJoker,
];

#[cfg(test)]
mod tests {
    use super::{Card, Number, Suit, Trump, FULL_DECK};

    #[test]
    fn test_deck_completeness() {
        assert_eq!(
            "🃑🃝🃜🃛🃚🃙🃘🃗🃖🃕🃔🃓🃒🃁🃍🃌🃋🃊🃉🃈🃇🃆🃅🃄🃃🃂🂱🂽🂼🂻🂺🂹🂸🂷🂶🂵🂴🂳🂲🂡🂭🂬🂫🂪🂩🂨🂧🂦🂥🂤🂣🂢🃟🃏",
            FULL_DECK
                .iter()
                .map(|card| card.as_char())
                .collect::<String>()
        );
    }

    #[test]
    fn test_ordering() {
        let mut hand = vec![
            Card::Suited {
                suit: Suit::Hearts,
                number: Number::Six,
            },
            Card::Suited {
                suit: Suit::Hearts,
                number: Number::Five,
            },
            Card::Suited {
                suit: Suit::Hearts,
                number: Number::Four,
            },
            Card::Suited {
                suit: Suit::Spades,
                number: Number::Three,
            },
            Card::Suited {
                suit: Suit::Spades,
                number: Number::Two,
            },
            Card::Suited {
                suit: Suit::Hearts,
                number: Number::Two,
            },
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
            "4♡5♡6♡3♤2♡2♤LJHJ"
        );
    }

    #[test]
    fn test_adjacent() {
        let trump = Trump::Standard {
            number: Number::Four,
            suit: Suit::Spades,
        };
        let two = Card::Suited {
            suit: Suit::Spades,
            number: Number::Two,
        };
        let three = Card::Suited {
            suit: Suit::Spades,
            number: Number::Three,
        };
        let spade_four = Card::Suited {
            suit: Suit::Spades,
            number: Number::Four,
        };
        let heart_four = Card::Suited {
            suit: Suit::Hearts,
            number: Number::Four,
        };
        let five = Card::Suited {
            suit: Suit::Spades,
            number: Number::Five,
        };
        let spade_ace = Card::Suited {
            suit: Suit::Spades,
            number: Number::Ace,
        };
        let heart_ace = Card::Suited {
            suit: Suit::Hearts,
            number: Number::Ace,
        };
        let spade_king = Card::Suited {
            suit: Suit::Spades,
            number: Number::King,
        };
        let heart_king = Card::Suited {
            suit: Suit::Hearts,
            number: Number::King,
        };

        assert_eq!(trump.successor(three), vec![five]);
        assert_eq!(trump.successor(spade_four), vec![Card::SmallJoker]);
        assert!(trump.successor(heart_four).contains(&spade_four));
        assert!(trump.successor(spade_ace).contains(&heart_four));
        assert!(trump.successor(heart_ace).is_empty());

        let no_trump = Trump::NoTrump {
            number: Number::Four,
        };
        assert_eq!(no_trump.successor(three), vec![five]);
        assert_eq!(no_trump.successor(spade_four), vec![Card::SmallJoker]);
        assert_eq!(no_trump.successor(heart_four), vec![Card::SmallJoker]);
        assert!(no_trump.successor(spade_ace).is_empty());
        assert!(no_trump.successor(heart_ace).is_empty());

        let trump_ace = Trump::Standard {
            number: Number::Ace,
            suit: Suit::Spades,
        };
        assert_eq!(trump_ace.successor(three), vec![spade_four]);
        assert_eq!(trump_ace.successor(spade_ace), vec![Card::SmallJoker]);
        assert_eq!(trump_ace.successor(heart_ace), vec![spade_ace]);
        assert!(trump_ace.successor(spade_king).contains(&heart_ace));
        assert!(trump_ace.successor(heart_king).is_empty());

        let no_trump_ace = Trump::NoTrump {
            number: Number::Ace,
        };
        assert_eq!(no_trump_ace.successor(three), vec![spade_four]);
        assert_eq!(no_trump_ace.successor(spade_ace), vec![Card::SmallJoker]);
        assert_eq!(no_trump_ace.successor(heart_ace), vec![Card::SmallJoker]);
        assert!(no_trump_ace.successor(spade_king).is_empty());
        assert!(no_trump_ace.successor(heart_king).is_empty());
    }
}
