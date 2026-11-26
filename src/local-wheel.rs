//! Contextual wheel navigation uses the scene's actionable controls and keeps focus in memory.
use super::{Control, Overlay, Page, State, Subpage, Target};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::mem::{Discriminant, discriminant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelFocusMode {
    First,
    #[default]
    #[serde(other)]
    Restore,
}

#[derive(Clone, Copy)]
pub(super) enum Intent {
    Direct,
    EnterPanel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Panel {
    Sidebar,
    Settings(usize, Option<Discriminant<Subpage>>, SettingsList),
    Menu,
    MenuInputs,
    TodayTabs,
    TodayBody(Option<crate::today::Tab>),
    RecentCards,
    Streams,
    Dialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsList {
    Main,
    GalleryTools,
    Visual(
        crate::home_visual::Category,
        Option<crate::home_visual::Prompt>,
    ),
    WeatherResults,
    CryptoResults,
    CryptoCoins,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ItemId {
    Target(Target),
    Indexed(Discriminant<Target>, String),
}

struct Item {
    panel: Panel,
    target: Target,
    id: ItemId,
}

#[derive(Default)]
struct ListMemory {
    order: Vec<ItemId>,
    focus: Option<ItemId>,
}

impl ListMemory {
    fn reconcile(&mut self, order: Vec<ItemId>) {
        if let Some(id) = &self.focus
            && !order.contains(id)
        {
            self.focus = self
                .order
                .iter()
                .position(|old| old == id)
                .and_then(|index| {
                    self.order[index + 1..]
                        .iter()
                        .chain(self.order[..index].iter().rev())
                        .find(|old| order.contains(old))
                })
                .or_else(|| order.first())
                .cloned();
        }
        self.order = order;
    }
}

#[derive(Default)]
struct ScreenMemory {
    lists: Vec<(Panel, ListMemory)>,
    owner: Option<Panel>,
    presented: Option<Target>,
}

impl ScreenMemory {
    fn list(&mut self, panel: Panel) -> &mut ListMemory {
        let index = self.lists.iter().position(|(key, _)| *key == panel);
        if let Some(index) = index {
            &mut self.lists[index].1
        } else {
            self.lists.push((panel, ListMemory::default()));
            &mut self.lists.last_mut().unwrap().1
        }
    }

    fn observe(
        &mut self,
        proposed: Panel,
        target: &Target,
        items: &[Item],
        intent: Option<Intent>,
        mode: PanelFocusMode,
    ) -> Target {
        let previous = self.owner;
        let explicit = intent.is_some()
            && (self.presented.as_ref() != Some(target) || previous != Some(proposed));
        let owner = if explicit || previous.is_none() {
            proposed
        } else {
            previous.unwrap_or(proposed)
        };
        self.owner = Some(owner);
        let mut panels = vec![owner];
        for item in items {
            if !panels.contains(&item.panel) {
                panels.push(item.panel);
            }
        }
        for panel in panels {
            self.list(panel).reconcile(
                items
                    .iter()
                    .filter(|item| item.panel == panel)
                    .map(|item| item.id.clone())
                    .collect(),
            );
        }
        let list = self.list(owner);
        let eligible = || items.iter().filter(|item| item.panel == owner);
        let requested = eligible().find(|item| item.target == *target);
        let remembered = || eligible().find(|item| list.focus.as_ref() == Some(&item.id));
        let first = || eligible().next();
        let next = if previous.is_some()
            && previous != Some(owner)
            && matches!(intent, Some(Intent::EnterPanel))
        {
            match mode {
                PanelFocusMode::Restore => remembered().or_else(first),
                PanelFocusMode::First => first(),
            }
        } else if explicit || previous.is_none() {
            requested.or_else(remembered).or_else(first)
        } else {
            remembered().or_else(first)
        };
        list.focus = next.map(|item| item.id.clone());
        next.map_or(Target::LocalContent, |item| item.target.clone())
    }
}

#[derive(Default)]
pub(super) struct LocalWheel {
    settings: ScreenMemory,
    settings_scope: Option<(usize, Option<Discriminant<Subpage>>)>,
    overlay: ScreenMemory,
    overlay_kind: Option<Discriminant<Overlay>>,
    covered: Option<(Discriminant<Overlay>, ScreenMemory)>,
    pub(super) intent: Option<Intent>,
    today_scroll_max: f32,
}

impl LocalWheel {
    pub(super) fn tracks_overlay(&self) -> bool {
        self.overlay_kind.is_some()
    }
}

/// A stepper's pointer controls share its logical row in the local list.
fn list_target(target: &Target) -> Cow<'_, Target> {
    use crate::home_visual::Field;
    match target {
        Target::DockScaleStep(_)
        | Target::DockHorizontalStep(_)
        | Target::CinemaScaleStep(_)
        | Target::VisualStep(..) => Cow::Owned(Field::from_target(target).unwrap().target()),
        Target::WheelStepsStep(_) => Cow::Borrowed(&Target::WheelSteps),
        Target::RemoteTimingStep(timing, _) => Cow::Owned(Target::RemoteTiming(*timing)),
        _ => Cow::Borrowed(target),
    }
}

impl State {
    fn local_panel(&self, target: &Target, today_tab: Option<crate::today::Tab>) -> Option<Panel> {
        if matches!(target, Target::DismissNotice | Target::RecentEmpty) {
            return None;
        }
        Some(match &self.overlay {
            Some(Overlay::Menu { .. }) => {
                if matches!(target, Target::Input(_)) {
                    Panel::MenuInputs
                } else {
                    Panel::Menu
                }
            }
            Some(Overlay::Today { .. }) => {
                if matches!(target, Target::TodayTab(_)) {
                    Panel::TodayTabs
                } else {
                    Panel::TodayBody(today_tab)
                }
            }
            Some(Overlay::Recents { .. }) if matches!(target, Target::Recent(_)) => {
                Panel::RecentCards
            }
            Some(Overlay::Cs2Match { .. }) if matches!(target, Target::Cs2Stream(_)) => {
                Panel::Streams
            }
            Some(_) => Panel::Dialog,
            None if self.page == Page::Settings => match target {
                Target::Nav(_) | Target::Retry => return None,
                Target::Sidebar(_) => Panel::Sidebar,
                _ => Panel::Settings(
                    self.section,
                    self.subpage.as_ref().map(discriminant),
                    match target {
                        _ if matches!(self.subpage, Some(Subpage::Visual(_))) => {
                            let Some(Subpage::Visual(edit)) = self.subpage else {
                                unreachable!()
                            };
                            SettingsList::Visual(edit.category, edit.prompt)
                        }
                        Target::WallpaperCycle | Target::WallpaperGradient => {
                            SettingsList::GalleryTools
                        }
                        Target::WeatherResult(_) => SettingsList::WeatherResults,
                        Target::CryptoResult(_) => SettingsList::CryptoResults,
                        Target::CryptoCoin(_) => SettingsList::CryptoCoins,
                        _ => SettingsList::Main,
                    },
                ),
            },
            None => return None,
        })
    }

    fn resolved_today_tab(&self, controls: &[Control]) -> Option<crate::today::Tab> {
        let tabs = || {
            controls.iter().filter_map(|control| match control.target {
                Target::TodayTab(tab) => Some(tab),
                _ => None,
            })
        };
        self.today_tab
            .filter(|active| tabs().any(|tab| tab == *active))
            .or_else(|| tabs().next())
    }

    fn local_id(&self, target: &Target) -> Option<ItemId> {
        let key = match target {
            Target::Background(index) => self.background_id(*index)?.to_owned(),
            Target::WeatherResult(index) => {
                let place = self.weather.results.get(*index)?;
                place.id.map_or_else(
                    || format!("{}|{}|{}", place.name, place.latitude, place.longitude),
                    |id| id.to_string(),
                )
            }
            Target::CryptoResult(index) => self.crypto.search.matches().get(*index)?.to_string(),
            Target::CryptoCoin(index) => self.crypto_choices().into_iter().nth(*index)?,
            Target::AppsApp(index) => self.app_cards.apps().into_iter().nth(*index)?,
            Target::Cs2Stream(index) => self.cs2_streams().get(*index)?.0.url.clone(),
            _ => return Some(ItemId::Target(target.clone())),
        };
        Some(ItemId::Indexed(discriminant(target), key))
    }

    fn local_items(&self, controls: &[Control]) -> Vec<Item> {
        if matches!(self.overlay, Some(Overlay::SystemKeyboard)) {
            return vec![];
        }
        let today_tab = self.resolved_today_tab(controls);
        let mut items = Vec::new();
        let mut add = |target: &Target| {
            if let (Some(panel), Some(id)) =
                (self.local_panel(target, today_tab), self.local_id(target))
                && !items
                    .iter()
                    .any(|item: &Item| item.panel == panel && item.id == id)
            {
                items.push(Item {
                    panel,
                    target: target.clone(),
                    id,
                });
            }
        };
        // Recents draws only the nearby cards, in paint order. Use the actual list for navigation.
        if matches!(self.overlay, Some(Overlay::Menu { .. })) {
            for target in self.menu_items() {
                add(&target);
            }
        } else if self.recents_open() {
            if self.recents.pending.is_some() {
                return vec![];
            }
            for entry in &self.recents.entries {
                add(&Target::Recent(entry.app.id.clone()));
            }
            for control in controls
                .iter()
                .filter(|c| matches!(c.target, Target::RecentRetry | Target::RecentCloseAll))
            {
                add(&control.target);
            }
        } else {
            for control in controls {
                if list_target(&control.target).as_ref() == &control.target {
                    add(&control.target);
                }
            }
        }
        items
    }

    pub fn refresh_local_focus(&mut self, controls: &[Control], today_scroll_max: f32) -> bool {
        let overlay_kind = self.overlay.as_ref().map(discriminant);
        let mut wheel = std::mem::take(&mut self.local_wheel);
        let intent = wheel.intent.take();
        wheel.today_scroll_max = today_scroll_max;
        if wheel.overlay_kind != overlay_kind {
            let covers_parent = matches!(&self.overlay, Some(Overlay::Cs2Match { restore, .. }) if restore.today_line())
                || matches!(
                    self.overlay,
                    Some(Overlay::Delete { .. } | Overlay::BackgroundDelete { .. })
                );
            let previous = std::mem::take(&mut wheel.overlay);
            if covers_parent && let Some(kind) = wheel.overlay_kind {
                wheel.covered = Some((kind, previous));
            } else if let Some((kind, memory)) = wheel.covered.take()
                && Some(kind) == overlay_kind
            {
                wheel.overlay = memory;
            }
            wheel.overlay_kind = overlay_kind;
        }
        if self.page != Page::Settings {
            wheel.settings = ScreenMemory::default();
            wheel.settings_scope = None;
        }
        let original = self.focus.clone();
        if (overlay_kind.is_some() || self.page == Page::Settings)
            && !matches!(self.overlay, Some(Overlay::SystemKeyboard))
        {
            let target = list_target(&original);
            let items = self.local_items(controls);
            let proposed = self.local_panel(target.as_ref(), self.resolved_today_tab(controls));
            let memory = if overlay_kind.is_some() {
                &mut wheel.overlay
            } else {
                let subpage = self.subpage.as_ref().map(discriminant);
                let scope = (self.section, subpage);
                if wheel.settings_scope != Some(scope) {
                    // Closing a subpage ends that screen's memory; section lists stay open.
                    if let Some((_, Some(old))) = wheel.settings_scope {
                        wheel.settings.lists.retain(|(panel, _)| !matches!(panel, Panel::Settings(_, Some(page), _) if *page == old));
                    }
                    if !matches!(wheel.settings.owner, Some(Panel::Sidebar)) {
                        wheel.settings.owner = None;
                    }
                    wheel.settings_scope = Some(scope);
                }
                &mut wheel.settings
            };
            if let Some(proposed) = proposed {
                let proposed = if *target == Target::LocalContent && intent.is_none() {
                    memory.owner.unwrap_or(proposed)
                } else {
                    proposed
                };
                let next = memory.observe(
                    proposed,
                    target.as_ref(),
                    &items,
                    intent,
                    self.preferences.evolution_home.panel_focus,
                );
                if &next != target.as_ref() {
                    self.focus = next;
                    if !self.pointer {
                        self.reveal_recent_focus();
                    }
                }
                memory.presented = Some(list_target(&self.focus).into_owned());
            } else if intent.is_some() {
                memory.owner = None;
            }
        }
        self.local_wheel = wheel;
        let changed = self.focus != original;
        self.dirty |= changed;
        changed
    }

    fn reveal_recent_focus(&mut self) {
        if self.recents_open()
            && let Target::Recent(id) = &self.focus
            && let Some(index) = self
                .recents
                .entries
                .iter()
                .position(|entry| &entry.app.id == id)
        {
            self.recents.selected = index;
        }
    }

    pub(super) fn contextual_wheel(&mut self, controls: &[Control], x: i32, y: i32) -> bool {
        if self.overlay.is_none() && self.page != Page::Settings {
            return false;
        }
        self.evolution_discard_wheel();
        let delta = if self.recents_open() && x != 0 {
            x.saturating_neg()
        } else {
            y
        };
        if delta == 0 || self.moving.is_some() {
            return true;
        }
        let max = self.local_wheel.today_scroll_max;
        self.refresh_local_focus(controls, max);
        let memory = if self.overlay.is_some() {
            &self.local_wheel.overlay
        } else {
            &self.local_wheel.settings
        };
        let owner = memory.owner;
        let items: Vec<_> = self
            .local_items(controls)
            .into_iter()
            .filter(|item| Some(item.panel) == owner)
            .collect();
        if items.is_empty()
            && matches!(self.overlay, Some(Overlay::Today { .. }))
            && owner == Some(Panel::TodayBody(self.resolved_today_tab(controls)))
        {
            let next = (self.today_scroll - delta as f32 * 90.).clamp(0., max);
            self.dirty |= self.today_scroll != next;
            self.today_scroll = next;
        }
        if items.len() < 2 {
            return true;
        }
        let mut index = items
            .iter()
            .position(|item| &item.target == list_target(&self.focus).as_ref())
            .unwrap_or(0);
        for _ in 0..delta.unsigned_abs() {
            index = if delta < 0 {
                (index + 1) % items.len()
            } else {
                index.checked_sub(1).unwrap_or(items.len() - 1)
            };
        }
        let target = &items[index].target;
        if target != &self.focus {
            self.focus = target.clone();
            self.local_wheel.intent = Some(Intent::Direct);
            self.pointer = false;
            self.reveal_recent_focus();
            self.dirty = true;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Preferences;
    use crate::state::{Direction, Effect, Rect};

    fn control(target: Target, row: usize) -> Control {
        Control {
            target,
            rect: Rect::new(400., crate::layout::LIST_TOP + row as f32 * 90., 700., 74.),
            scrollable: true,
        }
    }

    fn settings() -> State {
        let mut state = State::new(Preferences::default());
        state.open(2);
        state.focus = Target::Setting(0);
        state
    }

    fn wheel(state: &mut State, controls: &[Control], y: i32) {
        assert!(state.wheel_scroll(controls, Some((100., 300.)), 0, y));
        state.refresh_local_focus(controls, state.local_wheel.today_scroll_max);
    }

    #[test]
    fn p04_local_steps_keep_the_focused_item_visible() {
        for count in [3, 24] {
            let mut state = settings();
            let controls: Vec<_> = (0..count).map(|i| control(Target::Setting(i), i)).collect();
            for index in 1..count {
                wheel(&mut state, &controls, -1);
                assert_eq!(state.focus, Target::Setting(index));
            }
            assert!(state.reveal_focus(&controls) == (count > 3));
            let focused = controls.last().unwrap().rect;
            assert!(focused.y + focused.h - state.scroll <= state.scroll_bottom());
        }
    }

    #[test]
    fn p04_bursts_wrap_only_over_eligible_items_in_step_order() {
        let mut state = settings();
        state.preferences.evolution_home.wheel_steps =
            serde_json::from_value(serde_json::json!(3)).unwrap();
        let controls = [
            control(Target::Setting(0), 0),
            control(Target::Setting(2), 2),
            control(Target::Setting(4), 4),
        ];
        for (delta, expected) in [(-1, 2), (-1, 4), (-1, 0), (1, 4), (2, 0), (-5, 4), (1, 2)] {
            wheel(&mut state, &controls, delta);
            assert_eq!(state.focus, Target::Setting(expected));
        }
    }

    #[test]
    fn p04_empty_and_single_item_panels_consume_without_falling_through() {
        let mut state = settings();
        let only = [control(Target::Setting(0), 0)];
        for delta in [-5, 4] {
            wheel(&mut state, &only, delta);
            assert_eq!(state.focus, Target::Setting(0));
        }
        let sidebar = [control(Target::Sidebar(0), 0)];
        wheel(&mut state, &sidebar, -3);
        assert_eq!(state.focus, Target::LocalContent);
        assert_eq!(state.page, Page::Settings);
        assert!(state.activate().is_empty());
    }

    #[test]
    fn p04_information_moves_its_viewport_without_selecting_a_tab_or_line() {
        let mut state = State::new(Preferences::default());
        state.overlay = Some(Overlay::Today {
            restore: Target::TodayCard,
        });
        state.focus = Target::LocalContent;
        let controls = [control(Target::TodayTab(crate::today::Tab::Crypto), 0)];
        state.refresh_local_focus(&controls, 230.);
        for (delta, scroll) in [(-2, 180.), (-2, 230.), (-1, 230.), (3, 0.)] {
            wheel(&mut state, &controls, delta);
            assert_eq!(state.focus, Target::LocalContent);
            assert_eq!(state.today_scroll, scroll);
            assert_eq!(state.today_tab, None);
            assert!(matches!(state.overlay, Some(Overlay::Today { .. })));
        }
    }

    #[test]
    fn p04_only_the_focused_settings_panel_receives_the_wheel() {
        let mut state = settings();
        let controls = [
            control(Target::Sidebar(0), 0),
            control(Target::Sidebar(1), 1),
            control(Target::Setting(0), 0),
            control(Target::Setting(1), 1),
        ];
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::Setting(1));
        state.pointer_focus(Target::Sidebar(0));
        state.refresh_local_focus(&controls, 0.);
        state.scroll = 48.;
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::Sidebar(1));
        assert_eq!(state.section, 0);
        assert_eq!(state.scroll, 48.);
    }

    #[test]
    fn p04_nested_menu_reveals_inputs_and_skips_hidden_ports() {
        let mut state = State::new(Preferences::default());
        state.open_menu();
        state.menu_inputs = true;
        state.inventory.inputs = (0..12)
            .map(|i| crate::model::Input {
                id: format!("input{i}"),
                title: format!("HDMI {i}"),
                port: format!("HDMI {i}"),
                connected: Some(i != 1),
                signal: None,
            })
            .collect();
        state.focus = Target::Input("input0".into());
        let viewport = crate::view::MENU_INPUT_VIEWPORT;
        let controls: Vec<_> = state
            .menu_items()
            .into_iter()
            .filter(|target| matches!(target, Target::Input(_)))
            .enumerate()
            .map(|(row, target)| Control {
                target,
                rect: Rect::new(90., viewport.y + row as f32 * 68., 300., 64.),
                scrollable: true,
            })
            .collect();
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::Input("input2".into()));
        wheel(&mut state, &controls, -9);
        assert_eq!(state.focus, Target::Input("input11".into()));
        assert!(state.reveal_focus(&controls));
        assert!(
            controls.last().unwrap().rect.y + 64. - state.menu_input_scroll
                <= viewport.y + viewport.h
        );
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::Input("input0".into()));
        assert!(matches!(state.overlay, Some(Overlay::Menu { .. })));
    }

    #[test]
    fn p04_panel_entry_uses_the_preference_and_screen_close_discards_memory() {
        let controls = [
            control(Target::Sidebar(0), 0),
            control(Target::Setting(0), 0),
            control(Target::Setting(1), 1),
        ];
        for (mode, expected) in [(PanelFocusMode::Restore, 1), (PanelFocusMode::First, 0)] {
            let mut state = settings();
            state.preferences.evolution_home.panel_focus = mode;
            wheel(&mut state, &controls, -1);
            assert_eq!(state.focus, Target::Setting(1));
            state.direction(Direction::Left, &controls);
            state.refresh_local_focus(&controls, 0.);
            assert_eq!(state.focus, Target::Sidebar(0));
            state.direction(Direction::Right, &controls);
            state.refresh_local_focus(&controls, 0.);
            assert_eq!(state.focus, Target::Setting(expected));
            state.open(0);
            state.refresh_local_focus(&[], 0.);
            state.open(2);
            state.refresh_local_focus(&controls, 0.);
            state.direction(Direction::Right, &controls);
            state.refresh_local_focus(&controls, 0.);
            assert_eq!(state.focus, Target::Setting(0));
        }
    }

    #[test]
    fn p04_recovery_uses_stable_ids_and_the_previous_order() {
        let mut state = settings();
        state.subpage = Some(Subpage::WeatherLocation);
        state.focus = Target::WeatherResult(1);
        let location = |id| crate::weather::Location {
            name: format!("Place {id}"),
            region: String::new(),
            country: String::new(),
            latitude: 0.,
            longitude: 0.,
            id: Some(id),
        };
        let cases: &[(&[u64], Target)] = &[
            (&[1, 2, 3, 4], Target::WeatherResult(1)),
            (&[2, 1, 3, 4], Target::WeatherResult(0)),
            (&[4, 3, 1], Target::WeatherResult(2)),
            (&[4, 3], Target::WeatherResult(1)),
            (&[], Target::LocalContent),
        ];
        for (ids, expected) in cases {
            state.weather.results = ids.iter().copied().map(location).collect();
            let controls: Vec<_> = (0..ids.len())
                .map(|i| control(Target::WeatherResult(i), i))
                .collect();
            state.refresh_local_focus(&controls, 0.);
            assert_eq!(&state.focus, expected);
        }
    }

    #[test]
    fn p04_wheel_never_activates_dialog_buttons_or_changes_stepper_values() {
        let mut state = settings();
        state.overlay = Some(Overlay::Delete {
            id: "example".into(),
            restore: Target::Setting(0),
        });
        state.focus = Target::CancelDelete;
        let controls = [
            control(Target::ConfirmDelete, 0),
            control(Target::CancelDelete, 1),
        ];
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::ConfirmDelete);
        assert!(state.uninstalling.is_none());
        assert!(matches!(state.overlay, Some(Overlay::Delete { .. })));
        state.back();
        let steps = state.preferences.evolution_home.wheel_steps;
        state.subpage = Some(Subpage::Wheel { steps });
        state.focus = Target::WheelStepsStep(true);
        let controls = [
            control(Target::WheelSteps, 0),
            control(Target::WheelStepsStep(true), 0),
            control(Target::WheelApply, 1),
        ];
        wheel(&mut state, &controls, -1);
        assert_eq!(state.focus, Target::WheelApply);
        assert_eq!(state.subpage, Some(Subpage::Wheel { steps }));
        assert_eq!(state.preferences.evolution_home.wheel_steps, steps);
    }

    #[test]
    fn p04_only_the_panel_focus_policy_persists() {
        let mut state = settings();
        assert_eq!(
            state.preferences.evolution_home.panel_focus,
            PanelFocusMode::Restore
        );
        state.subpage = Some(Subpage::Navigation);
        state.focus = Target::Choice(3);
        assert!(state.activate().is_empty());
        assert_eq!(state.subpage, Some(Subpage::PanelFocus));
        state.focus = Target::Choice(1);
        assert_eq!(state.activate(), vec![Effect::Save]);
        assert_eq!(state.focus, Target::Choice(3));
        let stored = serde_json::to_value(&state.preferences.evolution_home).unwrap();
        assert_eq!(stored["panel_focus"], "first");
        assert!(stored.get("focus").is_none());
        let restored: crate::home_evolution::Preferences = serde_json::from_value(stored).unwrap();
        assert_eq!(restored.panel_focus, PanelFocusMode::First);
        let legacy: crate::home_evolution::Preferences =
            serde_json::from_value(serde_json::json!({"wheel_steps":2})).unwrap();
        assert_eq!(legacy.panel_focus, PanelFocusMode::Restore);
        assert_eq!(legacy.wheel_steps.value(), 2);
    }
}
