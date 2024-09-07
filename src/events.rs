//! Contains all reactive events that can be emitted in the app.

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when entering `GameState::DayStart`.
#[derive(Default)]
pub struct GameDayStart;

/// Reactive event emitted when entering `GameState::Play`.
#[derive(Default)]
pub struct GamePlay;

/// Reactive event emitted when entering `GameState::DayOver`.
#[derive(Default)]
pub struct GameDayOver;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when `GameClock` increments by one second. Used for updating the clock display.
pub struct GameClockIncremented;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event for toggling the settings display between on/off.
pub struct ToggleSettings;

/// Reactive event for toggling the settings display on.
pub struct ToggleSettingsOn;

/// Reactive event for toggling the settings display off.
pub struct ToggleSettingsOff;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when the player gains a power-up.
/// - This event indicates a power-up needs to be handled in the [`BufferedPowerUps`] resource.
pub struct PlayerPowerUp;

/// Reactive event emitted when the player dies.
pub struct PlayerDied;

/// Reactive event emitted when the day ends without the player dying.
pub struct PlayerSurvived;

//-------------------------------------------------------------------------------------------------------------------
