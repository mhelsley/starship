use crate::{
    config::Style,
    print::{Grapheme, UnicodeWidthGraphemes},
};
use nu_ansi_term::{AnsiString, Style as AnsiStyle};
use unicode_segmentation::UnicodeSegmentation;

/// Type that holds text with an associated style
#[derive(Clone)]
pub struct TextSegment {
    /// The segment's style. If None, will inherit the style of the module containing it.
    style: Option<Style>,

    /// The string value of the current segment.
    value: String,
}

impl TextSegment {
    // Returns the AnsiString of the segment value
    fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString<'_> {
        match self.style {
            Some(style) => style.to_ansi_style(prev).paint(&self.value),
            None => AnsiString::from(&self.value),
        }
    }
}

/// Type that holds a URL and text with an associated style
#[derive(Clone)]
pub struct URLSegment {
    displayed: TextSegment,
    url: String,
}

impl URLSegment {
    // Returns the AnsiString of the segment value
    fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString {
        let displayed = self.displayed.ansi_string(prev);

        let url = self.url.clone();
        displayed.hyperlink::<String>(url.into())
    }
}

#[derive(Clone)]
pub struct TitleSegment {
    // No styling
    title: String,
}

impl TitleSegment {
    // Returns the AnsiString of the segment value
    fn ansi_string(&self, _prev: Option<&AnsiStyle>) -> AnsiString {
        AnsiString::title(self.title.clone())
    }
}

#[cfg(test)]
mod url_seg_tests {
    use super::URLSegment;
    use nu_ansi_term::Color;

    #[test]
    fn ansi_string_link() {
        let style = Color::Blue.bold();

        let inputs = vec![("example.com", "example.com")];

        for (text, expected) in &inputs {
            let u = URLSegment {
                displayed: TextSegment {
                    value: String::from(*text),
                    style: Some(style),
                },
                url: String::from(format!("https://{}", *text)),
            };
            let actual = u.ansi_string();
            assert_eq!(style.paint(*expected), actual);
        }
    }
}

/// Type that holds fill text with an associated style
#[derive(Clone)]
pub struct FillSegment {
    /// The segment's style. If None, will inherit the style of the module containing it.
    style: Option<Style>,

    /// The string value of the current segment.
    value: String,
}

impl FillSegment {
    // Returns the AnsiString of the segment value, not including its prefix and suffix
    pub fn ansi_string(&self, width: Option<usize>, prev: Option<&AnsiStyle>) -> AnsiString<'_> {
        let s = match width {
            Some(w) => self
                .value
                .graphemes(true)
                .cycle()
                .scan(0usize, |len, g| {
                    *len += Grapheme(g).width();
                    if *len <= w { Some(g) } else { None }
                })
                .collect::<String>(),
            None => String::from(&self.value),
        };
        match self.style {
            Some(style) => style.to_ansi_style(prev).paint(s),
            None => AnsiString::from(s),
        }
    }
}

#[cfg(test)]
mod fill_seg_tests {
    use super::FillSegment;
    use nu_ansi_term::Color;

    #[test]
    fn ansi_string_width() {
        let width: usize = 10;
        let style = Color::Blue.bold();

        let inputs = vec![
            (".", ".........."),
            (".:", ".:.:.:.:.:"),
            ("-:-", "-:--:--:--"),
            ("🟦", "🟦🟦🟦🟦🟦"),
            ("🟢🔵🟡", "🟢🔵🟡🟢🔵"),
        ];

        for (text, expected) in &inputs {
            let f = FillSegment {
                value: String::from(*text),
                style: Some(style.into()),
            };
            let actual = f.ansi_string(Some(width), None);
            assert_eq!(style.paint(*expected), actual);
        }
    }
}

/// A segment is a styled text chunk ready for printing.
#[derive(Clone)]
pub enum Segment {
    Text(TextSegment),
    URL(URLSegment),
    Title(TitleSegment),
    Fill(FillSegment),
    LineTerm,
}

impl Segment {
    /// Creates new segments from a text with a style; breaking out `LineTerminators`.
    pub fn from_text<T>(style: Option<Style>, value: T) -> Vec<Self>
    where
        T: Into<String>,
    {
        let mut segs: Vec<Self> = Vec::new();
        value.into().split(LINE_TERMINATOR).for_each(|s| {
            if !segs.is_empty() {
                segs.push(Self::LineTerm);
            }
            segs.push(Self::Text(TextSegment {
                value: String::from(s),
                style,
            }));
        });
        segs
    }

    /// Creates a new URL segment
    pub fn link<T>(text: TextSegment, url: T) -> Self
    where
        T: Into<String>,
    {
        Self::URL(URLSegment {
            displayed: text,
            url: url.into(),
        })
    }

    /// Creates a new Title segment
    pub fn title<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self::Title(TitleSegment { title: text.into() })
    }

    /// Creates a new fill segment
    pub fn fill<T>(style: Option<Style>, value: T) -> Self
    where
        T: Into<String>,
    {
        Self::Fill(FillSegment {
            style,
            value: value.into(),
        })
    }

    pub fn style(&self) -> Option<AnsiStyle> {
        match self {
            Self::Fill(fs) => fs.style.map(|cs| cs.to_ansi_style(None)),
            Self::Text(ts) => ts.style.map(|cs| cs.to_ansi_style(None)),
            Self::URL(us) => us.displayed.style.map(|cs| cs.to_ansi_style(None)),
            Self::Title(_) => None,
            Self::LineTerm => None,
        }
    }

    pub fn set_style_if_empty(&mut self, style: Option<Style>) {
        match self {
            Self::Fill(fs) => {
                if fs.style.is_none() {
                    fs.style = style;
                }
            }
            Self::Text(ts) => {
                if ts.style.is_none() {
                    ts.style = style;
                }
            }
            Self::URL(us) => {
                if us.displayed.style.is_none() {
                    us.displayed.style = style
                }
            }
            Self::Title(_) | Self::LineTerm => {}
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Fill(fs) => &fs.value,
            Self::Text(ts) => &ts.value,
            Self::URL(us) => &us.displayed.value,
            Self::Title(ts) => &ts.title,
            Self::LineTerm => LINE_TERMINATOR_STRING,
        }
    }

    // Returns the AnsiString of the segment value, not including its prefix and suffix
    pub fn ansi_string(&self, prev: Option<&AnsiStyle>) -> AnsiString<'_> {
        match self {
            Self::Fill(fs) => fs.ansi_string(None, prev),
            Self::Text(ts) => ts.ansi_string(prev),
            Self::URL(us) => us.ansi_string(prev),
            Self::Title(ts) => ts.ansi_string(prev),
            Self::LineTerm => AnsiString::from(LINE_TERMINATOR_STRING),
        }
    }

    pub fn width_graphemes(&self) -> usize {
        match self {
            Self::Fill(fs) => fs.value.width_graphemes(),
            Self::Text(ts) => ts.value.width_graphemes(),
            Self::URL(us) => us.displayed.value.width_graphemes(),
            Self::Title(ts) => ts.title.width_graphemes(),
            Self::LineTerm => 0,
        }
    }
}

const LINE_TERMINATOR: char = '\n';
const LINE_TERMINATOR_STRING: &str = "\n";
