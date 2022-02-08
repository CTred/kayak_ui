use crate::core::{
    render_command::RenderCommand,
    Binding, rsx,
    styles::{Style, Units},
    widget, Bound, Children, Color, EventType, MutableBound, OnEvent, WidgetProps,
};
use kayak_core::CursorIcon;
use std::sync::{Arc, RwLock};
use kayak_core::styles::PositionType;
use kayak_font::{CoordinateSystem, KayakFont};

use crate::widgets::{Background, Clip, Element, If, Text};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct TextBoxProps {
    pub disabled: bool,
    pub value: String,
    pub on_change: Option<OnChange>,
    pub placeholder: Option<String>,
    pub styles: Option<Style>,
    pub children: Option<Children>,
    pub on_event: Option<OnEvent>,
    pub focusable: Option<bool>,
}

impl WidgetProps for TextBoxProps {
    fn get_children(&self) -> Option<Children> {
        self.children.clone()
    }

    fn set_children(&mut self, children: Option<Children>) {
        self.children = children;
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn get_on_event(&self) -> Option<OnEvent> {
        self.on_event.clone()
    }

    fn get_focusable(&self) -> Option<bool> {
        Some(!self.disabled)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangeEvent {
    pub value: String,
}

#[derive(Clone)]
pub struct OnChange(pub Arc<RwLock<dyn FnMut(ChangeEvent) + Send + Sync + 'static>>);

impl OnChange {
    pub fn new<F: FnMut(ChangeEvent) + Send + Sync + 'static>(f: F) -> OnChange {
        OnChange(Arc::new(RwLock::new(f)))
    }
}

impl PartialEq for OnChange {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for OnChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OnChange").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus(pub bool);

#[widget]
pub fn TextBox(props: TextBoxProps) {
    let TextBoxProps {
        on_change,
        placeholder,
        value,
        ..
    } = props.clone();

    props.styles = Some(
        Style::default()
            // Required styles
            .with_style(Style {
                render_command: RenderCommand::Layout.into(),
                ..Default::default()
            })
            // Apply any prop-given styles
            .with_style(&props.styles)
            // If not set by props, apply these styles
            .with_style(Style {
                top: Units::Pixels(0.0).into(),
                bottom: Units::Pixels(0.0).into(),
                height: Units::Pixels(26.0).into(),
                cursor: CursorIcon::Text.into(),
                ..Default::default()
            }),
    );

    let background_styles = Style {
        background_color: Color::new(0.176, 0.196, 0.215, 1.0).into(),
        border_radius: (5.0, 5.0, 5.0, 5.0).into(),
        height: Units::Pixels(26.0).into(),
        padding_left: Units::Pixels(5.0).into(),
        padding_right: Units::Pixels(5.0).into(),
        ..Default::default()
    };

    let has_focus = context.create_state(Focus(false)).unwrap();

    let mut current_value = value.clone();
    let cloned_on_change = on_change.clone();
    let cloned_has_focus = has_focus.clone();

    props.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::CharInput { c } => {
            if !cloned_has_focus.get().0 {
                return;
            }
            if is_backspace(c) {
                if !current_value.is_empty() {
                    current_value.truncate(current_value.len() - 1);
                }
            } else if !c.is_control() {
                current_value.push(c);
            }
            if let Some(on_change) = cloned_on_change.as_ref() {
                if let Ok(mut on_change) = on_change.0.write() {
                    on_change(ChangeEvent {
                        value: current_value.clone(),
                    });
                }
            }
        }
        EventType::Focus => cloned_has_focus.set(Focus(true)),
        EventType::Blur => cloned_has_focus.set(Focus(false)),
        _ => {}
    }));

    let font_name = Some("Roboto");
    let font: Binding<Option<KayakFont>> = context.get_asset(font_name.clone().unwrap_or("Roboto".into()));
    context.bind(&font);
    let mut should_render = true;
    let (layout_size, parent_size) =
        if let Some(parent_id) = context.get_valid_parent(parent_id.unwrap()) {
            if let Some(layout) = context.get_layout(&parent_id) {
                if let Some(font) = font.get() {
                    let measurement = font.measure(
                        CoordinateSystem::PositiveYDown,
                        &value,
                        14.0,
                        22.0,
                        (layout.width, layout.height),
                    );
                    (measurement, (layout.width, layout.height))
                } else {
                    should_render = false;
                    ((0.0, 0.0), (layout.width, layout.height))
                }
            } else {
                should_render = false;
                ((0.0, 0.0), (0.0, 0.0))
            }
        } else {
            should_render = false;
            ((0.0, 0.0), (0.0, 0.0))
        };

    println!("Layout: {:?}", layout_size);

    let text_styles = if value.is_empty() || (has_focus.get().0 && value.is_empty()) {
        Style {
            color: Color::new(0.5, 0.5, 0.5, 1.0).into(),
            ..Style::default()
        }
    } else {
        Style::default()
    };

    let cursor_styles = Style {
        background_color: Color::new(0.0, 1.0, 1.0, 1.0).into(),
        position_type: PositionType::SelfDirected.into(),
        render_command: RenderCommand::Quad.into(),
        left: Units::Pixels(layout_size.0 + 5.0).into(),
        top: Units::Pixels(3.0).into(),
        bottom: Units::Pixels(3.0).into(),
        width: Units::Pixels(1.0).into(),
        height: Units::Stretch(1.0).into(),
        ..Default::default()
    };

    let value = if value.is_empty() {
        placeholder.unwrap_or_else(|| value.clone())
    } else {
        value
    };

    let has_focus = has_focus.get();
    rsx! {
        <>
            <Background styles={Some(background_styles)}>
                <Clip>
                    <Text
                        content={value}
                        size={14.0}
                        line_height={Some(22.0)}
                        styles={Some(text_styles)}
                    />
                </Clip>
            </Background>
            <If condition={has_focus.0 && should_render}>
                <Element styles={Some(cursor_styles)} />
            </If>
        </>
    }
}

/// Checks if the given character contains the "Backspace" sequence
///
/// Context: [Wikipedia](https://en.wikipedia.org/wiki/Backspace#Common_use)
fn is_backspace(c: char) -> bool {
    c == '\u{8}' || c == '\u{7f}'
}
