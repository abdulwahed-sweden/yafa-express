//! The map view: projecting the country onto the screen, drawing its water and
//! its track, and the locomotive that runs over it.
//!
//! Everything here is drawn with gizmos rather than meshes. It costs nothing to
//! redraw each frame, and it lets the map be *drawn* rather than filled: rails
//! read as two running lines with sleepers between them, and the sea as the
//! parallel water lines an engraver would have cut around a coast.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use yafa_core::{CityId, GeoPoint, LineType, Network, Opening, coast};

use crate::components::*;
use crate::theme::{self, Fonts};

/// Pixels of map for one kilometre of ground.
///
/// Chosen so the whole network - 162 km from Akka down to Ghazzah - clears the
/// title above it and the station board below it. The water runs off the edges,
/// as it does on any printed sheet.
const PX_PER_KM: f32 = 2.82;
/// Where the centre of the network sits on screen, in pixels. Nudged left of
/// centre into the gap between the two columns of panels, and up to clear the
/// station board.
const MAP_OFFSET: Vec2 = Vec2::new(-15.0, 81.5);

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

/// How many water lines are drawn out from the shore.
const WATER_LINES: usize = 7;
/// Pixels between them.
const WATER_SPACING: f32 = 12.0;
/// Seaward direction, perpendicular to the overall run of the coast.
///
/// The water lines are the whole shore translated along this, rather than each
/// segment pushed out along its own normal: a translation can never fold in on
/// itself, which per-segment offsets do the moment the coast turns concave -
/// as it does at Haifa Bay.
const SEAWARD: Vec2 = Vec2::new(-0.93, 0.37);

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

impl MapView {
    /// Projects a geographic point straight to the screen.
    fn screen(&self, point: GeoPoint) -> Vec2 {
        self.world(point.to_map())
    }
}

// --- Setup ----------------------------------------------------------------

/// Spawns the camera, the names printed on the map, the station markers and
/// the train.
pub fn spawn_map(mut commands: Commands, rail: Res<Rail>, view: Res<MapView>, fonts: Res<Fonts>) {
    commands.spawn((
        Camera2d,
        Camera { clear_color: ClearColorConfig::Custom(theme::DESK), ..default() },
        MapCamera,
    ));

    water_names(&mut commands, &view, &fonts);
    scale_bar_figures(&mut commands, &view, &fonts);

    for city in CityId::ALL {
        let data = rail.city(city);
        let at = view.world(rail.position(city));
        let junction = data.line_type == LineType::Junction;

        // Junctions are drawn as diamonds, through stations as squares - the
        // same convention a printed railway map would use.
        let rotation = if junction {
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
                        Sprite::from_color(theme::MAP_INK, Vec2::splat(size)),
                        Transform::from_rotation(rotation).with_translation(Vec3::Z * 0.2),
                        Pickable::IGNORE,
                    ),
                    (
                        Sprite::from_color(
                            theme::line_color(data.line_type),
                            Vec2::splat(size - 3.4)
                        ),
                        Transform::from_rotation(rotation).with_translation(Vec3::Z * 0.3),
                        Pickable::IGNORE,
                    ),
                ],
            ))
            .with_child((
                StationLabel(city),
                Text2d::new(data.name.to_uppercase()),
                fonts.sign(theme::SIGN),
                TextColor(theme::MAP_INK),
                theme::TRACKED,
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

/// The names of the water, set in the open space they occupy.
fn water_names(commands: &mut Commands, view: &MapView, fonts: &Fonts) {
    let sea_ink = theme::SEA_INK.with_alpha(0.95);
    let mut name = |text: &str, at: Vec2, size: f32, turn: f32, anchor: Vec2| {
        commands.spawn((
            Text2d::new(text.to_owned()),
            fonts.sign(size),
            TextColor(sea_ink),
            theme::TRACKED_WIDE,
            Anchor(anchor),
            Transform::from_translation(at.extend(1.0)).with_rotation(Quat::from_rotation_z(turn)),
            Pickable::IGNORE,
        ));
    };

    // Set along the run of the coast, the way a sea name always is.
    name("THE MEDITERRANEAN", view.screen(GeoPoint::new(32.30, 34.26)), 16.0, 1.19, Vec2::ZERO);
    name("TABARIYYA", view.screen(GeoPoint::new(32.79, 35.70)), 10.0, 0.0, Vec2::new(-0.5, 0.0));
    name("BAHR LUT", view.screen(GeoPoint::new(31.62, 35.63)), 10.0, 0.0, Vec2::new(-0.5, 0.0));
}

/// A small 0-4-0 tank engine, assembled from plain coloured rectangles.
///
/// Drawn in screen pixels rather than map kilometres, and a little larger than
/// scale: at 2.8 px to the kilometre a true-size train would be a speck.
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
            (Sprite::from_color(IRON, Vec2::new(26.0, 11.0)), Transform::from_xyz(1.2, 0.0, 0.0)),
            // Cab, at the back.
            (
                Sprite::from_color(IRON_LIGHT, Vec2::new(11.0, 16.0)),
                Transform::from_xyz(-9.4, 1.8, 0.1),
            ),
            // Chimney, at the front.
            (
                Sprite::from_color(IRON_LIGHT, Vec2::new(4.4, 8.5)),
                Transform::from_xyz(10.0, 6.2, 0.1),
            ),
            // Brass band and lamp.
            (
                Sprite::from_color(theme::BRASS, Vec2::new(2.4, 11.0)),
                Transform::from_xyz(6.2, 0.0, 0.2),
            ),
            (
                Sprite::from_color(theme::BRASS, Vec2::new(3.0, 3.6)),
                Transform::from_xyz(15.0, 0.6, 0.2),
            ),
            // Running gear.
            (
                Sprite::from_color(theme::STAMP_RED, Vec2::new(23.0, 3.0)),
                Transform::from_xyz(0.0, -6.2, 0.2),
            ),
        ],
    ));
}

// --- The country ----------------------------------------------------------

/// Draws the water and the graticule beneath everything else.
pub fn draw_country(mut gizmos: Gizmos, view: Res<MapView>) {
    graticule(&mut gizmos, &view);

    let shore: Vec<Vec2> = coast::SHORE.iter().map(|&p| view.screen(p)).collect();
    water_lines(&mut gizmos, &shore, WATER_LINES);
    polyline(&mut gizmos, &shore, theme::SEA_INK.with_alpha(0.62));

    for ring in [coast::TABARIYYA.as_slice(), coast::BAHR_LUT.as_slice()] {
        let points: Vec<Vec2> = ring.iter().map(|&p| view.screen(p)).collect();
        closed_water(&mut gizmos, &points);
    }

    let jordan: Vec<Vec2> = coast::JORDAN.iter().map(|&p| view.screen(p)).collect();
    polyline(&mut gizmos, &jordan, theme::SEA_INK.with_alpha(0.45));
}

/// The faint grid of whole and half degrees, which reads as the ruling of the
/// sheet rather than as information.
fn graticule(gizmos: &mut Gizmos, view: &MapView) {
    const LATS: [f64; 5] = [31.0, 31.5, 32.0, 32.5, 33.0];
    const LONS: [f64; 4] = [34.0, 34.5, 35.0, 35.5];

    for lat in LATS {
        let a = view.screen(GeoPoint::new(lat, 33.6));
        let b = view.screen(GeoPoint::new(lat, 36.2));
        gizmos.line_2d(a, b, theme::GRATICULE);
    }
    for lon in LONS {
        let a = view.screen(GeoPoint::new(30.8, lon));
        let b = view.screen(GeoPoint::new(33.4, lon));
        gizmos.line_2d(a, b, theme::GRATICULE);
    }
}

/// Parallel lines stepped out from the shore into the sea, fading as they go -
/// the engraver's way of saying "water" without filling anything in.
fn water_lines(gizmos: &mut Gizmos, shore: &[Vec2], count: usize) {
    for step in 1..=count {
        let offset = SEAWARD * (step as f32 * WATER_SPACING);
        let alpha = 0.20 * (1.0 - (step - 1) as f32 / count as f32).powf(1.4);
        let color = theme::SEA_INK.with_alpha(alpha);
        for pair in shore.windows(2) {
            gizmos.line_2d(pair[0] + offset, pair[1] + offset, color);
        }
    }
}

/// A closed body of water: its outline, and the horizontal ruling inside it
/// that a printed map uses instead of a flat fill.
fn closed_water(gizmos: &mut Gizmos, ring: &[Vec2]) {
    hatch_ring(gizmos, ring, 4.0, theme::SEA_INK.with_alpha(0.30));
    closed_polyline(gizmos, ring, theme::SEA_INK.with_alpha(0.68));
}

/// Even-odd scanline ruling of a closed ring.
fn hatch_ring(gizmos: &mut Gizmos, ring: &[Vec2], spacing: f32, color: Color) {
    if ring.len() < 3 || spacing <= 0.0 {
        return;
    }
    let (min_y, max_y) =
        ring.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.y), hi.max(p.y)));

    let mut crossings = Vec::with_capacity(8);
    let mut y = min_y + spacing * 0.5;
    while y < max_y {
        crossings.clear();
        for index in 0..ring.len() {
            let a = ring[index];
            let b = ring[(index + 1) % ring.len()];
            // Half-open test, so a vertex exactly on the scanline is counted
            // once rather than twice.
            if (a.y <= y) != (b.y <= y) {
                let t = (y - a.y) / (b.y - a.y);
                crossings.push(a.x + (b.x - a.x) * t);
            }
        }
        crossings.sort_by(f32::total_cmp);
        for &[left, right] in crossings.as_chunks::<2>().0 {
            gizmos.line_2d(Vec2::new(left, y), Vec2::new(right, y), color);
        }
        y += spacing;
    }
}

fn closed_polyline(gizmos: &mut Gizmos, ring: &[Vec2], color: Color) {
    polyline(gizmos, ring, color);
    if let (Some(first), Some(last)) = (ring.first(), ring.last()) {
        gizmos.line_2d(*last, *first, color);
    }
}

fn polyline(gizmos: &mut Gizmos, points: &[Vec2], color: Color) {
    for pair in points.windows(2) {
        gizmos.line_2d(pair[0], pair[1], color);
    }
}

// --- The network ----------------------------------------------------------

/// Draws the whole network: built track as rails and sleepers, surveyed but
/// unbuilt track as a broken line.
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

// --- Marks on the stations ------------------------------------------------

/// Rings around stations: brass for a stamp collected, a bright halo for the
/// station currently being read about, and the ticket's two endpoints.
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
            gizmos.circle_2d(at, radius, theme::MAP_INK.with_alpha(0.25 + pulse * 0.4));
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
        gizmos.circle_2d(centre, radius, theme::MAP_INK.with_alpha(alpha));
    }
}

// --- Furniture ------------------------------------------------------------

/// Where the scale bar and compass sit: open water off the coast, clear of
/// both the panels and the station board that lies over the map's south.
const FURNITURE_ANCHOR: GeoPoint = GeoPoint::new(31.95, 34.08);
/// Ground distance the bar measures, in kilometres.
const SCALE_KM: f32 = 50.0;

/// The scale bar and the north arrow.
pub fn draw_furniture(mut gizmos: Gizmos, view: Res<MapView>) {
    let origin = view.screen(FURNITURE_ANCHOR);
    let length = SCALE_KM * view.px_per_km;
    let ink = theme::MAP_INK.with_alpha(0.55);

    // A plain bar with a tick at each end and one in the middle.
    gizmos.line_2d(origin, origin + Vec2::X * length, ink);
    for fraction in [0.0, 0.5, 1.0] {
        let at = origin + Vec2::X * (length * fraction);
        gizmos.line_2d(at, at + Vec2::Y * 5.0, ink);
    }

    // North arrow, standing above the bar.
    let base = origin + Vec2::new(length * 0.5, 26.0);
    let tip = base + Vec2::Y * 17.0;
    gizmos.line_2d(base, tip, ink);
    gizmos.line_2d(tip, tip + Vec2::new(-4.5, -6.0), ink);
    gizmos.line_2d(tip, tip + Vec2::new(4.5, -6.0), ink);
}

/// The figures printed against the scale bar and the arrow. Static, because
/// they never change once the projection is fixed.
fn scale_bar_figures(commands: &mut Commands, view: &MapView, fonts: &Fonts) {
    let origin = view.screen(FURNITURE_ANCHOR);
    let length = SCALE_KM * view.px_per_km;
    let ink = theme::MAP_INK.with_alpha(0.55);

    let mut figure = |text: String, at: Vec2, size: f32, anchor: Vec2| {
        commands.spawn((
            Text2d::new(text),
            fonts.data(size),
            TextColor(ink),
            Anchor(anchor),
            Transform::from_translation(at.extend(1.0)),
            Pickable::IGNORE,
        ));
    };

    figure("0".to_owned(), origin + Vec2::new(0.0, -9.0), 9.0, Vec2::ZERO);
    figure(format!("{:.0} KM", SCALE_KM), origin + Vec2::new(length, -9.0), 9.0, Vec2::ZERO);
    figure("N".to_owned(), origin + Vec2::new(length * 0.5, 54.0), 10.0, Vec2::ZERO);
}
