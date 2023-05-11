use cursive::views::{Button, Dialog, LinearLayout};

mod logic;

fn main() {
    let mut siv = cursive::default();

    siv.add_layer(
        Dialog::new()
            .title("Chess")
            .padding_lrtb(2, 2, 1, 1)
            .content(
                LinearLayout::vertical()
                    .child(Button::new_raw("New game", logic::show_options))
                    .child(Button::new_raw("Rules", |s| {
                        s.add_layer(Dialog::info("You probably know how to play!").title("Rules"))
                    }))
                    .child(Button::new_raw("Exit", |s| s.quit())),
            ),
    );

    siv.run();
}
