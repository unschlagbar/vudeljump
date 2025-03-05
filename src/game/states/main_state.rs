use iron_oxide::{graphics::formats::RGBA, ui::{Align::{self}, Button, Inline, Overflow, Padding, Position, Style, Text, UIUnit::{self, Pixel, Zero}, UiElement, UiSize::{self}, UiState}};


pub fn build_main() -> UiState {
    let style = Style::new(Align::Top, Zero, Pixel(10.0), UiSize::Size(UIUnit::Relative(0.3)), UiSize::Size(UIUnit::RelativeWidth(0.1)), RGBA::new(0, 0, 0, 120), RGBA::GREEN, 2.0, Pixel(10.0));
    let style2 = Style::new(Align::Center, Zero, Zero, UiSize::Size(UIUnit::Relative(0.8)), UiSize::Size(UIUnit::RelativeWidth(0.2)), RGBA::new(0, 0, 0, 120), RGBA::GREEN, 2.0, Pixel(10.0));

    let text_style = Style {
        position: Position::Inline(Inline { margin: [UIUnit::Pixel(0.0); 4], overflow: Overflow::clip() }),
        width: UiSize::Fill,
        height: UiSize::Size(UIUnit::Relative(0.6)),
        color: RGBA::GREEN,
        border: [0.0; 4],
        //corner: [UIUnit::RelativeHeight(0.5); 4],
        padding: Padding::new(0.0),
        ..Default::default()
    };

    let text_style2 = Style {
        position: Position::Inline(Inline { margin: [UIUnit::Pixel(0.0); 4], overflow: Overflow::clip() }),
        width: UiSize::Fill,
        height: UiSize::Size(UIUnit::RelativeHeight(0.6)),
        color: RGBA::RED,
        border: [0.0; 4],
        //corner: [UIUnit::RelativeHeight(0.5); 4],
        padding: Padding::new(0.0),
        ..Default::default()
    };


    let style_normal = Style::new(Align::Top, Zero, UIUnit::Relative(0.55), UiSize::Size(UIUnit::Relative(0.3)), UiSize::Size(UIUnit::RelativeWidth(0.1)), RGBA::BLACK, RGBA::GREEN, 2.0, Pixel(10.0));
    let style_hover = Style::new(Align::Top, Zero, UIUnit::Relative(0.55), UiSize::Size(UIUnit::Relative(0.3)), UiSize::Size(UIUnit::RelativeWidth(0.1)), RGBA::BLACK, RGBA::BLUE, 2.0, Pixel(10.0));
    let style_press = Style::new(Align::Top, Zero, UIUnit::Relative(0.55), UiSize::Size(UIUnit::Relative(0.3)), UiSize::Size(UIUnit::RelativeWidth(0.1)), RGBA::BLACK, RGBA::PURPLE, 2.0, Pixel(10.0));

    let score = UiElement::new(style, vec![Text::new(text_style, 0, "0", 3)]);
    let mut dead_message = UiElement::new(style2, vec![Text::new(text_style2, 0,"You dead", 3)]);
    dead_message.visible = false;

    let mut respawn_button = Button::new(style_normal, style_hover, style_press, Vec::with_capacity(0));
    respawn_button.visible = false;

    let state = UiState::create(vec![score, dead_message, respawn_button], Vec::with_capacity(0), true);
    state
}