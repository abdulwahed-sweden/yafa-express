//! Sound.
//!
//! The file names, the folders and the message each sound answers to are
//! fixed by [`docs/audio/sound-catalogue.md`]; the constants below are that
//! page in code, and the name of a constant is the name of its file.
//!
//! One system plays everything, reading the same messages the rest of the
//! game already sends. No draw system ever plays a sound.
//!
//! The clips themselves are not in the repository yet. Every path is loaded
//! anyway, and a sound whose file is missing is simply not played - so the
//! game runs, and the tests pass, with the whole folder empty. See
//! `assets/audio/README.md` for what is still wanted.

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::components::*;

/// Where each clip lives, as the catalogue names it.
pub mod paths {
    pub const TRAIN_WHISTLE_DEPART: &str = "audio/train/train_whistle_depart.ogg";
    pub const TRAIN_WHEELS_LOOP: &str = "audio/train/train_wheels_loop.ogg";
    pub const TRAIN_BRAKE_STEAM: &str = "audio/train/train_brake_steam.ogg";
    pub const STATION_BELL_ARRIVE: &str = "audio/train/station_bell_arrive.ogg";

    pub const UI_STATION_SELECT: &str = "audio/ui/ui_station_select.ogg";
    pub const UI_TICKET_TEAR: &str = "audio/ui/ui_ticket_tear.ogg";
    pub const UI_GATE_OPEN: &str = "audio/ui/ui_gate_open.ogg";
    pub const UI_GATE_CLOSED: &str = "audio/ui/ui_gate_closed.ogg";
    pub const UI_PASSPORT_STAMP: &str = "audio/ui/ui_passport_stamp.ogg";
    pub const UI_COINS_REVENUE: &str = "audio/ui/ui_coins_revenue.ogg";
    pub const UI_ERROR_SOFT: &str = "audio/ui/ui_error_soft.ogg";

    pub const AMB_COAST_WAVES: &str = "audio/ambience/amb_coast_waves.ogg";
    pub const AMB_MOUNTAIN_WIND: &str = "audio/ambience/amb_mountain_wind.ogg";
    pub const AMB_CITY_MARKET: &str = "audio/ambience/amb_city_market.ogg";

    pub const MUSIC_MENU_OUD_BAYATI: &str = "audio/music/music_menu_oud_bayati.ogg";
    pub const MUSIC_JOURNEY_LOW: &str = "audio/music/music_journey_low.ogg";
}

/// Every sound a message can ask for: the one-shots.
///
/// The loops - wheels, brakes, the ambiences and the music - are in the
/// catalogue and in [`paths`], but nothing drives them yet, so they are not
/// loaded and not listed here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    WhistleDepart,
    BellArrive,
    TicketTear,
    GateOpen,
    GateClosed,
    PassportStamp,
}

impl Sound {
    pub const ALL: [Sound; 6] = [
        Sound::WhistleDepart,
        Sound::BellArrive,
        Sound::TicketTear,
        Sound::GateOpen,
        Sound::GateClosed,
        Sound::PassportStamp,
    ];

    /// The file this sound is stored in.
    pub fn path(self) -> &'static str {
        match self {
            Sound::WhistleDepart => paths::TRAIN_WHISTLE_DEPART,
            Sound::BellArrive => paths::STATION_BELL_ARRIVE,
            Sound::TicketTear => paths::UI_TICKET_TEAR,
            Sound::GateOpen => paths::UI_GATE_OPEN,
            Sound::GateClosed => paths::UI_GATE_CLOSED,
            Sound::PassportStamp => paths::UI_PASSPORT_STAMP,
        }
    }

    /// How loud it sits against the others. The whistle is the loudest thing
    /// in the game and the interface clicks stay under it.
    pub fn gain(self) -> f32 {
        match self {
            Sound::WhistleDepart => 0.8,
            Sound::BellArrive => 0.7,
            Sound::PassportStamp => 0.65,
            Sound::GateOpen | Sound::GateClosed => 0.55,
            Sound::TicketTear => 0.45,
        }
    }
}

/// A handle for every one-shot, in the order of [`Sound::ALL`].
#[derive(Resource)]
pub struct Sfx([Handle<AudioSource>; Sound::ALL.len()]);

impl Sfx {
    /// Asks the asset server for every clip at once.
    pub fn load(assets: &AssetServer) -> Self {
        Self(Sound::ALL.map(|sound| assets.load(sound.path())))
    }

    /// The clip for a sound, whether or not its file turned up.
    pub fn handle(&self, sound: Sound) -> &Handle<AudioSource> {
        let slot = Sound::ALL.iter().position(|&s| s == sound).expect("every sound is in ALL");
        &self.0[slot]
    }
}

/// Reads the clips in, once, at startup.
///
/// Until the files land, the asset server logs one "path not found" for each
/// of them here. That is expected, and the line below says so in the same
/// place, so the log reads as a missing folder rather than a broken game.
pub fn load_sfx(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sfx::load(&assets));
    info!(
        "asked for {} sounds; any whose file is missing is simply not played - \
         see assets/audio/README.md",
        Sound::ALL.len()
    );
}

/// Plays a sound for every message that calls for one.
///
/// The only system in the game that plays anything.
pub fn play_sounds(
    mut commands: Commands,
    sfx: Res<Sfx>,
    clips: Res<Assets<AudioSource>>,
    mut departed: MessageReader<TrainDeparted>,
    mut reached: MessageReader<TrainReached>,
    mut stamped: MessageReader<PassportStamped>,
    mut torn: MessageReader<TicketTorn>,
    mut answered: MessageReader<GateAnswered>,
) {
    let mut queue: Vec<Sound> = Vec::new();

    queue.extend(torn.read().map(|_| Sound::TicketTear));
    queue.extend(answered.read().map(|&GateAnswered(verdict)| match verdict {
        GateVerdict::Passed => Sound::GateOpen,
        GateVerdict::Refused => Sound::GateClosed,
    }));
    queue.extend(departed.read().map(|_| Sound::WhistleDepart));
    queue.extend(reached.read().map(|_| Sound::BellArrive));
    // Only a stamp the traveller did not already have is worth the sound.
    queue.extend(stamped.read().filter(|stamp| stamp.fresh).map(|_| Sound::PassportStamp));

    for sound in queue {
        let clip = sfx.handle(sound);
        // A clip whose file is missing would sit in the queue for ever
        // waiting to be decoded, so nothing is spawned until it is really
        // there.
        if !clips.contains(clip) {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(clip.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sound.gain())),
        ));
    }
}

/// Everything that makes a noise.
///
/// Separate from [`crate::SimulationPlugin`] on purpose: the headless tests
/// have no asset server and no audio output, and the rules do not need one.
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sfx)
            .add_systems(Update, play_sounds.in_set(crate::GameSet::Present));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_has_its_own_file_under_the_catalogue_folder() {
        let mut seen = Vec::new();
        for sound in Sound::ALL {
            let path = sound.path();
            assert!(path.starts_with("audio/"), "{path} is not under the audio folder");
            assert!(path.ends_with(".ogg"), "the catalogue is OGG only: {path}");
            assert!(!seen.contains(&path), "two sounds share {path}");
            seen.push(path);
        }
    }

    #[test]
    fn nothing_is_louder_than_the_whistle() {
        for sound in Sound::ALL {
            assert!(sound.gain() > 0.0 && sound.gain() <= Sound::WhistleDepart.gain());
        }
    }
}
