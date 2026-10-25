//! The map view: projecting the country onto the screen, drawing its water and
//! its track, and the locomotive that runs over it.
//!
//! The linework - track, shorelines, the graticule, the scale bar - is gizmos,
//! redrawn each frame for nothing. The water is meshes, built once: 2D gizmos
//! are sorted last whatever their depth, so a fill drawn with them would cover
//! every station marker and label standing on it.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite_render::ColorMaterial;
use yafa_core::{CityId, GeoPoint, Label, LabelSide, LineType, Network, Opening, coast};

use crate::components::*;
use crate::theme::{self, Fonts};

/// Pixels of map for one kilometre of ground.
///
/// Chosen so the whole network - 162 km from Akka down to Ghazzah - clears the
/// title above it and the station board below it. The water runs off the edges,
/// as it does on any printed sheet.
const PX_PER_KM: f32 = 3.6;
/// Where the centre of the network sits on screen, in pixels. Nudged left of
/// centre into the gap between the two columns of panels, and up to clear the
/// station board.
const MAP_OFFSET: Vec2 = Vec2::new(-15.0, 48.0);

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

/// How far west of the frame the sea polygon is closed off, in pixels. The
/// window is 1280 logical pixels wide with the camera at its centre, so this
/// is comfortably outside anything that can be seen.
const SEA_EDGE: f32 = -760.0;
/// How far past the top and bottom of the frame the sea is carried, in pixels.
const FRAME_OVERRUN: f32 = 600.0;
/// Depth of the water, below the station markers and their labels.
const WATER_Z: f32 = -10.0;

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
                Text2d::new(data.name.to_owned()),
                fonts.sign(theme::SIGN),
                TextColor(theme::MAP_INK),
                // Placement is per-station data: at this scale no rule from
                // the corridor alone keeps Al-Quds clear of Bayt Lahm.
                label_anchor(data.label),
                Transform::from_translation(label_offset(data.label).extend(6.0)),
                Pickable::IGNORE,
            ));
    }

    spawn_train(&mut commands);
}

/// Which corner of the name box is pinned to the marker.
fn label_anchor(label: Label) -> Anchor {
    Anchor(match label.side {
        LabelSide::Left => Vec2::new(0.5, 0.0),
        LabelSide::Right => Vec2::new(-0.5, 0.0),
        LabelSide::Above => Vec2::new(0.0, -0.5),
        LabelSide::Below => Vec2::new(0.0, 0.5),
    })
}

/// How far the name sits from the marker, plus the station's own nudge.
fn label_offset(label: Label) -> Vec2 {
    let side = match label.side {
        LabelSide::Left => Vec2::new(-13.0, 0.0),
        LabelSide::Right => Vec2::new(13.0, 0.0),
        LabelSide::Above => Vec2::new(0.0, 15.0),
        LabelSide::Below => Vec2::new(0.0, -15.0),
    };
    side + Vec2::new(label.nudge.0, label.nudge.1)
}

/// The names of the water, set in the open space they occupy.
fn water_names(commands: &mut Commands, view: &MapView, fonts: &Fonts) {
    let sea_ink = theme::SEA_LABEL;
    let mut name = |text: &str, at: Vec2, size: f32, turn: f32, anchor: Vec2| {
        commands.spawn((
            Text2d::new(text.to_owned()),
            fonts.sign(size),
            TextColor(sea_ink),
            Anchor(anchor),
            Transform::from_translation(at.extend(1.0)).with_rotation(Quat::from_rotation_z(turn)),
            Pickable::IGNORE,
        ));
    };

    // Set along the run of the coast, the way a sea name always is.
    name(
        "The Mediterranean",
        view.screen(GeoPoint::new(32.30, 34.26)),
        theme::HEADING,
        1.19,
        Vec2::ZERO,
    );
    // Clear of the town of the same name, which is now a station on the
    // lake's western shore.
    name(
        "Tabariyya",
        view.screen(GeoPoint::new(32.90, 35.62)),
        theme::CAPTION,
        0.0,
        Vec2::new(-0.5, 0.0),
    );
    name(
        "Bahr Lut",
        view.screen(GeoPoint::new(31.62, 35.63)),
        theme::CAPTION,
        0.0,
        Vec2::new(-0.5, 0.0),
    );
}

/// A small 0-4-0 tank engine, assembled from plain coloured rectangles.
///
/// Drawn in screen pixels rather than map kilometres, and a little larger than
/// scale: at 2.8 px to the kilometre a true-size train would be a speck.
fn spawn_train(commands: &mut Commands) {
    const IRON: Color = Color::srgb_u8(0x25, 0x1E, 0x1A);
    const IRON_LIGHT: Color = Color::srgb_u8(0x3A, 0x30, 0x29);
    /// The lamp, the boiler band and the running gear. Warm paintwork on the
    /// engine, not a state the player is meant to read.
    const LIVERY: Color = Color::srgb_u8(0xB8, 0x65, 0x1F);

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
            // Band and lamp.
            (
                Sprite::from_color(LIVERY, Vec2::new(2.4, 11.0)),
                Transform::from_xyz(6.2, 0.0, 0.2),
            ),
            (
                Sprite::from_color(LIVERY, Vec2::new(3.0, 3.6)),
                Transform::from_xyz(15.0, 0.6, 0.2),
            ),
            // Running gear.
            (
                Sprite::from_color(LIVERY, Vec2::new(23.0, 3.0)),
                Transform::from_xyz(0.0, -6.2, 0.2),
            ),
        ],
    ));
}

// --- The country ----------------------------------------------------------

/// Draws the graticule. The water itself is geometry, not gizmos - see
/// [`spawn_water`].
pub fn draw_country(mut gizmos: Gizmos, view: Res<MapView>) {
    graticule(&mut gizmos, &view);
}

/// Builds the sea, the lake and the Dead Sea as meshes, once.
///
/// These have to be real geometry rather than a gizmo fill: 2D gizmos are
/// sorted last whatever their depth, so anything drawn with them covers every
/// sprite and label beneath it - which, for a fill the size of the
/// Mediterranean, means every coastal station on the map.
///
/// Kept apart from [`spawn_map`] because it is the one piece of map setup that
/// needs the renderer's asset stores, and the headless tests have none.
pub fn spawn_water(
    mut commands: Commands,
    view: Res<MapView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let fill = materials.add(ColorMaterial::from_color(theme::SEA_FILL));

    let shore: Vec<Vec2> = coast::SHORE.iter().map(|&p| view.screen(p)).collect();
    let mut water = vec![sea_mesh(&shore)];
    for ring in [coast::TABARIYYA.as_slice(), coast::BAHR_LUT.as_slice()] {
        let points: Vec<Vec2> = ring.iter().map(|&p| view.screen(p)).collect();
        water.push(fan_mesh(&points));
    }

    for mesh in water {
        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(fill.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, WATER_Z)),
            Pickable::IGNORE,
        ));
    }
}

/// The sea, as one quad per stretch of shore running out to a western edge
/// well outside the frame.
///
/// Where the coast turns back on itself - it does so once, at Haifa Bay -
/// neighbouring quads overlap. They are the same flat colour, so the overlap
/// is invisible, and this stays far simpler than triangulating the region.
fn sea_mesh(shore: &[Vec2]) -> Mesh {
    let (Some(first), Some(last)) = (shore.first(), shore.last()) else {
        return empty_mesh();
    };

    // Carry the ends past the top and bottom of the view as well.
    let mut edge = Vec::with_capacity(shore.len() + 2);
    edge.push(Vec2::new(first.x, first.y + FRAME_OVERRUN));
    edge.extend_from_slice(shore);
    edge.push(Vec2::new(last.x, last.y - FRAME_OVERRUN));

    let mut positions = Vec::with_capacity(edge.len() * 2);
    for point in &edge {
        positions.push([point.x, point.y, 0.0]);
        positions.push([SEA_EDGE, point.y, 0.0]);
    }

    let mut indices = Vec::new();
    for strip in 0..edge.len().saturating_sub(1) {
        let base = (strip * 2) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 3]);
        indices.extend_from_slice(&[base, base + 3, base + 2]);
    }

    mesh_from(positions, indices)
}

/// A closed body of water, as a fan from its own centroid.
fn fan_mesh(ring: &[Vec2]) -> Mesh {
    if ring.len() < 3 {
        return empty_mesh();
    }
    let centre = ring.iter().copied().sum::<Vec2>() / ring.len() as f32;

    let mut positions = Vec::with_capacity(ring.len() + 1);
    positions.push([centre.x, centre.y, 0.0]);
    positions.extend(ring.iter().map(|p| [p.x, p.y, 0.0]));

    let mut indices = Vec::new();
    for corner in 0..ring.len() {
        let next = corner % ring.len() + 1;
        let wrapped = (corner + 1) % ring.len() + 1;
        indices.extend_from_slice(&[0, next as u32, wrapped as u32]);
    }

    mesh_from(positions, indices)
}

fn mesh_from(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

fn empty_mesh() -> Mesh {
    mesh_from(Vec::new(), Vec::new())
}

/// Draws the shorelines over the fills.
pub fn draw_shorelines(mut gizmos: Gizmos, view: Res<MapView>) {
    let shore: Vec<Vec2> = coast::SHORE.iter().map(|&p| view.screen(p)).collect();
    polyline(&mut gizmos, &shore, theme::SEA_INK);

    for ring in [coast::TABARIYYA.as_slice(), coast::BAHR_LUT.as_slice()] {
        let points: Vec<Vec2> = ring.iter().map(|&p| view.screen(p)).collect();
        closed_polyline(&mut gizmos, &points, theme::SEA_INK);
    }

    let jordan: Vec<Vec2> = coast::JORDAN.iter().map(|&p| view.screen(p)).collect();
    polyline(&mut gizmos, &jordan, theme::SEA_INK);
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
            gizmos.circle_2d(at, STATION_SIZE + 3.5, theme::VISITED.with_alpha(0.85));
        }
        if desk.origin == Some(city) {
            gizmos.circle_2d(at, STATION_SIZE + 7.0, theme::VISITED);
        }
        if desk.destination == Some(city) {
            gizmos.circle_2d(at, STATION_SIZE + 7.0, theme::PENDING);
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

    figure("0".to_owned(), origin + Vec2::new(0.0, -11.0), theme::CAPTION, Vec2::ZERO);
    figure(
        format!("{:.0} km", SCALE_KM),
        origin + Vec2::new(length, -11.0),
        theme::CAPTION,
        Vec2::ZERO,
    );
    figure("N".to_owned(), origin + Vec2::new(length * 0.5, 56.0), theme::CAPTION, Vec2::ZERO);
}
