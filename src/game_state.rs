use std::collections::{HashMap, HashSet};

use anyhow::{bail, Error};
use rand::{seq::SliceRandom, RngCore};
use serde::{Deserialize, Serialize};

use crate::hands::Hands;
use crate::trick::Trick;
use crate::types::{Card, Number, PlayerID, Trump, FULL_DECK};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerID,
    pub name: String,
    level: Number,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Tractor,
    FindingFriends {
        num_friends: usize,
        friends: Vec<Friend>,
    },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Friend {
    card: Card,
    skip: usize,
    player_id: Option<PlayerID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameState {
    Initialize(InitializePhase),
    Draw(DrawPhase),
    Exchange(ExchangePhase),
    Play(PlayPhase),
    Done,
}

impl GameState {
    pub fn players(&self) -> Option<&'_ [Player]> {
        match self {
            GameState::Initialize(p) => Some(&p.players),
            GameState::Draw(p) => Some(&p.players),
            GameState::Exchange(p) => Some(&p.players),
            GameState::Play(p) => Some(&p.players),
            GameState::Done => None,
        }
    }

    pub fn player_name(&self, id: PlayerID) -> Result<&'_ str, Error> {
        if let Some(players) = self.players() {
            for p in players {
                if p.id == id {
                    return Ok(&p.name);
                }
            }
        }
        bail!("Couldn't find player name")
    }

    pub fn cards(&self, id: PlayerID) -> Vec<Card> {
        match self {
            GameState::Done | GameState::Initialize { .. } => vec![],
            GameState::Draw(DrawPhase { ref hands, .. })
            | GameState::Exchange(ExchangePhase { ref hands, .. })
            | GameState::Play(PlayPhase { ref hands, .. }) => {
                hands.cards(id).unwrap_or_else(|_| vec![])
            }
        }
    }

    pub fn for_player(&self, id: PlayerID) -> GameState {
        let mut s = self.clone();
        match s {
            GameState::Done | GameState::Initialize { .. } => (),
            GameState::Draw(DrawPhase {
                ref mut hands,
                ref mut kitty,
                ref mut deck,
                ..
            }) => {
                hands.redact_except(id);
                for card in kitty {
                    *card = Card::Unknown;
                }
                for card in deck {
                    *card = Card::Unknown;
                }
            }
            GameState::Exchange(ExchangePhase {
                ref mut hands,
                ref mut kitty,
                ref mut game_mode,
                landlord,
                ..
            }) => {
                hands.redact_except(id);
                if id != landlord {
                    for card in kitty {
                        *card = Card::Unknown;
                    }
                    if let GameMode::FindingFriends {
                        ref mut friends, ..
                    } = game_mode
                    {
                        friends.clear();
                    }
                }
            }
            GameState::Play(PlayPhase {
                ref mut hands,
                ref mut kitty,
                landlord,
                ..
            }) => {
                hands.redact_except(id);
                if id != landlord {
                    for card in kitty {
                        *card = Card::Unknown;
                    }
                }
            }
        }
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayPhase {
    num_decks: usize,
    game_mode: GameMode,
    hands: Hands,
    points: HashMap<PlayerID, Vec<Card>>,
    kitty: Vec<Card>,
    landlord: PlayerID,
    landlords_team: Vec<PlayerID>,
    players: Vec<Player>,
    trump: Trump,
    trick: Trick,
    last_trick: Option<Trick>,
}
impl PlayPhase {
    pub fn next_player(&self) -> PlayerID {
        self.trick.next_player().unwrap()
    }

    pub fn can_play_cards(&self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        Ok(self.trick.can_play_cards(id, &self.hands, cards)?)
    }

    pub fn play_cards(&mut self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        self.trick.play_cards(id, &mut self.hands, cards)?;
        Ok(())
    }

    pub fn take_back_cards(&mut self, id: PlayerID) -> Result<(), Error> {
        Ok(self.trick.take_back(id, &mut self.hands)?)
    }

    pub fn finish_trick(&mut self) -> Result<Vec<String>, Error> {
        let (winner, mut new_points, kitty_multipler) = self.trick.complete()?;
        let mut msgs = vec![];
        if let GameMode::FindingFriends {
            ref mut friends, ..
        } = self.game_mode
        {
            for played in self.trick.played_cards() {
                for card in played.cards.iter() {
                    for friend in friends.iter_mut() {
                        if friend.card == *card {
                            if friend.skip == 0 {
                                if friend.player_id.is_none() {
                                    friend.player_id = Some(played.id);
                                    if !self.landlords_team.contains(&played.id) {
                                        self.landlords_team.push(played.id);
                                        for player in &self.players {
                                            if player.id == played.id {
                                                msgs.push(format!(
                                                    "{} has joined the team",
                                                    player.name
                                                ))
                                            }
                                        }
                                    }
                                }
                            } else {
                                friend.skip -= 1;
                            }
                        }
                    }
                }
            }
        }
        let points = self.points.get_mut(&winner).unwrap();
        if self.hands.is_empty() {
            for _ in 0..kitty_multipler {
                new_points.extend(self.kitty.iter().filter(|c| c.points().is_some()).cloned());
            }
        }
        let winner_idx = self.players.iter().position(|p| p.id == winner).unwrap();
        if !new_points.is_empty() {
            let trump = self.trump;
            let num_points = new_points.iter().flat_map(|c| c.points()).sum::<usize>();
            points.extend(new_points);
            points.sort_by(|a, b| trump.compare(*a, *b));
            msgs.push(format!(
                "{} wins the trick and gets {} points",
                self.players[winner_idx].name, num_points
            ));
        } else {
            msgs.push(format!(
                "{} wins the trick, but gets no points :(",
                self.players[winner_idx].name
            ));
        }
        let new_trick = Trick::new(
            self.trump,
            (0..self.players.len()).map(|offset| {
                let idx = (winner_idx + offset) % self.players.len();
                self.players[idx].id
            }),
        );
        self.last_trick = Some(std::mem::replace(&mut self.trick, new_trick));

        Ok(msgs)
    }

    pub fn finish_game(&self) -> Result<(InitializePhase, Vec<String>), Error> {
        if !self.hands.is_empty() || !self.trick.played_cards().is_empty() {
            bail!("not done playing yet!")
        }

        let mut msgs = vec![];

        let non_landlords_points: usize = self
            .points
            .iter()
            .filter(|(id, _)| !self.landlords_team.contains(id))
            .flat_map(|(_, cards)| cards)
            .flat_map(|c| c.points())
            .sum();
        let point_segments = self.num_decks * 20;
        let landlord_won = non_landlords_points < 2 * point_segments;
        let (landlord_level_bump, non_landlord_level_bump) = if non_landlords_points == 0 {
            (3, 0)
        } else if non_landlords_points < point_segments {
            (2, 0)
        } else if non_landlords_points < 2 * point_segments {
            (1, 0)
        } else if non_landlords_points < 3 * point_segments {
            (0, 0)
        } else if non_landlords_points < 4 * point_segments {
            (0, 1)
        } else if non_landlords_points < 5 * point_segments {
            (0, 2)
        } else {
            (0, 3)
        };
        let mut players = self.players.clone();
        for player in &mut players {
            let bump = if self.landlords_team.contains(&player.id) {
                landlord_level_bump
            } else {
                non_landlord_level_bump
            };
            for _ in 0..bump {
                if let Some(next_level) = player.level.successor() {
                    player.level = next_level;
                }
            }
            if bump > 0 {
                msgs.push(format!(
                    "{} has advanced to rank {}",
                    player.name,
                    player.level.as_str()
                ));
            }
        }

        let landlord_idx = self
            .players
            .iter()
            .position(|p| p.id == self.landlord)
            .unwrap();
        let mut idx = (landlord_idx + 1) % players.len();
        let (next_landlord, next_landlord_idx) = loop {
            if landlord_won == self.landlords_team.contains(&players[idx].id) {
                break (players[idx].id, idx);
            }
            idx = (idx + 1) % players.len()
        };
        msgs.push(format!(
            "{} will start the next game",
            self.players[next_landlord_idx].name
        ));

        Ok((
            InitializePhase {
                game_mode: match self.game_mode {
                    GameMode::Tractor => GameMode::Tractor,
                    GameMode::FindingFriends { num_friends, .. } => GameMode::FindingFriends {
                        num_friends,
                        friends: vec![],
                    },
                },
                kitty_size: Some(self.kitty.len()),
                num_decks: Some(self.num_decks),
                landlord: Some(next_landlord),
                max_player_id: players.iter().map(|p| p.id.0).max().unwrap_or(0),
                players,
            },
            msgs,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePhase {
    num_decks: usize,
    game_mode: GameMode,
    hands: Hands,
    kitty: Vec<Card>,
    kitty_size: usize,
    landlord: PlayerID,
    players: Vec<Player>,
    trump: Trump,
}

impl ExchangePhase {
    pub fn move_card_to_kitty(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.landlord != id {
            bail!("not the landlord")
        }
        self.hands.remove(self.landlord, Some(card))?;
        self.kitty.push(card);
        Ok(())
    }

    pub fn move_card_to_hand(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.landlord != id {
            bail!("not the landlord")
        }
        if let Some(index) = self.kitty.iter().position(|c| *c == card) {
            self.kitty.swap_remove(index);
            self.hands.add(self.landlord, Some(card))?;
            Ok(())
        } else {
            bail!("card not in the kitty")
        }
    }

    pub fn set_friends(
        &mut self,
        id: PlayerID,
        iter: impl IntoIterator<Item = Friend>,
    ) -> Result<(), Error> {
        if self.landlord != id {
            bail!("not the landlord")
        }
        if let GameMode::FindingFriends {
            num_friends,
            ref mut friends,
        } = self.game_mode
        {
            let friend_set = iter.into_iter().collect::<HashSet<_>>();
            if num_friends != friend_set.len() {
                bail!("incorrect number of friends")
            }
            for friend in friend_set.iter() {
                if friend.player_id.is_some() {
                    bail!("you can't pick your friend on purpose")
                }
                if friend.card.is_joker() || friend.card.number() == Some(self.trump.number()) {
                    bail!(
                        "you can't pick a joker or a {} as your friend",
                        self.trump.number().as_str()
                    )
                }
                if friend.skip >= self.num_decks {
                    bail!("need to pick a card that exists!")
                }
            }
            friends.clear();
            friends.extend(friend_set);
            Ok(())
        } else {
            bail!("not playing finding friends")
        }
    }

    pub fn advance(&self, id: PlayerID) -> Result<PlayPhase, Error> {
        if id != self.landlord {
            bail!("only the landlord can advance the game")
        }
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the kitty")
        }
        if let GameMode::FindingFriends {
            num_friends,
            ref friends,
        } = self.game_mode
        {
            if friends.len() != num_friends {
                bail!("need to pick friends")
            }
        }

        let landlord_position = self
            .players
            .iter()
            .position(|p| p.id == self.landlord)
            .unwrap();
        let landlords_team = match self.game_mode {
            GameMode::Tractor => self
                .players
                .iter()
                .enumerate()
                .flat_map(|(idx, p)| {
                    if idx % 2 == landlord_position % 2 {
                        Some(p.id)
                    } else {
                        None
                    }
                })
                .collect(),
            GameMode::FindingFriends { .. } => vec![self.landlord],
        };
        let landlord_idx = self
            .players
            .iter()
            .position(|p| p.id == self.landlord)
            .unwrap();

        Ok(PlayPhase {
            num_decks: self.num_decks,
            game_mode: self.game_mode.clone(),
            hands: self.hands.clone(),
            kitty: self.kitty.clone(),
            trick: Trick::new(
                self.trump,
                (0..self.players.len()).map(|offset| {
                    let idx = (landlord_idx + offset) % self.players.len();
                    self.players[idx].id
                }),
            ),
            last_trick: None,
            points: self.players.iter().map(|p| (p.id, Vec::new())).collect(),
            players: self.players.clone(),
            landlord: self.landlord,
            trump: self.trump,
            landlords_team,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bid {
    id: PlayerID,
    card: Card,
    count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawPhase {
    num_decks: usize,
    game_mode: GameMode,
    deck: Vec<Card>,
    players: Vec<Player>,
    hands: Hands,
    bids: Vec<Bid>,
    position: usize,
    landlord: Option<PlayerID>,
    kitty: Vec<Card>,
    level: Number,
}
impl DrawPhase {
    pub fn draw_card(&mut self, id: PlayerID) -> Result<(), Error> {
        if id != self.players[self.position].id {
            bail!("not your turn!");
        }
        if let Some(next_card) = self.deck.pop() {
            self.hands.add(id, Some(next_card))?;
            self.position = (self.position + 1) % self.players.len();
            Ok(())
        } else {
            bail!("no cards left in deck")
        }
    }

    pub fn valid_bids(&self, id: PlayerID) -> Vec<Bid> {
        // Compute all valid bids.
        if self.bids.last().map(|b| b.id) == Some(id) {
            // If we're the current highest bidder, the only permissible bid is
            // one which is the same as the previous one, but has more cards
            let last_bid = self.bids.last().unwrap();
            let available = self
                .hands
                .counts(id)
                .and_then(|c| c.get(&last_bid.card).cloned())
                .unwrap_or(0);
            (last_bid.count + 1..available + 1)
                .map(|count| Bid {
                    card: last_bid.card,
                    count,
                    id,
                })
                .collect()
        } else if let Some(counts) = self.hands.counts(id) {
            // Construct all the valid bids from the player's hand
            let mut valid_bids = vec![];
            for (card, count) in counts {
                if !card.is_joker() && card.number() != Some(self.level) {
                    continue;
                }
                for inner_count in 1..count + 1 {
                    if card.is_joker() && inner_count == 1 {
                        continue;
                    }
                    let new_bid = Bid {
                        id,
                        card: *card,
                        count: inner_count,
                    };
                    if let Some(existing_bid) = self.bids.last() {
                        if new_bid.count > existing_bid.count {
                            valid_bids.push(new_bid);
                        } else if new_bid.count == existing_bid.count {
                            match (new_bid.card, existing_bid.card) {
                                (Card::BigJoker, Card::SmallJoker)
                                | (Card::BigJoker, Card::Suited { .. })
                                | (Card::SmallJoker, Card::Suited { .. }) => {
                                    valid_bids.push(new_bid);
                                }
                                _ => (),
                            }
                        }
                    } else {
                        valid_bids.push(new_bid);
                    }
                }
            }

            valid_bids
        } else {
            vec![]
        }
    }

    pub fn bid(&mut self, id: PlayerID, card: Card, count: usize) -> bool {
        let new_bid = Bid { id, card, count };
        if self.valid_bids(id).contains(&new_bid) {
            self.bids.push(new_bid);
            true
        } else {
            false
        }
    }

    pub fn advance(&self, id: PlayerID) -> Result<ExchangePhase, Error> {
        if !self.deck.is_empty() {
            bail!("deck has cards remaining")
        } else if self.bids.is_empty() {
            bail!("nobody has bid yet")
        } else {
            let winning_bid = self.bids.last().unwrap();
            let landlord = self.landlord.unwrap_or(winning_bid.id);
            if id != landlord {
                bail!("only the landlord can advance the game");
            }
            let trump = match winning_bid.card {
                Card::Unknown => bail!("can't bid with unknown cards!"),
                Card::SmallJoker | Card::BigJoker => Trump::NoTrump { number: self.level },
                Card::Suited { suit, .. } => Trump::Standard {
                    suit,
                    number: self.level,
                },
            };
            let mut hands = self.hands.clone();
            hands.set_trump(trump);
            Ok(ExchangePhase {
                num_decks: self.num_decks,
                game_mode: self.game_mode.clone(),
                kitty_size: self.kitty.len(),
                kitty: self.kitty.clone(),
                players: self.players.clone(),
                landlord,
                hands,
                trump,
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializePhase {
    max_player_id: usize,
    players: Vec<Player>,
    num_decks: Option<usize>,
    kitty_size: Option<usize>,
    game_mode: GameMode,
    landlord: Option<PlayerID>,
}
impl InitializePhase {
    pub fn new() -> Self {
        Self {
            max_player_id: 0,
            players: Vec::new(),
            kitty_size: None,
            num_decks: None,
            game_mode: GameMode::Tractor,
            landlord: None,
        }
    }

    pub fn add_player(&mut self, name: String) -> PlayerID {
        let id = PlayerID(self.max_player_id);
        self.max_player_id += 1;
        self.players.push(Player {
            id,
            name,
            level: Number::Two,
        });
        self.kitty_size = None;
        id
    }

    pub fn remove_player(&mut self, id: PlayerID) {
        self.players.retain(|p| p.id != id);
        if self.landlord == Some(id) {
            self.landlord = None;
        }
    }

    pub fn reorder_players(&mut self, order: &[PlayerID]) -> Result<(), Error> {
        let uniq = order.iter().cloned().collect::<HashSet<PlayerID>>();
        if uniq.len() != self.players.len() {
            bail!("Incorrect number of players");
        }
        let mut new_players = Vec::with_capacity(self.players.len());
        for id in order {
            match self.players.iter().filter(|p| p.id == *id).next() {
                Some(player) => new_players.push(player.clone()),
                None => bail!("player ID not found"),
            }
        }
        self.players = new_players;
        Ok(())
    }

    pub fn set_num_decks(&mut self, num_decks: usize) {
        if num_decks > 0 {
            self.num_decks = Some(num_decks);
        }
    }

    pub fn set_landlord(&mut self, landlord: Option<PlayerID>) -> Result<(), Error> {
        match landlord {
            Some(landlord) => {
                if self
                    .players
                    .iter()
                    .filter(|p| p.id == landlord)
                    .next()
                    .is_some()
                {
                    self.landlord = Some(landlord)
                } else {
                    bail!("player ID not found")
                }
            }
            None => self.landlord = None,
        }
        Ok(())
    }

    pub fn set_rank(&mut self, player_id: PlayerID, level: Number) -> Result<(), Error> {
        match self.players.iter_mut().filter(|p| p.id == player_id).next() {
            Some(ref mut player) => {
                player.level = level;
            }
            None => bail!("player ID not found"),
        }
        Ok(())
    }

    pub fn set_kitty_size(&mut self, size: usize) -> Result<(), Error> {
        if self.players.is_empty() {
            bail!("no players")
        }
        let deck_len = self.players.len() * FULL_DECK.len();
        if size >= deck_len {
            bail!("kitty size too large")
        }

        if deck_len % self.players.len() != size % self.players.len() {
            bail!("kitty must be a multiple of the remaining cards")
        }
        self.kitty_size = Some(size);
        Ok(())
    }

    pub fn set_game_mode(&mut self, game_mode: GameMode) {
        self.game_mode = game_mode;
    }

    pub fn start(&self) -> Result<DrawPhase, Error> {
        if self.players.len() < 4 {
            bail!("not enough players")
        }

        let game_mode = match self.game_mode {
            GameMode::FindingFriends { num_friends, .. }
                if num_friends > 0 && num_friends <= self.players.len() - 1 =>
            {
                GameMode::FindingFriends {
                    num_friends,
                    friends: vec![],
                }
            }
            GameMode::FindingFriends { .. } => GameMode::FindingFriends {
                num_friends: (self.players.len() / 2) - 1,
                friends: vec![],
            },
            GameMode::Tractor if self.players.len() % 2 == 0 => GameMode::Tractor,
            GameMode::Tractor => bail!("can only play tractor with an even number of players"),
        };

        let num_decks = self.num_decks.unwrap_or(self.players.len() / 2);
        let mut deck = Vec::with_capacity(num_decks * FULL_DECK.len());
        for _ in 0..num_decks {
            deck.extend(FULL_DECK.iter());
        }
        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        let kitty_size = match self.kitty_size {
            Some(size) if deck.len() % self.players.len() == size % self.players.len() => size,
            Some(_) => bail!("kitty size doesn't match player count"),
            None => {
                let mut kitty_size = deck.len() % self.players.len();
                if kitty_size == 0 {
                    kitty_size = self.players.len();
                }
                if kitty_size < 5 {
                    kitty_size += self.players.len();
                }
                kitty_size
            }
        };

        let position = self
            .landlord
            .and_then(|landlord| self.players.iter().position(|p| p.id == landlord))
            .unwrap_or(rng.next_u32() as usize % self.players.len());
        let level = self.players[position].level;

        Ok(DrawPhase {
            deck: (&deck[0..deck.len() - kitty_size]).to_vec(),
            kitty: (&deck[deck.len() - kitty_size..]).to_vec(),
            hands: Hands::new(self.players.iter().map(|p| p.id), level),
            bids: Vec::new(),
            players: self.players.clone(),
            landlord: self.landlord,
            position,
            num_decks,
            game_mode,
            level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::InitializePhase;

    use crate::types::cards;

    #[test]
    fn reinforce_bid() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into());
        let p2 = init.add_player("p2".into());
        let p3 = init.add_player("p3".into());
        let p4 = init.add_player("p4".into());
        let mut draw = init.start().unwrap();
        // Hackily ensure that everyone can bid.
        draw.deck = vec![
            cards::S_2,
            cards::D_2,
            cards::C_2,
            cards::H_2,
            cards::S_2,
            cards::D_2,
            cards::C_2,
            cards::H_2,
        ];
        draw.position = 0;

        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();
        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();

        assert!(draw.bid(p1, cards::H_2, 1));
        assert!(draw.bid(p1, cards::H_2, 2));
    }
}
