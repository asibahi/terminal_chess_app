use cursive::{
    direction::Direction,
    event::{Callback, Event, EventResult, MouseEvent},
    theme::{BaseColor, Color, ColorStyle},
    view::CannotFocus,
    views::{Dialog, LinearLayout, Panel, SelectView},
    Cursive, Printer, Vec2,
};
use rand::seq::SliceRandom;
use shakmaty::{Chess, Color as CColor, Position, Role, Square};

struct BoardView {
    board: Chess,
    focused: Option<Square>,
    rng: rand::rngs::ThreadRng,
}

impl BoardView {
    pub fn new() -> Self {
        let board = Chess::default();

        BoardView {
            board,
            focused: None,
            rng: rand::thread_rng(),
        }
    }

    fn get_sq(&self, mouse_pos: Vec2, offset: Vec2) -> Option<Square> {
        mouse_pos
            .checked_sub(offset)
            .map(|pos| pos.map_x(|x| x / 3))
            .and_then(|pos| {
                if pos.fits_in(Vec2::new(8, 8)) {
                    Some(Square::new((pos.x + 8 * (7 - pos.y)).try_into().unwrap()))
                } else {
                    None
                }
            })
    }

    fn move_and_reply(&mut self, mv: shakmaty::Move) -> Option<EventResult> {
        self.board.play_unchecked(&mv);

        if self.board.is_checkmate() {
            return Some(EventResult::Consumed(Some(Callback::from_fn(|s| {
                game_over(s, "Game Over. You win.")
            }))));
        } else if self.board.is_game_over() {
            return Some(EventResult::Consumed(Some(Callback::from_fn(|s| {
                game_over(s, "Game Over.")
            }))));
        };

        let legals = self.board.legal_moves();
        let cpu_move = legals.choose(&mut self.rng).unwrap();

        self.board.play_unchecked(cpu_move);

        if self.board.is_checkmate() {
            return Some(EventResult::Consumed(Some(Callback::from_fn(|s| {
                game_over(s, "Game Over. I win. Hahaha.")
            }))));
        } else if self.board.is_game_over() {
            return Some(EventResult::Consumed(Some(Callback::from_fn(|s| {
                game_over(s, "Game Over.")
            }))));
        };

        None
    }
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer) {
        for file in 0..8 {
            for rank in 0..8 {
                let y = 7 - rank;
                let x = file * 3;

                let sq = Square::new(file + 8 * rank);

                let text = match self.board.board().piece_at(sq) {
                    Some(p) => {
                        let symbol = piece_to_char(p);
                        format!(" {} ", symbol)
                    }
                    None => "   ".to_owned(),
                };

                let color = if self.focused == Some(sq) {
                    Color::Dark(BaseColor::Yellow)
                } else if sq.is_dark() {
                    Color::RgbLowRes(1, 1, 1)
                } else {
                    Color::RgbLowRes(4, 4, 4)
                };

                printer.with_color(
                    ColorStyle::new(Color::Dark(BaseColor::Black), color),
                    |printer| printer.print((x, y), &text),
                );
            }
        }
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Mouse {
                offset,
                position,
                event: MouseEvent::Press(_),
            } => match self.focused {
                None => {
                    if let Some(sq) = self.get_sq(position, offset) {
                        if self.board.us().contains(sq) {
                            self.focused = Some(sq);
                            return EventResult::Consumed(None);
                        }
                    }
                }
                Some(from) => {
                    if let Some(to) = self.get_sq(position, offset) {
                        let input_move = self
                            .board
                            .legal_moves()
                            .into_iter()
                            .find(|m| m.from() == Some(from) && m.to() == to);

                        if let Some(event_result) =
                            input_move.and_then(|mv| self.move_and_reply(mv))
                        {
                            return event_result;
                        }

                        self.focused = None;
                        return EventResult::Consumed(None);
                    }
                }
            },
            _ => (),
        }

        EventResult::Ignored
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2::new(8, 8).map_x(|x| 3 * x)
    }
}

fn piece_to_char(p: shakmaty::Piece) -> char {
    match (p.color, p.role) {
        (CColor::Black, Role::Pawn) => '\u{265F}',
        (CColor::Black, Role::Knight) => '\u{265E}',
        (CColor::Black, Role::Bishop) => '\u{265D}',
        (CColor::Black, Role::Rook) => '\u{265C}',
        (CColor::Black, Role::Queen) => '\u{265B}',
        (CColor::Black, Role::King) => '\u{265A}',
        (CColor::White, Role::Pawn) => '\u{2659}',
        (CColor::White, Role::Knight) => '\u{2658}',
        (CColor::White, Role::Bishop) => '\u{2657}',
        (CColor::White, Role::Rook) => '\u{2656}',
        (CColor::White, Role::Queen) => '\u{2655}',
        (CColor::White, Role::King) => '\u{2654}',
    }
}

pub fn show_options(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::new()
            .title("Select Variant")
            .content(
                SelectView::new()
                    .item("Chess", "Chess")
                    .item("Racing Kings", "Racing Kings")
                    .on_submit(|s, option: &str| {
                        s.pop_layer();
                        if option == "Chess" {
                            new_game(s)
                        } else {
                            s.add_layer(Dialog::info("Coming soon"))
                        };
                    }),
            )
            .dismiss_button("Back"),
    );
}

fn new_game(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::new()
            .title("Chess")
            .content(LinearLayout::horizontal().child(Panel::new(BoardView::new())))
            .button("Quit game", |s| {
                s.pop_layer();
            }),
    );

    siv.add_layer(Dialog::info(
        "Controls:
Click with the mouse on the piece you want to move,
then click on the square you want to move it to.",
    ));
}

fn game_over(siv: &mut Cursive, msg: &str) {
    siv.pop_layer();
    siv.add_layer(Dialog::info(msg))
}
