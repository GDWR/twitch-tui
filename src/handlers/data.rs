use std::borrow::Cow;

use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use lazy_static::lazy_static;
use regex::Regex;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row},
};

use crate::{
    handlers::config::{FrontendConfig, Palette, Theme},
    utils::{
        colors::hsl_to_rgb,
        styles::{
            DATETIME_DARK, DATETIME_LIGHT, HIGHLIGHT_NAME_DARK, HIGHLIGHT_NAME_LIGHT, SYSTEM_CHAT,
        },
    },
};

lazy_static! {
    pub static ref FUZZY_FINDER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub system: bool,
    pub payload: String,
}

impl MessageData {
    pub fn new(author: String, system: bool, payload: String) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            system,
            payload,
        }
    }

    fn hash_username(&self, palette: &Palette) -> Color {
        let hash = f64::from(
            self.author
                .as_bytes()
                .iter()
                .map(|&b| u32::from(b))
                .sum::<u32>(),
        );

        let (hue, saturation, lightness) = match palette {
            Palette::Pastel => (hash % 360. + 1., 0.5, 0.75),
            Palette::Vibrant => (hash % 360. + 1., 1., 0.6),
            Palette::Warm => ((hash % 100. + 1.) * 1.2, 0.8, 0.7),
            Palette::Cool => ((hash % 100. + 1.).mul_add(1.2, 180.), 0.6, 0.7),
        };

        let rgb = hsl_to_rgb(hue, saturation, lightness);

        Rgb(rgb[0], rgb[1], rgb[2])
    }

    fn wrap_message(
        &self,
        combined_message: String,
        frontend_config: &FrontendConfig,
        width: usize,
    ) -> Vec<String> {
        // Total width of the window subtracted by any margin, then the two border line lengths.
        let wrap_limit = width - (frontend_config.margin as usize * 2) - 2;

        textwrap::wrap(&combined_message, wrap_limit)
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }

    pub fn to_spans(
        &self,
        frontend_config: &FrontendConfig,
        width: usize,
        search_highlight: Option<String>,
        username_highlight: Option<String>,
    ) -> (Vec<Spans>, u32) {
        let time_sent = self
            .time_sent
            .format(&frontend_config.date_format)
            .to_string();

        let raw_message_start = format!("{} {}: ", time_sent, &self.author);

        let raw_message = format!("{}{}", raw_message_start, &self.payload);

        let search = if let Some(user_search) = search_highlight {
            FUZZY_FINDER.fuzzy_indices(&raw_message[raw_message_start.len()..], &user_search)
        } else {
            None
        };

        let raw_message_wrapped = self.wrap_message(raw_message, frontend_config, width);

        let mut wrapped_message_spans = vec![];

        // let time_spent_styled_span = Span::styled(
        //     &wrapped_message[0][..time_sent.len()],
        //     match frontend_config.theme {
        //         Theme::Light => DATETIME_LIGHT,
        //         _ => DATETIME_DARK,
        //     },
        // );

        let mut start_vec = vec![
            Span::styled(
                raw_message_wrapped[0][..time_sent.len()].to_string(),
                match frontend_config.theme {
                    Theme::Light => DATETIME_LIGHT,
                    _ => DATETIME_DARK,
                },
            ),
            Span::raw(" "),
            Span::styled(
                self.author.clone(),
                if self.system {
                    SYSTEM_CHAT
                } else {
                    Style::default().fg(self.hash_username(&frontend_config.palette))
                },
            ),
            Span::raw(": "),
        ];

        start_vec.extend(if let Some((_, indices)) = search {
            raw_message_wrapped[0][raw_message_start.len()..]
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if indices.contains(&i) {
                        Span::styled(
                            c.to_string(),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw(c.to_string())
                    }
                })
                .collect::<Vec<Span>>()
        } else {
            vec![Span::raw(
                raw_message_wrapped[0][raw_message_start.len()..].to_string(),
            )]
        });

        wrapped_message_spans.push(Spans::from(start_vec));

        if raw_message_wrapped.len() > 1 {
            wrapped_message_spans.extend(
                raw_message_wrapped[1..]
                    .iter()
                    .map(|s| Spans::from(vec![Span::raw(s.to_string())]))
                    .collect::<Vec<Spans>>(),
            );
        }

        (wrapped_message_spans, 0)
    }

    // pub fn to_spans(
    //     &self,
    //     frontend_config: &FrontendConfig,
    //     width: usize,
    //     search_highlight: Option<String>,
    //     username_highlight: Option<String>,
    // ) -> (Vec<Spans>, u32) {
    //     let time_sent = self
    //         .time_sent
    //         .format(&frontend_config.date_format)
    //         .to_string();

    //     let message_spans = self
    //         .wrap_message(frontend_config, width, time_sent.clone())
    //         .iter()
    //         .map(|s| {
    //             if let Some(search) = search_highlight.clone() {
    //                 if let Some((_, indices)) = FUZZY_FINDER.fuzzy_indices(s, search.as_str()) {
    //                     return s
    //                         .chars()
    //                         .enumerate()
    //                         .map(|(i, c)| {
    //                             if indices.contains(&i) {
    //                                 Span::styled(
    //                                     c.to_string(),
    //                                     Style::default()
    //                                         .fg(Color::Red)
    //                                         .add_modifier(Modifier::BOLD),
    //                                 )
    //                             } else {
    //                                 Span::raw(s.to_string())
    //                             }
    //                         })
    //                         .collect::<Vec<Span>>();
    //                 }
    //             }

    //             return vec![Span::raw(s.clone())];
    //         })
    //         .flatten()
    //         .collect::<Vec<Span>>();

    //     let mut info = if frontend_config.date_shown {
    //         vec![
    //             Span::styled(
    //                 time_sent,
    // match frontend_config.theme {
    //     Theme::Light => DATETIME_LIGHT,
    //     _ => DATETIME_DARK,
    // },
    //             ),
    //             Span::raw(" "),
    //         ]
    //     } else {
    //         vec![]
    //     };

    //     if frontend_config.username_shown {
    //         info.extend(vec![
    //             Span::styled(
    //                 self.author.clone(),
    //                 if self.system {
    //                     SYSTEM_CHAT
    //                 } else {
    //                     Style::default().fg(self.hash_username(&frontend_config.palette))
    //                 },
    //             ),
    //             Span::raw(": "),
    //             message_spans[0].clone(),
    //         ]);
    //     }

    //     let mut info_spans = vec![Spans::from(info)];

    //     if message_spans.len() > 1 {
    //         for extra in message_spans[1..].iter().cloned() {
    //             info_spans.push(Spans::from(extra));
    //         }
    //     }

    //     (info_spans, 0)

    //     // let username_highlight_style = username_highlight.map_or_else(Style::default, |username| {
    //     //     if Regex::new(format!("^.*{username}.*$").as_str())
    //     //         .unwrap()
    //     //         .is_match(&message)
    //     //     {
    //     //         match frontend_config.theme {
    //     //             Theme::Light => HIGHLIGHT_NAME_LIGHT,
    //     //             _ => HIGHLIGHT_NAME_DARK,
    //     //         }
    //     //     } else {
    //     //         Style::default()
    //     //     }
    //     // });

    //     // let mut num_search_matches = 0;

    //     // if msg_cells.len() > 1 {
    //     //     for cell in msg_cells.iter().skip(1) {
    //     //         let mut wrapped_msg = vec![Cell::from(""), cell.clone()];

    //     //         if frontend_config.date_shown {
    //     //             wrapped_msg.insert(0, Cell::from(""));
    //     //         }

    //     //         row_vector.push(Row::new(wrapped_msg));
    //     //     }
    //     // }
    // }
}

#[derive(Debug, Copy, Clone)]
pub struct DataBuilder<'conf> {
    pub date_format: &'conf str,
}

impl<'conf> DataBuilder<'conf> {
    pub const fn new(date_format: &'conf str) -> Self {
        DataBuilder { date_format }
    }

    pub fn user(user: String, payload: String) -> MessageData {
        MessageData::new(user, false, payload)
    }

    pub fn system(self, payload: String) -> MessageData {
        MessageData::new("System".to_string(), true, payload)
    }

    pub fn twitch(self, payload: String) -> MessageData {
        MessageData::new("Twitch".to_string(), true, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            MessageData::new("human".to_string(), false, "beep boop".to_string())
                .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
