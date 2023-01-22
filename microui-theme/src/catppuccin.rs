use microui::{Color, WidgetColors, WidgetColor};

/// The colors for a theme variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Theme {
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,
    pub text: Color,
    pub subtext1: Color,
    pub subtext0: Color,
    pub overlay2: Color,
    pub overlay1: Color,
    pub overlay0: Color,
    pub surface2: Color,
    pub surface1: Color,
    pub surface0: Color,
    pub base: Color,
    pub mantle: Color,
    pub crust: Color
}

pub const LATTE: Theme = Theme {
    rosewater: Color::rgb(220, 138, 120),
    flamingo: Color::rgb(221, 120, 120),
    pink: Color::rgb(234, 118, 203),
    mauve: Color::rgb(136, 57, 239),
    red: Color::rgb(210, 15, 57),
    maroon: Color::rgb(230, 69, 83),
    peach: Color::rgb(254, 100, 11),
    yellow: Color::rgb(223, 142, 29),
    green: Color::rgb(64, 160, 43),
    teal: Color::rgb(23, 146, 153),
    sky: Color::rgb(4, 165, 229),
    sapphire: Color::rgb(32, 159, 181),
    blue: Color::rgb(30, 102, 245),
    lavender: Color::rgb(114, 135, 253),
    text: Color::rgb(76, 79, 105),
    subtext1: Color::rgb(92, 95, 119),
    subtext0: Color::rgb(108, 111, 133),
    overlay2: Color::rgb(124, 127, 147),
    overlay1: Color::rgb(140, 143, 161),
    overlay0: Color::rgb(156, 160, 176),
    surface2: Color::rgb(172, 176, 190),
    surface1: Color::rgb(188, 192, 204),
    surface0: Color::rgb(204, 208, 218),
    base: Color::rgb(239, 241, 245),
    mantle: Color::rgb(230, 233, 239),
    crust: Color::rgb(220, 224, 232)
};

pub const FRAPPE: Theme = Theme {
    rosewater: Color::rgb(242, 213, 207),
    flamingo: Color::rgb(238, 190, 190),
    pink: Color::rgb(244, 184, 228),
    mauve: Color::rgb(202, 158, 230),
    red: Color::rgb(231, 130, 132),
    maroon: Color::rgb(234, 153, 156),
    peach: Color::rgb(239, 159, 118),
    yellow: Color::rgb(229, 200, 144),
    green: Color::rgb(166, 209, 137),
    teal: Color::rgb(129, 200, 190),
    sky: Color::rgb(153, 209, 219),
    sapphire: Color::rgb(133, 193, 220),
    blue: Color::rgb(140, 170, 238),
    lavender: Color::rgb(186, 187, 241),
    text: Color::rgb(198, 208, 245),
    subtext1: Color::rgb(181, 191, 226),
    subtext0: Color::rgb(165, 173, 206),
    overlay2: Color::rgb(148, 156, 187),
    overlay1: Color::rgb(131, 139, 167),
    overlay0: Color::rgb(115, 121, 148),
    surface2: Color::rgb(98, 104, 128),
    surface1: Color::rgb(81, 87, 109),
    surface0: Color::rgb(65, 69, 89),
    base: Color::rgb(48, 52, 70),
    mantle: Color::rgb(41, 44, 60),
    crust: Color::rgb(35, 38, 52)
};

pub const MACCHIATO: Theme = Theme {
    rosewater: Color::rgb(244, 219, 214),
    flamingo: Color::rgb(240, 198, 198),
    pink: Color::rgb(245, 189, 230),
    mauve: Color::rgb(198, 160, 246),
    red: Color::rgb(237, 135, 150),
    maroon: Color::rgb(238, 153, 160),
    peach: Color::rgb(245, 169, 127),
    yellow: Color::rgb(238, 212, 159),
    green: Color::rgb(166, 218, 149),
    teal: Color::rgb(139, 213, 202),
    sky: Color::rgb(145, 215, 227),
    sapphire: Color::rgb(125, 196, 228),
    blue: Color::rgb(138, 173, 244),
    lavender: Color::rgb(183, 189, 248),
    text: Color::rgb(202, 211, 245),
    subtext1: Color::rgb(184, 192, 224),
    subtext0: Color::rgb(165, 173, 203),
    overlay2: Color::rgb(147, 154, 183),
    overlay1: Color::rgb(128, 135, 162),
    overlay0: Color::rgb(110, 115, 141),
    surface2: Color::rgb(91, 96, 120),
    surface1: Color::rgb(73, 77, 100),
    surface0: Color::rgb(54, 58, 79),
    base: Color::rgb(36, 39, 58),
    mantle: Color::rgb(30, 32, 48),
    crust: Color::rgb(24, 25, 38)
};

pub const MOCHA: Theme = Theme {
    rosewater: Color::rgb(245, 224, 220),
    flamingo: Color::rgb(242, 205, 205),
    pink: Color::rgb(245, 194, 231),
    mauve: Color::rgb(203, 166, 247),
    red: Color::rgb(243, 139, 168),
    maroon: Color::rgb(235, 160, 172),
    peach: Color::rgb(250, 179, 135),
    yellow: Color::rgb(249, 226, 175),
    green: Color::rgb(166, 227, 161),
    teal: Color::rgb(148, 226, 213),
    sky: Color::rgb(137, 220, 235),
    sapphire: Color::rgb(116, 199, 236),
    blue: Color::rgb(137, 180, 250),
    lavender: Color::rgb(180, 190, 254),
    text: Color::rgb(205, 214, 244),
    subtext1: Color::rgb(186, 194, 222),
    subtext0: Color::rgb(166, 173, 200),
    overlay2: Color::rgb(147, 153, 178),
    overlay1: Color::rgb(127, 132, 156),
    overlay0: Color::rgb(108, 112, 134),
    surface2: Color::rgb(88, 91, 112),
    surface1: Color::rgb(69, 71, 90),
    surface0: Color::rgb(49, 50, 68),
    base: Color::rgb(30, 30, 46),
    mantle: Color::rgb(24, 24, 37),
    crust: Color::rgb(17, 17, 27)
};

impl Theme {
    pub fn widget_colors(&self) -> WidgetColors {
        use WidgetColor::*;

        let mut c = WidgetColors([Color::TRANSPARENT; WidgetColors::COUNT]);
        
        c[Text] = self.text;
        c[Border] = self.lavender;
        c[WindowBackground] = self.base;
        c[TitleBackground] = self.crust;
        c[TitleText] = self.subtext1;
        c[PanelBackground] = self.mantle;
        c[Button] = self.base;
        c[ButtonHover] = self.overlay0;
        c[ButtonFocus] = self.overlay1;
        c[Base] = self.surface0;
        c[BaseHover] = self.surface1;
        c[BaseFocus] = self.surface2;
        c[ScrollBase] = self.surface1;
        c[ScrollThumb] = self.overlay0;

        c
    }
}
