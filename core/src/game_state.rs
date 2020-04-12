use std::collections::{HashMap, HashSet};

use anyhow::{bail, Error};
use rand::{seq::SliceRandom, RngCore};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::hands::Hands;
use crate::trick::Trick;
use crate::types::{Card, Number, PlayerID, Trump, FULL_DECK};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageVariant {
    ResettingGame,
    StartingGame,
    TrickWon {
        winner: PlayerID,
        points: usize,
    },
    RankAdvanced {
        player: PlayerID,
        new_rank: Number,
    },
    NewLandlordForNextGame {
        landlord: PlayerID,
    },
    PointsInKitty {
        points: usize,
        multiplier: usize,
    },
    JoinedGame {
        player: PlayerID,
    },
    JoinedTeam {
        player: PlayerID,
    },
    LeftGame {
        name: String,
    },
    KittySizeSet {
        size: Option<usize>,
    },
    NumDecksSet {
        num_decks: Option<usize>,
    },
    NumFriendsSet {
        num_friends: Option<usize>,
    },
    GameModeSet {
        game_mode: GameModeSettings,
    },
    TookBackPlay,
    PlayedCards {
        player_name: String,
        cards: Vec<Card>,
    },
    SetDefendingPointVisibility {
        visible: bool,
    },
    SetLandlord {
        landlord: Option<PlayerID>,
    },
    SetRank {
        rank: Number,
    },
    MadeBid {
        card: Card,
        count: usize,
    },
}

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameModeSettings {
    Tractor,
    FindingFriends { num_friends: Option<usize> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagatedState {
    game_mode: GameModeSettings,
    hide_landlord_points: Option<bool>,
    kitty_size: Option<usize>,
    num_decks: Option<usize>,
    max_player_id: usize,
    players: Vec<Player>,
    observers: Vec<Player>,
    landlord: Option<PlayerID>,
    chat_link: Option<String>,
}

impl PropagatedState {
    pub fn set_game_mode(
        &mut self,
        game_mode: GameModeSettings,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.game_mode = game_mode;
        Ok(vec![MessageVariant::GameModeSet { game_mode }])
    }

    fn num_players_changed(&mut self) -> Result<Vec<MessageVariant>, Error> {
        let mut msgs = vec![];
        msgs.extend(self.set_num_decks(None)?);

        if let GameModeSettings::FindingFriends {
            ref mut num_friends,
            ..
        } = self.game_mode
        {
            if num_friends.is_some() {
                *num_friends = None;
                msgs.push(MessageVariant::GameModeSet {
                    game_mode: self.game_mode,
                });
            }
        }
        Ok(msgs)
    }

    pub fn add_player(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        let id = PlayerID(self.max_player_id);
        if self.players.iter().any(|p| p.name == name)
            || self.observers.iter().any(|p| p.name == name)
        {
            bail!("player with name already exists!")
        }

        let mut msgs = vec![MessageVariant::JoinedGame { player: id }];

        self.max_player_id += 1;
        self.players.push(Player {
            id,
            name,
            level: Number::Two,
        });

        msgs.extend(self.num_players_changed()?);
        Ok((id, msgs))
    }

    pub fn reorder_players(&mut self, order: &[PlayerID]) -> Result<(), Error> {
        let uniq = order.iter().cloned().collect::<HashSet<PlayerID>>();
        if uniq.len() != self.players.len() {
            bail!("Incorrect number of players");
        }
        let mut new_players = Vec::with_capacity(self.players.len());
        for id in order {
            match self.players.iter().find(|p| p.id == *id) {
                Some(player) => new_players.push(player.clone()),
                None => bail!("player ID not found"),
            }
        }
        self.players = new_players;
        Ok(())
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        let id = PlayerID(self.max_player_id);
        if self.players.iter().any(|p| p.name == name)
            || self.observers.iter().any(|p| p.name == name)
        {
            bail!("player with name already exists!")
        }

        self.max_player_id += 1;
        self.observers.push(Player {
            id,
            name,
            level: Number::Two,
        });
        Ok(id)
    }

    pub fn remove_player(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.players.iter().find(|p| p.id == id).cloned() {
            let mut msgs = vec![MessageVariant::LeftGame { name: player.name }];
            if self.landlord == Some(id) {
                self.landlord = None;
            }
            self.players.retain(|p| p.id != id);
            msgs.extend(self.num_players_changed()?);
            Ok(msgs)
        } else {
            bail!("player not found")
        }
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.observers.retain(|p| p.id != id);
        Ok(())
    }

    pub fn set_chat_link(&mut self, chat_link: Option<String>) -> Result<(), Error> {
        if chat_link.as_ref().map(|link| link.len()).unwrap_or(0) >= 128 {
            bail!("link too long");
        }
        if let Some(ref chat_link) = chat_link {
            if let Ok(u) = Url::parse(&chat_link) {
                if u.scheme() != "https" {
                    bail!("must be https URL")
                }
            } else {
                bail!("Invalid URL")
            }
        }
        self.chat_link = chat_link;
        Ok(())
    }

    pub fn set_num_decks(
        &mut self,
        num_decks: Option<usize>,
    ) -> Result<Vec<MessageVariant>, Error> {
        if num_decks == Some(0) {
            bail!("At least one deck is necessary to play the game")
        }
        let mut msgs = vec![];
        if self.num_decks != num_decks {
            msgs.push(MessageVariant::NumDecksSet { num_decks });
            self.num_decks = num_decks;
            msgs.extend(self.set_kitty_size(None)?);
        }
        Ok(msgs)
    }

    pub fn set_kitty_size(
        &mut self,
        kitty_size: Option<usize>,
    ) -> Result<Option<MessageVariant>, Error> {
        if self.kitty_size == kitty_size {
            return Ok(None);
        }
        if let Some(size) = kitty_size {
            if self.players.is_empty() {
                bail!("no players")
            }
            let deck_len = self.num_decks.unwrap_or(self.players.len() / 2) * FULL_DECK.len();
            if size >= deck_len {
                bail!("kitty size too large")
            }

            if deck_len % self.players.len() != size % self.players.len() {
                bail!("kitty must be a multiple of the remaining cards")
            }
            self.kitty_size = Some(size);
        } else {
            self.kitty_size = None;
        }
        Ok(Some(MessageVariant::KittySizeSet {
            size: self.kitty_size,
        }))
    }

    pub fn set_landlord(&mut self, landlord: Option<PlayerID>) -> Result<(), Error> {
        match landlord {
            Some(landlord) => {
                if self.players.iter().any(|p| p.id == landlord) {
                    self.landlord = Some(landlord)
                } else {
                    bail!("player ID not found")
                }
            }
            None => self.landlord = None,
        }
        Ok(())
    }

    pub fn hide_landlord_points(&mut self, should_hide: bool) {
        if should_hide {
            self.hide_landlord_points = Some(true);
        } else {
            self.hide_landlord_points = None;
        }
    }

    pub fn make_observer(&mut self, player_id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.players.iter().find(|p| p.id == player_id).cloned() {
            self.players.retain(|p| p.id != player_id);
            self.observers.push(player);
            self.num_players_changed()
        } else {
            bail!("player not found")
        }
    }

    pub fn make_player(&mut self, player_id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.observers.iter().find(|p| p.id == player_id).cloned() {
            self.observers.retain(|p| p.id != player_id);
            self.players.push(player);
            self.num_players_changed()
        } else {
            bail!("player not found")
        }
    }

    pub fn make_all_observers_into_players(&mut self) -> Result<Vec<MessageVariant>, Error> {
        if self.observers.is_empty() {
            return Ok(vec![]);
        }
        let mut msgs = vec![];
        while let Some(player) = self.observers.pop() {
            msgs.push(MessageVariant::JoinedGame { player: player.id });
            self.players.push(player);
        }
        msgs.extend(self.num_players_changed()?);
        Ok(msgs)
    }

    pub fn set_rank(&mut self, player_id: PlayerID, level: Number) -> Result<(), Error> {
        match self.players.iter_mut().find(|p| p.id == player_id) {
            Some(ref mut player) => {
                player.level = level;
            }
            None => bail!("player not found"),
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Friend {
    card: Card,
    skip: usize,
    player_id: Option<PlayerID>,
}

#[allow(clippy::large_enum_variant)]
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
            GameState::Initialize(p) => Some(&p.propagated.players),
            GameState::Draw(p) => Some(&p.propagated.players),
            GameState::Exchange(p) => Some(&p.propagated.players),
            GameState::Play(p) => Some(&p.propagated.players),
            GameState::Done => None,
        }
    }

    pub fn observers(&self) -> Option<&'_ [Player]> {
        match self {
            GameState::Draw(p) => Some(&p.propagated.observers),
            GameState::Exchange(p) => Some(&p.propagated.observers),
            GameState::Play(p) => Some(&p.propagated.observers),
            GameState::Initialize(p) => Some(&p.propagated.observers),
            GameState::Done => None,
        }
    }

    pub fn is_player(&self, id: PlayerID) -> bool {
        self.players()
            .map(|p| p.iter().any(|pp| pp.id == id))
            .unwrap_or(false)
    }

    pub fn player_name(&self, id: PlayerID) -> Result<&'_ str, Error> {
        if let Some(players) = self.players() {
            for p in players {
                if p.id == id {
                    return Ok(&p.name);
                }
            }
        }
        if let Some(observers) = self.observers() {
            for p in observers {
                if p.id == id {
                    return Ok(&p.name);
                }
            }
        }
        bail!("Couldn't find player name")
    }

    pub fn player_id(&self, name: &str) -> Result<PlayerID, Error> {
        if let Some(players) = self.players() {
            for p in players {
                if p.name == name {
                    return Ok(p.id);
                }
            }
        }
        if let Some(observers) = self.observers() {
            for p in observers {
                if p.name == name {
                    return Ok(p.id);
                }
            }
        }
        bail!("Couldn't find player id")
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

    pub fn register(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        if let Ok(pid) = self.player_id(&name) {
            return Ok((pid, vec![]));
        }
        match self {
            GameState::Initialize(ref mut p) => p.add_player(name),
            GameState::Draw(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Exchange(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Play(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Done => bail!("Game is done"),
        }
    }

    pub fn kick(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(ref mut p) => p.remove_player(id),
            GameState::Draw(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Exchange(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Play(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Done => bail!("Game is done"),
        }
    }

    pub fn set_chat_link(&mut self, chat_link: Option<String>) -> Result<(), Error> {
        match self {
            GameState::Initialize(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Draw(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Exchange(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Play(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Done => bail!("Game is done"),
        }
    }

    pub fn reset(&mut self) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(_) => bail!("Game has not started yet!"),
            GameState::Draw(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
            GameState::Exchange(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
            GameState::Play(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
            GameState::Done => bail!("Game is done"),
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
                ref mut points,
                ref trick,
                ref landlords_team,
                ref propagated,
                landlord,
                ..
            }) => {
                if propagated.hide_landlord_points.unwrap_or(false) {
                    for (k, v) in points.iter_mut() {
                        if landlords_team.contains(&k) {
                            v.clear();
                        }
                    }
                }
                hands.redact_except(id);
                // Don't redact at the end of the game.
                let game_ongoing = !hands.is_empty() || !trick.played_cards().is_empty();
                if game_ongoing && id != landlord {
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
    propagated: PropagatedState,
    hands: Hands,
    points: HashMap<PlayerID, Vec<Card>>,
    kitty: Vec<Card>,
    landlord: PlayerID,
    landlords_team: Vec<PlayerID>,
    trump: Trump,
    trick: Trick,
    last_trick: Option<Trick>,
}

impl PlayPhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

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

    pub fn finish_trick(&mut self) -> Result<Vec<MessageVariant>, Error> {
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
                                        for player in &self.propagated.players {
                                            if player.id == played.id {
                                                msgs.push(MessageVariant::JoinedTeam {
                                                    player: player.id,
                                                });
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
        let kitty_points = self
            .kitty
            .iter()
            .filter(|c| c.points().is_some())
            .copied()
            .collect::<Vec<_>>();

        if self.hands.is_empty() {
            for _ in 0..kitty_multipler {
                new_points.extend(kitty_points.iter().copied());
            }
            if !kitty_points.is_empty() && kitty_multipler > 0 {
                msgs.push(MessageVariant::PointsInKitty {
                    points: kitty_points.iter().flat_map(|c| c.points()).sum::<usize>(),
                    multiplier: kitty_multipler,
                });
            }
        }
        let winner_idx = self
            .propagated
            .players
            .iter()
            .position(|p| p.id == winner)
            .unwrap();
        if !new_points.is_empty() {
            let trump = self.trump;
            let num_points = new_points.iter().flat_map(|c| c.points()).sum::<usize>();
            points.extend(new_points);
            points.sort_by(|a, b| trump.compare(*a, *b));
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: num_points,
            });
        } else {
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: 0,
            });
        }
        let new_trick = Trick::new(
            self.trump,
            (0..self.propagated.players.len()).map(|offset| {
                let idx = (winner_idx + offset) % self.propagated.players.len();
                self.propagated.players[idx].id
            }),
        );
        self.last_trick = Some(std::mem::replace(&mut self.trick, new_trick));

        Ok(msgs)
    }

    pub fn game_finished(&self) -> bool {
        self.hands.is_empty() && self.trick.played_cards().is_empty()
    }

    pub fn finish_game(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        if !self.game_finished() {
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
        let mut propagated = self.propagated.clone();
        for player in &mut propagated.players {
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
                msgs.push(MessageVariant::RankAdvanced {
                    player: player.id,
                    new_rank: player.level,
                });
            }
        }

        let landlord_idx = self
            .propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord)
            .unwrap();
        let mut idx = (landlord_idx + 1) % propagated.players.len();
        let (next_landlord, next_landlord_idx) = loop {
            if landlord_won == self.landlords_team.contains(&propagated.players[idx].id) {
                break (propagated.players[idx].id, idx);
            }
            idx = (idx + 1) % propagated.players.len()
        };

        msgs.push(MessageVariant::NewLandlordForNextGame {
            landlord: self.propagated.players[next_landlord_idx].id,
        });
        propagated.set_landlord(Some(next_landlord))?;
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePhase {
    propagated: PropagatedState,
    num_decks: usize,
    game_mode: GameMode,
    hands: Hands,
    kitty: Vec<Card>,
    kitty_size: usize,
    landlord: PlayerID,
    trump: Trump,
}

impl ExchangePhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

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
            bail!("only the leader can advance the game")
        }
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the bottom")
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
            .propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord)
            .unwrap();
        let landlords_team = match self.game_mode {
            GameMode::Tractor => self
                .propagated
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
            .propagated
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
                (0..self.propagated.players.len()).map(|offset| {
                    let idx = (landlord_idx + offset) % self.propagated.players.len();
                    self.propagated.players[idx].id
                }),
            ),
            last_trick: None,
            points: self
                .propagated
                .players
                .iter()
                .map(|p| (p.id, Vec::new()))
                .collect(),
            landlord: self.landlord,
            trump: self.trump,
            propagated: self.propagated.clone(),
            landlords_team,
        })
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
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
    propagated: PropagatedState,
    hands: Hands,
    bids: Vec<Bid>,
    position: usize,
    kitty: Vec<Card>,
    level: Number,
}
impl DrawPhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn draw_card(&mut self, id: PlayerID) -> Result<(), Error> {
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

    #[allow(clippy::comparison_chain)]
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
            (last_bid.count + 1..=available)
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
                for inner_count in 1..=*count {
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
            let landlord = self.propagated.landlord.unwrap_or(winning_bid.id);
            if id != landlord {
                bail!("only the leader can advance the game");
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
                propagated: self.propagated.clone(),
                landlord,
                hands,
                trump,
            })
        }
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializePhase {
    propagated: PropagatedState,
}

impl InitializePhase {
    pub fn new() -> Self {
        Self {
            propagated: PropagatedState {
                max_player_id: 0,
                players: Vec::new(),
                kitty_size: None,
                num_decks: None,
                game_mode: GameModeSettings::Tractor,
                landlord: None,
                hide_landlord_points: None,
                observers: vec![],
                chat_link: None,
            },
        }
    }

    pub fn add_player(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        self.propagated.add_player(name)
    }

    pub fn remove_player(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        self.propagated.remove_player(id)
    }

    pub fn make_observer(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        self.propagated.make_observer(id)
    }

    pub fn make_player(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        self.propagated.make_player(id)
    }

    pub fn reorder_players(&mut self, order: &[PlayerID]) -> Result<(), Error> {
        self.propagated.reorder_players(order)
    }

    pub fn set_num_decks(
        &mut self,
        num_decks: Option<usize>,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.propagated.set_num_decks(num_decks)
    }

    pub fn hide_landlord_points(&mut self, should_hide: bool) {
        self.propagated.hide_landlord_points(should_hide)
    }

    pub fn set_landlord(&mut self, landlord: Option<PlayerID>) -> Result<(), Error> {
        self.propagated.set_landlord(landlord)
    }

    pub fn set_rank(&mut self, player_id: PlayerID, level: Number) -> Result<(), Error> {
        self.propagated.set_rank(player_id, level)
    }

    pub fn set_kitty_size(&mut self, size: Option<usize>) -> Result<Option<MessageVariant>, Error> {
        self.propagated.set_kitty_size(size)
    }

    pub fn set_game_mode(
        &mut self,
        game_mode: GameModeSettings,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.propagated.set_game_mode(game_mode)
    }

    pub fn start(&self) -> Result<DrawPhase, Error> {
        if self.propagated.players.len() < 4 {
            bail!("not enough players")
        }

        let game_mode = match self.propagated.game_mode {
            // Always override the number of friends for finding friends
            // TODO: consider exposing this in the future properly...
            GameModeSettings::FindingFriends {
                num_friends: Some(num_friends),
                ..
            } if num_friends + 1 < self.propagated.players.len() => GameMode::FindingFriends {
                num_friends,
                friends: vec![],
            },
            GameModeSettings::FindingFriends { .. } => GameMode::FindingFriends {
                num_friends: (self.propagated.players.len() / 2) - 1,
                friends: vec![],
            },
            GameModeSettings::Tractor if self.propagated.players.len() % 2 == 0 => {
                GameMode::Tractor
            }
            GameModeSettings::Tractor => {
                bail!("can only play tractor with an even number of players")
            }
        };

        let num_decks = self
            .propagated
            .num_decks
            .unwrap_or(self.propagated.players.len() / 2);
        let mut deck = Vec::with_capacity(num_decks * FULL_DECK.len());
        for _ in 0..num_decks {
            deck.extend(FULL_DECK.iter());
        }
        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        let kitty_size = match self.propagated.kitty_size {
            Some(size)
                if deck.len() % self.propagated.players.len()
                    == size % self.propagated.players.len() =>
            {
                size
            }
            Some(_) => bail!("kitty size doesn't match player count"),
            None => {
                let mut kitty_size = deck.len() % self.propagated.players.len();
                if kitty_size == 0 {
                    kitty_size = self.propagated.players.len();
                }
                if kitty_size < 5 {
                    kitty_size += self.propagated.players.len();
                }
                kitty_size
            }
        };

        let position = self
            .propagated
            .landlord
            .and_then(|landlord| {
                self.propagated
                    .players
                    .iter()
                    .position(|p| p.id == landlord)
            })
            .unwrap_or(rng.next_u32() as usize % self.propagated.players.len());
        let level = self.propagated.players[position].level;

        Ok(DrawPhase {
            deck: (&deck[0..deck.len() - kitty_size]).to_vec(),
            kitty: (&deck[deck.len() - kitty_size..]).to_vec(),
            hands: Hands::new(self.propagated.players.iter().map(|p| p.id), level),
            propagated: self.propagated.clone(),
            bids: Vec::new(),
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
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
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
