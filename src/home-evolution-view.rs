//! Composes the five Home areas with the existing native scene primitives.
use super::*;
use crate::home_evolution::{
    CARD_FOCUS_GROWTH, CARD_SHADOW_BLUR, CARD_SHADOW_DROP, CONTENTS, DOCK_CAPTION_HEIGHT,
    DOCK_CAPTION_TOP, VISIBLE, WidgetApp, content,
};
include!("home-evolution-details.rs");

fn clipped(rect: Rect, viewport: Rect) -> Rect {
    let x = rect.x.max(viewport.x);
    let y = rect.y.max(viewport.y);
    let right = (rect.x + rect.w).min(viewport.x + viewport.w);
    let bottom = (rect.y + rect.h).min(viewport.y + viewport.h);
    Rect::new(x, y, (right - x).max(0.), (bottom - y).max(0.))
}

fn card(out: &mut Scene, state: &State, target: Target, rect: Rect) -> Rect {
    let focused = state.focus == target;
    // Fixed hit regions: drawing the focus lift never redistributes neighboring controls.
    out.control(target, rect, false);
    let shown = if focused {
        Rect::new(
            rect.x - rect.w * CARD_FOCUS_GROWTH,
            rect.y - rect.h * CARD_FOCUS_GROWTH,
            rect.w * (1. + 2. * CARD_FOCUS_GROWTH),
            rect.h * (1. + 2. * CARD_FOCUS_GROWTH),
        )
    } else {
        rect
    };
    if focused {
        out.shadow(shown, 18., CARD_SHADOW_BLUR, CARD_SHADOW_DROP, 0.35);
        out.ring(shown, 18.);
    }
    shown
}

fn single_line(
    out: &mut Scene,
    assets: &Assets,
    label: &str,
    rect: Rect,
    style: TextStyle,
    centered: bool,
) {
    out.text(
        label,
        rect,
        assets.fit_text(label, style, rect.w),
        WHITE,
        centered,
    );
}

fn right_line(out: &mut Scene, assets: &Assets, label: &str, area: Rect, style: TextStyle) -> Rect {
    let style = assets.fit_text(label, style, area.w);
    let width = assets.width(label, style) + 1.;
    let rect = Rect::new(area.x + area.w - width, area.y, width, area.h);
    out.text(label, rect, style, WHITE, false);
    rect
}

fn top_buttons(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    layout: &crate::home_evolution::HeaderGeometry,
) {
    let home = state.evolution.as_ref().unwrap();
    let focused = if state.pointer {
        home.pointer_target.as_ref()
    } else {
        Some(&state.focus)
    };
    for (i, (target, hit)) in state
        .evolution_top_targets()
        .iter()
        .zip(&layout.buttons)
        .enumerate()
    {
        let (icon, label) = match target {
            Target::EvolutionGlobal(action) => {
                (action.icon(), action.label(state.preferences.language))
            }
            Target::TodayCard => ("clock", state.preferences.language.t("today")),
            _ => unreachable!(),
        };
        let amount = home.button_focus[i].value;
        let shown = hit.inset(14. - 11. * amount);
        out.control(target.clone(), *hit, false);
        if amount > 0.
            && let Some(rgb) = state.preferences.focus_glow.rgb()
        {
            // Offset only the vector strokes. Their silhouette casts the colored light.
            let (taps, alpha) = crate::renderer::shadow_taps(72.);
            for [dx, dy] in taps {
                out.icon(
                    assets,
                    icon,
                    Rect::new(shown.x + dx * 1.7, shown.y + dy * 1.7, shown.w, shown.h),
                    c(rgb, alpha * amount),
                );
            }
        }
        out.shadowed_icon(assets, icon, shown, WHITE);
        if focused == Some(target) {
            let style = text(21., 650, 0.).shadow();
            let width = (assets.width(label, style) + 2.).clamp(hit.w, 720.);
            let rect = layout.action_label(*hit, width);
            let start = out.draw.len();
            single_line(out, assets, label, rect, style, true);
            out.fade(
                start,
                amount * ((home.top_label.value - 0.8) / 0.2).clamp(0., 1.),
                1.,
            );
        }
    }
}

fn widgets(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    now: OffsetDateTime,
    layout: &crate::home_evolution::HeaderGeometry,
    viewport: Rect,
) {
    use crate::home_evolution::WeatherFixture;
    let home = state.evolution.as_ref().unwrap();
    let prefs = &state.preferences;
    let language = prefs.language;
    let expanded = home.top_expansion.value;
    let cinema = home.expansion.value;
    let y = layout.clock.y;
    let left = layout.clock.x;
    if cinema < 0.5 {
        let start = out.draw.len();
        single_line(
            out,
            assets,
            &language.clock(now, prefs.hour_24, prefs.show_seconds),
            Rect::new(left, y, layout.clock.w, 64.),
            text(60., 500, -0.04).shadow(),
            false,
        );
        if prefs.show_date {
            single_line(
                out,
                assets,
                &prefs.date_format.format(language, now),
                Rect::new(left, y + 64., layout.clock.w, 30.),
                text(25., 500, 0.).shadow(),
                false,
            );
        }
        if prefs.show_greeting {
            single_line(
                out,
                assets,
                &language.greeting(now.hour(), &prefs.greeting_name),
                Rect::new(left, y + 94., layout.clock.w, 28.),
                text(21., 450, 0.).shadow(),
                false,
            );
        }
        out.fade(start, 1. - cinema * 2., 1.);
    }
    if cinema > 0.4 {
        let start = out.draw.len();
        single_line(
            out,
            assets,
            &crate::home_evolution::compact_header(prefs, now),
            Rect::new(left, y + 7., layout.clock.w, 42.),
            text(28., 550, -0.02).shadow(),
            false,
        );
        out.fade(start, ((cinema - 0.4) / 0.6).min(1.), 1.);
    }

    let right = layout.weather.x + layout.weather.w;
    if prefs.show_weather {
        let fixture = home.weather_fixture;
        let place = prefs.weather_location.as_ref();
        let reading = match fixture {
            WeatherFixture::Ready => Some(weather::Reading {
                temperature: 24.,
                code: 2,
                is_day: true,
            }),
            WeatherFixture::None => place.and_then(|p| state.weather.reading(p)),
            _ => None,
        };
        let condition = reading.map(weather::condition);
        let icon = condition.as_ref().map_or_else(
            || "weather-cloud".to_owned(),
            |c| format!("weather-{}", c.icon),
        );
        let label = reading.map_or_else(
            || {
                language
                    .t(
                        if fixture == WeatherFixture::Loading
                            || (fixture == WeatherFixture::None && state.weather.loading)
                        {
                            "loadingWeather"
                        } else {
                            "weatherUnavailable"
                        },
                    )
                    .to_owned()
            },
            |r| {
                format!(
                    "{} · {}",
                    weather::format_temperature(r.temperature, prefs.temperature_unit),
                    language.t(condition.as_ref().unwrap().key)
                )
            },
        );
        let line = right_line(
            out,
            assets,
            &label,
            Rect::new(layout.weather.x + 62., y + 8., layout.weather.w - 62., 46.),
            text(25. + 6. * expanded, 550, 0.).shadow(),
        );
        out.shadowed_icon(
            assets,
            &icon,
            Rect::new(line.x - 51., y + 4., 40., 40.),
            WHITE,
        );
        let location = if fixture != WeatherFixture::None {
            Some(format!(
                "São Paulo · {}",
                language.t("evolutionWeatherFixture")
            ))
        } else {
            place.map(|p| {
                if state.weather.failed && reading.is_some() {
                    format!("{} · {}", p.name, language.t("lastReading"))
                } else {
                    p.name.clone()
                }
            })
        };
        if let Some(location) = location {
            right_line(
                out,
                assets,
                &location,
                Rect::new(layout.weather.x, y + 49., layout.weather.w, 32.),
                text(20., 500, 0.).shadow(),
            );
        }
    }

    if expanded > 0. {
        let body_y = layout.calendar().y;
        // Reveal both halves with the same boundary while cinema and Dock make room.
        let start = out.draw.len();
        let body = Rect::new(
            left - 10.,
            body_y,
            right - left + 20.,
            (layout.bottom - body_y).max(0.),
        );
        out.draw.push(Draw::Clip(Some(clipped(body, viewport))));
        calendar(
            out,
            state,
            assets,
            now,
            Rect::new(left, body_y, layout.clock.w, 370.),
        );
        if prefs.show_weather {
            forecast(
                out,
                state,
                assets,
                now,
                Rect::new(layout.weather.x, body_y, layout.weather.w, 370.),
            );
        }
        out.draw.push(Draw::Clip(Some(viewport)));
        out.fade(start, (expanded * 1.6).min(1.), 1.);
    }

    // Match the weather column, keeping the compact clock outside the hit areas.
    for (app, rect) in [
        (WidgetApp::Calendar, layout.clock),
        (WidgetApp::Weather, layout.weather),
    ] {
        if (app == WidgetApp::Calendar && layout.calendar().h <= 0.)
            || !state.evolution_widget_eligible(app)
            || state.evolution_moving()
        {
            continue;
        }
        let target = Target::EvolutionWidget(app);
        if state.focus == target {
            out.border(rect.inset(-5.), 14., 2., c(0xffffff, 0.7));
            // Wait for the full inline reveal before drawing in its footer.
            if expanded == 1. {
                let footer = Rect::new(rect.x + 10., rect.y + rect.h - 19., rect.w - 20., 19.);
                let hint = language.t("evolutionWidgetOpenHint");
                let style = text(18., 500, 0.).shadow();
                match app {
                    WidgetApp::Calendar => single_line(out, assets, hint, footer, style, false),
                    WidgetApp::Weather => {
                        right_line(out, assets, hint, footer, style);
                    }
                }
            }
        }
        out.control(target, rect, false);
    }
}

fn cinema(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    rect: Rect,
    height: f32,
    viewport: Rect,
) {
    if clipped(rect.inset(-18.), viewport).h == 0. {
        return;
    }
    let home = state.evolution.as_ref().unwrap();
    let expansion = home.expansion.value;
    let (background, selected_title, detail) = CONTENTS[content(home.selected)];
    let language = state.preferences.language;
    let visible = VISIBLE.min(home.catalog_len.unwrap_or(CONTENTS.len()));

    // Keep the photo at its native ratio and reveal it through the expanding region.
    if expansion > 0. {
        let art_height = rect.w * 9. / 16.;
        let art = Rect::new(
            rect.x,
            rect.y + (rect.h - art_height) / 2.,
            rect.w,
            art_height,
        );
        out.draw.push(Draw::Clip(Some(clipped(rect, viewport))));
        let bottom = rect.y + rect.h;
        let fade = 64_f32.min(rect.h / 4.);
        out.draw.push(Draw::EdgeFade(Some([
            rect.y,
            rect.y + fade,
            bottom - fade,
            bottom,
        ])));
        let right = rect.x + rect.w;
        out.draw.push(Draw::EdgeFadeX(Some([
            rect.x,
            rect.x + 76.,
            right - 76.,
            right,
        ])));
        out.draw.push(Draw::TintedImage(
            assets.background_shaded(state, background, false),
            art,
            0.,
            false,
            [1., 1., 1., expansion * 0.92],
        ));
        out.draw.push(Draw::RoundedGradient(
            rect,
            0.,
            c(0x0a101a, 0.48 * expansion),
            c(0x0a101a, 0.83 * expansion),
        ));
        out.draw.push(Draw::EdgeFade(None));
        out.draw.push(Draw::EdgeFadeX(None));
        out.draw.push(Draw::Clip(Some(viewport)));
    }

    if height <= 0. {
        return;
    }
    let width = height * 16. / 9.;
    let gap = 27.;
    let strip_width = visible as f32 * (width + gap) - gap;
    let x = (1920. - strip_width) / 2.;
    let y = rect.y + rect.h - height - 7.;
    let label_style = text(17., 650, 0.04).shadow();
    let label = language.t("evolutionFiction").to_uppercase();
    let label_x = x + (rect.x + 24. - x) * expansion;
    out.text(
        label,
        Rect::new(label_x, rect.y + 2., 560., 24.),
        label_style,
        WHITE,
        false,
    );
    if y - rect.y > 150. {
        let start = out.draw.len();
        out.text(
            selected_title,
            Rect::new(rect.x + 24., rect.y + 44., rect.w - 48., 68.),
            text(54., 650, -0.035).shadow(),
            WHITE,
            false,
        );
        out.text(
            detail,
            Rect::new(rect.x + 26., rect.y + 117., rect.w - 52., 34.),
            text(24., 500, 0.).shadow(),
            WHITE,
            false,
        );
        out.fade(start, expansion, 1.);
    }

    let strip = Rect::new(x - 19., y - 18., strip_width + 38., height + 36.);
    out.draw.push(Draw::Clip(Some(clipped(strip, viewport))));
    let first = home.carousel.value.floor() as i64;
    for position in first - 1..first + visible as i64 + 2 {
        let n =
            content(position.rem_euclid(home.catalog_len.unwrap_or(CONTENTS.len()).max(1) as i64));
        let (art, title, _) = CONTENTS[n];
        let rect = Rect::new(
            x + (position as f32 - home.carousel.value) * (width + gap),
            y,
            width,
            height,
        );
        if rect.x + rect.w <= x || rect.x >= x + strip_width {
            continue;
        }
        let shown = card(
            out,
            state,
            Target::EvolutionContent(format!("{art}:{n}")),
            rect,
        );
        let hit = &mut out.controls.last_mut().unwrap().rect;
        let right = (hit.x + hit.w).min(x + strip_width);
        hit.x = hit.x.max(x);
        hit.w = right - hit.x;
        out.image(
            assets.background_shaded(state, art, false),
            shown,
            16.,
            false,
        );
        let shade = shown.h.min(42.);
        out.draw.push(Draw::RoundedGradient(
            Rect::new(shown.x, shown.y + shown.h - shade, shown.w, shade),
            10.,
            c(0x080b12, 0.24),
            c(0x080b12, 0.90),
        ));
        single_line(
            out,
            assets,
            title,
            Rect::new(
                shown.x + 10.,
                shown.y + shown.h - 28. - 6. * expansion,
                shown.w - 20.,
                25.,
            ),
            text(18., 550, 0.).shadow(),
            false,
        );
        if n == content(home.selected)
            && expansion > 0.
            && state.preferences.evolution_home.rotation_enabled
            && home.catalog_len.unwrap_or(CONTENTS.len()) > 1
        {
            let progress = home.rotation.progress();
            // Inside the artwork, below its caption. This is the rotation's own elapsed time.
            let track = Rect::new(shown.x + 12., shown.y + shown.h - 5., shown.w - 24., 2.);
            out.shape(track, 1., c(0xffffff, 0.15 * expansion));
            if progress > 0. {
                out.shape(
                    Rect::new(track.x, track.y, track.w * progress, track.h),
                    1.,
                    c(0xaee8ff, expansion),
                );
            }
        }
    }
    out.draw.push(Draw::Clip(Some(viewport)));
}

fn dock(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    layout: &crate::home_evolution::Geometry,
    items: &[Target],
) {
    let language = state.preferences.language;
    if items.is_empty() && state.error.is_some() {
        let rect = Rect::new(824., layout.dock_bounds.y + 22., 272., 54.);
        let target = Target::Retry;
        if state.focus == target {
            out.ring(rect, 18.);
        }
        out.glass(rect, 18.);
        out.control(target, rect, false);
        let label = language.t("retry");
        single_line(
            out,
            assets,
            label,
            Rect::new(rect.x + 10., rect.y + 12., rect.w - 20., 32.),
            text(22., 600, 0.),
            true,
        );
        return;
    }
    let pane = layout.dock_bounds;
    if clipped(pane, layout.viewport).h == 0. {
        return;
    }
    let contraction = state.evolution.as_ref().unwrap().dock_caption_transition();
    out.glass_tinted(pane, 30., c(0x0d1420, 0.58), 0.17);
    if let Some(rect) = layout.empty_hint
        && clipped(rect, layout.viewport).h > 0.
    {
        let label = language.format(
            "evolutionDockEmpty",
            &[("apps", language.t("evolutionAllApps"))],
        );
        out.text(
            &label,
            rect,
            assets.fit_text(&label, text(22., 500, 0.).shadow(), pane.w - 32.),
            c(0xffffff, 0.80),
            true,
        );
    }
    for (slot, rect) in layout.dock.iter().enumerate() {
        let target = &items[layout.dock_start + slot];
        let shown = card(out, state, target.clone(), *rect);
        // Small expanded logos keep a taller pointer target without changing visual scale.
        let hit = &mut out.controls.last_mut().unwrap().rect;
        let height = hit.h.max(64.);
        hit.y -= (height - hit.h) / 2.;
        hit.h = height;
        let (label, caption) = match target {
            Target::App(id) => {
                let Some(app) = state.inventory.apps.iter().find(|app| &app.id == id) else {
                    continue;
                };
                app_face(out, assets, app, shown, language, 18.);
                (
                    app.title.as_str(),
                    state.focus == *target || state.preferences.show_home_names,
                )
            }
            Target::EvolutionGlobal(action) => {
                out.draw.push(Draw::RoundedGradient(
                    shown,
                    18.,
                    c(0x323e55, 0.98),
                    c(0x141d31, 0.99),
                ));
                let (x, y) = shown.center();
                let size = shown.h.min(150.) * 0.38;
                out.icon(
                    assets,
                    action.icon(),
                    Rect::new(x - size / 2., y - size / 2., size, size),
                    WHITE,
                );
                (action.label(language), true)
            }
            _ => unreachable!(),
        };
        if caption && contraction < 0.5 {
            let start = out.draw.len();
            single_line(
                out,
                assets,
                label,
                Rect::new(
                    rect.x,
                    rect.y + rect.h + DOCK_CAPTION_TOP,
                    rect.w,
                    DOCK_CAPTION_HEIGHT,
                ),
                text(20., 550, 0.),
                true,
            );
            out.fade(start, 1. - 2. * contraction, 1.);
        }
    }
}

fn inputs(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    layout: &crate::home_evolution::Geometry,
    inputs: &[&Input],
) {
    let language = state.preferences.language;
    if layout.inputs.is_empty() {
        return;
    }
    out.text(
        language.t("inputs"),
        Rect::new(
            layout.inputs[0].x - 146.,
            layout.inputs[0].y + 14.,
            130.,
            28.,
        ),
        text(20., 600, 0.).shadow(),
        WHITE,
        false,
    );
    for (slot, &rect) in layout.inputs.iter().enumerate() {
        let input = inputs[layout.input_start + slot];
        let target = Target::Input(input.id.clone());
        out.control(target.clone(), rect, false);
        out.glass_tinted(rect, 16., c(0x121926, 0.62), 0.16);
        if state.focus == target {
            out.ring(rect, 16.);
        }
        out.icon(
            assets,
            &format!(
                "input-{}",
                input_icon(input, state.preferences.input_name(input))
            ),
            Rect::new(rect.x + 13., rect.y + 12., 30., 30.),
            WHITE,
        );
        let title = state.preferences.input_title(input);
        single_line(
            out,
            assets,
            title,
            Rect::new(rect.x + 53., rect.y + 4., rect.w - 65., 26.),
            text(20., 600, 0.),
            false,
        );
        single_line(
            out,
            assets,
            language.t("evolutionInputDemo"),
            Rect::new(rect.x + 53., rect.y + 30., rect.w - 65., 21.),
            text(14., 450, 0.),
            false,
        );
    }
}

pub fn render(out: &mut Scene, state: &State, assets: &Assets, now: OffsetDateTime) {
    let home = state.evolution.as_ref().unwrap();
    let prefs = &state.preferences.evolution_home;
    let items = state.evolution_dock_items();
    let inputs_list = state.evolution_inputs();
    let layout = state.evolution_geometry();
    let mut header = crate::home_evolution::header_geometry(state);
    let first_scroll_control = out.controls.len();
    out.draw.push(Draw::Clip(Some(layout.viewport)));

    if prefs.show_center && home.catalog_len.unwrap_or(CONTENTS.len()) > 0 {
        cinema(
            out,
            state,
            assets,
            layout.center,
            layout.cinema_card_height,
            layout.viewport,
        );
    }
    dock(out, state, assets, &layout, &items);
    inputs(out, state, assets, &layout, &inputs_list);
    header.clock.y -= layout.offset;
    header.weather.y -= layout.offset;
    header.bottom -= layout.offset;
    if prefs.show_top && header.bottom + 5. > layout.viewport.y {
        widgets(out, state, assets, now, &header, layout.viewport);
    }
    for control in &mut out.controls[first_scroll_control..] {
        control.rect = clipped(control.rect, layout.viewport);
    }
    let mut i = first_scroll_control;
    out.controls.retain(|control| {
        let keep = i > 0 || (control.rect.w > 0. && control.rect.h > 0.);
        i = i.saturating_sub(1);
        keep
    });
    // Header and mandatory actions are the final Home layer, outside cinema clipping/fades.
    out.draw.push(Draw::Clip(None));
    out.draw.push(Draw::EdgeFade(None));
    out.draw.push(Draw::EdgeFadeX(None));
    out.text(
        "LUMA EVOLUTION  /  PREVIEW NATIVO",
        Rect::new(80., 34., 840., 27.),
        text(17., 650, 0.06).shadow(),
        WHITE,
        false,
    );
    top_buttons(out, state, assets, &header);
    use crate::home_evolution::ScrollHints;
    for (show, icon, rect) in [
        (layout.scroll.up, "evolution-scroll-up", ScrollHints::UP),
        (layout.scroll.down, "down", ScrollHints::DOWN),
    ] {
        if show {
            out.shadowed_icon(assets, icon, rect, c(0xffffff, 0.64));
        }
    }

    if state.evolution_moving() {
        let label = state.preferences.language.t("evolutionMove");
        let rect = Rect::new(580., 33., 760., 40.);
        out.glass_tinted(rect, 16., c(0x101724, 0.8), 0.12);
        single_line(
            out,
            assets,
            label,
            Rect::new(rect.x + 10., rect.y + 7., rect.w - 20., 27.),
            text(20., 600, 0.),
            true,
        );
    }
}
