//! Geometry and navigation of the two upper Home areas.
use super::*;
use crate::i18n::Language;

pub const LEFT: f32 = 80.;
pub const RIGHT: f32 = 1840.;
pub const WIDGET_WIDTH: f32 = 824.;
const ACTION_TOP: f32 = 28.;
const ACTION_SIZE: f32 = 60.;
const LABEL_GAP: f32 = 4.;
const LABEL_HEIGHT: f32 = 28.;
const WIDGET_GAP: f32 = 12.;

pub struct HeaderGeometry {
    pub clock: Rect,
    pub weather: Rect,
    pub buttons: Vec<Rect>,
    pub actions_bottom: f32,
    pub summary_height: f32,
    pub bottom: f32,
}

impl HeaderGeometry {
    pub fn calendar(&self) -> Rect {
        let top = self.clock.y + self.summary_height + WIDGET_GAP;
        Rect::new(self.clock.x, top, self.clock.w, (self.bottom - top).max(0.))
    }

    pub fn action_label(&self, icon: Rect, width: f32) -> Rect {
        Rect::new(
            (icon.center().0 - width / 2.).clamp(LEFT, RIGHT - width),
            ACTION_TOP + ACTION_SIZE + LABEL_GAP,
            width,
            LABEL_HEIGHT,
        )
    }

    pub fn diagnostic(&self) -> serde_json::Value {
        let rect = |r: Rect| [r.x, r.y, r.w, r.h];
        serde_json::json!({
            "clock": rect(self.clock), "calendar": rect(self.calendar()), "weather": rect(self.weather),
            "actions": self.buttons.iter().copied().map(rect).collect::<Vec<_>>(),
            "actions_bottom": self.actions_bottom, "bottom": self.bottom,
        })
    }
}

pub fn header_geometry(state: &State) -> HeaderGeometry {
    header_geometry_for(state, false)
}

fn header_geometry_for(state: &State, settled_dock: bool) -> HeaderGeometry {
    let home = state.evolution.as_ref().unwrap();
    let (label, expanded, compact) = if settled_dock {
        (0., 0., 0.)
    } else {
        (
            home.top_label.value,
            home.top_expansion.value,
            home.expansion.value,
        )
    };
    let count = state.evolution_top_targets().len();
    let buttons = (0..count)
        .map(|i| {
            Rect::new(
                RIGHT - ACTION_SIZE - (count - 1 - i) as f32 * 80.,
                ACTION_TOP,
                ACTION_SIZE,
                ACTION_SIZE,
            )
        })
        .collect();
    let actions_bottom = if count == 0 {
        // Keep the preview identifier clear when there are no action hit targets.
        62.
    } else {
        ACTION_TOP + ACTION_SIZE + (LABEL_GAP + LABEL_HEIGHT) * label
    };
    let y = actions_bottom + WIDGET_GAP;
    let has_location = state.preferences.weather_location.is_some()
        || home.weather_fixture != WeatherFixture::None;
    let summary_height =
        (122. - 68. * compact).max(if state.preferences.show_weather && has_location {
            83.
        } else {
            0.
        });
    let height = summary_height + 382. * expanded;
    let bottom = if state.preferences.evolution_home.show_top {
        y + height
    } else {
        actions_bottom
    };
    HeaderGeometry {
        clock: Rect::new(LEFT, y, WIDGET_WIDTH, height),
        weather: Rect::new(RIGHT - WIDGET_WIDTH, y, WIDGET_WIDTH, height),
        buttons,
        actions_bottom,
        summary_height,
        bottom,
    }
}

/// Month names and weekday abbreviations are bundled, just like Language::date.
pub fn calendar_labels(language: Language) -> (&'static str, &'static str, bool) {
    match language {
        Language::Pt => (
            "Janeiro|Fevereiro|Março|Abril|Maio|Junho|Julho|Agosto|Setembro|Outubro|Novembro|Dezembro",
            "Dom|Seg|Ter|Qua|Qui|Sex|Sáb",
            true,
        ),
        Language::En => (
            "January|February|March|April|May|June|July|August|September|October|November|December",
            "Sun|Mon|Tue|Wed|Thu|Fri|Sat",
            true,
        ),
        Language::De => (
            "Januar|Februar|März|April|Mai|Juni|Juli|August|September|Oktober|November|Dezember",
            "Mo|Di|Mi|Do|Fr|Sa|So",
            false,
        ),
        Language::Es => (
            "Enero|Febrero|Marzo|Abril|Mayo|Junio|Julio|Agosto|Septiembre|Octubre|Noviembre|Diciembre",
            "Lun|Mar|Mié|Jue|Vie|Sáb|Dom",
            false,
        ),
        Language::Fr => (
            "Janvier|Février|Mars|Avril|Mai|Juin|Juillet|Août|Septembre|Octobre|Novembre|Décembre",
            "Lun|Mar|Mer|Jeu|Ven|Sam|Dim",
            false,
        ),
        Language::It => (
            "Gennaio|Febbraio|Marzo|Aprile|Maggio|Giugno|Luglio|Agosto|Settembre|Ottobre|Novembre|Dicembre",
            "Lun|Mar|Mer|Gio|Ven|Sab|Dom",
            false,
        ),
        Language::Ru => (
            "Январь|Февраль|Март|Апрель|Май|Июнь|Июль|Август|Сентябрь|Октябрь|Ноябрь|Декабрь",
            "Пн|Вт|Ср|Чт|Пт|Сб|Вс",
            false,
        ),
        Language::Pl => (
            "Styczeń|Luty|Marzec|Kwiecień|Maj|Czerwiec|Lipiec|Sierpień|Wrzesień|Październik|Listopad|Grudzień",
            "Pon|Wt|Śr|Czw|Pt|Sob|Ndz",
            false,
        ),
        Language::Nl => (
            "Januari|Februari|Maart|April|Mei|Juni|Juli|Augustus|September|Oktober|November|December",
            "Ma|Di|Wo|Do|Vr|Za|Zo",
            false,
        ),
        Language::Sv => (
            "Januari|Februari|Mars|April|Maj|Juni|Juli|Augusti|September|Oktober|November|December",
            "Mån|Tis|Ons|Tor|Fre|Lör|Sön",
            false,
        ),
        Language::Cs => (
            "Leden|Únor|Březen|Duben|Květen|Červen|Červenec|Srpen|Září|Říjen|Listopad|Prosinec",
            "Po|Út|St|Čt|Pá|So|Ne",
            false,
        ),
        Language::Vi => (
            "Tháng 1|Tháng 2|Tháng 3|Tháng 4|Tháng 5|Tháng 6|Tháng 7|Tháng 8|Tháng 9|Tháng 10|Tháng 11|Tháng 12",
            "T2|T3|T4|T5|T6|T7|CN",
            false,
        ),
        Language::Zh | Language::ZhTw => (
            "一月|二月|三月|四月|五月|六月|七月|八月|九月|十月|十一月|十二月",
            "一|二|三|四|五|六|日",
            false,
        ),
    }
}

impl State {
    pub fn compact_cinema_percent(&self) -> u8 {
        self.evolution
            .as_ref()
            .and_then(|home| home.cinema_scale_override)
            .unwrap_or(self.preferences.evolution_home.compact_cinema_scale)
            .percent()
    }

    fn evolution_flow(
        &self,
        items: &[Target],
        header: &HeaderGeometry,
        settled_dock: bool,
    ) -> (DockFlow, f32) {
        let home = self.evolution.as_ref().unwrap();
        let prefs = &self.preferences.evolution_home;
        let (widgets, expansion, scale) = if settled_dock {
            (0., 0., 1.)
        } else {
            (
                home.top_expansion.value,
                home.expansion.value,
                home.dock_scale(prefs.cinema_dock_scale),
            )
        };
        let neutral = f32::from(self.compact_cinema_percent());
        let compact = neutral + (50. - neutral) * widgets;
        let percent = compact + (100. - compact) * expansion;
        let wanted = EXPANDED_CARD_HEIGHT * (percent / 100.);
        let upper_bottom = if self.evolution_has_cinema() {
            header.bottom + 12. + wanted + MIN_STRIP_PADDING + CINEMA_DOCK_GAP
        } else if prefs.show_top {
            header.bottom + 12.
        } else {
            header.actions_bottom + 4.
        };
        let empty = (self.error.is_none() || !items.is_empty())
            && !items.iter().any(|item| matches!(item, Target::App(_)));
        let caption = settled_dock || home.dock_caption_transition() < 0.5;
        (
            DockFlow::new(
                prefs,
                items.len(),
                empty,
                scale,
                header.actions_bottom,
                upper_bottom,
                caption,
            ),
            wanted,
        )
    }

    pub(super) fn evolution_dock_destination(
        &self,
        items: &[Target],
        slot: usize,
        direction: Direction,
    ) -> Option<usize> {
        let (flow, _) = self.evolution_flow(items, &header_geometry(self), false);
        flow.destination(
            slot,
            direction,
            self.preferences.evolution_home.dock_horizontal,
        )
    }

    pub(crate) fn evolution_remember_dock(&mut self, items: &[Target], index: usize) {
        let settled = !self.evolution.as_ref().unwrap().dock_in_view;
        let header = header_geometry_for(self, settled);
        let (flow, _) = self.evolution_flow(items, &header, settled);
        let previous = items.iter().position(|item| *item == self.focus);
        let scroll = &mut self.evolution.as_mut().unwrap().dock_scroll;
        let restored = flow.restored_offset(items, scroll);
        let restored = if !settled {
            previous.map_or(restored, |slot| flow.reveal(slot, restored))
        } else {
            restored
        };
        scroll.offset = flow.reveal(index, restored);
        if scroll.offset == restored
            && let Some((target, y)) = &mut scroll.anchor
            && let Some(slot) = items.iter().position(|item| item == target)
        {
            // Keep the existing anchor when the new focus already fits the viewport.
            *y = flow.row_y(slot / flow.columns);
        } else {
            scroll.anchor = Some((items[index].clone(), flow.row_y(index / flow.columns)));
        }
    }

    pub fn evolution_geometry(&self) -> Geometry {
        let home = self.evolution.as_ref().unwrap();
        let items = self.evolution_dock_items();
        let inputs = self.evolution_inputs();
        let header = header_geometry(self);
        let (flow, wanted) = self.evolution_flow(&items, &header, false);
        let offset = if home.inputs_revealed && !inputs.is_empty() {
            flow.input_scroll_limit()
        } else if home.dock_in_view {
            // Project the session anchor into the displayed layout without storing any
            // corrections required by an in-flight contextual transition.
            let offset = flow.restored_offset(&items, &home.dock_scroll);
            let target = if self.pointer {
                home.dock_scroll.anchor.as_ref().map(|(target, _)| target)
            } else {
                Some(&self.focus)
            };
            target
                .and_then(|target| items.iter().position(|item| item == target))
                .map_or(offset, |slot| flow.reveal(slot, offset))
        } else {
            0.
        };
        let mut layout = flow.layout(offset, inputs.len(), home.input_anchor);
        let bottom = flow.pane.y - CINEMA_DOCK_GAP;
        // On a crowded Home, use spare spacing before shrinking cards. The remaining header
        // gap also clears the Area 2 focus outline, which extends 5 px past the widget bounds.
        let header_gap = (bottom - header.bottom - wanted - MIN_STRIP_PADDING).clamp(12., 18.);
        let expanded_top = header.bottom + header_gap;
        let strip_padding =
            (bottom - expanded_top - wanted).clamp(MIN_STRIP_PADDING, STRIP_PADDING);
        let compact_top = bottom - wanted - strip_padding;
        let center_top =
            (compact_top + (expanded_top - compact_top) * home.expansion.value).max(expanded_top);
        layout.center = Rect::new(
            LEFT,
            center_top - offset,
            RIGHT - LEFT,
            (bottom - center_top).max(0.),
        );
        // Measure the budget from the header. Subtracting the animated center's
        // coordinates again can round a fitting 40% card below the minimum.
        let available = bottom - expanded_top - strip_padding;
        let card_height = if available + 0.001 >= wanted {
            wanted
        } else {
            available
        };
        if self.evolution_has_cinema() && card_height >= EXPANDED_CARD_HEIGHT * 0.4 {
            layout.cinema_card_height = card_height;
        }
        layout.scroll.up |= self.preferences.evolution_home.show_top
            && header.clock.y < flow.viewport.y + offset
            || (layout.cinema_card_height > 0.
                || (self.evolution_has_cinema()
                    && home.expansion.value > 0.
                    && layout.center.h > 0.))
                && center_top < flow.viewport.y + offset;
        // Reordering directs keys to the draft. Its banner occupies the upper margin.
        if self.evolution_moving() {
            layout.scroll = ScrollHints::default();
        }
        layout
    }
}

const CINEMA_DOCK_GAP: f32 = 28.;
const EXPANDED_CARD_HEIGHT: f32 = 164.;
const STRIP_PADDING: f32 = 46.;
// Caption and its insets plus the space below the cards.
const MIN_STRIP_PADDING: f32 = 35.;
