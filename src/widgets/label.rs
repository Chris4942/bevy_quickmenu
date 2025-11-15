use super::Widget;
use crate::style::StyleEntry;
use crate::types::{MenuAssets, WidgetLabel};
use bevy::prelude::*;

pub struct LabelWidget<'a> {
    text: &'a WidgetLabel,
    style: &'a StyleEntry,
}

impl<'a> LabelWidget<'a> {
    pub fn new(text: &'a WidgetLabel, style: &'a StyleEntry) -> Self {
        Self { text, style }
    }
}

impl<'a> Widget for LabelWidget<'a> {
    fn build(self, parent: &mut ChildSpawnerCommands, assets: &MenuAssets) {
        let LabelWidget { text, style, .. } = self;

        let (bg, fg) = (style.normal.bg, style.selected.fg);

        let font = TextFont {
            font: assets.font.clone(),
            font_size: style.size,
            ..default()
        };

        let color = TextColor(fg);

        parent
            .spawn((
                Node {
                    margin: style.margin,
                    padding: style.padding,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(bg),
            ))
            .with_children(|parent| {
                for child in text.bundle(&font, &color) {
                    parent.spawn(child);
                }
            });
    }
}
