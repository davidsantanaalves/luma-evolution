use super::{Press, begin_ok};
use crate::home_evolution::{CONTENTS, Home, content};
use crate::magic_remote::{Settings, Timing};
use crate::model::{Preferences, demo_inventory};
use crate::state::{Direction, Effect, MAGIC_REMOTE_ROW, Overlay, State, Subpage, Target};
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

fn preview() -> State {
    let mut state = State::new(Preferences::default());
    state.evolution = Some(Home::preview());
    state.set_inventory(demo_inventory(), None);
    state.open(0);
    state.preferences.evolution_home.rotation_enabled = false;
    state
}

fn after(now: Instant, ms: u64) -> Instant {
    now + Duration::from_millis(ms)
}

fn card(index: usize) -> Target {
    Target::EvolutionContent(format!("{}:{index}", CONTENTS[index].0))
}

fn selected(state: &State) -> usize {
    content(state.evolution.as_ref().unwrap().selected)
}

#[test]
fn p05_defaults_and_legacy_settings_use_only_approved_steps() {
    let prefs = Preferences::default();
    assert_eq!(
        Settings::saved(&prefs.evolution_home),
        Settings {
            hold_ms: 1000,
            hover_ms: 250,
            leave_ms: 350,
            pointer_expands: true,
        }
    );
    for (timing, min, max, step) in [
        (Timing::Hold, 500, 2000, 100),
        (Timing::Hover, 100, 600, 50),
        (Timing::Leave, 100, 800, 50),
    ] {
        assert_eq!(timing.normalize(0), min);
        assert_eq!(timing.normalize(u64::MAX), max);
        let mut value = min;
        for expected in (min..=max).step_by(step) {
            assert_eq!(value, expected);
            value = timing.step(value, true);
        }
        assert_eq!(value, max);
        assert_eq!(timing.step(min, false), min);
    }
    let existing: Preferences = serde_json::from_value(serde_json::json!({
        "evolution_home": {"hover_ms": 420, "leave_ms": 999, "pointer_expands": false}
    }))
    .unwrap();
    let draft = Settings::saved(&existing.evolution_home);
    assert_eq!(
        (draft.hold_ms, draft.hover_ms, draft.leave_ms),
        (1000, 400, 800)
    );
    assert!(!draft.pointer_expands);
}

#[test]
fn p05_ok_short_and_long_are_exclusive_in_apps_and_dock() {
    for page in [0, 1] {
        for threshold in [500, 1000, 2000] {
            let mut state = preview();
            state.open(page);
            state.preferences.evolution_home.hold_ms = threshold;
            let app = state.apps()[0].id.clone();
            state.focus = Target::App(app.clone());
            let now = Instant::now();
            let short = Press::new(&state, Some(Keycode::Return), now);
            assert_eq!(
                short.release(&mut state, after(now, threshold - 1)),
                [Effect::Launch(app.clone())]
            );
            assert!(state.overlay.is_none());
            for poll_before_keyup in [true, false] {
                let mut long = Press::new(&state, Some(Keycode::Return), now);
                if poll_before_keyup {
                    long.hold_if_due(&mut state, after(now, threshold - 1));
                    assert!(!long.held);
                    long.hold_if_due(&mut state, after(now, threshold));
                    assert!(long.held);
                }
                assert!(long.release(&mut state, after(now, threshold)).is_empty());
                assert!(matches!(&state.overlay, Some(Overlay::Options { id, .. }) if id == &app));
                state.back();
            }
        }
    }
}

#[test]
fn p05_repeats_and_context_changes_cannot_retarget_ok() {
    let mut state = preview();
    let app = state.apps()[0].id.clone();
    state.focus = Target::App(app.clone());
    let now = Instant::now();
    let mut active = None;
    begin_ok(&mut active, &state, Keycode::Return, false, now);
    begin_ok(&mut active, &state, Keycode::Return, true, after(now, 500));
    begin_ok(
        &mut active,
        &state,
        Keycode::KpEnter,
        false,
        after(now, 500),
    );
    assert_eq!(active.as_ref().unwrap().start, now);
    assert_eq!(active.as_ref().unwrap().key, Some(Keycode::Return));
    state.open(1);
    state.focus = Target::App(app.clone());
    assert!(
        active
            .take()
            .unwrap()
            .release(&mut state, after(now, 1500))
            .is_empty()
    );
    assert!(state.overlay.is_none());
    let press = Press::new(&state, Some(Keycode::Return), now);
    state.inventory.apps.retain(|candidate| candidate.id != app);
    assert!(press.release(&mut state, after(now, 1500)).is_empty());
    assert!(state.overlay.is_none());
}

#[test]
fn p05_hover_changes_backdrop_after_the_delay_and_restarts_on_another_id() {
    let mut state = preview();
    let old = selected(&state);
    let a = (old + 1) % CONTENTS.len();
    let b = (old + 2) % CONTENTS.len();
    state.pointer_focus(card(a));
    let start = state.evolution.as_ref().unwrap().hover.as_ref().unwrap().1;
    assert_eq!(state.focus, card(a));
    state.evolution_tick_at(0., after(start, 249));
    assert_eq!(selected(&state), old);
    state.focus = card(b);
    state.evolution_pointer(Some(&card(b)), after(start, 249));
    state.evolution_tick_at(0., after(start, 498));
    assert_eq!(selected(&state), old);
    state.evolution_tick_at(0., after(start, 499));
    assert_eq!(selected(&state), b);
    assert!(state.evolution.as_ref().unwrap().expanded);
}

#[test]
fn p05_click_before_hover_uses_the_clicked_card_without_a_context_menu() {
    let mut state = preview();
    let clicked = (selected(&state) + 1) % CONTENTS.len();
    state.pointer_focus(card(clicked));
    let start = state.evolution.as_ref().unwrap().hover.as_ref().unwrap().1;
    assert_ne!(selected(&state), clicked);
    let press = Press::new(&state, None, start);
    assert!(press.release(&mut state, after(start, 1)).is_empty());
    assert_eq!(selected(&state), clicked);
    assert!(state.evolution.as_ref().unwrap().hover.is_none());
    let mut long = Press::new(&state, Some(Keycode::Return), start);
    long.hold_if_due(&mut state, after(start, 3000));
    assert!(!long.held);
    assert!(state.overlay.is_none());
}

#[test]
fn p05_hidden_pointer_and_direction_cancel_pending_hover() {
    let mut state = preview();
    let old = selected(&state);
    state.pointer_focus(card((old + 2) % CONTENTS.len()));
    let now = Instant::now();
    state.evolution_pointer_hidden(now);
    state.evolution_tick_at(0., after(now, 1000));
    assert_eq!(selected(&state), old);
    state.pointer_focus(card((old + 2) % CONTENTS.len()));
    state.direction(Direction::Right, &[]);
    let directional = selected(&state);
    assert_ne!(directional, old);
    assert!(!state.evolution_pointer_motion(0, 0));
    state.evolution_tick_at(0., after(now, 1000));
    assert_eq!(selected(&state), directional);
    assert!(!state.pointer);
    assert!(state.evolution.as_ref().unwrap().hover.is_none());
}

#[test]
fn p05_exit_tolerance_respects_return_and_directional_ownership() {
    let mut state = preview();
    let target = card(selected(&state));
    state.pointer_focus(target.clone());
    let start = state.evolution.as_ref().unwrap().hover.as_ref().unwrap().1;
    state.evolution_tick_at(0., after(start, 250));
    state.evolution_pointer(None, after(start, 250));
    state.evolution_tick_at(0., after(start, 599));
    assert!(state.evolution.as_ref().unwrap().expanded);
    state.evolution_pointer(Some(&target), after(start, 599));
    state.evolution_tick_at(0., after(start, 600));
    assert!(state.evolution.as_ref().unwrap().expanded);
    state.evolution_pointer(None, after(start, 600));
    state.evolution_tick_at(0., after(start, 950));
    assert!(!state.evolution.as_ref().unwrap().expanded);
    state.direction(Direction::Right, &[]);
    state.evolution_pointer_hidden(after(start, 950));
    state.evolution_tick_at(0., after(start, 1300));
    assert!(state.evolution.as_ref().unwrap().expanded);
    state.evolution.as_mut().unwrap().expanded = false;
    state.preferences.evolution_home.pointer_expands = false;
    state.pointer_focus(target);
    let restarted = state.evolution.as_ref().unwrap().hover.as_ref().unwrap().1;
    state.evolution_tick_at(0., after(restarted, 250));
    assert!(!state.evolution.as_ref().unwrap().expanded);
}

#[test]
fn p05_settings_draft_confirmation_apply_cancel_and_reopen() {
    let mut state = preview();
    state.open(2);
    let open = |state: &mut State| {
        state.focus = Target::Setting(MAGIC_REMOTE_ROW);
        assert!(state.activate().is_empty());
    };
    open(&mut state);
    state.direction(Direction::Right, &[]);
    assert_eq!(state.preferences.evolution_home.hold_ms, 1000);
    state.back();
    assert_eq!(state.focus, Target::RemoteContinue);
    state.activate();
    assert!(
        matches!(state.subpage, Some(Subpage::MagicRemote { confirming_exit: false, draft }) if draft.hold_ms == 1100)
    );
    state.focus = Target::RemoteCancel;
    state.activate();
    assert!(state.subpage.is_none());
    assert_eq!(state.preferences.evolution_home.hold_ms, 1000);
    open(&mut state);
    for kind in [Timing::Hold, Timing::Hover, Timing::Leave] {
        state.focus = Target::RemoteTiming(kind);
        state.direction(Direction::Right, &[]);
    }
    state.back();
    state.focus = Target::RemoteApply;
    let effects = state.activate();
    let [Effect::SaveMagicRemote(draft)] = effects.as_slice() else {
        panic!("Expected only the save")
    };
    let draft = *draft;
    assert_eq!(
        (draft.hold_ms, draft.hover_ms, draft.leave_ms),
        (1100, 300, 400)
    );
    let mut stored = state.preferences.clone();
    draft.apply(&mut stored.evolution_home);
    let restored: Preferences =
        serde_json::from_value(serde_json::to_value(stored).unwrap()).unwrap();
    state.remote_settings_saved(draft);
    open(&mut state);
    assert_eq!(Settings::saved(&restored.evolution_home), draft);
    assert!(
        matches!(state.subpage, Some(Subpage::MagicRemote { draft: current, confirming_exit: false }) if current == draft)
    );
}
