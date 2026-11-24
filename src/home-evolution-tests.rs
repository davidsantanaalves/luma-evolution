use super::*;
use crate::home_evolution::{
    Area, GlobalAction, GlobalPosition, Home, WidgetApp, WidgetAppFixture,
};
use std::time::{Duration, Instant};

fn preview() -> State {
    let mut state = ready();
    state.evolution = Some(Home::default());
    state.open(0);
    state
}

fn p02_preview() -> (State, Vec<String>) {
    let inventory = demo_inventory();
    let ids: Vec<_> = inventory
        .apps
        .iter()
        .take(23)
        .map(|app| app.id.clone())
        .collect();
    let mut state = State::new(Preferences {
        favorites: Some(ids.clone()),
        ..Preferences::default()
    });
    state.evolution = Some(Home::default());
    state.set_inventory(inventory, None);
    state.open(0);
    (state, ids)
}

#[test]
fn p02_dock_reconciles_in_previous_order_and_ignores_a_stale_press() {
    let (mut state, ids) = p02_preview();
    state.evolution_focus_dock(Target::App(ids[2].clone()));
    state.pressed = Some(state.focus.clone());
    let mut inventory = state.inventory.clone();
    inventory.apps.retain(|app| !ids[2..5].contains(&app.id));
    state.set_inventory(inventory, None);
    assert_eq!(state.focus, Target::App(ids[5].clone()));
    assert!(state.pressed.is_none());
    assert!(state.overlay.is_none() && state.notice.is_none());

    state.evolution_focus_dock(Target::App(ids[22].clone()));
    let mut inventory = state.inventory.clone();
    inventory.apps.retain(|app| !ids[20..].contains(&app.id));
    state.set_inventory(inventory, None);
    assert_eq!(state.focus, Target::App(ids[19].clone()));

    state.options();
    state.focus = Target::Option(1);
    assert_eq!(state.activate(), vec![Effect::Save]);
    assert_eq!(state.focus, Target::App(ids[18].clone()));
    state.options();
    state.focus = Target::Option(0);
    assert_eq!(state.activate(), vec![Effect::Save]);
    assert_eq!(state.focus, Target::App(ids[17].clone()));

    state.preferences.favorites = Some(vec![]);
    state.reconcile_focus();
    assert_eq!(state.focus, Target::EvolutionGlobal(GlobalAction::Apps));
    assert!(state.evolution_geometry().empty_hint.is_some());
}

#[test]
fn p02_dock_preserves_newer_pointer_focus_and_remembers_eligible_ids() {
    let (mut state, ids) = p02_preview();
    let selected = Target::App(ids[18].clone());
    state.evolution_focus_dock(selected.clone());
    let before = state.evolution_geometry().offset;
    assert!(before > 0.);
    state
        .preferences
        .favorites
        .as_mut()
        .unwrap()
        .rotate_right(9);
    state.reconcile_focus();
    assert_eq!(state.focus, selected);
    assert!(state.evolution_geometry().offset < before);

    let global = Target::EvolutionGlobal(GlobalAction::Settings);
    state.pointer_focus(global.clone());
    state.preferences.hidden.push(ids[18].clone());
    state.reconcile_focus();
    assert_eq!(state.focus, global);
    assert!(state.pointer);
    assert_eq!(state.evolution_dock(), Target::App(ids[19].clone()));
    state.preferences.hidden.clear();
    state.reconcile_focus();
    assert_eq!(state.focus, global);
    assert_eq!(state.evolution_dock(), Target::App(ids[19].clone()));
}

#[test]
fn p02_dock_uses_the_old_successor_when_removal_and_reordering_coincide() {
    let (mut state, ids) = p02_preview();
    state.evolution_focus_dock(Target::App(ids[2].clone()));
    state.preferences.favorites = Some([18, 0, 1, 5, 4].map(|index| ids[index].clone()).to_vec());
    state.reconcile_focus();
    assert_eq!(state.focus, Target::App(ids[4].clone()));
}

#[test]
fn p02_dock_considers_a_favorite_that_becomes_eligible_in_the_same_inventory_update() {
    let (mut state, ids) = p02_preview();
    let mut unavailable = state.inventory.clone();
    unavailable.apps.retain(|app| app.id != ids[1]);
    state.set_inventory(unavailable, None);
    assert_eq!(state.focus, Target::App(ids[0].clone()));

    let mut updated = demo_inventory();
    updated.apps.retain(|app| app.id != ids[0]);
    state.set_inventory(updated, None);
    assert_eq!(state.focus, Target::App(ids[1].clone()));
}

#[test]
fn p02_applying_horizontal_policy_keeps_the_dock_selection_and_scale() {
    use crate::home_evolution::DockHorizontal;
    use crate::state::DOCK_ROW;
    let (mut state, ids) = p02_preview();
    let selected = Target::App(ids[20].clone());
    state.evolution_focus_dock(selected.clone());
    let offset = state.evolution_geometry().offset;
    let scale = state.preferences.evolution_home.cinema_dock_scale;
    state.open(2);
    state.focus = Target::Setting(DOCK_ROW);
    state.activate();
    state.direction(Direction::Down, &[]);
    state.direction(Direction::Left, &[]);
    assert_eq!(state.focus, Target::DockHorizontal);
    state.direction(Direction::Down, &[]);
    assert_eq!(
        state.activate(),
        vec![Effect::SaveDockSettings {
            scale,
            horizontal: DockHorizontal::Stop
        }]
    );
    state.dock_settings_saved(scale, DockHorizontal::Stop);
    assert_eq!(state.focus, Target::Setting(DOCK_ROW));
    state.open(0);
    assert_eq!(state.focus, selected);
    assert_eq!(state.evolution_geometry().offset, offset);
    assert_eq!(state.preferences.evolution_home.cinema_dock_scale, scale);
}

#[test]
fn p02_new_dock_session_loads_order_without_loading_focus() {
    let (mut state, ids) = p02_preview();
    state.preferences.hidden.push(ids[0].clone());
    state.evolution_focus_dock(Target::App(ids[9].clone()));
    state.open(1);
    state.open(0);
    assert_eq!(state.focus, Target::App(ids[9].clone()));
    let disk = serde_json::to_string(&state.preferences).unwrap();
    let mut fresh = State::new(serde_json::from_str(&disk).unwrap());
    fresh.evolution = Some(Home::default());
    fresh.set_inventory(state.inventory, None);
    fresh.open(0);
    assert_eq!(fresh.focus, Target::App(ids[1].clone()));
}

#[test]
fn p02_dock_policy_is_migrated_independently_and_does_not_change_moving() {
    use crate::home_evolution::DockHorizontal;
    for value in [json!(null), json!("custom"), json!(2)] {
        let prefs: Preferences = serde_json::from_value(
            json!({"evolution_home": {"cinema_dock_scale": 75, "dock_horizontal": value}}),
        )
        .unwrap();
        assert_eq!(
            prefs.evolution_home.dock_horizontal,
            DockHorizontal::Sequential
        );
        assert_eq!(prefs.evolution_home.cinema_dock_scale.label(), "75%");
    }
    for policy in [
        DockHorizontal::Stop,
        DockHorizontal::Sequential,
        DockHorizontal::WrapRow,
    ] {
        let (mut state, ids) = p02_preview();
        let columns = state.preferences.evolution_home.columns();
        state.preferences.evolution_home.dock_horizontal = policy;
        let selected = Target::App(ids[columns - 1].clone());
        state.evolution_start_move(selected.clone());
        state.direction(Direction::Right, &[]);
        assert_eq!(state.evolution_dock_items()[columns], selected);
        assert_eq!(state.focus, selected);
        state.back();
        assert_eq!(state.evolution_dock_items()[columns - 1], selected);
    }
}

#[test]
fn p02_spatial_destination_survives_an_in_flight_cinema_transition() {
    let (mut state, ids) = p02_preview();
    state.direction(Direction::Up, &[]);
    state.evolution_tick(0.23);
    assert!(state.evolution.as_ref().unwrap().expansion.moving());
    state.direction(Direction::Down, &[]);
    state.evolution_tick(0.04);
    for _ in 0..3 {
        state.direction(Direction::Down, &[]);
    }
    let last = Target::App(ids[21].clone());
    assert_eq!(state.focus, last);
    state.evolution_tick(1.);
    let layout = state.evolution_geometry();
    assert_eq!(state.focus, last);
    assert!(layout.offset > 0. && layout.scroll.down);
    let input = state.direction(Direction::Down, &[]);
    assert!(input.is_empty() && matches!(state.focus, Target::Input(_)));
    state.direction(Direction::Up, &[]);
    assert_eq!(state.focus, last);
    assert_eq!(state.evolution_geometry().offset, layout.offset);
}

#[test]
fn home_preview_back_restores_dock_and_keeps_catalog_selection_in_memory() {
    let mut state = preview();
    let dock = state.focus.clone();
    state.direction(Direction::Up, &controls(&state));
    assert!(matches!(state.focus, Target::EvolutionContent(_)));
    state.direction(Direction::Right, &controls(&state));
    assert_eq!(state.evolution.as_ref().unwrap().selected, 1);
    assert!(state.back().is_empty());
    assert_eq!(state.focus, dock);
    assert!(!state.evolution.as_ref().unwrap().expanded);
    assert_eq!(state.evolution.as_ref().unwrap().selected, 1);
    let stored = serde_json::to_string(&state.preferences).unwrap();
    assert!(!stored.contains("selected") && !stored.contains("expanded"));
}

#[test]
fn home_preview_regions_can_be_disabled_without_invisible_focus_targets() {
    let mut state = preview();
    state.preferences.evolution_home.show_top = false;
    state.preferences.evolution_home.show_center = false;
    let controls = controls(&state);
    assert!(!controls.iter().any(|control| matches!(
        control.target,
        Target::EvolutionWidget(_) | Target::EvolutionContent(_)
    )));
    state.direction(Direction::Up, &controls);
    assert_eq!(state.focus, Target::EvolutionGlobal(GlobalAction::Apps));
}

#[test]
fn preview_clicks_never_launch_fictional_titles_or_change_normal_preferences() {
    let mut state = preview();
    state.pointer_focus(Target::EvolutionContent("forest:2".into()));
    assert!(state.activate().is_empty());
    assert_eq!(state.evolution.as_ref().unwrap().selected, 2);
    assert!(state.notice.is_some());
    let normal = ready();
    assert!(normal.evolution.is_none());
}

#[test]
fn preview_hover_is_delayed_and_directionals_cancel_pending_hover() {
    let mut state = preview();
    state.pointer_focus(Target::EvolutionContent("forest:2".into()));
    state.evolution_tick(0.01);
    assert!(!state.evolution.as_ref().unwrap().expanded);
    state.evolution.as_mut().unwrap().hover.as_mut().unwrap().1 -=
        std::time::Duration::from_millis(251);
    state.evolution.as_mut().unwrap().rotation.elapsed = std::time::Duration::from_secs(5);
    state.evolution_tick(0.01);
    assert!(state.evolution.as_ref().unwrap().expanded);
    assert_eq!(state.evolution.as_ref().unwrap().selected, 2);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.progress(), 0.);
    state.pointer_focus(Target::EvolutionContent("summit:3".into()));
    state.direction(Direction::Down, &controls(&state));
    assert!(state.evolution.as_ref().unwrap().hover.is_none());
    assert!(matches!(state.focus, Target::App(_)));
}

#[test]
fn pointer_hover_preserves_second_dock_row_geometry() {
    let mut state = preview();
    state.preferences.favorites = Some(
        state
            .inventory
            .apps
            .iter()
            .take(15)
            .map(|app| app.id.clone())
            .collect(),
    );
    state.open(0);
    state.direction(Direction::Down, &controls(&state));
    let before = controls(&state);
    let first = before
        .iter()
        .find(|control| matches!(control.target, Target::App(_)))
        .unwrap();
    state.pointer_focus(first.target.clone());
    let after = controls(&state);
    for old in before
        .iter()
        .filter(|control| matches!(control.target, Target::App(_)))
    {
        let new = after
            .iter()
            .find(|control| control.target == old.target)
            .unwrap();
        assert_eq!(
            (old.rect.x, old.rect.y, old.rect.w, old.rect.h),
            (new.rect.x, new.rect.y, new.rect.w, new.rect.h)
        );
    }
}

#[test]
fn p01_dock_restores_the_offset_then_reconciles_the_remembered_id() {
    let mut state = preview();
    crate::scroll_preview::append(&mut state.inventory, 56, state.preferences.language);
    state.preferences.favorites = Some(
        state
            .inventory
            .apps
            .iter()
            .map(|app| app.id.clone())
            .collect(),
    );
    let prefs = &mut state.preferences.evolution_home;
    prefs.show_top = false;
    prefs.show_center = false;
    prefs.dock_size = crate::home_evolution::DockSize::Compact;
    state.open(0);
    let columns = state.preferences.evolution_home.columns();
    let rows = state.evolution_geometry().rows;
    for _ in 1..rows {
        state.direction(Direction::Down, &[]);
    }
    let last = state.focus.clone();
    let before = state.evolution_geometry();
    assert!(before.offset > 0.);
    let last_screen_y = before.dock.last().unwrap().y;
    state.direction(Direction::Down, &[]);
    assert!(matches!(state.focus, Target::Input(_)));
    assert!(state.evolution_geometry().offset > before.offset);
    state.direction(Direction::Up, &[]);
    assert_eq!(state.focus, last);
    assert_eq!(state.evolution_geometry().offset, before.offset);

    state.direction(Direction::Down, &[]);
    let removed = &state.preferences.favorites.as_ref().unwrap()[..columns];
    state
        .inventory
        .apps
        .retain(|app| !removed.contains(&app.id));
    state.reconcile_focus();
    state.direction(Direction::Up, &[]);
    assert_eq!(state.focus, last);
    let after = state.evolution_geometry();
    assert!(after.offset < before.offset);
    assert_eq!(after.dock.last().unwrap().y, last_screen_y);

    state.direction(Direction::Down, &[]);
    state.preferences.favorites = Some(vec![state.inventory.apps[0].id.clone()]);
    state.preferences.evolution_home.dock_size = crate::home_evolution::DockSize::Large;
    state.reconcile_focus();
    state.direction(Direction::Up, &[]);
    let layout = state.evolution_geometry();
    assert_ne!(state.focus, last);
    assert_eq!(state.focus, state.evolution_dock_items()[0]);
    assert_eq!(layout.offset, 0.);
    assert_eq!((layout.rows, layout.visible_rows), (1, 1));
}

#[test]
fn p01_dock_keeps_the_offset_across_cinema_expansion() {
    let mut state = preview();
    state.preferences.favorites = Some(
        state
            .inventory
            .apps
            .iter()
            .take(22)
            .map(|app| app.id.clone())
            .collect(),
    );
    state.open(0);
    for _ in 0..3 {
        state.direction(Direction::Down, &[]);
    }
    let offset = state.evolution_geometry().offset;
    assert!(offset > 0.);
    for _ in 0..3 {
        state.direction(Direction::Up, &[]);
        assert_eq!(state.evolution_geometry().offset, offset);
    }
    let first = state.focus.clone();
    state.direction(Direction::Up, &[]);
    state.evolution_tick(1.);
    assert!(matches!(state.focus, Target::EvolutionContent(_)));
    assert_eq!(state.evolution_geometry().offset, 0.);
    state.direction(Direction::Down, &[]);
    state.evolution_tick(1.);
    assert_eq!(state.focus, first);
    assert_eq!(state.evolution_geometry().offset, offset);
}

#[test]
fn p01_empty_hint_has_no_target_and_disappears_without_stealing_focus() {
    for position in GlobalPosition::ALL {
        let mut state = preview();
        let id = state.inventory.apps[0].id.clone();
        state.preferences.favorites = Some(vec![id.clone(), "missing.app".into()]);
        state.preferences.hidden.push(id.clone());
        state.preferences.evolution_home.global_position = position;
        state.open(0);
        let focus = state.focus.clone();
        let layout = state.evolution_geometry();
        let hint = layout.empty_hint.unwrap();
        assert!(layout.dock_bounds.h < 270.);
        let controls = controls(&state);
        let (x, y) = hint.center();
        assert!(state.hit_test(&controls, x, y).is_none());
        assert!(
            !controls
                .iter()
                .any(|control| matches!(control.target, Target::App(_)))
        );
        for action in GlobalAction::ALL {
            assert_eq!(
                controls
                    .iter()
                    .filter(|control| control.target == Target::EvolutionGlobal(action))
                    .count(),
                1
            );
        }

        state.preferences.hidden.retain(|hidden| hidden != &id);
        state.reconcile_focus();
        assert!(state.evolution_geometry().empty_hint.is_none());
        assert_eq!(state.focus, focus);
    }
}

#[test]
fn preview_preferences_round_trip_without_restoring_navigation_state() {
    let mut state = preview();
    state.preferences.evolution_home.show_top = false;
    state.preferences.evolution_home.hover_ms = 410;
    state.direction(Direction::Up, &controls(&state));
    let json = serde_json::to_vec(&state.preferences).unwrap();
    let restored: Preferences = serde_json::from_slice(&json).unwrap();
    assert!(!restored.evolution_home.show_top);
    assert_eq!(restored.evolution_home.hover_ms, 410);
    assert!(State::new(restored).evolution.is_none());
    let legacy: Preferences = serde_json::from_str("{}").unwrap();
    assert_eq!(legacy.evolution_home.hover_ms, 250);
}

#[test]
fn global_navigation_defaults_to_top_even_with_invalid_preferences() {
    for value in [
        serde_json::Value::Null,
        json!("obsolete"),
        json!(false),
        json!({"dock":true}),
    ] {
        let prefs: Preferences = serde_json::from_value(json!({
            "favorites": ["netflix"],
            "evolution_home": {"global_position": value, "global_slots": "invalid"}
        }))
        .unwrap();
        assert_eq!(prefs.evolution_home.global_position, GlobalPosition::Top);
        assert_eq!(prefs.favorites, Some(vec!["netflix".into()]));
        assert_eq!(prefs.evolution_home.global_slots, [0, 1]);
    }
    assert_eq!(
        Preferences::default().evolution_home.global_position,
        GlobalPosition::Top
    );
}

#[test]
fn changing_navigation_position_restores_the_same_action_and_preserves_favorites() {
    let mut state = preview();
    let favorites = state.preferences.favorites(&state.inventory.apps);
    let settings = Target::EvolutionGlobal(GlobalAction::Settings);
    let apps = Target::EvolutionGlobal(GlobalAction::Apps);
    for position in [GlobalPosition::Dock, GlobalPosition::Top] {
        state.focus = settings.clone();
        state.activate();
        assert_eq!(state.page, Page::Settings);
        state.focus = Target::Setting(crate::state::NAVIGATION_ROW);
        state.activate();
        assert_eq!(state.subpage, Some(crate::state::Subpage::Navigation));
        state.focus = Target::Choice(position.index());
        assert_eq!(state.activate(), vec![Effect::Save]);
        assert_eq!(state.focus, Target::Setting(crate::state::NAVIGATION_ROW));
        state.back();
        state.back();
        assert_eq!(state.focus, settings);
        assert_eq!(
            state.preferences.favorites(&state.inventory.apps),
            favorites
        );
        let dock = state.evolution_dock_items();
        let top = state.evolution_top_targets();
        for action in [&apps, &settings] {
            assert_eq!(
                top.iter()
                    .chain(&dock)
                    .filter(|item| *item == action)
                    .count(),
                1
            );
            assert_eq!(dock.contains(action), position == GlobalPosition::Dock);
        }
        let persisted = serde_json::to_value(&state.preferences).unwrap();
        let loaded: Preferences = serde_json::from_value(persisted).unwrap();
        assert_eq!(loaded.evolution_home.global_position, position);
    }
}

#[test]
fn permanent_actions_move_without_changing_real_favorites_and_have_no_remove_operation() {
    let mut state = preview();
    state.preferences.evolution_home.global_position = GlobalPosition::Dock;
    let original = state.evolution_dock_items();
    let favorites = state.preferences.favorites.clone();
    let target = Target::EvolutionGlobal(GlobalAction::Apps);
    state.evolution_focus_dock(target.clone());
    state.options();
    assert!(state.overlay.is_none());
    assert!(state.evolution_moving());
    assert!(
        !controls(&state)
            .iter()
            .any(|control| matches!(control.target, Target::Option(_)))
    );
    state.direction(Direction::Down, &controls(&state));
    let moved = state.evolution_dock_items();
    assert_ne!(moved, original);
    state.back();
    assert_eq!(state.evolution_dock_items(), original);
    state.options();
    state.direction(Direction::Down, &controls(&state));
    assert_eq!(state.activate(), vec![Effect::Save]);
    assert_eq!(state.evolution_dock_items(), moved);
    assert_eq!(state.focus, target);
    assert_eq!(state.preferences.favorites, favorites);
    state.preferences.favorites = Some(vec![]);
    state.open(0);
    assert_eq!(state.evolution_dock_items().len(), 2);
    assert!(
        controls(&state)
            .iter()
            .any(|control| control.target == target)
    );
}

#[test]
fn inputs_follow_the_last_dock_row_and_reconcile_when_disconnected() {
    let mut state = preview();
    state.preferences.favorites = Some(
        state
            .inventory
            .apps
            .iter()
            .take(15)
            .map(|app| app.id.clone())
            .collect(),
    );
    state.open(0);
    let first_dock = state.evolution_geometry().dock_start;
    state.wheel_scroll(&[], None, 0, -1);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
    state.evolution_tick(1.);
    let layout = state.evolution_geometry();
    assert!(layout.offset > 0.);
    assert_eq!(
        layout.dock_start + layout.dock.len(),
        state.evolution_dock_items().len()
    );
    assert!(!layout.scroll.down && !layout.inputs.is_empty());
    state.wheel_scroll(&[], None, 0, 1);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Dock));
    state.evolution_tick(1.);
    assert_eq!(state.evolution_geometry().dock_start, first_dock);
    state.direction(Direction::Down, &[]);
    assert!(
        !controls(&state)
            .iter()
            .any(|c| matches!(c.target, Target::Input(_)))
    );
    state.direction(Direction::Down, &controls(&state));
    assert!(matches!(state.focus, Target::App(_)));
    state.direction(Direction::Down, &controls(&state));
    let last = state.focus.clone();
    assert!(matches!(last, Target::App(_)));
    assert!(
        !controls(&state)
            .iter()
            .any(|c| matches!(c.target, Target::Input(_)))
    );
    state.direction(Direction::Down, &controls(&state));
    let input = state.focus.clone();
    let Target::Input(id) = input else {
        panic!("expected first available input")
    };
    assert_eq!(state.activate(), vec![Effect::Launch(id)]);
    state.direction(Direction::Up, &controls(&state));
    assert_eq!(state.focus, last);
    assert!(
        !controls(&state)
            .iter()
            .any(|c| matches!(c.target, Target::Input(_)))
    );
    state.direction(Direction::Down, &controls(&state));
    state.set_inputs(vec![]);
    assert_eq!(state.focus, last);
    assert!(
        !controls(&state)
            .iter()
            .any(|control| matches!(control.target, Target::Input(_)))
    );
}

#[test]
fn legacy_square_preference_migrates_without_losing_favorites_or_other_settings() {
    for (size, width) in [("compact", 192.), ("standard", 224.), ("large", 256.)] {
        let prefs: Preferences = serde_json::from_value(json!({
            "favorites": ["netflix", "youtube", "disney"],
            "greeting_name": "Ana", "hour_24": false,
            "evolution_home": {"square_icons": true, "dock_size": size,
                "global_position": "dock", "global_slots": [4, 2], "hover_ms": 420}
        }))
        .unwrap();
        assert_eq!(prefs.evolution_home.card(), (width, width / 1.6));
        let saved = serde_json::to_value(&prefs).unwrap();
        assert!(saved["evolution_home"].get("square_icons").is_none());
        let restored: Preferences = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.favorites, prefs.favorites);
        assert_eq!(restored.greeting_name, "Ana");
        assert!(!restored.hour_24);
        assert_eq!(
            restored.evolution_home.global_position,
            GlobalPosition::Dock
        );
        assert_eq!(restored.evolution_home.global_slots, [4, 2]);
        assert_eq!(restored.evolution_home.hover_ms, 420);
        assert_eq!(restored.evolution_home.card(), prefs.evolution_home.card());
    }
}

#[test]
fn compact_header_uses_current_clock_region_and_widget_preferences() {
    let mut prefs = Preferences {
        language: crate::i18n::Language::Pt,
        hour_24: true,
        ..Preferences::default()
    };
    let now = time::OffsetDateTime::parse(
        "2026-11-21T20:41:00-03:00",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let header = crate::home_evolution::compact_header(&prefs, now);
    assert!(header.contains("Boa noite") && header.contains("20:41"));
    assert!(header.contains(&prefs.date_format.format(prefs.language, now)));
    prefs.date_format = crate::model::DateFormat::YearMonthDay;
    prefs.hour_24 = false;
    prefs.show_seconds = true;
    let header = crate::home_evolution::compact_header(&prefs, now);
    assert!(header.contains("2026-11-21"));
    assert!(header.contains(&prefs.language.clock(now, false, true)));
    prefs.show_date = false;
    prefs.show_greeting = false;
    assert_eq!(
        crate::home_evolution::compact_header(&prefs, now),
        prefs.language.clock(now, false, true)
    );
}

#[test]
fn rapid_reversals_keep_focus_visible_and_pointer_press_freezes_geometry() {
    let mut state = preview();
    let favorites = state.evolution_dock_items();
    for _ in 0..8 {
        state.direction(Direction::Up, &controls(&state));
        state.evolution_tick(0.08);
        state.direction(Direction::Right, &controls(&state));
        state.direction(Direction::Down, &controls(&state));
        state.evolution_tick(0.04);
        assert!(controls(&state).iter().any(|c| c.target == state.focus));
        assert_eq!(state.evolution_dock_items(), favorites);
    }
    state.direction(Direction::Up, &controls(&state));
    state.evolution_tick(0.1);
    state.pointer = true;
    state.pressed = Some(state.focus.clone());
    let before = state.evolution.as_ref().unwrap().expansion.value;
    state.evolution_tick(0.2);
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, before);
    state.pressed = None;
    state.pointer = false;
    state.preferences.reduced_motion = true;
    state.evolution_tick(0.01);
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, 1.);
}

#[test]
fn reduced_and_no_animation_profiles_snap_on_the_first_frame() {
    use crate::home_evolution::MotionProfile;
    for profile in [MotionProfile::Reduced, MotionProfile::None] {
        let mut state = preview();
        state.preferences.evolution_home.motion_profile = profile;
        state.direction(Direction::Up, &controls(&state));
        state.evolution_tick(0.);
        assert_eq!(state.evolution.as_ref().unwrap().expansion.value, 1.);
        state.direction(Direction::Down, &controls(&state));
        state.evolution_tick(0.);
        assert_eq!(state.evolution.as_ref().unwrap().expansion.value, 0.);
    }
}

#[test]
fn evolution_home_never_opens_the_inherited_side_menu() {
    let mut state = preview();
    state.direction(Direction::Left, &controls(&state));
    state.open_menu();
    assert!(state.overlay.is_none());
    state.direction(Direction::Up, &controls(&state));
    state.direction(Direction::Left, &controls(&state));
    assert_eq!(state.evolution.as_ref().unwrap().selected, -1);
    assert!(state.overlay.is_none());
    state.direction(Direction::Up, &controls(&state));
    state.direction(Direction::Up, &controls(&state));
    assert_eq!(state.focus, Target::EvolutionGlobal(GlobalAction::Apps));
    state.activate();
    assert_eq!(state.page, Page::Apps);
}

#[test]
fn settings_provides_tv_settings_and_about_and_keeps_the_parent_row() {
    let mut state = preview();
    state.focus = Target::EvolutionGlobal(GlobalAction::Settings);
    state.activate();
    state.section = crate::state::SYSTEM_SECTION;
    for index in [crate::state::TV_SETTINGS_ROW, crate::state::ABOUT_ROW] {
        assert!(
            controls(&state)
                .iter()
                .any(|control| control.target == Target::Setting(index))
        );
    }
    state.focus = Target::Setting(crate::state::TV_SETTINGS_ROW);
    assert_eq!(
        state.activate(),
        vec![Effect::Launch("com.palm.app.settings".into())]
    );
    state.focus = Target::Setting(crate::state::ABOUT_ROW);
    state.activate();
    assert_eq!(state.subpage, Some(crate::state::Subpage::About));
    assert!(
        controls(&state)
            .iter()
            .any(|control| control.target == Target::CloseDialog)
    );
    state.back();
    assert_eq!(state.focus, Target::Setting(crate::state::ABOUT_ROW));
}

#[test]
fn center_widgets_remain_reachable_from_today() {
    let mut state = preview();
    state.preferences.show_shows = true;
    state.preferences.shows_today = false;
    assert_eq!(
        state.preferences.center(),
        Some(crate::model::Center::Shows)
    );
    assert!(state.evolution_widget_in_today(crate::model::Center::Shows));
    assert!(state.evolution_top_targets().contains(&Target::TodayCard));
    state.focus = Target::TodayCard;
    state.activate();
    assert!(matches!(state.overlay, Some(Overlay::Today { .. })));
    state.back();
    assert_eq!(state.focus, Target::TodayCard);
}

#[test]
fn dock_margins_and_incomplete_rows_are_geometrically_symmetric() {
    use crate::home_evolution::DockSize;
    for size in [DockSize::Compact, DockSize::Standard, DockSize::Large] {
        let mut state = preview();
        state.preferences.evolution_home.dock_size = size;
        state.preferences.evolution_home.show_top = false;
        state.preferences.evolution_home.show_center = false;
        let columns = state.preferences.evolution_home.columns();
        for count in [1, columns, columns + 2, 30] {
            state.preferences.favorites = Some(
                state
                    .inventory
                    .apps
                    .iter()
                    .take(count)
                    .map(|app| app.id.clone())
                    .collect(),
            );
            state.open(0);
            for expansion in [0., 0.5, 1.] {
                state.evolution.as_mut().unwrap().expansion.value = expansion;
                let g = state.evolution_geometry();
                let first = g.dock.first().unwrap();
                let bottom = g.dock.iter().map(|r| r.y + r.h).fold(0., f32::max);
                let top_margin = first.y - g.dock_bounds.y;
                assert!((top_margin - (g.dock_bounds.y + g.dock_bounds.h - bottom)).abs() < 0.001);
                for row in g.dock.chunks(columns) {
                    let center = (row[0].x + row.last().unwrap().x + row.last().unwrap().w) / 2.;
                    assert!((center - 960.).abs() < 0.001);
                    assert!(row.iter().all(|r| (r.w / r.h - 1.6).abs() < 0.001));
                }
            }
        }
    }
}

#[test]
fn weather_uses_the_right_safe_edge_below_the_actions_without_moving_hit_targets() {
    let mut state = preview();
    let idle = crate::home_evolution::header_geometry(&state);
    assert_eq!(idle.weather.x + idle.weather.w, 1840.);
    assert!(idle.weather.y > idle.buttons[0].y + idle.buttons[0].h);
    state.focus = Target::EvolutionGlobal(GlobalAction::Settings);
    state.evolution_tick(0.3);
    let focused = crate::home_evolution::header_geometry(&state);
    assert_eq!(idle.buttons[0].y, focused.buttons[0].y);
    assert_eq!(
        focused.clock.y - idle.clock.y,
        focused.weather.y - idle.weather.y
    );
    assert!(focused.clock.y > idle.clock.y);
    state.preferences.evolution_home.global_position = GlobalPosition::Dock;
    let dock = crate::home_evolution::header_geometry(&state);
    assert_eq!(dock.weather.x + dock.weather.w, 1840.);
    assert!(dock.buttons.is_empty() && dock.weather.y < idle.weather.y);
}

#[test]
fn leaving_button_hover_clears_the_label_and_geometry_without_losing_remote_memory() {
    let mut state = preview();
    let target = Target::EvolutionGlobal(GlobalAction::Settings);
    let idle = crate::home_evolution::header_geometry(&state);
    state.pointer_focus(target.clone());
    state.evolution_tick(0.3);
    assert!(crate::home_evolution::header_geometry(&state).clock.y > idle.clock.y);
    state.evolution_pointer(None, std::time::Instant::now());
    state.evolution_tick(0.3);
    assert_eq!(
        crate::home_evolution::header_geometry(&state).clock.y,
        idle.clock.y
    );
    assert_eq!(state.focus, target);
    assert!(
        !state
            .evolution
            .as_ref()
            .unwrap()
            .button_focus
            .iter()
            .any(|t| t.moving())
    );
    assert_eq!(
        state
            .evolution_top_targets()
            .iter()
            .filter(|t| matches!(t, Target::EvolutionGlobal(_)))
            .count(),
        2
    );
}

fn widget_apps(state: &mut State, calendar: bool, weather: bool) {
    state.preferences.show_weather = true;
    state.evolution.as_mut().unwrap().widget_apps = WidgetAppFixture { calendar, weather };
    state.reconcile_focus();
}

#[test]
fn widget_app_matrix_is_independent_of_weather_data() {
    for (calendar, weather, expected) in [
        (true, true, vec![WidgetApp::Calendar, WidgetApp::Weather]),
        (false, true, vec![WidgetApp::Weather]),
        (true, false, vec![]),
        (false, false, vec![]),
    ] {
        let mut state = preview();
        widget_apps(&mut state, calendar, weather);
        state.evolution.as_mut().unwrap().weather_fixture = if weather {
            crate::home_evolution::WeatherFixture::Error
        } else {
            crate::home_evolution::WeatherFixture::Ready
        };
        state.direction(Direction::Up, &[]);
        state.direction(Direction::Up, &[]);
        assert_eq!(
            state.focus,
            expected
                .first()
                .map_or(Target::EvolutionGlobal(GlobalAction::Apps), |app| {
                    Target::EvolutionWidget(*app)
                })
        );
        if expected.is_empty() {
            state.wheel_scroll(&[], None, 0, -1);
        }
        state.evolution_tick(1.);
        let controls = controls(&state);
        let actual: Vec<_> = controls
            .iter()
            .filter_map(|control| match control.target {
                Target::EvolutionWidget(app) => Some(app),
                _ => None,
            })
            .collect();
        assert_eq!(actual, expected);
        assert!(state.evolution.as_ref().unwrap().top_open);
        assert!(
            state
                .evolution
                .as_ref()
                .unwrap()
                .simulated_widget_open
                .is_none()
        );
    }
}

#[test]
fn widgets_have_whole_block_hit_regions_and_require_explicit_activation() {
    let mut state = preview();
    widget_apps(&mut state, true, true);
    let compact = crate::home_evolution::header_geometry(&state);
    assert_eq!(
        state.hit_test(
            &controls(&state),
            compact.clock.x + 30.,
            compact.clock.y + 30.
        ),
        None
    );
    state.direction(Direction::Up, &[]);
    state.direction(Direction::Up, &[]);
    state.evolution_tick(1.);
    let header = crate::home_evolution::header_geometry(&state);
    let controls = controls(&state);
    for y in [30., 75., 105., header.summary_height + 120.] {
        assert_eq!(
            state.hit_test(&controls, header.clock.x + 30., header.clock.y + y),
            Some(Target::EvolutionWidget(WidgetApp::Calendar))
        );
    }
    assert_eq!(
        (header.clock.y, header.clock.w, header.clock.h),
        (header.weather.y, header.weather.w, header.weather.h)
    );
    let cache = state.weather.cache.clone();
    for (app, rect) in [
        (WidgetApp::Calendar, header.clock),
        (WidgetApp::Weather, header.weather),
    ] {
        let (x, y) = rect.center();
        let target = state.hit_test(&controls, x, y).unwrap();
        assert_eq!(target, Target::EvolutionWidget(app));
        state.pointer_focus(target);
        assert_ne!(
            state.evolution.as_ref().unwrap().simulated_widget_open,
            Some(app)
        );
        assert!(state.activate().is_empty());
        assert_eq!(
            state.evolution.as_ref().unwrap().simulated_widget_open,
            Some(app)
        );
        assert!(state.overlay.is_none());
    }
    state.evolution_hover_gap(Some((970., 280.)), Instant::now());
    assert_eq!(state.focus, Target::EvolutionArea(Area::Widgets));
    assert!(state.evolution.as_ref().unwrap().top_open);
    state.evolution_hover_gap(Some((970., 680.)), Instant::now());
    assert!(!state.evolution.as_ref().unwrap().top_open);
    assert_eq!(state.weather.cache, cache);
}

#[test]
fn widget_focus_recovers_locally_and_back_restores_the_dock() {
    let mut state = preview();
    widget_apps(&mut state, true, true);
    let dock = state.focus.clone();
    state.direction(Direction::Up, &[]);
    state.direction(Direction::Right, &[]);
    let selected = state.evolution.as_ref().unwrap().selected;
    state.direction(Direction::Up, &[]);
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Calendar));
    state.direction(Direction::Right, &[]);
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Weather));
    state.preferences.show_weather = false;
    state.reconcile_focus();
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Calendar));
    state.direction(Direction::Right, &[]);
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Calendar));
    state.preferences.show_weather = true;
    state.direction(Direction::Right, &[]);
    state.direction(Direction::Left, &[]);
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Calendar));
    widget_apps(&mut state, false, true);
    assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Weather));
    widget_apps(&mut state, true, false);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Widgets));
    assert!(state.activate().is_empty());
    let home = state.evolution.as_ref().unwrap();
    assert!(home.top_open && home.simulated_widget_open.is_none());
    assert_eq!(home.selected, selected);
    state.back();
    assert_eq!(state.focus, dock);
    assert!(!state.evolution.as_ref().unwrap().top_open);
}

#[test]
fn wheel_expands_areas_before_directions_can_select_items() {
    let mut state = preview();
    widget_apps(&mut state, true, true);
    let initial_dock = state.focus.clone();
    for pointer in [false, true] {
        state.pointer = pointer;
        for area in [Area::Cinema, Area::Widgets] {
            assert!(state.wheel_scroll(&[], None, 0, 1));
            assert_eq!(state.focus, Target::EvolutionArea(area));
            assert!(state.activate().is_empty());
            assert_eq!(state.pointer, pointer);
        }
        assert_eq!(state.evolution.as_ref().unwrap().selected, 0);
        assert!(state.evolution.as_ref().unwrap().top_open);
        assert!(
            state
                .evolution
                .as_ref()
                .unwrap()
                .simulated_widget_open
                .is_none()
        );
        state.direction(Direction::Right, &[]);
        assert_eq!(state.focus, Target::EvolutionWidget(WidgetApp::Calendar));
        for area in [Area::Cinema, Area::Dock] {
            state.wheel_scroll(&[], None, 0, -1);
            assert_eq!(state.focus, Target::EvolutionArea(area));
        }
        state.direction(Direction::Down, &[]);
        assert_eq!(state.focus, initial_dock);
    }
    state.evolution.as_mut().unwrap().step(2);
    state.wheel_scroll(&[], None, 0, 1);
    assert_eq!(state.evolution.as_ref().unwrap().selected, 2);
    state.direction(Direction::Right, &[]);
    assert_eq!(state.focus, Target::EvolutionContent("dunes:0".into()));
    assert_eq!(state.evolution.as_ref().unwrap().selected, 0);
}

#[test]
fn p03_wheel_reaches_every_area_and_enters_the_first_eligible_control() {
    let (mut state, ids) = p02_preview();
    widget_apps(&mut state, true, true);
    let first_top = state.evolution_top_targets()[0].clone();
    let first_input = Target::Input(state.evolution_inputs()[0].id.clone());
    let now = Instant::now();

    for pointer in [false, true] {
        state.evolution_focus_dock(Target::App(ids[18].clone()));
        state.pointer = pointer;
        for (steps, area, expected) in [
            (3, Area::Actions, first_top.clone()),
            (
                -1,
                Area::Widgets,
                Target::EvolutionWidget(WidgetApp::Calendar),
            ),
            (-1, Area::Cinema, Target::EvolutionContent("dunes:0".into())),
            (-1, Area::Dock, Target::App(ids[0].clone())),
            (-1, Area::Inputs, first_input.clone()),
        ] {
            assert!(state.evolution_wheel(steps, now));
            assert_eq!(state.focus, Target::EvolutionArea(area));
            assert!(state.activate().is_empty());
            assert_eq!(state.pointer, pointer && area == Area::Actions);
            assert!(state.direction(Direction::Left, &[]).is_empty());
            assert_eq!(state.focus, expected);
        }
    }

    state.open(0);
    widget_apps(&mut state, false, false);
    assert!(state.evolution_wheel(2, now));
    assert_eq!(state.focus, Target::EvolutionArea(Area::Widgets));
    assert!(state.direction(Direction::Up, &[]).is_empty());
    assert_eq!(state.focus, Target::EvolutionArea(Area::Widgets));

    state.open(0);
    state.preferences.evolution_home.show_top = false;
    state.preferences.evolution_home.show_center = false;
    assert!(state.evolution_wheel(1, now));
    assert_eq!(state.focus, Target::EvolutionArea(Area::Actions));
    assert!(state.evolution_wheel(-2, now));
    assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
}

#[test]
fn p03_wheel_counts_thresholds_and_discards_excess_at_both_extremes() {
    use crate::home_evolution::WheelSteps;
    let now = Instant::now();
    for threshold in [2, 3] {
        let mut state = preview();
        state.preferences.evolution_home.wheel_steps =
            serde_json::from_value::<WheelSteps>(json!(threshold)).unwrap();
        let initial = state.focus.clone();
        state.evolution_wheel(threshold - 1, now);
        assert_eq!(state.focus, initial);
        state.evolution_wheel(1, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Cinema));
        state.evolution_wheel(i32::MAX, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Actions));
        state.evolution_wheel(-(threshold - 1), now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Actions));
        state.evolution_wheel(-1, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Widgets));
        state.evolution_wheel(i32::MIN, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
        state.evolution_wheel(threshold - 1, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
        state.evolution_wheel(1, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Dock));
    }
}

#[test]
fn p03_wheel_restarts_on_inversion_and_expires_at_800_ms_between_steps() {
    let now = Instant::now();
    for saldo in [1, 2] {
        let mut state = preview();
        state.preferences.evolution_home.wheel_steps = serde_json::from_value(json!(3)).unwrap();
        let dock = state.focus.clone();
        state.evolution_wheel(saldo, now);
        state.evolution_wheel(-1, now);
        state.evolution_wheel(-1, now);
        assert_eq!(state.focus, dock);
        state.evolution_wheel(-1, now);
        assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
    }

    for pause in [799, 800, 801] {
        let mut state = preview();
        state.preferences.evolution_home.wheel_steps = serde_json::from_value(json!(3)).unwrap();
        let dock = state.focus.clone();
        let second = now + Duration::from_millis(799);
        state.evolution_wheel(1, now);
        state.evolution_wheel(1, second);
        assert_eq!(state.focus, dock);
        let third = second + Duration::from_millis(pause);
        assert!(!state.evolution_wheel(0, third));
        state.evolution_wheel(1, third);
        if pause >= 800 {
            assert_eq!(state.focus, dock);
            state.evolution_wheel(1, third);
            assert_eq!(state.focus, dock);
            state.evolution_wheel(1, third);
        }
        assert_eq!(state.focus, Target::EvolutionArea(Area::Cinema));
        assert!(state.activate().is_empty());
    }
}

#[test]
fn p03_wheel_resets_only_on_intentional_motion_or_direction() {
    let now = Instant::now();
    for (pointer, motion, direction, discarded) in [
        (false, (0, 0), None, false),
        (true, (0, 0), None, false),
        (false, (1, 0), None, true),
        (true, (0, -1), None, true),
        (false, (0, 0), Some(Direction::Right), true),
    ] {
        let mut state = preview();
        state.pointer = pointer;
        state.preferences.evolution_home.wheel_steps = serde_json::from_value(json!(3)).unwrap();
        state.evolution_wheel(2, now);
        let focus = state.focus.clone();
        assert_eq!(
            state.evolution_pointer_motion(motion.0, motion.1),
            motion != (0, 0)
        );
        assert_eq!(state.focus, focus);
        if let Some(direction) = direction {
            assert!(state.direction(direction, &[]).is_empty());
        }
        let focus = state.focus.clone();
        state.evolution_wheel(1, now);
        if discarded {
            assert_eq!(state.focus, focus);
            state.evolution_wheel(2, now);
        }
        assert_eq!(state.focus, Target::EvolutionArea(Area::Cinema));
    }
}

#[test]
fn p03_wheel_retargets_expansion_and_viewport_before_the_previous_tween_finishes() {
    let (mut state, _) = p02_preview();
    state.preferences.reduced_motion = false;
    let now = Instant::now();
    state.evolution_wheel(1, now);
    state.evolution_tick(0.12);
    let expansion = state.evolution.as_ref().unwrap().expansion.value;
    assert!(expansion > 0. && expansion < 1.);
    state.evolution_wheel(2, now);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Actions));
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, expansion);
    state.evolution_tick(0.10);
    let closing = state.evolution.as_ref().unwrap().expansion.value;
    assert!(closing > 0. && closing < expansion);
    state.evolution_wheel(-2, now);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Cinema));
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, closing);
    state.evolution_tick(0.06);
    assert!(state.evolution.as_ref().unwrap().expansion.value > closing);

    state.evolution_wheel(-2, now);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Inputs));
    state.evolution_tick(0.12);
    let offset = state.evolution_geometry().offset;
    assert!(state.evolution.as_ref().unwrap().wheel_viewport_moving());
    assert!(offset > 0.);
    state.evolution_wheel(2, now);
    assert_eq!(state.focus, Target::EvolutionArea(Area::Cinema));
    assert_eq!(state.evolution_geometry().offset, offset);
    state.evolution_tick(1.);
    assert_eq!(state.evolution_geometry().offset, 0.);
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, 1.);
}

#[test]
fn p03_wheel_is_ignored_in_menu_dialog_settings_and_move() {
    let mut state = preview();
    state.preferences.evolution_home.wheel_steps = serde_json::from_value(json!(2)).unwrap();
    let dock = state.focus.clone();
    let now = Instant::now();
    state.evolution_wheel(1, now);
    // The Home preview has no side-menu entry point. Exercise a covered Home surface directly.
    let menu = Target::Nav(0);
    state.overlay = Some(Overlay::Menu {
        restore: dock.clone(),
    });
    state.focus = menu.clone();
    assert!(!state.wheel_scroll(&[], None, 0, 100));
    assert_eq!(state.focus, menu);
    state.back();
    assert_eq!(state.focus, dock);
    state.evolution_wheel(1, now);
    assert_eq!(state.focus, dock);

    state.options();
    let option = state.focus.clone();
    assert!(state.overlay.is_some());
    assert!(!state.wheel_scroll(&[], None, 0, 100));
    assert_eq!(state.focus, option);
    state.back();
    state.evolution_start_move(dock.clone());
    assert!(state.evolution_moving());
    assert!(!state.wheel_scroll(&[], None, 0, 100));
    assert_eq!(state.focus, dock);

    state.open(2);
    let setting = state.focus.clone();
    assert!(!state.wheel_scroll(&[], None, 0, 100));
    assert_eq!(state.page, Page::Settings);
    assert_eq!(state.focus, setting);
}

#[test]
fn p03_wheel_preference_decodes_only_supported_values() {
    use crate::home_evolution::Preferences;
    assert_eq!(Preferences::default().wheel_steps.value(), 1);
    for (stored, expected) in [
        (json!(1), 1),
        (json!(2), 2),
        (json!(3), 3),
        (json!(4), 1),
        (json!(-1), 1),
        (json!(2.0), 1),
        (json!("3"), 1),
        (json!(null), 1),
    ] {
        let prefs: Preferences =
            serde_json::from_value(json!({"wheel_steps": stored, "show_center": false})).unwrap();
        assert!(!prefs.show_center);
        assert_eq!(prefs.wheel_steps.value(), expected);
        assert_eq!(
            serde_json::to_value(prefs).unwrap()["wheel_steps"],
            expected
        );
    }
}

#[test]
fn cinema_scale_has_one_reference_across_home_areas() {
    let mut state = preview();
    widget_apps(&mut state, true, true);
    let stored = serde_json::to_string(&state.preferences).unwrap();
    let normal = state.evolution_geometry();
    assert_eq!(normal.cinema_card_height, 164.);
    state.focus = Target::EvolutionGlobal(GlobalAction::Settings);
    state.evolution_tick(1.);
    assert_eq!(
        state.evolution_geometry().cinema_card_height,
        normal.cinema_card_height
    );
    state.direction(Direction::Down, &[]);
    state.evolution_tick(1.);
    let expanded_widgets = state.evolution_geometry();
    let header = crate::home_evolution::header_geometry(&state);
    assert!(expanded_widgets.center.y > header.bottom);
    assert_eq!(
        expanded_widgets.cinema_card_height,
        normal.cinema_card_height * 0.5
    );
    assert_eq!(state.evolution.as_ref().unwrap().expansion.value, 0.);
    assert!(expanded_widgets.dock[0].w < normal.dock[0].w * 0.8);
    assert!(state.evolution.as_ref().unwrap().rotation.paused);
    state.direction(Direction::Down, &[]);
    state.evolution_tick(1.);
    assert!(matches!(state.focus, Target::EvolutionContent(_)));
    assert_eq!(
        state.evolution_geometry().cinema_card_height,
        normal.cinema_card_height
    );
    assert_eq!(stored, serde_json::to_string(&state.preferences).unwrap());
}

#[test]
fn compact_cinema_scale_preserves_saved_values_and_validates_steps() {
    use crate::home_evolution::CinemaScale;
    let mut scale = CinemaScale::default();
    for percent in (40u8..=100).step_by(10).rev() {
        assert_eq!(serde_json::to_value(scale).unwrap(), percent);
        assert_eq!(
            serde_json::from_value::<CinemaScale>(json!(percent)).unwrap(),
            scale
        );
        scale = scale.stepped(false);
    }
    assert_eq!(scale.percent(), 40);
    let mut state = preview();
    state.preferences.evolution_home.compact_cinema_scale = scale;
    assert_eq!(state.evolution_geometry().cinema_card_height, 65.6);
    assert_eq!(CinemaScale::default().stepped(true).percent(), 100);
    for invalid in [
        json!(null),
        json!(39),
        json!(45),
        json!(110),
        json!(330),
        json!("80"),
    ] {
        let prefs: Preferences = serde_json::from_value(json!({
            "evolution_home": { "cinema_dock_scale": 85, "compact_cinema_scale": invalid }
        }))
        .unwrap();
        assert_eq!(
            prefs.evolution_home.compact_cinema_scale,
            CinemaScale::default()
        );
        assert_eq!(prefs.evolution_home.cinema_dock_scale.label(), "85%");
    }
}

#[test]
fn dock_scale_validates_saved_percentages_without_resetting_other_preferences() {
    use crate::home_evolution::DockScale;
    let legacy: Preferences = serde_json::from_value(json!({
        "evolution_home": { "dock_size": "large" }
    }))
    .unwrap();
    assert_eq!(
        legacy.evolution_home.cinema_dock_scale,
        DockScale::default()
    );
    for invalid in [json!(null), json!(37), json!(101), json!(43), json!("85")] {
        let prefs: Preferences = serde_json::from_value(json!({
            "evolution_home": { "dock_size": "large", "cinema_dock_scale": invalid }
        }))
        .unwrap();
        assert_eq!(prefs.evolution_home.cinema_dock_scale, DockScale::default());
        assert_eq!(prefs.evolution_home.card(), legacy.evolution_home.card());
    }
    let mut scale = DockScale::default().stepped(false);
    for percent in (40..=100).step_by(5) {
        assert_eq!(serde_json::to_value(scale).unwrap(), percent);
        assert_eq!(
            serde_json::from_value::<DockScale>(json!(percent)).unwrap(),
            scale
        );
        scale = scale.stepped(true);
    }
    assert_eq!(scale.label(), "100%");
}

#[test]
fn dock_scale_changes_only_the_cinema_context_and_preserves_the_dock_window() {
    let mut state = preview();
    let dock = state.evolution_dock_items();
    let base = state.evolution_geometry();
    state.direction(Direction::Up, &[]);
    state.evolution_tick(1.);
    let expanded = state.evolution_geometry();
    let cinema_focus = state.focus.clone();
    assert!((expanded.dock[0].w - base.dock[0].w * 0.4).abs() < 0.001);
    state.preferences.evolution_home.cinema_dock_scale = serde_json::from_value(json!(85)).unwrap();
    let adjusted = state.evolution_geometry();
    assert_eq!(adjusted.dock_start, expanded.dock_start);
    assert_eq!(adjusted.dock.len(), expanded.dock.len());
    assert!((adjusted.dock[0].w - base.dock[0].w * 0.85).abs() < 0.001);
    assert!((adjusted.dock[0].h - base.dock[0].h * 0.85).abs() < 0.001);
    assert_eq!(state.evolution_dock_items(), dock);
    assert_eq!(state.focus, cinema_focus);
    assert!(state.back().is_empty());
    state.evolution_tick(1.);
    assert_eq!(state.focus, dock[0]);
    assert_eq!(state.evolution_geometry().dock[0].w, base.dock[0].w);
    state.wheel_scroll(&[], None, 0, 1);
    state.wheel_scroll(&[], None, 0, 1);
    state.evolution_tick(1.);
    assert!((state.evolution_geometry().dock[0].w - base.dock[0].w * 0.58).abs() < 0.001);
}

#[test]
fn dock_settings_stage_apply_cancel_and_confirm_restore() {
    use crate::home_evolution::{DockHorizontal, DockScale};
    use crate::state::{DOCK_ROW, Subpage};
    let mut state = preview();
    let saved = DockScale::default();
    let edited = saved.stepped(true);
    state.open(2);
    assert!(
        controls(&state)
            .iter()
            .any(|c| c.target == Target::Setting(DOCK_ROW))
    );
    state.focus = Target::Setting(DOCK_ROW);
    assert!(state.activate().is_empty());
    state.direction(Direction::Right, &[]);
    assert_eq!(
        state.subpage,
        Some(Subpage::Dock {
            scale: edited,
            horizontal: DockHorizontal::Sequential,
            confirming_restore: false
        })
    );
    assert_eq!(state.preferences.evolution_home.cinema_dock_scale, saved);
    state.focus = Target::DockApply;
    assert_eq!(
        state.activate(),
        vec![Effect::SaveDockSettings {
            scale: edited,
            horizontal: DockHorizontal::Sequential
        }]
    );
    assert_eq!(state.preferences.evolution_home.cinema_dock_scale, saved);
    state.dock_settings_saved(edited, DockHorizontal::Sequential);
    assert_eq!(state.focus, Target::Setting(DOCK_ROW));
    state.activate();
    state.focus = Target::DockRestore;
    assert!(state.activate().is_empty());
    assert_eq!(state.focus, Target::DockCancel);
    assert!(state.back().is_empty());
    assert_eq!(state.focus, Target::DockRestore);
    state.activate();
    state.focus = Target::DockConfirmRestore;
    assert!(state.activate().is_empty());
    assert_eq!(
        state.subpage,
        Some(Subpage::Dock {
            scale: saved,
            horizontal: DockHorizontal::Sequential,
            confirming_restore: false
        })
    );
    assert_eq!(state.preferences.evolution_home.cinema_dock_scale, edited);
    state.focus = Target::DockCancel;
    assert!(state.activate().is_empty());
    assert_eq!(state.focus, Target::Setting(DOCK_ROW));
    assert_eq!(state.preferences.evolution_home.cinema_dock_scale, edited);
}

#[test]
fn scroll_indicator_uses_the_live_dock_and_input_inventory_and_has_no_hit_target() {
    use crate::home_evolution::{DockSize, ScrollHints};
    let mut state = preview();
    let ids: Vec<_> = state
        .inventory
        .apps
        .iter()
        .take(15)
        .map(|app| app.id.clone())
        .collect();
    state.preferences.favorites = Some(ids.clone());
    state.open(0);
    let scroll = |state: &State| state.evolution_geometry().scroll;
    assert_eq!(
        scroll(&state),
        ScrollHints {
            up: false,
            down: true
        }
    );
    let (x, y) = ScrollHints::DOWN.center();
    assert!(state.hit_test(&controls(&state), x, y).is_none());

    for _ in 0..2 {
        state.direction(Direction::Down, &controls(&state));
    }
    assert_eq!(
        scroll(&state),
        ScrollHints {
            up: true,
            down: true
        }
    );
    state.direction(Direction::Down, &controls(&state));
    assert!(matches!(state.focus, Target::Input(_)));
    assert_eq!(
        scroll(&state),
        ScrollHints {
            up: true,
            down: false
        }
    );
    let (x, y) = ScrollHints::UP.center();
    assert!(state.hit_test(&controls(&state), x, y).is_none());
    state.set_inputs(vec![]);
    assert_eq!(
        scroll(&state),
        ScrollHints {
            up: true,
            down: false
        }
    );

    state.preferences.evolution_home.dock_size = DockSize::Compact;
    state.reconcile_focus();
    assert_eq!(scroll(&state), ScrollHints::default());
    state.preferences.favorites = Some(ids[..3].to_vec());
    state.reconcile_focus();
    assert_eq!(scroll(&state), ScrollHints::default());
}

#[test]
fn rotation_progress_and_switch_share_one_monotonic_cycle_and_pause_restarts_it() {
    use crate::home_evolution::Rotation;
    let mut rotation = Rotation::default();
    let now = std::time::Instant::now();
    assert!(!rotation.tick(now, true, 10, 0));
    for (seconds, expected) in [(0., 0.), (2.5, 0.25), (5., 0.5), (7.5, 0.75), (9.9, 0.99)] {
        assert!(!rotation.tick(
            now + std::time::Duration::from_secs_f32(seconds),
            true,
            10,
            0
        ));
        assert!((rotation.progress() - expected).abs() < 0.001);
    }
    assert!(!rotation.tick(now + std::time::Duration::from_secs(100), false, 10, 0));
    assert!((rotation.progress() - 0.99).abs() < 0.001);
    assert!(!rotation.tick(now + std::time::Duration::from_secs(200), true, 10, 0));
    assert_eq!(rotation.progress(), 0.);
    assert!(rotation.tick(now + std::time::Duration::from_secs(210), true, 10, 0));
    assert_eq!(rotation.progress(), 0.);
    assert_eq!(rotation.advances, 1);
    assert!(!rotation.tick(now + std::time::Duration::from_secs(211), true, 4, 1));
    assert_eq!(rotation.progress(), 0.);
    assert!(rotation.tick(now + std::time::Duration::from_secs(215), true, 4, 1));
    assert!(!rotation.tick(now + std::time::Duration::from_secs(216), true, 4, 5));
    assert_eq!(rotation.progress(), 0.);
}

#[test]
fn cinema_focus_pointer_disabled_rotation_and_small_catalogs_never_advance() {
    let mut state = preview();
    state.direction(Direction::Up, &controls(&state));
    state.evolution_tick(0.1);
    assert!(state.evolution.as_ref().unwrap().rotation.paused);
    state.pointer_focus(state.focus.clone());
    assert!(state.evolution.as_ref().unwrap().pointer_in_cinema);
    state.evolution_tick(0.1);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.advances, 0);
    for count in [0, 1] {
        state.evolution_focus_dock(state.evolution_dock());
        state.evolution.as_mut().unwrap().catalog_len = Some(count);
        state.evolution.as_mut().unwrap().pointer_in_cinema = false;
        state.evolution_tick(0.1);
        assert!(state.evolution.as_ref().unwrap().rotation.paused);
    }
    state.preferences.evolution_home.rotation_enabled = false;
    state.evolution_tick(0.1);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.advances, 0);
}

#[test]
fn automatic_rotation_visits_every_title_and_wraps_without_reordering_favorites() {
    let mut state = preview();
    let favorites = state.evolution_dock_items();
    state.evolution_tick(0.);
    for step in 1..=12 {
        state.evolution.as_mut().unwrap().rotation.started =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(10));
        state.evolution_tick(0.);
        let home = state.evolution.as_ref().unwrap();
        assert_eq!(home.selected, step);
        assert_eq!(
            crate::home_evolution::content(home.selected),
            step as usize % 8
        );
        assert_eq!(home.rotation.progress(), 0.);
        assert_eq!(state.evolution_dock_items(), favorites);
    }
    state.evolution.as_mut().unwrap().suspended = true;
    state.evolution_tick(0.);
    assert!(state.evolution.as_ref().unwrap().rotation.paused);
    state.open(0);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.progress(), 0.);
}

#[test]
fn input_processed_at_an_expired_deadline_wins_over_automatic_rotation() {
    let mut state = preview();
    state.evolution_tick(0.);
    let home = state.evolution.as_mut().unwrap();
    home.rotation.started = Some(std::time::Instant::now() - std::time::Duration::from_secs(10));
    home.defer_rotation = true;
    state.evolution_tick(0.);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.advances, 0);
    state.direction(Direction::Up, &controls(&state));
    state.evolution.as_mut().unwrap().defer_rotation = false;
    state.evolution_tick(0.);
    assert_eq!(state.evolution.as_ref().unwrap().rotation.advances, 0);
    assert!(state.evolution.as_ref().unwrap().rotation.paused);
}

#[test]
fn calendar_labels_cover_every_supported_language_and_rotation_preferences_fall_back() {
    for language in crate::i18n::Language::ALL {
        let (months, days, _) = crate::home_evolution::calendar_labels(language);
        assert_eq!(months.split('|').count(), 12);
        assert_eq!(days.split('|').count(), 7);
    }
    for invalid in [json!(null), json!(-4), json!("bad"), json!(0), json!(301)] {
        let prefs: Preferences = serde_json::from_value(
            json!({"favorites":["netflix"],"evolution_home":{"rotation_seconds":invalid}}),
        )
        .unwrap();
        assert_eq!(prefs.evolution_home.rotation_seconds, 10);
        assert_eq!(prefs.favorites, Some(vec!["netflix".into()]));
    }
}
