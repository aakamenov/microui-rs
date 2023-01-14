use std::{
    mem,
    ops::{Index, IndexMut}
};

use crate::{
    FontId,
    geometry::{Vec2, vec2}
};

const WIDGET_COLOR_COUNT: usize = mem::variant_count::<WidgetColor>();

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

pub struct WidgetColors(pub [Color; WIDGET_COLOR_COUNT]);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum WidgetColor {
    Text = 0,
    Border = 1,
    WindowBackground = 2,
    TitleBackground = 3,
    TitleText = 4,
    PanelBackground = 5,
    Button = 6,
    ButtonHover = Self::Button as u8 + 1u8,
    ButtonFocus = Self::Button as u8 + 2u8,
    Base = 9,
    BaseHover = Self::Base as u8 + 1u8,
    BaseFocus = Self::Base as u8 + 2u8,
    ScrollBase = 12,
    ScrollThumb = 13
}

pub struct Style {
    pub font: FontId,
    pub size: Vec2,
    pub padding: u16,
    pub spacing: u16,
    pub indent: u16,
    pub title_height: u16,
    pub footer_height: u16,
    pub scrollbar_size: u16,
    pub thumb_size: u16,
    pub colors: WidgetColors
}

impl Color {
    pub const TRANSPARENT: Color = Self::rgba(0, 0, 0, 0);

    #[inline(always)]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline(always)]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

impl Index<WidgetColor> for WidgetColors {
    type Output = Color;

    #[inline(always)]
    fn index(&self, index: WidgetColor) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<WidgetColor> for WidgetColors {
    #[inline(always)]
    fn index_mut(&mut self, index: WidgetColor) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: FontId::default(),
            size: vec2(68, 10),
            padding: 5,
            spacing: 4,
            indent: 24,
            title_height: 24,
            footer_height: 20,
            scrollbar_size: 12,
            thumb_size: 8,
            colors: WidgetColors::default()
        }
    }
}

impl Default for WidgetColors {
    fn default() -> Self {
        use WidgetColor::*;

        let mut c = Self([Color::TRANSPARENT; WIDGET_COLOR_COUNT]);
        
        c[Text] = Color::rgb(230, 230, 230);
        c[Border] = Color::rgb(25, 25, 25);
        c[WindowBackground] = Color::rgb(50, 50, 50);
        c[TitleBackground] = Color::rgb(25, 25, 25);
        c[TitleText] = Color::rgb(240, 240, 240);
        c[Button] = Color::rgb(75, 75, 75);
        c[ButtonHover] = Color::rgb(95, 95, 95);
        c[ButtonFocus] = Color::rgb(115, 115, 115);
        c[Base] = Color::rgb(30, 30, 30);
        c[BaseHover] = Color::rgb(35, 35, 35);
        c[BaseFocus] = Color::rgb(40, 40, 40);
        c[ScrollBase] = Color::rgb(43, 43, 43);
        c[ScrollThumb] = Color::rgb(30, 30, 30);

        c
    }
}
