use anyhow::{anyhow, bail, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::bidding::Bid;
use shengji_mechanics::deck::Deck;
use shengji_mechanics::hands::Hands;
use shengji_mechanics::types::{Card, PlayerID, Rank, Trump};

use crate::message::MessageVariant;
use crate::settings::{FirstLandlordSelectionPolicy, GameMode, KittyBidPolicy, PropagatedState};

use crate::game_state::exchange_phase::ExchangePhase;
use crate::game_state::initialize_phase::InitializePhase;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DrawPhase {
    num_decks: usize,
    game_mode: GameMode,
    deck: Vec<Card>,
    propagated: PropagatedState,
    hands: Hands,
    bids: Vec<Bid>,
    #[serde(default)]
    autobid: Option<Bid>,
    position: usize,
    #[serde(default)]
    deck_cut_player: Option<PlayerID>,
    kitty: Vec<Card>,
    #[serde(default)]
    revealed_cards: usize,
    level: Option<Rank>,
    #[serde(default)]
    removed_cards: Vec<Card>,
    #[serde(default)]
    decks: Vec<Deck>,
    player_requested_reset: Option<PlayerID>,
}

impl DrawPhase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        propagated: PropagatedState,
        position: usize,
        deck: Vec<Card>,
        kitty: Vec<Card>,
        num_decks: usize,
        game_mode: GameMode,
        level: Option<Rank>,
        decks: Vec<Deck>,
        removed_cards: Vec<Card>,
    ) -> Self {
        let deck_cut_player = if propagated.players.is_empty() {
            None
        } else {
            Some(
                propagated.players
                    [(position + propagated.players.len() - 1) % propagated.players.len()]
                .id,
            )
        };
        DrawPhase {
            hands: Hands::new(propagated.players.iter().map(|p| p.id)),
            deck,
            kitty,
            propagated,
            position,
            deck_cut_player,
            num_decks,
            decks,
            game_mode,
            level,
            removed_cards,
            bids: Vec::new(),
            revealed_cards: 0,
            autobid: None,
            player_requested_reset: None,
        }
    }

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn propagated_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }

    pub fn removed_cards(&self) -> &[Card] {
        &self.removed_cards
    }

    pub fn deck(&self) -> &[Card] {
        &self.deck
    }

    pub fn kitty(&self) -> &[Card] {
        &self.kitty
    }

    pub fn deck_cut_player(&self) -> Option<PlayerID> {
        self.deck_cut_player
    }

    #[cfg(test)]
    pub fn deck_mut(&mut self) -> &mut Vec<Card> {
        &mut self.deck
    }

    #[cfg(test)]
    pub fn position_mut(&mut self) -> &mut usize {
        &mut self.position
    }

    #[cfg(test)]
    pub fn kitty_mut(&mut self) -> &mut Vec<Card> {
        &mut self.kitty
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        if let Some(player) = self.deck_cut_player {
            return Ok(player);
        }
        if self.deck.is_empty() {
            let (first_bid, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
            let landlord = self.propagated.landlord.unwrap_or(
                match self.propagated.first_landlord_selection_policy {
                    FirstLandlordSelectionPolicy::ByWinningBid => winning_bid.id,
                    FirstLandlordSelectionPolicy::ByFirstBid => first_bid.id,
                },
            );

            Ok(landlord)
        } else {
            Ok(self.propagated.players[self.position].id)
        }
    }

    pub fn draw_card(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.deck_cut_player.is_some() {
            bail!("the deck must be cut before cards can be drawn");
        }
        if id != self.propagated.players[self.position].id {
            bail!("not your turn!");
        }
        if let Some(next_card) = self.deck.pop() {
            self.hands.add(id, Some(next_card))?;
            self.position = (self.position + 1) % self.propagated.players.len();
            Ok(())
        } else {
            bail!("no cards left in deck")
        }
    }

    pub fn cut_deck(&mut self, id: PlayerID, count: usize) -> Result<(), Error> {
        if self.deck_cut_player != Some(id) {
            bail!("not your turn to cut the deck");
        }

        let total_cards = self.deck.len() + self.kitty.len();
        if count > total_cards {
            bail!("cut must be between 0 and {total_cards}");
        }

        // `deck.pop()` draws from the top, so the drawable portion is stored
        // bottom-to-top. Reconstruct the physical top-to-bottom order including
        // the kitty, rotate the requested top cards to the bottom, then split it
        // back without changing the kitty size.
        let num_drawable_cards = self.deck.len();
        let mut top_to_bottom = self
            .deck
            .iter()
            .rev()
            .chain(self.kitty.iter())
            .copied()
            .collect::<Vec<_>>();
        top_to_bottom.rotate_left(count);
        self.deck = top_to_bottom[..num_drawable_cards]
            .iter()
            .rev()
            .copied()
            .collect();
        self.kitty = top_to_bottom[num_drawable_cards..].to_vec();
        self.deck_cut_player = None;
        Ok(())
    }

    pub fn reveal_card(&mut self) -> Result<MessageVariant, Error> {
        if !self.deck.is_empty() {
            bail!("can't reveal card until deck is fully drawn")
        }
        if !self.bids.is_empty() {
            bail!("can't reveal card if at least one bid has been made")
        }
        let id = self
            .propagated
            .landlord
            .ok_or_else(|| anyhow!("can't reveal card if landlord hasn't been selected yet"))?;

        let landlord_level = self
            .propagated
            .players
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("Couldn't find landlord level?"))?
            .rank();

        if landlord_level == Rank::NoTrump {
            bail!("can't reveal card if the level is no trump!");
        }

        if self.revealed_cards >= self.kitty.len() || self.autobid.is_some() {
            bail!("can't reveal any more cards")
        }

        let level = self
            .propagated
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.rank())
            .ok_or_else(|| anyhow!("can't find landlord level?"))?;

        let card = self.kitty[self.revealed_cards];

        match self.propagated.kitty_bid_policy {
            KittyBidPolicy::FirstCard => {
                self.autobid = Some(Bid {
                    count: 1,
                    id,
                    card,
                    epoch: 0,
                });
            }
            KittyBidPolicy::FirstCardOfLevelOrHighest
                if card.is_joker() || card.number().map(Rank::Number) == Some(level) =>
            {
                self.autobid = Some(Bid {
                    count: 1,
                    id,
                    card,
                    epoch: 0,
                });
            }
            KittyBidPolicy::FirstCardOfLevelOrHighest
                if self.revealed_cards >= self.kitty.len() - 1 =>
            {
                let mut sorted_kitty = self.kitty.clone();
                sorted_kitty.sort_by(|a, b| {
                    Trump::NoTrump {
                        number: match level {
                            Rank::Number(n) => Some(n),
                            Rank::NoTrump => None,
                        },
                    }
                    .compare(*a, *b)
                });
                if let Some(highest_card) = sorted_kitty.last() {
                    self.autobid = Some(Bid {
                        count: 1,
                        id,
                        card: *highest_card,
                        epoch: 0,
                    });
                }
            }
            _ => (),
        }
        self.revealed_cards += 1;

        Ok(MessageVariant::RevealedCardFromKitty)
    }

    pub fn bid(&mut self, id: PlayerID, card: Card, count: usize) -> bool {
        if self.revealed_cards > 0 {
            return false;
        }
        Bid::bid(
            id,
            card,
            count,
            &mut self.bids,
            self.autobid,
            &self.hands,
            &self.propagated.players,
            self.propagated.landlord,
            self.propagated.bid_policy,
            self.propagated.bid_reinforcement_policy,
            self.propagated.joker_bid_policy,
            self.num_decks,
            0,
        )
    }

    pub fn take_back_bid(&mut self, id: PlayerID) -> Result<(), Error> {
        Bid::take_back_bid(id, self.propagated.bid_takeback_policy, &mut self.bids, 0)
    }

    pub fn done_drawing(&self) -> bool {
        self.deck.is_empty()
    }

    pub fn advance(&self, id: PlayerID) -> Result<ExchangePhase, Error> {
        if !self.deck.is_empty() {
            bail!("deck has cards remaining")
        }

        let (landlord, landlord_level) = {
            let landlord = match self.propagated.landlord {
                Some(landlord) => landlord,
                None => {
                    let (first_bid, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
                    match self.propagated.first_landlord_selection_policy {
                        FirstLandlordSelectionPolicy::ByWinningBid => winning_bid.id,
                        FirstLandlordSelectionPolicy::ByFirstBid => first_bid.id,
                    }
                }
            };

            if id != landlord {
                bail!("only the leader can advance the game");
            }
            let landlord_level = self
                .propagated
                .players
                .iter()
                .find(|p| p.id == landlord)
                .ok_or_else(|| anyhow!("Couldn't find landlord level?"))?
                .rank();
            (landlord, landlord_level)
        };
        let trump = match landlord_level {
            Rank::NoTrump => Trump::NoTrump { number: None },
            Rank::Number(landlord_level) => {
                // Note: this is not repeated in all cases above, but it is
                // repeated in some. It's OK because the bid calculation is
                // fast.
                let (_, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
                match winning_bid.card {
                    Card::Unknown => bail!("can't bid with unknown cards!"),
                    Card::SmallJoker | Card::BigJoker => Trump::NoTrump {
                        number: Some(landlord_level),
                    },
                    Card::Suited { suit, .. } => Trump::Standard {
                        suit,
                        number: landlord_level,
                    },
                }
            }
        };
        let mut hands = self.hands.clone();
        hands.set_trump(trump);
        Ok(ExchangePhase::new(
            self.propagated.clone(),
            self.num_decks,
            self.game_mode.clone(),
            self.kitty.clone(),
            landlord,
            hands,
            trump,
            self.bids.clone(),
            self.autobid,
            self.removed_cards.clone(),
            self.decks.clone(),
        ))
    }

    pub fn request_reset(
        &mut self,
        player: PlayerID,
    ) -> Result<(Option<InitializePhase>, Vec<MessageVariant>), Error> {
        match self.player_requested_reset {
            Some(p) => {
                // ignore duplicate reset requests from same player
                if p == player {
                    return Ok((None, vec![]));
                }

                let (s, m) = self.return_to_initialize()?;
                Ok((Some(s), m))
            }
            None => {
                self.player_requested_reset = Some(player);
                Ok((None, vec![MessageVariant::ResetRequested]))
            }
        }
    }

    pub fn cancel_reset(&mut self) -> Option<MessageVariant> {
        if self.player_requested_reset.is_some() {
            self.player_requested_reset = None;
            return Some(MessageVariant::ResetCanceled);
        }
        None
    }

    fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase::from_propagated(propagated), msgs))
    }

    pub fn destructively_redact_for_player(&mut self, player: PlayerID) {
        self.hands.destructively_redact_except_for_player(player);
        for card in &mut self.kitty[self.revealed_cards..] {
            *card = Card::Unknown;
        }
        self.deck.fill(Card::Unknown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shengji_mechanics::types::cards;

    #[test]
    fn player_before_first_drawer_cuts_entire_deck() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        init.add_player("p2".into()).unwrap();
        init.add_player("p3".into()).unwrap();
        let p4 = init.add_player("p4".into()).unwrap().0;
        let mut draw = init.start(p1).unwrap();

        draw.position = 0;
        draw.deck_cut_player = Some(p4);
        draw.deck = vec![cards::S_6, cards::S_5, cards::S_4];
        draw.kitty = vec![cards::S_7, cards::S_8];

        assert_eq!(draw.next_player().unwrap(), p4);
        assert!(draw.draw_card(p1).is_err());
        assert!(draw.cut_deck(p1, 2).is_err());
        assert!(draw.cut_deck(p4, 6).is_err());

        draw.cut_deck(p4, 2).unwrap();
        assert_eq!(draw.deck, vec![cards::S_8, cards::S_7, cards::S_6]);
        assert_eq!(draw.kitty, vec![cards::S_4, cards::S_5]);
        assert_eq!(draw.next_player().unwrap(), p1);

        draw.draw_card(p1).unwrap();
        assert_eq!(draw.hands.get(p1).unwrap().get(&cards::S_6), Some(&1));
        assert!(draw.cut_deck(p4, 0).is_err());
    }
}
