use std::{cell::RefCell, rc::Rc};

use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key, MouseEvent},
    theme::{BaseColor, Color, ColorStyle},
    view::CannotFocus,
    views::{Dialog, Panel, SelectView},
    Cursive, Printer, Vec2,
};
use rand::seq::SliceRandom;
use shakmaty::{Chess, Color as CColor, Position, Rank, Role, Square};

struct BoardView {
    board: Chess,
    focused: Option<Square>,
    highlighted: Option<Square>,
    rng: rand::rngs::ThreadRng,
    promotion: Rc<RefCell<Option<Role>>>,
}

impl BoardView {
    pub fn new() -> Self {
        let board = Chess::default();

        BoardView {
            board,
            focused: None,
            highlighted: None,
            rng: rand::thread_rng(),
            promotion: Rc::new(RefCell::new(None)),
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

        fn game_over(siv: &mut Cursive, msg: &str) {
            siv.pop_layer();
            siv.add_layer(Dialog::info(msg))
        }

        if self.board.is_checkmate() {
            return Some(EventResult::with_cb(|s| {
                game_over(s, "Game Over. You win.")
            }));
        } else if self.board.is_game_over() {
            return Some(EventResult::with_cb(|s| game_over(s, "Game Over.")));
        };

        let legals = self.board.legal_moves();
        let cpu_move = legals.choose(&mut self.rng).unwrap();

        self.board.play_unchecked(cpu_move);

        if self.board.is_checkmate() {
            return Some(EventResult::with_cb(|s| {
                game_over(s, "Game Over. I win. Hahaha.")
            }));
        } else if self.board.is_game_over() {
            return Some(EventResult::with_cb(|s| game_over(s, "Game Over.")));
        };

        None
    }

    fn process_focus_change(&mut self, sq: Square) -> EventResult {
        match self.focused {
            None if self.board.us().contains(sq) => {
                self.focused = Some(sq);


                if sq.rank() == Rank::Seventh && self.board.board().role_at(sq) == Some(Role::Pawn)
                {
                    let p = self.promotion.clone();
                    EventResult::with_cb(move |s| {
                        let p = p.clone();
                        s.add_layer(
                            Dialog::new().content(
                                SelectView::new()
                                    .item("Queen", Role::Queen)
                                    .item("Rook", Role::Rook)
                                    .item("Bishop", Role::Bishop)
                                    .item("Knight", Role::Knight)
                                    .on_submit(move |s, &piece| {
                                        s.pop_layer();
                                        *p.borrow_mut() = Some(piece);
                                    }),
                            ),
                        );
                    })
                } else {
                    *self.promotion.borrow_mut() = None;
                    EventResult::Consumed(None)
                }
            }

            Some(from) => {
                let input_move = self.board.legal_moves().into_iter().find(|m| {
                    m.from() == Some(from)
                        && m.to() == sq
                        && if self.promotion.borrow().is_some() {
                            m.promotion() == *self.promotion.borrow()
                        } else {
                            true
                        }
                });


                match input_move.and_then(|mv| self.move_and_reply(mv)) {
                    Some(event_result) => event_result,
                    None => {
                        self.focused = None;
                        EventResult::Consumed(None)
                    }
                }
            }
            _ => EventResult::Ignored,
        }
    }
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer) {
        for file in 0..8 {
            for rank in 0..8 {
                let x = file * 3;
                let y = 7 - rank;

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
                } else if self.highlighted == Some(sq) {
                    Color::Light(BaseColor::Yellow)
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
            // Mouse Input
            Event::Mouse {
                offset,
                position,
                event: MouseEvent::Press(_),
            } => {
                if let Some(sq) = self.get_sq(position, offset) {
                    self.process_focus_change(sq)
                } else {
                    EventResult::Ignored
                }
            }

            // Keyboard Input
            Event::Key(Key::Left | Key::Right | Key::Up | Key::Down) | Event::Char(' ')
                if self.highlighted.is_none() =>
            {
                self.highlighted = Some(Square::A1);
                EventResult::Consumed(None)
            }
            Event::Char(' ') => self.process_focus_change(self.highlighted.unwrap()),
            Event::Key(key) => {
                let sq = self.highlighted.unwrap();
                match key {
                    Key::Right => {
                        self.highlighted = sq.offset(1);
                        EventResult::Consumed(None)

                    }
                    Key::Left => {
                        self.highlighted = sq.offset(-1);
                        EventResult::Consumed(None)
                    }

                    Key::Up => {
                        self.highlighted = sq.offset(8);
                        EventResult::Consumed(None)
                    }
                    Key::Down => {
                        self.highlighted = sq.offset(-8);
                        EventResult::Consumed(None)
                    }
                    _ => EventResult::Ignored,
                }
            }
            _ => EventResult::Ignored,
        }
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
                    .item_str("Chess")
                    .item_str("Atomic")
                    .on_submit(|s, option: &str| {
                        s.pop_layer();
                        match option {
                            "Chess" => new_game(s),
                            _ => s.add_layer(Dialog::info("Coming soon")),
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
            .content(Panel::new(BoardView::new()))
            .button("Quit Game", |s| {
                s.pop_layer();
            }),
    );

    siv.add_layer(Dialog::info(
        "Controls:
Click with the mouse on the piece you want to move,
then click on the square you want to move it to.
Or use Arrows and Space.",
    ));
}
