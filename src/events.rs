//! Contains all reactive events that can be emitted in the app.

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when entering `GameState::DayStart`.
#[derive(Default)]
pub struct GameDayStart;

/// Reactive event emitted when entering `GameState::Play`.
#[derive(Default)]
pub struct GamePlay;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event for toggling the settings display between on/off.
pub struct ToggleSettings;

/// Reactive event for toggling the settings display on.
pub struct ToggleSettingsOn;

/// Reactive event for toggling the settings display off.
pub struct ToggleSettingsOff;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when the player levels up.
pub struct PlayerLevelUp;

/// Reactive event emitted when the player dies.
pub struct PlayerDied;

/// Reactive event emitted when the day ends without the player dying.
pub struct DayEnded;

//-------------------------------------------------------------------------------------------------------------------
