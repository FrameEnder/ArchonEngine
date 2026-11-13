//! Triple Triad's rules: a match on the 3x3 board, each card's four numbers, the captures, and the special rules
//! (TripleTriadRule rows: Roulette 1, All Open 2, Three Open 3, Sudden Death 4, Reverse 5, Fallen Ace 6, Same 7,
//! Plus 8, Ascension 9, Descension 10, Order 11, Chaos 12, Swap 13, Random 14, Draft 15.) See
//! `docs/src/fork/gold-saucer.md` (Triple Triad).
//!
//! The rules are written after FFTriadBuddy's game simulation (github.com/MgAl2O4/FFTriadBuddy, `sources/gamelogic`:
//! `TriadGameSimulation.cs`, `TriadGameModifier.cs`), which its author checked against the game for years: MIT
//! License, Copyright (c) 2019 MgAl2O4. Its notice: "Permission is hereby granted, free of charge, to any person
//! obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software
//! without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute,
//! sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
//! subject to the following conditions: The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND."
//!
//! What it takes from there: a card captures a neighbour of the other side whose facing number is lower; Same (two or
//! more neighbours' facing numbers equal to the card's, at least one of the other side's: those are taken) and Plus
//! (two or more neighbours whose sums with the card's numbers match: the other side's among them are taken) work on the
//! card just placed only, and what they take goes on to capture as a plain card would (the combo), with Reverse still
//! in force; Reverse takes on lower; Fallen Ace lets a 1 take a 10 (with Reverse, a 10 can't take a 1); Ascension and
//! Descension raise or lower every card of a type by one for each of its type on the board (between 1 and 10); Order
//! plays the hand in order; the match is counted with the unplayed card of the side that went second, and Sudden Death
//! replays a draw (up to three times) with the cards each side holds at its end.
//!
//! The NPC's choice of move (`best_move`) is Kawari's: the move that leaves it the most cards, nearest first. Retail's
//! isn't known. Nothing calls this from a connection yet: retail's packets for a match aren't recorded.

/// A card as the game has it (TripleTriadCardResident): its numbers, top, right, bottom and left (1-10, 10 the "A"),
/// and its type (TripleTriadCardType: 0 none, then Primal, Scion, Beastman, Garlean.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub id: u32,
    pub numbers: [u8; 4],
    pub kind: u8,
}

pub const TOP: usize = 0;
pub const RIGHT: usize = 1;
pub const BOTTOM: usize = 2;
pub const LEFT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The player (blue.)
    Player,
    /// The NPC (red.)
    Npc,
}

impl Side {
    pub fn other(self) -> Self {
        match self {
            Side::Player => Side::Npc,
            Side::Npc => Side::Player,
        }
    }
}

/// TripleTriadRule rows.
pub mod rule {
    pub const ROULETTE: u8 = 1;
    pub const ALL_OPEN: u8 = 2;
    pub const THREE_OPEN: u8 = 3;
    pub const SUDDEN_DEATH: u8 = 4;
    pub const REVERSE: u8 = 5;
    pub const FALLEN_ACE: u8 = 6;
    pub const SAME: u8 = 7;
    pub const PLUS: u8 = 8;
    pub const ASCENSION: u8 = 9;
    pub const DESCENSION: u8 = 10;
    pub const ORDER: u8 = 11;
    pub const CHAOS: u8 = 12;
    pub const SWAP: u8 = 13;
    pub const RANDOM: u8 = 14;
    pub const DRAFT: u8 = 15;
    /// What Roulette may become: every rule but itself, the open rules and Draft (FFTriadBuddy's list.)
    pub const ROULETTE_CHOICES: [u8; 11] = [
        SUDDEN_DEATH,
        REVERSE,
        FALLEN_ACE,
        SAME,
        PLUS,
        ASCENSION,
        DESCENSION,
        ORDER,
        CHAOS,
        SWAP,
        RANDOM,
    ];
}

/// A card on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub card: Card,
    pub owner: Side,
    /// Ascension's or Descension's change to its numbers.
    pub modifier: i8,
}

impl Placed {
    /// Its number on a side, with Ascension or Descension (never below 1 or above 10.)
    pub fn number(&self, side: usize) -> i8 {
        (self.card.numbers[side] as i8 + self.modifier).clamp(1, 10)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// It's this side's turn.
    Turn(Side),
    Won(Side),
    Draw,
}

/// A match.
#[derive(Debug, Clone)]
pub struct Match {
    pub board: [Option<Placed>; 9],
    pub hands: [Vec<Option<Card>>; 2],
    pub rules: Vec<u8>,
    pub outcome: Outcome,
    /// Who went first (the other side's unplayed card counts for it at the end.)
    pub first: Side,
    /// Ascension's and Descension's count by card type.
    type_counts: [i8; 8],
    /// How many times Sudden Death has replayed it.
    pub replays: u8,
}

/// The board's neighbours of a square (0-8, row by row), by the side of the card there that faces them.
pub fn neighbours(square: usize) -> [Option<usize>; 4] {
    let (x, y) = (square % 3, square / 3);
    [
        (y > 0).then(|| square - 3),
        (x < 2).then(|| square + 1),
        (y < 2).then(|| square + 3),
        (x > 0).then(|| square - 1),
    ]
}

fn index(side: Side) -> usize {
    match side {
        Side::Player => 0,
        Side::Npc => 1,
    }
}

impl Match {
    /// A new match: each side's five cards, the rules in force (Roulette already resolved), who goes first.
    pub fn new(player: [Card; 5], npc: [Card; 5], rules: Vec<u8>, first: Side) -> Self {
        Self {
            board: [None; 9],
            hands: [
                player.iter().copied().map(Some).collect(),
                npc.iter().copied().map(Some).collect(),
            ],
            rules,
            outcome: Outcome::Turn(first),
            first,
            type_counts: [0; 8],
            replays: 0,
        }
    }

    pub fn has(&self, rule: u8) -> bool {
        self.rules.contains(&rule)
    }

    /// The cards (their places in the hand) a side may play now: Order, its first card left; otherwise any.
    pub fn playable(&self, side: Side) -> Vec<usize> {
        let hand = &self.hands[index(side)];
        let left: Vec<usize> = (0..hand.len()).filter(|i| hand[*i].is_some()).collect();
        if self.has(rule::ORDER) {
            left.into_iter().take(1).collect()
        } else {
            left
        }
    }

    pub fn empty_squares(&self) -> Vec<usize> {
        (0..9).filter(|s| self.board[*s].is_none()).collect()
    }

    /// Plays a side's card (its place in the hand) on a square. False if it isn't that side's turn, the card isn't
    /// playable or the square is taken.
    pub fn play(&mut self, side: Side, card: usize, square: usize) -> bool {
        if self.outcome != Outcome::Turn(side)
            || square >= 9
            || self.board[square].is_some()
            || !self.playable(side).contains(&card)
        {
            return false;
        }
        let Some(played) = self.hands[index(side)][card].take() else {
            return false;
        };
        let mut placed = Placed {
            card: played,
            owner: side,
            modifier: 0,
        };
        if self.has(rule::ASCENSION) || self.has(rule::DESCENSION) {
            placed.modifier = self.type_modifier(played.kind);
        }
        self.board[square] = Some(placed);

        let mut chain = self.special_captures(square);
        self.plain_captures(square, true, &mut Vec::new());
        // What Same or Plus took captures on as a plain card would, and what that takes too.
        while !chain.is_empty() {
            let mut next = Vec::new();
            for taken in chain {
                self.plain_captures(taken, false, &mut next);
            }
            chain = next;
        }
        if played.kind != 0 && (self.has(rule::ASCENSION) || self.has(rule::DESCENSION)) {
            self.count_type(played.kind);
        }

        self.outcome = if self.board.iter().all(Option::is_some) {
            self.count()
        } else {
            Outcome::Turn(side.other())
        };
        if self.outcome == Outcome::Draw && self.has(rule::SUDDEN_DEATH) && self.replays < 3 {
            self.replay();
        }
        true
    }

    fn type_modifier(&self, kind: u8) -> i8 {
        if kind == 0 {
            return 0;
        }
        let count = self.type_counts[kind as usize];
        if self.has(rule::ASCENSION) {
            count
        } else {
            -count
        }
    }

    /// A card of `kind` came onto the board: every card of its type goes up (Ascension) or down (Descension) by one.
    fn count_type(&mut self, kind: u8) {
        self.type_counts[kind as usize] += 1;
        let modifier = self.type_modifier(kind);
        for placed in self.board.iter_mut().flatten() {
            if placed.card.kind == kind {
                placed.modifier = modifier;
            }
        }
    }

    /// Same and Plus, for the card just placed: what they take (to go on capturing.)
    fn special_captures(&mut self, square: usize) -> Vec<usize> {
        let Some(card) = self.board[square] else {
            return Vec::new();
        };
        let facing: Vec<(usize, Placed, i8, i8)> = neighbours(square)
            .iter()
            .enumerate()
            .filter_map(|(side, n)| {
                let n = (*n)?;
                let other = self.board[n]?;
                Some((n, other, card.number(side), other.number((side + 2) % 4)))
            })
            .collect();
        let mut taken = Vec::new();
        if self.has(rule::SAME) {
            let same: Vec<&(usize, Placed, i8, i8)> = facing
                .iter()
                .filter(|(_, _, mine, theirs)| mine == theirs)
                .collect();
            if same.len() >= 2 {
                for (n, other, _, _) in same {
                    if other.owner != card.owner && !taken.contains(n) {
                        taken.push(*n);
                    }
                }
            }
        }
        if self.has(rule::PLUS) {
            for (i, (n, other, mine, theirs)) in facing.iter().enumerate() {
                let matched = facing
                    .iter()
                    .enumerate()
                    .any(|(j, (_, _, m, t))| j != i && m + t == mine + theirs);
                if matched && other.owner != card.owner && !taken.contains(n) {
                    taken.push(*n);
                }
            }
        }
        for n in &taken {
            if let Some(other) = self.board[*n].as_mut() {
                other.owner = card.owner;
            }
        }
        taken
    }

    /// A card takes its neighbours of the other side whose facing number is lower (Reverse: higher; Fallen Ace: a 1
    /// takes a 10, with Reverse a 10 can't take a 1.) Fallen Ace counts only for the card just placed, Reverse for the
    /// whole chain (FFTriadBuddy's order.)
    fn plain_captures(&mut self, square: usize, placed_now: bool, chain: &mut Vec<usize>) {
        let Some(card) = self.board[square] else {
            return;
        };
        let reverse = self.has(rule::REVERSE);
        let fallen_ace = placed_now && self.has(rule::FALLEN_ACE);
        for (side, n) in neighbours(square).iter().enumerate() {
            let Some(n) = *n else { continue };
            let Some(other) = self.board[n] else { continue };
            if other.owner == card.owner {
                continue;
            }
            let (mut mine, mut theirs) = (card.number(side), other.number((side + 2) % 4));
            if fallen_ace {
                if reverse && mine == 10 && theirs == 1 {
                    mine = 0;
                } else if !reverse && mine == 1 && theirs == 10 {
                    theirs = 0;
                }
            }
            let captured = if reverse {
                mine < theirs
            } else {
                mine > theirs
            };
            if captured {
                if let Some(other) = self.board[n].as_mut() {
                    other.owner = card.owner;
                }
                if !placed_now {
                    chain.push(n);
                }
            }
        }
    }

    /// The count at the end: each side's cards on the board, and the unplayed card of the side that went second.
    fn count(&self) -> Outcome {
        let mut player = self
            .board
            .iter()
            .flatten()
            .filter(|p| p.owner == Side::Player)
            .count();
        if self.hands[0].iter().any(Option::is_some) {
            player += 1;
        }
        match player {
            6.. => Outcome::Won(Side::Player),
            5 => Outcome::Draw,
            _ => Outcome::Won(Side::Npc),
        }
    }

    /// Sudden Death: each side takes the cards it holds on the board (and its unplayed one) and they play again; the
    /// side that played fewer goes first.
    fn replay(&mut self) {
        let mut hands: [Vec<Option<Card>>; 2] = [Vec::new(), Vec::new()];
        for placed in self.board.iter().flatten() {
            hands[index(placed.owner)].push(Some(placed.card));
        }
        for (i, hand) in self.hands.iter().enumerate() {
            hands[i].extend(hand.iter().flatten().map(|card| Some(*card)));
        }
        let first = self.first.other();
        self.board = [None; 9];
        self.hands = hands;
        self.type_counts = [0; 8];
        self.first = first;
        self.outcome = Outcome::Turn(first);
        self.replays += 1;
    }

    /// How many cards a side holds now (on the board, and in hand for the side that went second.)
    pub fn holds(&self, side: Side) -> usize {
        self.board
            .iter()
            .flatten()
            .filter(|p| p.owner == side)
            .count()
    }
}

/// Kawari's NPC: the move (card in hand, square) that leaves it holding the most cards after it, the first of equals.
pub fn best_move(game: &Match, side: Side) -> Option<(usize, usize)> {
    let mut best: Option<((usize, usize), usize)> = None;
    for card in game.playable(side) {
        for square in game.empty_squares() {
            let mut trial = game.clone();
            if !trial.play(side, card, square) {
                continue;
            }
            let held = match trial.outcome {
                Outcome::Won(winner) if winner == side => 100,
                _ => trial.holds(side),
            };
            if best.is_none_or(|(_, most)| held > most) {
                best = Some(((card, square), held));
            }
        }
    }
    best.map(|(found, _)| found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: u32, numbers: [u8; 4], kind: u8) -> Card {
        Card { id, numbers, kind }
    }

    fn weak(id: u32) -> Card {
        card(id, [1, 1, 1, 1], 0)
    }

    fn game(rules: Vec<u8>, player: [Card; 5], npc: [Card; 5]) -> Match {
        Match::new(player, npc, rules, Side::Player)
    }

    #[test]
    fn a_higher_number_takes() {
        let mut g = game(
            vec![],
            [card(1, [5, 5, 5, 5], 0), weak(2), weak(3), weak(4), weak(5)],
            [
                card(6, [4, 4, 4, 4], 0),
                weak(7),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        assert!(g.play(Side::Npc, 0, 4) == false, "not its turn");
        assert!(g.play(Side::Player, 1, 0));
        // The NPC's 4s beside it: its left 4 against the player's right 1.
        assert!(g.play(Side::Npc, 0, 1));
        assert_eq!(g.board[0].unwrap().owner, Side::Npc);
        // The player's 5 below the NPC's card: its top 5 against the bottom 4.
        assert!(g.play(Side::Player, 0, 4));
        assert_eq!(g.board[1].unwrap().owner, Side::Player);
        assert_eq!(g.board[0].unwrap().owner, Side::Npc, "not next to it");
    }

    #[test]
    fn same_and_its_combo() {
        // Squares 0 and 2 the NPC's, 3 the NPC's below 0; the player's card at 1 matches 0 and 2 (Same), and 0 then
        // takes 3 (the combo.)
        let mut g = game(
            vec![rule::SAME],
            [card(1, [9, 3, 1, 3], 0), weak(2), weak(3), weak(4), weak(5)],
            [
                card(6, [1, 3, 9, 1], 0),
                card(7, [1, 1, 1, 3], 0),
                card(8, [2, 1, 1, 1], 0),
                weak(9),
                weak(10),
            ],
        );
        g.outcome = Outcome::Turn(Side::Npc);
        assert!(g.play(Side::Npc, 0, 0));
        assert!(g.play(Side::Player, 4, 8));
        assert!(g.play(Side::Npc, 1, 2));
        assert!(g.play(Side::Player, 3, 7));
        assert!(g.play(Side::Npc, 2, 3));
        assert!(g.play(Side::Player, 0, 1));
        let owners: Vec<Side> = [0, 2, 3]
            .iter()
            .map(|s| g.board[*s].unwrap().owner)
            .collect();
        assert_eq!(owners, vec![Side::Player; 3]);
    }

    #[test]
    fn plus_matches_sums() {
        // Placed at 4: up 3 + 0's bottom 4 = 7, left 2 + 3's right 5 = 7.
        let mut g = game(
            vec![rule::PLUS],
            [card(1, [3, 1, 1, 2], 0), weak(2), weak(3), weak(4), weak(5)],
            [
                card(6, [1, 1, 4, 1], 0),
                card(7, [1, 5, 1, 1], 0),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        g.outcome = Outcome::Turn(Side::Npc);
        assert!(g.play(Side::Npc, 0, 1));
        assert!(g.play(Side::Player, 4, 8));
        assert!(g.play(Side::Npc, 1, 3));
        assert!(g.play(Side::Player, 0, 4));
        assert_eq!(g.board[1].unwrap().owner, Side::Player);
        assert_eq!(g.board[3].unwrap().owner, Side::Player);
    }

    #[test]
    fn reverse_and_fallen_ace() {
        let mut g = game(
            vec![rule::REVERSE],
            [card(1, [1, 1, 1, 1], 0), weak(2), weak(3), weak(4), weak(5)],
            [
                card(6, [5, 5, 5, 5], 0),
                weak(7),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        g.outcome = Outcome::Turn(Side::Npc);
        assert!(g.play(Side::Npc, 0, 0));
        assert!(g.play(Side::Player, 0, 1));
        assert_eq!(g.board[0].unwrap().owner, Side::Player, "lower takes");

        let mut g = game(
            vec![rule::FALLEN_ACE],
            [card(1, [1, 1, 1, 1], 0), weak(2), weak(3), weak(4), weak(5)],
            [
                card(6, [10, 10, 10, 10], 0),
                weak(7),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        g.outcome = Outcome::Turn(Side::Npc);
        assert!(g.play(Side::Npc, 0, 0));
        assert!(g.play(Side::Player, 0, 1));
        assert_eq!(g.board[0].unwrap().owner, Side::Player, "a 1 takes an A");
    }

    #[test]
    fn ascension_raises_a_type() {
        let mut g = game(
            vec![rule::ASCENSION],
            [
                card(1, [5, 5, 5, 5], 2),
                card(2, [5, 5, 5, 5], 2),
                weak(3),
                weak(4),
                weak(5),
            ],
            [
                card(6, [5, 5, 5, 5], 0),
                weak(7),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        assert!(g.play(Side::Player, 0, 8));
        assert_eq!(g.board[8].unwrap().number(TOP), 6);
        assert!(g.play(Side::Npc, 0, 0));
        // The second Primal comes on as 6s and both go to 7: its 7 takes the NPC's 5.
        assert!(g.play(Side::Player, 1, 1));
        assert_eq!(g.board[1].unwrap().number(LEFT), 7);
        assert_eq!(g.board[8].unwrap().number(TOP), 7);
        assert_eq!(g.board[0].unwrap().owner, Side::Player);
    }

    #[test]
    fn order_plays_the_hand_in_order() {
        let g = game(
            vec![rule::ORDER],
            [weak(1), weak(2), weak(3), weak(4), weak(5)],
            [weak(6); 5],
        );
        assert_eq!(g.playable(Side::Player), vec![0]);
        assert_eq!(
            game(vec![], [weak(1); 5], [weak(6); 5])
                .playable(Side::Player)
                .len(),
            5
        );
    }

    /// Nine 1s: nothing captures, so the player (first) has 5 on the board and no card left: a draw; Sudden Death
    /// replays it with the NPC first.
    #[test]
    fn a_draw_and_sudden_death() {
        let play_out = |g: &mut Match| {
            while let Outcome::Turn(side) = g.outcome {
                let (card, square) = (g.playable(side)[0], g.empty_squares()[0]);
                assert!(g.play(side, card, square));
                if g.replays > 0 && g.empty_squares().len() == 9 {
                    break;
                }
            }
        };
        let mut g = game(vec![], [weak(1); 5], [weak(6); 5]);
        play_out(&mut g);
        assert_eq!(g.outcome, Outcome::Draw);
        let mut g = game(vec![rule::SUDDEN_DEATH], [weak(1); 5], [weak(6); 5]);
        play_out(&mut g);
        assert_eq!(g.replays, 1);
        assert_eq!(g.outcome, Outcome::Turn(Side::Npc));
        assert_eq!(g.hands[0].iter().flatten().count(), 5);
    }

    #[test]
    fn the_npc_takes_what_it_can() {
        let mut g = game(
            vec![],
            [weak(1); 5],
            [
                card(6, [9, 9, 9, 9], 0),
                weak(7),
                weak(8),
                weak(9),
                weak(10),
            ],
        );
        assert!(g.play(Side::Player, 0, 4));
        let (card, square) = best_move(&g, Side::Npc).unwrap();
        assert_eq!(card, 0);
        assert!([1, 3, 5, 7].contains(&square), "beside the player's card");
    }
}
