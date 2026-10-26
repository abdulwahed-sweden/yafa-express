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
use yafa_core::{CityId, GeoPoint, Label, LabelSide, LineType, Network, Phase, coast};

use crate::components::*;
use crate::strings::{Language, Phrase, fill};
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
/// Spawns the geographic station markers, labels and furniture.
///
/// Kept, but no longer in the draw path: the map is the schematic diagram
/// now. This is what the geographic view was, if it is ever wanted back.
pub fn spawn_geographic_markers(
    mut commands: Commands,
    rail: Res<Rail>,
    view: Res<MapView>,
    fonts: Res<Fonts>,
) {
    water_names(&mut commands, &view, &fonts);
    scale_bar_figures(&mut commands, &view, &fonts);

    for city in CityId::ALL {
        let data = rail.city(city);
        let at = view.world(rail.position(city));
        let junction = data.line_type == LineType::Junction;
        let rotation = if junction {
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)
        } else {
            Quat::IDENTITY
        };
        let size = if junction { STATION_SIZE * 1.15 } else { STATION_SIZE };

        commands
            .spawn((
                StationDot(city),
                Sprite::from_color(Color::NONE, Vec2::splat(STATION_HIT)),
                Transform::from_translation(at.extend(5.0)),
                Pickable::default(),
                Name::new(data.name),
                children![
                    (
                        StationFace { city, inner: false },
                        Sprite::from_color(theme::MAP_INK, Vec2::splat(size)),
                        Transform::from_rotation(rotation).with_translation(Vec3::Z * 0.2),
                        Pickable::IGNORE,
                    ),
                    (
                        StationFace { city, inner: true },
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
                label_anchor(data.label),
                Transform::from_translation(label_offset(data.label).extend(6.0)),
                Pickable::IGNORE,
            ));
    }
}

pub fn spawn_map(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera { clear_color: ClearColorConfig::Custom(theme::DESK), ..default() },
        MapCamera,
        // Required, not decorative: `UiPickingSettings { require_markers: true }`
        // considers only cameras carrying this. Without it the UI picking
        // backend silently ignores every button in the game, and the whole
        // interface goes dead to the mouse while the map stays live.
        UiPickingCamera,
    ));
}

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

/// A small 0-4-0 tank engine, drawn as polygons.
///
/// Rectangles could never give it a smokebox front, a sloped cab roof or a
/// cowcatcher, so the whole engine is a handful of convex polygons in one
/// mesh apiece. It is drawn half as large again as the sprites were: at 3.6
/// pixels to the kilometre a true-size train is a speck, and this one has to
/// read as a locomotive.
const ENGINE_SCALE: f32 = 1.5;

/// Paintwork, warm against the iron. Not a state the player should read.
const LIVERY: Color = Color::srgb_u8(0xB8, 0x65, 0x1F);
const IRON: Color = Color::srgb_u8(0x25, 0x1E, 0x1A);
const IRON_LIGHT: Color = Color::srgb_u8(0x3A, 0x30, 0x29);

/// One piece of the engine: an outline in engine-local pixels, and its colour.
type Part = (&'static [[f32; 2]], Color);

/// The engine, nose to the right, built from the frames up.
const ENGINE: [Part; 7] = [
    // Footplate and frames.
    (&[[-11.0, -4.5], [13.0, -4.5], [13.0, -2.0], [-11.0, -2.0]], IRON),
    // Boiler, tapering to the smokebox.
    (&[[-5.0, -2.0], [11.0, -2.0], [12.5, 0.5], [11.0, 3.5], [-5.0, 3.5]], IRON_LIGHT),
    // Smokebox front, a touch proud of the boiler.
    (&[[11.0, -2.0], [14.0, -0.5], [14.0, 1.5], [11.5, 3.5]], IRON),
    // Cab, with the roof sloped back.
    (&[[-11.0, -2.0], [-4.0, -2.0], [-4.0, 7.0], [-9.5, 8.5], [-11.0, 7.0]], IRON_LIGHT),
    // Chimney.
    (&[[7.0, 3.5], [10.5, 3.5], [10.0, 8.5], [7.5, 8.5]], IRON),
    // Boiler band.
    (&[[2.0, -2.0], [3.5, -2.0], [3.5, 3.5], [2.0, 3.5]], LIVERY),
    // Cowcatcher.
    (&[[13.0, -4.5], [17.0, -5.5], [17.0, -4.0], [13.0, -2.0]], LIVERY),
];

pub fn spawn_train(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (meshes, materials) = (&mut *meshes, &mut *materials);
    let parts: Vec<_> = ENGINE
        .iter()
        .enumerate()
        .map(|(layer, (outline, color))| {
            let points: Vec<Vec2> =
                outline.iter().map(|&[x, y]| Vec2::new(x, y) * ENGINE_SCALE).collect();
            (
                Mesh2d(meshes.add(fan_mesh(&points))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(*color))),
                Transform::from_translation(Vec3::Z * (layer as f32 * 0.01)),
                Pickable::IGNORE,
            )
        })
        .collect();

    commands
        .spawn((
            Train,
            Transform::from_translation(Vec3::new(0.0, 0.0, 20.0)),
            Visibility::Hidden,
            Name::new("Locomotive"),
        ))
        .with_children(|engine| {
            for part in parts {
                engine.spawn(part);
            }
        });
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
            SeaFill,
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
    almanac: Res<Almanac>,
    runs: Query<&TrainRun>,
) {
    // Segments the train is working, or that the pending ticket would buy.
    let highlighted = active_segments(&rail, &desk, runs.iter().next(), almanac.year());

    for segment in rail.segments() {
        let a = view.world(rail.position(segment.from));
        let b = view.world(rail.position(segment.to));
        let lit = highlighted.contains(&segment.id.0);
        let open = segment.opening.is_open(almanac.year());
        let color = match (lit, open) {
            (true, _) => theme::line_color(segment.line),
            (false, true) => theme::line_color_dim(segment.line),
            // Track that exists on paper, or has not been laid yet this year.
            (false, false) => theme::line_color_dim(segment.line).mix(&theme::DESK, 0.45),
        };

        if open {
            draw_track(&mut gizmos, a, b, color, lit);
        } else {
            draw_dashed(&mut gizmos, a, b, color);
        }
    }
}

/// Which segments to pick out: the route being run, else the route on offer.
pub(crate) fn active_segments(
    rail: &Rail,
    desk: &Desk,
    run: Option<&TrainRun>,
    year: u16,
) -> Vec<usize> {
    if let Some(run) = run {
        return run.route().legs.iter().map(|leg| leg.segment.0).collect();
    }
    desk.issue(rail, year)
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

/// Washes out stations the network has not reached yet.
///
/// A station with no open track is still drawn - it is a real place, and its
/// history can still be read - but it is plainly not somewhere a train can be
/// sent this year.
pub fn refresh_station_state(
    rail: Res<Rail>,
    almanac: Res<Almanac>,
    lang: Res<Language>,
    mut faces: Query<(&StationFace, &mut Sprite)>,
    mut labels: Query<(&StationLabel, &mut TextColor, &mut Text2d)>,
) {
    for (face, mut sprite) in &mut faces {
        let open = rail.station_is_open(face.city, almanac.year());
        sprite.color = match (open, face.inner) {
            (true, true) => theme::line_color(rail.city(face.city).line_type),
            (true, false) => theme::MAP_INK,
            (false, true) => theme::INK_FAINT,
            (false, false) => theme::BORDER,
        };
    }
    for (StationLabel(city), mut color, mut text) in &mut labels {
        color.0 = if rail.station_is_open(*city, almanac.year()) {
            theme::MAP_INK
        } else {
            theme::INK_FAINT
        };
        // The names on the map read in whichever script the rest of the
        // interface does.
        let data = rail.city(*city);
        let name = match *lang {
            Language::English => data.name,
            Language::Arabic => data.arabic,
        };
        if text.0 != name {
            text.0 = name.to_owned();
        }
    }
}

/// A trail of steam, laid along the track the engine has just left.
///
/// The puffs are placed in the engine's own frame rather than the world's:
/// each one is pushed back along the heading from the chimney and lifted away
/// from the boiler, so the trail swings with the train instead of always
/// drifting the same way up the screen. They grow and fade with age, and the
/// whole trail scrolls through them so it reads as moving air.
pub fn draw_steam(
    mut gizmos: Gizmos,
    view: Res<MapView>,
    rail: Res<Rail>,
    runs: Query<&TrainRun>,
    time: Res<Time>,
) {
    const PUFFS: usize = 5;
    /// Where the chimney stands, in engine-local pixels.
    const CHIMNEY: Vec2 = Vec2::new(8.8 * ENGINE_SCALE, 8.5 * ENGINE_SCALE);

    let Ok(run) = runs.single() else { return };
    if run.standing_at().is_some() {
        return;
    }

    let at = view.world(run.position(&rail));
    let (hx, hy) = run.heading(&rail);
    let forward = Vec2::new(hx, hy);
    // The engine's own up, so the plume lifts off the boiler whichever way
    // the train is pointing.
    let up = Vec2::new(-forward.y, forward.x);
    let chimney = at + forward * CHIMNEY.x + up * CHIMNEY.y;

    let scroll = (time.elapsed_secs() * 1.6).fract();
    for puff in 0..PUFFS {
        let age = puff as f32 + scroll;
        // Back along the track, and rising as it is left behind.
        let centre = chimney - forward * (6.0 + age * 11.0) + up * (3.0 + age * 4.5);
        let radius = 2.4 + age * 2.3;
        let alpha = (0.34 - age * 0.062).max(0.0);
        if alpha > 0.0 {
            gizmos.circle_2d(centre, radius, theme::SLATE.with_alpha(alpha));
        }
    }
}

// --- The schematic ---------------------------------------------------------

/// The frame the SL concept demo draws into, and this map reproduces.
const FRAME: Vec2 = Vec2::new(1000.0, 400.0);
/// The dark ground the diagram sits on.
const GROUND: Color = Color::srgb_u8(0x2B, 0x2F, 0x36);
/// A station the player has selected on the ticket.
const PICKED: Color = Color::srgb_u8(0xFF, 0xD1, 0x66);
/// Radius of an ordinary station dot, in schematic units.
const DOT: f32 = 7.0;
/// Half-size of an interchange capsule, in schematic units.
const CAPSULE: Vec2 = Vec2::new(9.0, 13.0);

/// Where the schematic frame sits in the world, and how big it is drawn.
#[derive(Resource)]
pub struct SchematicView {
    pub scale: f32,
    /// World position of the frame's top-left corner.
    pub origin: Vec2,
}

impl SchematicView {
    /// Projects a schematic point into the world.
    ///
    /// The frame is an SVG viewBox, so its y runs down the page while the
    /// world's runs up it.
    pub fn at(&self, point: (i32, i32)) -> Vec2 {
        self.origin + Vec2::new(point.0 as f32, -point.1 as f32) * self.scale
    }

    /// A length in schematic units, in world pixels.
    pub fn len(&self, units: f32) -> f32 {
        units * self.scale
    }
}

/// Fits the frame into the column between the header and the progress strip.
pub fn make_schematic_view() -> SchematicView {
    // Measured against the 1440 x 940 window: the centre column runs from the
    // ticket panel to the city panel, and the strip takes the foot of it.
    const AREA_CENTRE: Vec2 = Vec2::new(-15.0, 38.0);
    const AREA: Vec2 = Vec2::new(712.0, 560.0);

    let scale = (AREA / FRAME).min_element();
    let drawn = FRAME * scale;
    SchematicView { scale, origin: AREA_CENTRE + Vec2::new(-drawn.x, drawn.y) * 0.5 }
}

/// The stations the concept names as interchanges, drawn as capsules.
const INTERCHANGES: [CityId; 7] = [
    CityId::Ghazzah,
    CityId::Yafa,
    CityId::AlLadd,
    CityId::AlQuds,
    CityId::Tulkarm,
    CityId::Haifa,
    CityId::Beisan,
];

/// Lays the dark ground the diagram is drawn on.
pub fn spawn_schematic_ground(mut commands: Commands, view: Res<SchematicView>) {
    let drawn = FRAME * view.scale;
    commands.spawn((
        SchematicGround,
        Sprite::from_color(GROUND, drawn),
        Transform::from_translation(
            (view.origin + Vec2::new(drawn.x, -drawn.y) * 0.5).extend(-20.0),
        ),
        Pickable::IGNORE,
    ));
}

/// Routes a segment between two schematic points using only horizontal,
/// vertical and 45-degree legs.
///
/// The diagram's whole legibility rests on that rule, so the corner is put in
/// rather than drawn through: run the diagonal for as far as both axes have
/// to travel, then straight for whatever is left.
fn elbow(from: Vec2, to: Vec2) -> [Vec2; 3] {
    let delta = to - from;
    let diagonal = delta.x.abs().min(delta.y.abs());
    let corner = from + Vec2::new(diagonal * delta.x.signum(), diagonal * delta.y.signum());
    [from, corner, to]
}

/// Draws the network as the schematic diagram.
pub fn draw_schematic(
    mut gizmos: Gizmos,
    rail: Res<Rail>,
    view: Res<SchematicView>,
    almanac: Res<Almanac>,
    desk: Res<Desk>,
    runs: Query<&TrainRun>,
) {
    let year = almanac.year();
    let lit = active_segments(&rail, &desk, runs.iter().next(), year);

    // The line itself first, then the chosen route over the top of it in
    // white, the way the concept has it.
    for segment in rail.segments() {
        let open = segment.opening.is_open(year);
        let color = if open {
            theme::line_color(segment.line)
        } else {
            theme::line_color(segment.line).mix(&GROUND, 0.62)
        };
        let bend = elbow(
            view.at(rail.city(segment.from).schematic),
            view.at(rail.city(segment.to).schematic),
        );
        if open {
            thick_polyline(&mut gizmos, &bend, color, view.len(9.0));
        } else {
            dashed_polyline(&mut gizmos, &bend, color);
        }
    }

    for index in &lit {
        let segment = &rail.segments()[*index];
        let bend = elbow(
            view.at(rail.city(segment.from).schematic),
            view.at(rail.city(segment.to).schematic),
        );
        thick_polyline(&mut gizmos, &bend, Color::WHITE.with_alpha(0.9), view.len(13.0));
    }
}

/// Gizmo lines are a pixel wide, so weight is drawn by stacking them across
/// the line's own normal.
fn thick_polyline(gizmos: &mut Gizmos, points: &[Vec2], color: Color, width: f32) {
    let steps = (width.round() as i32).max(1);
    for pair in points.windows(2) {
        let along = pair[1] - pair[0];
        if along.length_squared() < f32::EPSILON {
            continue;
        }
        let across = Vec2::new(-along.y, along.x).normalize();
        for step in 0..steps {
            let offset = across * (step as f32 - (steps - 1) as f32 * 0.5);
            gizmos.line_2d(pair[0] + offset, pair[1] + offset, color);
        }
    }
}

/// Track that is only drawn on paper, in the same bends.
fn dashed_polyline(gizmos: &mut Gizmos, points: &[Vec2], color: Color) {
    for pair in points.windows(2) {
        draw_dashed(gizmos, pair[0], pair[1], color);
    }
}

/// Spawns a station's mark, its two names and its hit target.
///
/// The names are set at -40 degrees the way the concept asks, Arabic over a
/// smaller Latin line in mono, so that dense stretches of the coast stay
/// readable without the labels colliding.
pub fn spawn_schematic_stations(
    mut commands: Commands,
    rail: Res<Rail>,
    view: Res<SchematicView>,
    fonts: Res<Fonts>,
) {
    for city in CityId::ALL {
        let data = rail.city(city);
        let at = view.at(data.schematic);
        let capsule = INTERCHANGES.contains(&city);

        // Al-Ladd sits alone on the link band with room beside it, so its
        // name is set level there rather than turned like the rest.
        let turned = city != CityId::AlLadd;
        let above = data.line_type == LineType::Coastal || city == CityId::AlLadd;
        let label_at = if turned {
            at + Vec2::new(0.0, if above { view.len(22.0) } else { -view.len(22.0) })
        } else {
            at + Vec2::new(view.len(16.0), -view.len(5.0))
        };
        let turn = if turned { 40f32.to_radians() } else { 0.0 };
        let anchor = if turned && !above {
            Anchor(Vec2::new(0.5, 0.0))
        } else {
            Anchor(Vec2::new(-0.5, 0.0))
        };
        let second = if above { -view.len(14.0) } else { view.len(14.0) };

        // A transparent pad is what the pointer hits, at the 36 px the
        // concept asks for. The visible mark is geometry and is built in
        // `spawn_schematic_marks`, which needs the renderer's asset stores.
        commands.spawn((
            StationDot(city),
            SchematicStation { city, capsule },
            Sprite::from_color(Color::NONE, Vec2::splat(view.len(36.0))),
            Transform::from_translation(at.extend(4.0)),
            Pickable::default(),
            Name::new(data.name),
        ));

        commands.spawn((
            StationLabel(city),
            Text2d::new(data.arabic),
            fonts.sign(view.len(15.0)),
            TextColor(Color::WHITE),
            anchor,
            Transform::from_translation(label_at.extend(6.0))
                .with_rotation(Quat::from_rotation_z(turn)),
            Pickable::IGNORE,
        ));
        commands.spawn((
            StationLatin(city),
            Text2d::new(data.name),
            fonts.data(view.len(10.0)),
            TextColor(theme::SLATE),
            anchor,
            Transform::from_translation((label_at + Vec2::new(0.0, second)).extend(6.0))
                .with_rotation(Quat::from_rotation_z(turn)),
            Pickable::IGNORE,
        ));
    }

    terminus_badges(&mut commands, &view, &fonts);
}

/// Builds the visible marks: a round dot, or a capsule for an interchange.
///
/// Separate from [`spawn_schematic_stations`] because it needs the mesh and
/// material stores, which the headless tests have not got.
pub fn spawn_schematic_marks(
    mut commands: Commands,
    view: Res<SchematicView>,
    stations: Query<(Entity, &SchematicStation)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dot = meshes.add(fan_mesh(&circle_outline(view.len(DOT))));
    let capsule =
        meshes.add(fan_mesh(&capsule_outline(Vec2::new(view.len(CAPSULE.x), view.len(CAPSULE.y)))));

    for (entity, station) in &stations {
        let paint = materials.add(ColorMaterial::from_color(Color::WHITE));
        commands.entity(entity).insert(StationMark { city: station.city, paint: paint.clone() });
        commands.entity(entity).with_child((
            Mesh2d(if station.capsule { capsule.clone() } else { dot.clone() }),
            MeshMaterial2d(paint),
            Transform::from_translation(Vec3::Z * 0.1),
            Pickable::IGNORE,
        ));
    }
}

/// A ring of points around the origin.
fn circle_outline(radius: f32) -> Vec<Vec2> {
    const SIDES: usize = 20;
    (0..SIDES)
        .map(|i| {
            let turn = i as f32 / SIDES as f32 * std::f32::consts::TAU;
            Vec2::new(turn.cos(), turn.sin()) * radius
        })
        .collect()
}

/// A stadium: a rectangle with both ends rounded off, standing upright.
fn capsule_outline(half: Vec2) -> Vec<Vec2> {
    const PER_END: usize = 10;
    let straight = (half.y - half.x).max(0.0);
    let mut points = Vec::with_capacity(PER_END * 2);
    for end in [1.0f32, -1.0] {
        for i in 0..=PER_END {
            let turn = i as f32 / PER_END as f32 * std::f32::consts::PI;
            let (sin, cos) = turn.sin_cos();
            points.push(Vec2::new(cos * half.x * end, sin * half.x * end + straight * end));
        }
    }
    points
}

/// The line number in a coloured square at each end of each line, as on an SL
/// map: 1 for the coast, 2 for the inland line.
fn terminus_badges(commands: &mut Commands, view: &SchematicView, fonts: &Fonts) {
    const BADGES: [((i32, i32), &str, LineType); 4] = [
        ((40, 110), "1", LineType::Coastal),
        ((950, 110), "1", LineType::Coastal),
        ((200, 310), "2", LineType::Mountain),
        ((990, 310), "2", LineType::Mountain),
    ];
    for (point, number, line) in BADGES {
        let at = view.at(point);
        commands.spawn((
            Sprite::from_color(theme::line_color(line), Vec2::splat(view.len(22.0))),
            Transform::from_translation(at.extend(5.0)),
            Pickable::IGNORE,
        ));
        commands.spawn((
            Text2d::new(number),
            fonts.data_strong(theme::CAPTION),
            TextColor(Color::WHITE),
            Transform::from_translation(at.extend(6.0)),
            Pickable::IGNORE,
        ));
    }
}

/// Where the engine stands on the diagram, and which way it is pointing.
///
/// The journey knows only geography, so how far along its current leg it has
/// come is read back from the geographic positions and then walked out along
/// the schematic bend. That keeps the calculation out of `yafa_core`, which
/// has no business knowing the map is a diagram.
pub fn schematic_train(run: &TrainRun, rail: &Rail, view: &SchematicView) -> (Vec2, f32) {
    let leg = run.current_leg();
    let bend = elbow(view.at(rail.city(leg.from).schematic), view.at(rail.city(leg.to).schematic));

    let (start, end) = (rail.position(leg.from), rail.position(leg.to));
    let here = run.position(rail);
    let span = Vec2::new(end.x_km - start.x_km, end.y_km - start.y_km).length();
    let gone = Vec2::new(here.x_km - start.x_km, here.y_km - start.y_km).length();
    let travelled = if span > f32::EPSILON { (gone / span).clamp(0.0, 1.0) } else { 0.0 };

    point_along(&bend, travelled)
}

/// Puts the engine where the journey has reached on the diagram.
pub fn place_train(
    rail: Res<Rail>,
    view: Res<SchematicView>,
    mut trains: Query<(&TrainRun, &mut Transform), With<Train>>,
) {
    let Ok((run, mut transform)) = trains.single_mut() else { return };
    let (at, heading) = schematic_train(run, &rail, &view);
    transform.translation = at.extend(20.0);
    transform.rotation = Quat::from_rotation_z(heading);
}

/// Walks `t` of the way along a polyline, and reports the heading there.
fn point_along(points: &[Vec2], t: f32) -> (Vec2, f32) {
    let legs: Vec<f32> = points.windows(2).map(|p| (p[1] - p[0]).length()).collect();
    let total: f32 = legs.iter().sum();
    if total < f32::EPSILON {
        return (points[0], 0.0);
    }

    let mut want = total * t.clamp(0.0, 1.0);
    for (index, length) in legs.iter().enumerate() {
        if want <= *length || index + 2 == points.len() {
            let along = points[index + 1] - points[index];
            let step = if *length > f32::EPSILON { want / length } else { 0.0 };
            return (points[index] + along * step, along.y.atan2(along.x));
        }
        want -= length;
    }
    (points[points.len() - 1], 0.0)
}

/// Colours the station marks: picked, visited, open, or not yet reached.
pub fn refresh_schematic_stations(
    rail: Res<Rail>,
    almanac: Res<Almanac>,
    desk: Res<Desk>,
    book: Res<PassportBook>,
    marks: Query<&StationMark>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut arabic: Query<(&StationLabel, &mut TextColor), Without<StationLatin>>,
) {
    let year = almanac.year();
    for mark in &marks {
        let Some(ref mut paint) = materials.get_mut(&mark.paint) else { continue };
        let picked = desk.origin == Some(mark.city) || desk.destination == Some(mark.city);
        paint.color = if picked {
            PICKED
        } else if book.passport.is_stamped(mark.city) {
            theme::VISITED
        } else if rail.station_is_open(mark.city, year) {
            Color::WHITE
        } else {
            theme::SLATE
        };
    }
    for (StationLabel(city), mut color) in &mut arabic {
        let picked = desk.origin == Some(*city) || desk.destination == Some(*city);
        color.0 = if picked {
            PICKED
        } else if rail.station_is_open(*city, year) {
            Color::WHITE
        } else {
            theme::SLATE
        };
    }
}

// --- The in-car progress strip ---------------------------------------------

/// The frame the concept's in-car screen is drawn into.
const STRIP: Vec2 = Vec2::new(900.0, 170.0);
/// The most stops a strip will label. The longest route on the network is
/// shorter than this; anything beyond it would be unreadable anyway.
pub const MAX_STOPS: usize = 14;

/// Where the strip sits in the world, and how big it is drawn.
#[derive(Resource)]
pub struct StripView {
    pub scale: f32,
    pub origin: Vec2,
}

impl StripView {
    pub fn at(&self, point: Vec2) -> Vec2 {
        self.origin + Vec2::new(point.x, -point.y) * self.scale
    }

    pub fn len(&self, units: f32) -> f32 {
        units * self.scale
    }
}

/// Fits the strip into the band between the map and the station board.
pub fn make_strip_view() -> StripView {
    const AREA_CENTRE: Vec2 = Vec2::new(-15.0, -196.0);
    const AREA: Vec2 = Vec2::new(712.0, 150.0);

    let scale = (AREA / STRIP).min_element();
    let drawn = STRIP * scale;
    StripView { scale, origin: AREA_CENTRE + Vec2::new(-drawn.x, drawn.y) * 0.5 }
}

/// Lays the strip's ground and the text that rides on it.
///
/// A fixed set of label slots is spawned once and shown or hidden as the route
/// changes, because a route is between two and a dozen stops long and
/// respawning text every frame would be wasteful and would flicker.
pub fn spawn_progress_strip(mut commands: Commands, view: Res<StripView>, fonts: Res<Fonts>) {
    let drawn = STRIP * view.scale;
    commands.spawn((
        StripGround,
        Sprite::from_color(GROUND, drawn),
        Transform::from_translation(
            (view.origin + Vec2::new(drawn.x, -drawn.y) * 0.5).extend(-20.0),
        ),
        Pickable::IGNORE,
    ));

    commands.spawn((
        StripStatus,
        Text2d::new(""),
        fonts.sign(view.len(16.0)),
        TextColor(Color::WHITE),
        Anchor(Vec2::new(0.5, 0.0)),
        Transform::from_translation(view.at(Vec2::new(850.0, 30.0)).extend(2.0)),
        Pickable::IGNORE,
    ));

    for slot in 0..MAX_STOPS {
        commands.spawn((
            StripStop(slot),
            Text2d::new(""),
            fonts.sign(view.len(14.0)),
            TextColor(Color::WHITE),
            Anchor(Vec2::new(-0.5, 0.0)),
            Transform::from_translation(Vec3::ZERO),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }

    // The terminus sits in a white box at the end of the line. The two names
    // are its children, so the box alone is moved and shown, and they follow.
    commands.spawn((
        StripTerminusBox,
        Sprite::from_color(Color::WHITE, Vec2::new(view.len(150.0), view.len(92.0))),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Hidden,
        Pickable::IGNORE,
        children![
            (
                StripTerminusName,
                Text2d::new(""),
                fonts.sign_bold(view.len(20.0)),
                TextColor(theme::INK),
                Transform::from_translation(Vec3::new(0.0, view.len(10.0), 0.1)),
                Pickable::IGNORE,
            ),
            (
                StripTerminusLatin,
                Text2d::new(""),
                fonts.data(view.len(12.0)),
                TextColor(theme::INK_SOFT),
                Transform::from_translation(Vec3::new(0.0, -view.len(16.0), 0.1)),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// Where each stop of the current route sits on the strip.
fn stop_positions(view: &StripView, stops: usize) -> Vec<Vec2> {
    let gap = if stops > 1 { (620.0 / (stops - 1) as f32).min(90.0) } else { 0.0 };
    (0..stops).map(|i| view.at(Vec2::new(50.0 + i as f32 * gap, 118.0))).collect()
}

/// Draws the line, the stops and the train on the strip.
pub fn draw_progress_strip(
    mut gizmos: Gizmos,
    rail: Res<Rail>,
    view: Res<StripView>,
    runs: Query<&TrainRun>,
) {
    let Ok(run) = runs.single() else { return };
    let stops: Vec<CityId> = run.route().stops().collect();
    let xs = stop_positions(&view, stops.len());
    let (Some(first), Some(last)) = (xs.first(), xs.last()) else { return };

    // Ahead of the train in white, behind it in the route's own colour.
    let here = strip_train(run, &xs);
    thick_polyline(&mut gizmos, &[*first, *last], Color::WHITE, view.len(8.0));
    let colour = theme::line_color(run.current_leg().line);
    thick_polyline(&mut gizmos, &[*first, here], colour, view.len(8.0));

    let reached = reached_count(run);
    for (index, at) in xs.iter().enumerate() {
        let next = index == reached.min(stops.len() - 1);
        let passed = index < reached;
        let radius = if next { view.len(8.0) } else { view.len(6.0) };
        let fill = if passed || next { colour } else { Color::WHITE };
        disc(&mut gizmos, *at, radius, fill);
    }

    // The train itself, a small marker on the line.
    let half = Vec2::new(view.len(14.0), view.len(7.0));
    gizmos.rect_2d(Isometry2d::from_translation(here), half * 2.0, PICKED);
    disc(&mut gizmos, here, view.len(5.0), PICKED);

    // A short tail of line into the terminus box.
    thick_polyline(
        &mut gizmos,
        &[*last, *last + Vec2::new(view.len(40.0), 0.0)],
        Color::WHITE,
        view.len(8.0),
    );
    let _ = rail;
}

/// A filled circle, drawn from rings because gizmos only stroke.
fn disc(gizmos: &mut Gizmos, centre: Vec2, radius: f32, color: Color) {
    let rings = (radius.round() as i32).max(1);
    for ring in 0..=rings {
        gizmos.circle_2d(centre, radius * ring as f32 / rings as f32, color);
    }
}

/// How many stops of the route the train has already called at.
fn reached_count(run: &TrainRun) -> usize {
    let stops: Vec<CityId> = run.route().stops().collect();
    match run.standing_at() {
        Some(city) => stops.iter().position(|&s| s == city).unwrap_or(0),
        None => stops.iter().position(|&s| s == run.current_leg().from).unwrap_or(0),
    }
}

/// Where the train marker sits along the strip.
fn strip_train(run: &TrainRun, xs: &[Vec2]) -> Vec2 {
    if xs.is_empty() {
        return Vec2::ZERO;
    }
    let travelled = run.progress() * (xs.len() - 1) as f32;
    let index = (travelled.floor() as usize).min(xs.len() - 1);
    let next = (index + 1).min(xs.len() - 1);
    xs[index].lerp(xs[next], travelled - index as f32)
}

/// Fills the strip's text from the current route.
///
/// Passed stops are dimmed, the next one is set larger and heavier, and the
/// terminus sits in its white box at the end - the three states the concept
/// asks for. With no train running the strip is empty and says so.
pub fn refresh_progress_strip(
    rail: Res<Rail>,
    view: Res<StripView>,
    lang: Res<Language>,
    fonts: Res<Fonts>,
    runs: Query<&TrainRun>,
    mut stops_text: Query<
        (&StripStop, &mut Text2d, &mut Transform, &mut Visibility, &mut TextColor, &mut TextFont),
        (
            Without<StripStatus>,
            Without<StripTerminusBox>,
            Without<StripTerminusName>,
            Without<StripTerminusLatin>,
        ),
    >,
    mut status: Query<
        &mut Text2d,
        (
            With<StripStatus>,
            Without<StripStop>,
            Without<StripTerminusName>,
            Without<StripTerminusLatin>,
        ),
    >,
    mut terminus: Query<
        (&mut Transform, &mut Visibility),
        (With<StripTerminusBox>, Without<StripStop>),
    >,
    mut names: Query<&mut Text2d, (With<StripTerminusName>, Without<StripStop>)>,
    mut latin: Query<
        &mut Text2d,
        (With<StripTerminusLatin>, Without<StripStop>, Without<StripTerminusName>),
    >,
) {
    let run = runs.iter().next();
    let stops: Vec<CityId> = run.map(|r| r.route().stops().collect()).unwrap_or_default();
    let xs = stop_positions(&view, stops.len());
    let reached = run.map_or(0, reached_count);

    for (StripStop(slot), mut text, mut transform, mut visible, mut colour, mut font) in
        &mut stops_text
    {
        match stops.get(*slot) {
            Some(city) => {
                let next = *slot == reached.min(stops.len().saturating_sub(1));
                let passed = *slot < reached;
                *visible = Visibility::Visible;
                text.0 = station_label(*lang, rail.city(*city)).to_owned();
                // Dimmed once it is behind you, bigger and bolder when it is
                // the one coming up.
                colour.0 = if passed && !next { theme::SLATE } else { Color::WHITE };
                *font =
                    if next { fonts.sign_bold(view.len(17.0)) } else { fonts.sign(view.len(14.0)) };
                *transform = Transform::from_translation(
                    (xs[*slot] + Vec2::new(0.0, view.len(16.0))).extend(2.0),
                )
                .with_rotation(Quat::from_rotation_z(40f32.to_radians()));
            }
            None => *visible = Visibility::Hidden,
        }
    }

    let end = xs.last().copied();
    let box_at = end.map(|at| at + Vec2::new(view.len(115.0), 0.0));

    if let Ok((mut transform, mut visible)) = terminus.single_mut() {
        match box_at {
            Some(at) => {
                *visible = Visibility::Visible;
                *transform = Transform::from_translation(at.extend(1.0));
            }
            None => *visible = Visibility::Hidden,
        }
    }
    if let Ok(mut text) = names.single_mut() {
        text.0 = stops.last().map(|c| rail.city(*c).arabic.to_owned()).unwrap_or_default();
    }
    if let Ok(mut text) = latin.single_mut() {
        text.0 = stops.last().map(|c| rail.city(*c).name.to_owned()).unwrap_or_default();
    }

    if let Ok(mut text) = status.single_mut() {
        text.0 = match run {
            None => lang.say(Phrase::WaitingForTicket).to_owned(),
            Some(run) => {
                let (phrase, city) = match run.phase() {
                    Phase::Arrived => (Phrase::ArrivedAt, run.route().destination()),
                    Phase::Running => (Phrase::InTransitTo, run.current_leg().to),
                    _ => (Phrase::StandingAt, run.current_leg().from),
                };
                fill(lang.say(phrase), &[station_label(*lang, rail.city(city))])
            }
        };
    }
}

/// A station's name in whichever script the interface is reading.
fn station_label(lang: Language, city: &yafa_core::City) -> &'static str {
    match lang {
        Language::English => city.name,
        Language::Arabic => city.arabic,
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

/// Moves the light on the map with the season.
///
/// The ground and the water carry it rather than an overlay: 2D gizmos sort
/// last whatever their depth, so a translucent sheet laid over the map would
/// tint the water and the engine but leave every rail, ring and shoreline
/// untouched, and the map would come apart.
pub fn refresh_season(
    almanac: Res<Almanac>,
    mut cameras: Query<&mut Camera, With<MapCamera>>,
    water: Query<&MeshMaterial2d<ColorMaterial>, With<SeaFill>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let through = almanac.through_year();

    if let Ok(mut camera) = cameras.single_mut() {
        camera.clear_color = ClearColorConfig::Custom(theme::Season::page(through));
    }
    if let Some(handle) = water.iter().next()
        && let Some(ref mut material) = materials.get_mut(&handle.0)
    {
        material.color = theme::Season::water(through);
    }
}
