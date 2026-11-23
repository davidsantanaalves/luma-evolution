/// The localized month grid lives in the left half of the expanded widget area.
fn calendar(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    now: OffsetDateTime,
    area: Rect,
) {
    use crate::home_evolution::calendar_labels;
    let language = state.preferences.language;
    let today = now.date();
    let (months, weekdays, sunday) = calendar_labels(language);
    let month = months
        .split('|')
        .nth(today.month() as usize - 1)
        .unwrap();
    let title = match language {
        Language::Zh | Language::ZhTw => format!("{}年{month}", today.year()),
        _ => format!("{month} {}", today.year()),
    };
    let x = area.x;
    let y = area.y;
    let style = text(28., 650, 0.).shadow();
    single_line(
        out,
        assets,
        &title,
        Rect::new(x, y + 2., 555., 38.),
        style,
        false,
    );
    let pitch = 104.;
    for (i, day) in weekdays.split('|').enumerate() {
        out.text(
            day,
            Rect::new(x + i as f32 * pitch, y + 53., 84., 28.),
            text(19., 550, 0.).shadow(),
            WHITE,
            true,
        );
    }
    let first = today.replace_day(1).unwrap();
    let offset = if sunday {
        first.weekday().number_days_from_sunday()
    } else {
        first.weekday().number_days_from_monday()
    };
    for day in 1..=31 {
        let Ok(date) = first.replace_day(day) else {
            break;
        };
        let index = u32::from(offset) + u32::from(day) - 1;
        let rect = Rect::new(
            x + (index % 7) as f32 * pitch,
            y + 86. + (index / 7) as f32 * 46.,
            84.,
            38.,
        );
        if date == today {
            out.shape(rect, 10., c(0x276990, 0.96));
        }
        out.text(
            day.to_string(),
            Rect::new(rect.x, rect.y + 6., rect.w, 29.),
            text(24., 550, 0.).shadow(),
            WHITE,
            true,
        );
    }
}

/// Forecast columns share the existing preview fixture and temperature preferences.
fn forecast(
    out: &mut Scene,
    state: &State,
    assets: &Assets,
    now: OffsetDateTime,
    area: Rect,
) {
    use crate::home_evolution::WeatherFixture;
    let language = state.preferences.language;
    let unit = state.preferences.temperature_unit;
    let fixture = state.evolution.as_ref().unwrap().weather_fixture;
    let right = area.x + area.w;
    let y = area.y;
    let heading = text(25., 650, 0.).shadow();
    if fixture != WeatherFixture::Ready {
        right_line(
            out,
            assets,
            language.t("evolutionForecastUnavailable"),
            Rect::new(area.x, y + 4., area.w, 40.),
            text(22., 500, 0.).shadow(),
        );
        return;
    }
    for (title, hourly, count, top, pitch) in [
        ("evolutionHourly", true, 6_u8, y + 4., 116.),
        ("evolutionDaily", false, 3_u8, y + 164., 246.),
    ] {
        right_line(
            out,
            assets,
            language.t(title),
            Rect::new(area.x, top, area.w, 38.),
            heading,
        );
        for index in 0..count {
            let edge = right - f32::from(count - 1 - index) * pitch;
            let rect = Rect::new(edge - pitch + 13., top + 55., pitch - 13., 78.);
            let (when, temperature) = if hourly {
                let at = now + time::Duration::hours(i64::from(index) + 1);
                (
                    language.clock(at, state.preferences.hour_24, false),
                    weather::format_temperature(24. + f64::from(index % 3), unit),
                )
            } else {
                let at = now + time::Duration::days(i64::from(index) + 1);
                (
                    language.date(at),
                    format!(
                        "{} / {}",
                        weather::format_temperature(19. + f64::from(index), unit),
                        weather::format_temperature(26. + f64::from(index), unit)
                    ),
                )
            };
            right_line(
                out,
                assets,
                &when,
                Rect::new(rect.x + 8., rect.y + 3., rect.w - 12., 28.),
                text(19., 500, 0.).shadow(),
            );
            right_line(
                out,
                assets,
                &temperature,
                Rect::new(rect.x + 8., rect.y + 36., rect.w - 12., 36.),
                text(27., 650, 0.).shadow(),
            );
        }
    }
}
