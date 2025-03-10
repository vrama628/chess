use ratatui::{
    crossterm::event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
    },
    layout::Flex,
    prelude::*,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    piece::{PieceColor, PieceType},
    position::Position,
    Game, Outcome,
};

pub struct Tui {
    game: Game,
    click_targets: Vec<(Rect, Position)>,
    selected_tile: Option<(Position, BTreeSet<Position>)>,
    selected_promotion: Option<(
        Position,
        Position,
        BTreeMap<ratatui::layout::Position, PieceType>,
    )>,
    last_move: Option<(Position, Position)>,
}

impl Tui {
    pub fn new() -> Self {
        let game = Game::new();
        let click_targets = Vec::new();
        let selected_tile = None;
        let selected_promotion = None;
        let last_move = None;
        Self {
            game,
            click_targets,
            selected_tile,
            selected_promotion,
            last_move,
        }
    }

    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> std::io::Result<Option<Outcome>> {
        let outcome = loop {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            if let Some(outcome) = self.game.status() {
                while !matches!(event::read()?, Event::Key(_)) {}
                break Some(outcome);
            }
            let event = event::read()?;
            if self.handle(event) {
                break None;
            }
        };
        return Ok(outcome);
    }

    /// returns whether to exit
    fn handle(&mut self, event: Event) -> bool {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc | KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => true,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: _,
            }) => {
                let click = ratatui::layout::Position { x: column, y: row };
                if let Some((from, to, ref click_targets)) = self.selected_promotion {
                    if let Some(&piece_type) = click_targets.get(&click) {
                        self.game = self.game.promote(from, to, piece_type);
                        self.last_move = Some((from, to));
                        self.selected_promotion = None;
                        return false;
                    }
                }
                for &(rect, position) in &self.click_targets {
                    if rect.contains(click) {
                        match self.selected_tile {
                            None => self.select_tile(position),
                            Some((from, ref moves)) => {
                                if moves.contains(&position) {
                                    if self.game.is_promotion(from, position) {
                                        // promotion click targets will be populated upon rendering
                                        self.selected_promotion =
                                            Some((from, position, BTreeMap::new()));
                                    } else {
                                        self.game = self.game.r#move(from, position);
                                        self.last_move = Some((from, position));
                                    }
                                    self.selected_tile = None;
                                } else {
                                    self.select_tile(position);
                                }
                            }
                        }
                        return false;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn select_tile(&mut self, position: Position) {
        self.selected_tile = self.game.moves(self.game.turn()).remove_entry(&position);
        self.selected_promotion = None;
    }
}

fn highlight_last_move<'a>(line: Line<'a>) -> Line<'a> {
    let bg = match line.style.bg {
        Some(Color::DarkGray) => Color::Yellow,
        Some(Color::Gray) => Color::LightYellow,
        color => panic!("unexpected background color {color:?}"),
    };
    line.bg(bg)
}

impl Widget for &mut Tui {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [area] = Layout::vertical([8]).flex(Flex::Center).areas(area);
        let [board_area, info_area] = Layout::horizontal([8 * 2, 5])
            .spacing(1)
            .flex(Flex::Center)
            .areas(area);

        // board
        let ranks = Layout::vertical([Constraint::Fill(1); 8]).split(board_area);
        self.click_targets.clear();
        for (rank, rect) in ranks.iter().copied().rev().enumerate() {
            let files = Layout::horizontal([Constraint::Fill(1); 8]).split(rect);
            for (file, rect) in files.iter().copied().enumerate() {
                let position = Position { rank, file };
                let mut line = position.square();
                if self
                    .last_move
                    .is_some_and(|(from, to)| position == from || position == to)
                {
                    line = highlight_last_move(line);
                }
                if let Some(piece) = self.game.get(position) {
                    line.push_span(piece.render())
                } else {
                    line.push_span(" ")
                }
                if self
                    .selected_tile
                    .as_ref()
                    .is_some_and(|(p, _)| *p == position)
                    || self
                        .selected_promotion
                        .as_ref()
                        .is_some_and(|(from, to, _)| *from == position || *to == position)
                {
                    line.push_span(Span::raw("●").fg(Color::LightYellow))
                }
                if self
                    .selected_tile
                    .as_ref()
                    .is_some_and(|(_, moves)| moves.contains(&position))
                {
                    line.push_span(Span::raw("○").fg(Color::LightGreen))
                }
                line.render(rect, buf);
                self.click_targets.push((rect, position));
            }
        }

        // info
        let [black_turn_area, promotion_area, white_turn_area] = Layout::vertical([2, 1, 2])
            .flex(Flex::SpaceBetween)
            .areas(info_area);
        let turn = self.game.turn();
        let turn_area = match turn {
            PieceColor::White => white_turn_area,
            PieceColor::Black => black_turn_area,
        };
        let mut text = Text::default();
        let turn_span = Span::raw(turn.to_string())
            .fg(turn.render())
            .bg(Color::Gray);
        text.push_span(turn_span);
        if self.game.check(turn) {
            let check_line = Line::raw("check").bg(Color::LightRed).fg(Color::Gray);
            text.push_line(check_line);
        }
        text.render(turn_area, buf);

        // promotion
        if let Some((_, _, click_targets)) = &mut self.selected_promotion {
            click_targets.clear();
            for (area, piece) in promotion_area.columns().zip([
                PieceType::Queen,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
            ]) {
                piece
                    .render()
                    .fg(turn.render())
                    .bg(Color::Gray)
                    .render(area, buf);
                click_targets.insert(area.as_position(), piece);
            }
        }
    }
}
