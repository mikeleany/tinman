//! Module to implement a chess game
//
//  Copyright 2020 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
use std::slice::SliceIndex;
use std::ops::Index;
use std::iter::FusedIterator;
use std::iter::FromIterator;
use std::sync::Arc;
use std::time::Duration;
use std::convert::TryInto;
use std::collections::HashMap;
use super::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A structure to represent a sequence of moves and resulting positions
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveSequence {
    moves: Vec<ArcMove>,        // TODO: make this Vec<DetachedMove>
    final_pos: Arc<Position>,   // TODO: make this positions: Vec<Position>
}

impl MoveSequence {
    /// Constructs an empty `MoveSequence` starting at the standard starting position.
    pub fn new() -> MoveSequence {
        MoveSequence {
            moves: Vec::new(),
            final_pos: Arc::new(Position::new()),
        }
    }

    /// Constructs an empty `MoveSequence` starting at the given initial position.
    pub fn starting_at(initial_pos: Arc<Position>) -> MoveSequence {
        MoveSequence {
            moves: Vec::new(),
            final_pos: initial_pos,
        }
    }

    /// Shortens the move sequence, keeping the first `len` Moves and dropping the rest.
    ///
    /// If `len` is greater than the move sequence's current length, this has no effect.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.final_pos = self.moves[len].position_arc().clone();
            self.moves.truncate(len);
        }
    }

    /// Extracts a slice containing the entire move sequence.
    pub fn as_slice(&self) -> &[ArcMove] {
        self.moves.as_slice()
    }

    /// Adds a move onto the end of the move sequence and returns the resulting position.
    ///
    /// # Errors
    ///
    /// Returns an error if `mv.position()` is not the same as `self.final_position()` or if
    /// `mv.make_arc()` returns and error.
    pub fn push(&mut self, mv: ArcMove) -> Result<&Arc<Position>> {
        if mv.position() == self.final_pos.as_ref() {
            self.final_pos = mv.make_arc()?;
            self.moves.push(mv);

            Ok(&self.final_pos)
        } else {
            Err(Error::MovePositionMismatch)
        }
    }

    /// Removes the last move from the move sequence and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<ArcMove> {
        if let Some(mv) = self.moves.pop() {
            self.final_pos = mv.position_arc().clone();

            Some(mv)
        } else {
            None
        }
    }

    /// Moves all moves in `other` to `self`, leaving other empty.
    ///
    /// # Errors
    ///
    /// Returns an error if `other.initial_position()` is not the same as `self.final_position()`.
    pub fn append(&mut self, other: &mut MoveSequence) -> Result<&Arc<Position>> {
        if other.initial_position().as_ref() == self.final_pos.as_ref() {
            self.moves.append(&mut other.moves);
            self.final_pos = other.final_position().clone();

            Ok(&self.final_pos)
        } else {
            Err(Error::MovePositionMismatch)
        }
    }

    /// Removes all moves from the move sequence, leaving only the initial position.
    pub fn clear(&mut self) {
        if !self.is_empty() {
            self.final_pos = self.moves[0].position_arc().clone();
            self.moves.clear();
        }
    }

    /// Returns the number of moves in the move sequence.
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Returns `true` if the move sequence contains no moves.
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Returns the first move in the sequence.
    pub fn first(&self) -> Option<&ArcMove> {
        self.moves.first()
    }

    /// Returns the last move in the sequence.
    pub fn last(&self) -> Option<&ArcMove> {
        self.moves.last()
    }

    /// Returns the initial position of the move sequence.
    pub fn initial_position(&self) -> &Arc<Position> {
        if !self.is_empty() {
            self.moves[0].position_arc()
        } else {
            &self.final_pos
        }
    }

    /// Returns the final position of the move sequence.
    pub fn final_position(&self) -> &Arc<Position> {
        &self.final_pos
    }

    /// Returns a reference to a move or subslice of moves, depending on the type of index.
    ///
    /// - If given an integer, retrns a reference to the move at that location, or `None` if out of
    ///   bounds
    /// - If given a range, returns the subslice corresponding to that range, or `None` if out of
    ///   bounds.
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[ArcMove]>>::Output>
        where I: SliceIndex<[ArcMove]> {
        self.moves.get(index)
    }

    /// Returns a reference to the position at `index` or `None` if out of bounds.
    ///
    /// Note that an index of `self.len()` is in bounds and will return the final position, which
    /// is the result of the last move.
    pub fn position(&self, index: usize) -> Option<&Arc<Position>> {
        use std::cmp::Ordering;

        match index.cmp(&self.len()) {
            Ordering::Less => Some(&self.moves[index].position_arc()),
            Ordering::Equal => Some(&self.final_pos),
            Ordering::Greater => None,
        }
    }

    /// Returns an iterator over the move sequence.
    pub fn iter(&self) -> Iter {
        self.moves.iter()
    }

    /// Returns an iterator over the positions in the move sequence, from the intial position up to
    /// and including the final position.
    pub fn positions(&self) -> Positions {
        Positions {
            moves: self.moves.iter(),
            final_pos: Some(&self.final_pos),
        }
    }

    /// Returns true if the sequence ends in the repetition of an earlier position.
    ///
    /// In other words, returns true if `self.final_position()` matches another position within
    /// the sequence.
    pub fn repetition(&self) -> bool {
        let key = self.final_pos.zobrist_key();

        for m in self.moves.iter() {
            if key == m.position().zobrist_key() {
                return true;
            }
        }

        false
    }

    /// Returns true if the sequence ends in three-fold repetition.
    ///
    /// In other words, returns true if `self.final_position()` matches at least two other
    /// positions within the sequence.
    pub fn three_fold_repetition(&self) -> bool {
        let key = self.final_pos.zobrist_key();

        let mut count = 1;
        for m in self.moves.iter() {
            if key == m.position().zobrist_key() {
                count += 1;
                if count >= 3 {
                    return true;
                }
            }
        }

        false
    }
}

impl IntoIterator for MoveSequence {
    type Item = ArcMove;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter()
    }
}

impl TryFrom<ArcMove> for MoveSequence {
    type Error = Error;

    fn try_from(mv: ArcMove) -> Result<Self> {
        let final_pos = Arc::new(mv.make()?);

        Ok(MoveSequence {
            moves: vec![ mv ],
            final_pos,
        })
    }
}

impl FromIterator<ArcMove> for Result<MoveSequence> {
    fn from_iter<I: IntoIterator<Item=ArcMove>>(iter: I) -> Self {
        let mut iter = iter.into_iter();

        let mut seq = if let Some(mv) = iter.next() {
            mv.try_into()?
        } else {
            MoveSequence::new()
        };

        for mv in iter {
            seq.push(mv)?;
        }

        Ok(seq)
    }
}

impl<I> Index<I> for MoveSequence where I: SliceIndex<[ArcMove]> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.moves[index]
    }
}

impl fmt::Display for MoveSequence {
    /// The move is formatted as follows:
    ///
    /// "{}" -- Space delimited sequence of SAN (eg e4 e5 Nf3)
    ///
    /// "{:+}" -- As formatted in PGN (eg 1. e4 e5 2. Nf3)
    ///
    /// "{:#}" -- A space delimited sequence in Coordinate Notation (eg e2e4 e7e5 g1f3)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();

        if f.alternate() {
            for mv in self.iter() {
                s += &format!("{:#} ", mv);
            }
        } else {
            for mv in self.iter() {
                if f.sign_plus() && mv.position().turn() == Color::White {
                    s += &format!("{}. ", mv.position().move_number());
                }
                s += &format!("{} ", mv);
            }
        }
        s.pop(); // remove space from end of string

        s.fmt(f)
    }
}

impl FromStr for MoveSequence {
    type Err = Error;

    fn from_str(s: &str) -> Result<MoveSequence> {
        let re = regex::Regex::new(r"\s+|\s*\d+\s*\.\s*").expect("regex error");
        let mut seq = MoveSequence::new();

        for mv in re.split(s) {
            if mv.is_empty()
                || mv.starts_with('*')
                || mv.starts_with("1-0")
                || mv.starts_with("0-1")
                || mv.starts_with("1/2-1/2") {
                continue;
            }

            let pos = Arc::clone(seq.final_position());
            let mv = mv.parse::<MoveBuilder>()?.validate(&pos)?;
            seq.push(mv.into())?;
        }

        Ok(seq)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Iterator over the moves in a MoveSequence
pub type Iter<'a>=std::slice::Iter<'a, ArcMove>;

////////////////////////////////////////////////////////////////////////////////////////////////////
/// An iterator over the positions in the move sequence, from the intial position up to and
/// including the final position.
#[derive(Debug, Clone)]
pub struct Positions<'a> {
    moves: Iter<'a>,
    final_pos: Option<&'a Arc<Position>>,
}

impl<'a> Iterator for Positions<'a> {
    type Item = &'a Arc<Position>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mv) = self.moves.next() {
            Some(mv.position_arc())
        } else if let Some(pos) = self.final_pos {
            self.final_pos = None;

            Some(pos)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.final_pos.is_some() {
            let size = self.moves.len() + 1;

            (size, Some(size))
        } else {
            self.moves.size_hint()
        }
    }
}

impl DoubleEndedIterator for Positions<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.final_pos {
            self.final_pos = None;

            Some(pos)
        } else if let Some(mv) = self.moves.next() {
            Some(mv.position_arc())
        } else {
            None
        }
    }
}

impl ExactSizeIterator for Positions<'_> { }
impl FusedIterator for Positions<'_> { }

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Time controls for a game
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeControl {
    /// Time is not limited. Typically the engine should continue searching until told to stop or
    /// a maximum depth has been reached.
    Infinite,
    /// Each player has a fixed time to play the entire game.
    SuddenDeath(Duration),
    /// Each player begins with `base` time, which is incremented by `inc` each time the player
    /// makes a move.
    Incremental{
        /// The amount of time each player has at the beginning of the game.
        base: Duration,
        /// The amount of time added to each player's time after each move.
        inc: Duration,
    },
    /// Each player begins with `base` time, and each session of `mps` moves, `base` gets added
    /// to each player's remaining time.
    Session{
        /// The mount of time each player has at the beginning of the game, and the amount added on
        /// after each session.
        base: Duration,
        /// The number of moves per session
        mps: usize,
    },
    /// Each player must make each move in the specified number of seconds.
    Exact(Duration),
}

impl Default for TimeControl {
    fn default() -> Self {
        TimeControl::Infinite
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Chess clock for a game
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Clock {
    remaining: [ Duration; Color::COUNT ],
    tc: TimeControl,
}

impl Clock {
    /// Creates a new clock with the given time control
    pub fn new(tc: TimeControl) -> Self {
        use TimeControl::*;

        let remaining = match tc {
            Infinite => Duration::default(),
            SuddenDeath(base) => base,
            Incremental{ base, .. } => base,
            Session{ base, .. } => base,
            Exact(base) => base,
        };

        Clock {
            remaining: [ remaining, remaining ],
            tc,
        }
    }

    /// Returns the time control associated with this clock.
    pub fn time_control(&self) -> TimeControl {
        self.tc
    }

    /// Get the remaining time for `color`.
    pub fn remaining(&self, color: Color) -> Duration {
        self.remaining[color as usize]
    }

    /// Update the clock based on the `elapsed` time and the time control being used.
    pub fn update(&mut self, color: Color, elapsed: Duration, moves: usize) -> bool {
        if let Some(remaining) = self.remaining[color as usize].checked_sub(elapsed) {
            self.remaining[color as usize] = remaining;
        } else {
            self.remaining[color as usize] = Duration::from_secs(0);
            return false; // no time remaining
        }

        match self.tc {
            TimeControl::Incremental{ inc, .. } => self.remaining[color as usize] += inc,
            TimeControl::Session{ base, mps } => {
                if moves % mps == 0 {
                    self.remaining[color as usize] += base;
                }
            },
            _ => { }
        }

        true
    }

    /// Sets the remaining time for `color` to `time`.
    pub fn set(&mut self, color: Color, time: Duration) {
        self.remaining[color as usize] = time;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// The result of a game
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameResult {
    /// The given color has won.
    Win(Color, Option<WinReason>),
    /// The game has ended in a draw.
    Draw(Option<DrawReason>),
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameResult::Win(Color::White, Some(reason)) => format!("1-0 {{{}}}", reason).fmt(f),
            GameResult::Win(Color::White, None) => "1-0".fmt(f),
            GameResult::Win(Color::Black, Some(reason)) => format!("0-1 {{{}}}", reason).fmt(f),
            GameResult::Win(Color::Black, None) => "0-1".fmt(f),
            GameResult::Draw(Some(reason)) => format!("1/2-1/2 {{{}}}", reason).fmt(f),
            GameResult::Draw(None) => "1/2-1/2".fmt(f),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Win reason
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WinReason {
    /// The game was won by checkmate.
    Checkmate,
    /// One player resigned.
    Resignation,
    /// One player's time ran out.
    Time,
    /// The opponent forfeits.
    Forfeiture
}

impl fmt::Display for WinReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WinReason::Checkmate => "checkmate".fmt(f),
            WinReason::Resignation => "by resignation".fmt(f),
            WinReason::Time => "time expired".fmt(f),
            WinReason::Forfeiture => "by forfeiture".fmt(f),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Draw reason
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DrawReason {
    /// Stalemate
    Stalemate,
    /// Draw by the fifty-move rule
    FiftyMoves,
    /// Draw by three-fold repetition
    Repetition,
    /// Insufficient material
    Material,
    /// Draw by agreement
    Agreement,
}

impl fmt::Display for DrawReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawReason::Stalemate => "stalemate".fmt(f),
            DrawReason::FiftyMoves => "fifty-move rule".fmt(f),
            DrawReason::Repetition => "repetition".fmt(f),
            DrawReason::Material => "insufficient material".fmt(f),
            DrawReason::Agreement => "by agreement".fmt(f),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// A chess game
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Game {
    clock: Clock,
    moves: MoveSequence,
    result: Option<GameResult>,
}

impl Game {
    /// Creates a new game from the standard starting position
    pub fn new() -> Self {
        Game::default()
    }

    /// Creates a new game using `pos` as the starting position
    pub fn starting_at(pos: Position) -> Self {
        Game {
            moves: MoveSequence::starting_at(Arc::new(pos)),
            ..Default::default()
        }
    }

    /// Sets the time control for the game. Default is `Infinite`.
    pub fn set_time_control(&mut self, tc: TimeControl) -> &mut Self {
        self.clock = Clock::new(tc);

        self
    }

    /// Sets the result of the game.
    pub fn set_result(&mut self, result: GameResult) {
        self.result = Some(result);
    }

    /// Returns a reference counted pointer to the current position
    pub fn position(&self) -> &Arc<Position> {
        self.moves.final_position()
    }

    /// Returns the game's position history
    pub fn history(&self) -> &MoveSequence {
        &self.moves
    }

    /// Returns the time remaining for `color`
    pub fn time_remaining(&self, color: Color) -> Duration {
        self.clock.remaining(color)
    }

    /// Returns the game's clock
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// Returns a mutable reference to the game's clock
    pub fn clock_mut(&mut self) -> &mut Clock {
        &mut self.clock
    }

    /// Returns the result of the game, or `None` if the game isn't over
    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    /// Make the given move
    pub fn make_move(&mut self, mv: ArcMove) -> Result<&mut Self> {
        self.moves.push(mv)?;

        self.check_game_result();

        Ok(self)
    }

    /// Make the given move and (if successful) update the clock based on `elapsed` time and the
    /// game's time control.
    pub fn make_move_timed(&mut self, mv: ArcMove, elapsed: Duration) -> Result<&mut Self> {
        let color = mv.color();

        self.make_move(mv)?;
        self.clock.update(color, elapsed, (self.moves.len() + 1)/2);

        Ok(self)
    }

    /// Make the given move
    pub fn make_move_from_str(&mut self, mv: &str) -> Result<&mut Self> {
        let mv: ArcMove = MoveBuilder::from_str(mv)?
            .validate(self.position())?
            .into();

        self.make_move(mv)
    }

    /// Make the given move and (if successful) update the clock based on `elapsed` time and the
    /// game's time control.
    pub fn make_move_from_str_timed(&mut self, mv: &str, elapsed: Duration) -> Result<&mut Self> {
        let mv: ArcMove = MoveBuilder::from_str(mv)?
            .validate(self.position())?
            .into();

        self.make_move_timed(mv, elapsed)
    }

    /// Undoes the last move. Returns false if there are no moves to undo.
    pub fn undo(&mut self) -> bool {
        self.moves.pop().is_some()
    }

    /// Checks if the game is over, and sets `self.result` appropriately
    fn check_game_result(&mut self) {
        if self.position().fifty_moves() {
            self.result = Some(GameResult::Draw(Some(DrawReason::FiftyMoves)));
            return;
        } else if self.position().insufficient_material() {
            self.result = Some(GameResult::Draw(Some(DrawReason::Material)));
            return;
        } else if self.moves.three_fold_repetition() {
            self.result = Some(GameResult::Draw(Some(DrawReason::Repetition)));
            return;
        }

        for mv in self.position().moves() {
            if mv.make().is_ok() {
                self.result = None;
                return;
            }
        }

        // no legal moves if we reach this point
        if self.position().in_check() {
            self.result = Some(GameResult::Win(
                !self.position().turn(),
                Some(WinReason::Checkmate)
            ));
        } else {
            self.result = Some(GameResult::Draw(Some(DrawReason::Stalemate)));
        }
    }

    /// Returns a PGN representation of the game, using the given tags.
    ///
    /// This method will add a "Result" tag, and, if needed, the "SetUp" and "FEN" tags to the
    /// given tags.
    pub fn to_pgn(&self, tags: &HashMap<String, String>) -> String {
        let mut tags = tags.to_owned();

        if self.moves.initial_position().zobrist_key() != Position::default().zobrist_key() {
            tags.insert("SetUp".to_owned(), "1".to_owned());
            tags.insert("FEN".to_owned(), self.moves.initial_position().to_string());
        }

        let result = match self.result {
            Some(GameResult::Win(Color::White, _)) => "1-0",
            Some(GameResult::Win(Color::Black, _)) => "0-1",
            Some(GameResult::Draw(_)) => "1/2-1/2",
            _ => "*",
        };
        tags.insert("Result".to_owned(), result.to_owned());

        let mut tag_list = String::new();
        for &name in &["Event", "Site", "Date", "Round", "White", "Black", "Result"] {
            let value = match (name, tags.remove(name)) {
                (_, Some(value)) => value,
                ("Date", None) => "????.??.??".to_owned(),
                (_, None) => "?".to_owned(),
            };

            tag_list += &format!("[{} \"{}\"]\n", name, value);
        }

        let mut names: Vec<_> = tags.keys().collect();
        names.sort_unstable();

        for name in names {
            tag_list += &format!("[{} \"{}\"]\n", name, tags[name]);
        }

        let result = if let Some(result) = self.result {
            result.to_string()
        } else {
            "*".to_owned()
        };

        let mut move_text = String::new();
        let mut width = 0;
        for word in format!("{:+} {}", self.moves, result).split(' ') {
            // split into lines of no more than than 80 BYTES each
            if width + word.len() < 80 && width > 0 {
                move_text += " ";
                width += 1;
            } else if width > 0{
                move_text += "\n";
                width = 0;
            }
            move_text += word;
            width += word.len();
        }

        let pgn = format!("{}\n{}\n", tag_list, move_text);

        pgn
    }
}
