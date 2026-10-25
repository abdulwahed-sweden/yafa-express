//! The map view: projecting the network onto the screen, drawing the track,
//! and the locomotive that runs over it.
//!
//! Track is drawn with gizmos rather than meshes. It costs nothing to redraw
//! every frame, and it lets the rails read as rails - two running lines with
//! sleepers between them - instead of as a flat stroke.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use yafa_core::{CityId, LineType, Network, Opening};

use crate::components::*;
use crate::theme;

/// Pixels of map for one kilometre of ground.
///
/// Chosen so the whole network - 162 km from Akka down to Ghazzah - clears the
/// title above it and the station board below it without being cropped.
const PX_PER_KM: f32 = 3.0;
/// Where the centre of the network sits on screen, in pixels. Nudged left of
/// centre to sit in the gap between the two columns of panels, and up to clear
/// the station board.
const MAP_OFFSET: Vec2 = Vec2::new(-15.0, 54.0);

/// Half the gauge, in pixels: how far each running rail sits from the centre
/// line of the track.
const RAIL_GAUGE: f32 = 1.9;
/// Pixels between sleepers.
const SLEEPER_SPACING: f32 = 10.0;
/// Half-length of a sleeper, in pixels.
const SLEEPER_REACH: f32 = 3.4;

/// Radius of a station marker, in pixels.
const STATION_SIZE: f32 = 8.0;
/// Size of the invisible pad that catches clicks on a station.
const STATION_HIT: f32 = 26.0;

/// Builds the projection from the network's own extent, so the map stays
/// centred if the city table ever changes.
pub fn make_map_view(network: &Network) -> MapView {
    let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for city in CityId::ALL {
        let p = network.position(city);
        min = min.min(Vec2::new(p.x_km, p.y_km));
        max = max.max(Vec2::new(p.x_km, p.y_km));
    }
    MapView { px_per_km: PX_PER_KM, centre: (min + max) * 0.5, offset: MAP_OFFSET }
}

/// Spawns the camera, the station markers and their labels, and the train.
pub fn spawn_map(mut commands: Commands, rail: Res<Rail>, view: Res<MapView>) {
    commands.spawn((
        Camera2d,
        Camera { clear_color: ClearColorConfig::Custom(theme::DESK), ..default() },
        MapCamera,
    ));

    for city in CityId::ALL {
        let data = rail.city(city);
        let at = view.world(rail.position(city));
        let junction = data.line_type == LineType::Junction;

        // Junctions are drawn as diamonds, through stations as squares - the
        // same convention a printed railway map would use.
        let marker_rotation = if junction {
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)
        } else {
            Quat::IDENTITY
        };
        let size = if junction { STATION_SIZE * 1.15 } else { STATION_SIZE };

        commands
            .spawn((
                StationDot(city),
                // A transparent pad, larger than the marker, so stations are
                // comfortable to click. The marker itself is a child.
                Sprite::from_color(Color::NONE, Vec2::splat(STATION_HIT)),
                Transform::from_translation(at.extend(5.0)),
                Pickable::default(),
                Name::new(data.name),
                children![
                    (
                        Sprite::from_color(theme::PAPER, Vec2::splat(size)),
                        Transform::from_rotation(marker_rotation).with_translation(Vec3::Z * 0.2),
                        Pickable::IGNORE,
                    ),
                    (
                        Sprite::from_color(
                            theme::line_color(data.line_type),
                            Vec2::splat(size - 3.4)
                        ),
                        Transform::from_rotation(marker_rotation).with_translation(Vec3::Z * 0.3),
                        Pickable::IGNORE,
                    ),
                ],
            ))
            .with_child((
                StationLabel(city),
                Text2d::new(data.name),
                theme::text_font(11.0),
                TextColor(theme::PAPER),
                // Coastal names are set out over the sea, inland names over the
                // hills, so neither crowds the track between them.
                label_anchor(data.line_type),
                Transform::from_translation(label_offset(data.line_type).extend(6.0)),
                Pickable::IGNORE,
            ));
    }

    spawn_train(&mut commands);
}

fn label_anchor(line: LineType) -> Anchor {
    match line {
        LineType::Coastal => Anchor(Vec2::new(0.5, 0.0)),
        _ => Anchor(Vec2::new(-0.5, 0.0)),
    }
}

fn label_offset(line: LineType) -> Vec2 {
    match line {
        LineType::Coastal => Vec2::new(-11.0, 0.0),
        _ => Vec2::new(11.0, 0.0),
    }
}

/// A small 0-4-0 tank engine, assembled from plain coloured rectangles.
fn spawn_train(commands: &mut Commands) {
    const IRON: Color = Color::srgb_u8(0x25, 0x1E, 0x1A);
    const IRON_LIGHT: Color = Color::srgb_u8(0x3A, 0x30, 0x29);

    commands.spawn((
        Train,
        Transform::from_translation(Vec3::new(0.0, 0.0, 20.0)),
        Visibility::Hidden,
        Name::new("Locomotive"),
        children![
            // Boiler.
            (Sprite::from_color(IRON, Vec2::new(21.0, 9.0)), Transform::from_xyz(1.0, 0.0, 0.0),),
            // Cab, at the back.
            (
                Sprite::from_color(IRON_LIGHT, Vec2::new(9.0, 13.0)),
                Transform::from_xyz(-7.5, 1.5, 0.1),
            ),
            // Chimney, at the front.
            (
                Sprite::from_color(IRON_LIGHT, Vec2::new(3.5, 7.0)),
                Transform::from_xyz(8.0, 5.0, 0.1),
            ),
            // Brass band and lamp.
            (
                Sprite::from_color(theme::BRASS, Vec2::new(2.0, 9.0)),
                Transform::from_xyz(5.0, 0.0, 0.2),
            ),
            (
                Sprite::from_color(theme::BRASS, Vec2::new(2.5, 3.0)),
                Transform::from_xyz(12.0, 0.5, 0.2),
            ),
            // Running gear.
            (
                Sprite::from_color(theme::STAMP_RED, Vec2::new(19.0, 2.5)),
                Transform::from_xyz(0.0, -5.0, 0.2),
            ),
        ],
    ));
}

/// Draws the whole network: built track as rails and sleepers, surveyed but
/// unbuilt track as a dashed line.
pub fn draw_network(
    mut gizmos: Gizmos,
    rail: Res<Rail>,
    view: Res<MapView>,
    desk: Res<Desk>,
    runs: Query<&TrainRun>,
) {
    // Segments the train is working, or that the pending ticket would buy.
    let highlighted = active_segments(&rail, &desk, runs.iter().next());

    for segment in rail.segments() {
        let a = view.world(rail.position(segment.from));
        let b = view.world(rail.position(segment.to));
        let lit = highlighted.contains(&segment.id.0);
        let color =
            if lit { theme::line_color(segment.line) } else { theme::line_color_dim(segment.line) };

        match segment.opening {
            Opening::Opened(_) => draw_track(&mut gizmos, a, b, color, lit),
            Opening::Projected => draw_dashed(&mut gizmos, a, b, color),
        }
    }
}

/// Which segments to pick out: the route being run, else the route on offer.
fn active_segments(rail: &Rail, desk: &Desk, run: Option<&TrainRun>) -> Vec<usize> {
    if let Some(run) = run {
        return run.route().legs.iter().map(|leg| leg.segment.0).collect();
    }
    desk.issue(rail)
        .map(|route| route.legs.iter().map(|leg| leg.segment.0).collect())
        .unwrap_or_default()
}

/// Two running rails with sleepers between them.
fn draw_track(gizmos: &mut Gizmos, a: Vec2, b: Vec2, color: Color, lit: bool) {
    let along = b - a;
    let length = along.length();
    if length < f32::EPSILON {
        return;
    }
    let dir = along / length;
    let side = Vec2::new(-dir.y, dir.x);

    // The ballast: a wash under the rails, so lit track reads as a solid line.
    if lit {
        gizmos.line_2d(a, b, color.with_alpha(0.5));
    }

    let offset = side * RAIL_GAUGE;
    gizmos.line_2d(a + offset, b + offset, color);
    gizmos.line_2d(a - offset, b - offset, color);

    let sleeper_color = color.with_alpha(if lit { 0.75 } else { 0.35 });
    let count = (length / SLEEPER_SPACING) as usize;
    for i in 1..count {
        let at = a + dir * (i as f32 * SLEEPER_SPACING);
        gizmos.line_2d(at + side * SLEEPER_REACH, at - side * SLEEPER_REACH, sleeper_color);
    }
}

/// A broken line, for track that was surveyed but never laid.
fn draw_dashed(gizmos: &mut Gizmos, a: Vec2, b: Vec2, color: Color) {
    const DASH: f32 = 7.0;
    const GAP: f32 = 6.0;

    let along = b - a;
    let length = along.length();
    if length < f32::EPSILON {
        return;
    }
    let dir = along / length;

    let mut travelled = 0.0;
    while travelled < length {
        let end = (travelled + DASH).min(length);
        gizmos.line_2d(a + dir * travelled, a + dir * end, color);
        travelled = end + GAP;
    }
}

/// Rings around stations: brass for a stamp collected, a bright halo for the
/// station currently being read about, and a pulse for the ticket's endpoints.
pub fn draw_station_rings(
    mut gizmos: Gizmos,
    rail: Res<Rail>,
    view: Res<MapView>,
    book: Res<PassportBook>,
    focus: Res<Focus>,
    desk: Res<Desk>,
    time: Res<Time>,
) {
    let pulse = (time.elapsed_secs() * 2.6).sin() * 0.5 + 0.5;

    for city in CityId::ALL {
        let at = view.world(rail.position(city));

        if book.passport.is_stamped(city) {
            gizmos.circle_2d(at, STATION_SIZE + 3.5, theme::BRASS.with_alpha(0.85));
        }
        if desk.origin == Some(city) {
            gizmos.circle_2d(at, STATION_SIZE + 7.0, theme::OLIVE);
        }
        if desk.destination == Some(city) {
            gizmos.circle_2d(at, STATION_SIZE + 7.0, theme::STAMP_RED);
        }
        if focus.0 == Some(city) {
            let radius = STATION_SIZE + 10.0 + pulse * 3.0;
            gizmos.circle_2d(at, radius, theme::PAPER.with_alpha(0.25 + pulse * 0.4));
        }
    }
}

/// A trail of steam behind a moving locomotive.
pub fn draw_steam(
    mut gizmos: Gizmos,
    view: Res<MapView>,
    rail: Res<Rail>,
    runs: Query<&TrainRun>,
    time: Res<Time>,
) {
    let Ok(run) = runs.single() else { return };
    if run.standing_at().is_some() {
        return;
    }

    let at = view.world(run.position(&rail));
    let (hx, hy) = run.heading(&rail);
    let back = -Vec2::new(hx, hy);
    let drift = time.elapsed_secs();

    // Puffs fall back along the track and lift away from it, each one older,
    // wider and fainter than the one in front.
    for puff in 0..3 {
        let age = puff as f32 + (drift * 1.7).fract();
        let lift = Vec2::new(-back.y, back.x) * (age * 2.4);
        let centre = at + back * (11.0 + age * 7.5) + lift + Vec2::Y * (4.0 + age * 2.0);
        let radius = 2.0 + age * 2.6;
        let alpha = (0.30 - age * 0.085).max(0.0);
        gizmos.circle_2d(centre, radius, theme::PAPER.with_alpha(alpha));
    }
}
