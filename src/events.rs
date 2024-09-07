//! Contains all reactive events that can be emitted in the app.

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when entering `GameState::DayStart`.
#[derive(Default, Copy, Clone)]
pub struct GameDayStart;

/// Reactive event emitted when entering `GameState::Play`.
#[derive(Default, Copy, Clone)]
pub struct GamePlay;

/// Reactive event emitted when entering `GameState::DayOver`.
#[derive(Default, Copy, Clone)]
pub struct GameDayOver;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when `GameClock` increments by one second. Used for updating the clock display.
#[derive(Default, Copy, Clone)]
pub struct GameClockIncremented;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event for toggling the settings display between on/off.
#[derive(Default, Copy, Clone)]
pub struct ToggleSettings;

/// Reactive event for toggling the settings display on.
#[derive(Default, Copy, Clone)]
pub struct ToggleSettingsOn;

/// Reactive event for toggling the settings display off.
#[derive(Default, Copy, Clone)]
pub struct ToggleSettingsOff;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event emitted when the player gains a power-up.
/// - This event indicates a power-up needs to be handled in the [`BufferedPowerUps`] resource.
#[derive(Default, Copy, Clone)]
pub struct PlayerPowerUp;

/// Reactive event emitted when the player dies.
#[derive(Default, Copy, Clone)]
pub struct PlayerDied;

/// Reactive event emitted when the day ends without the player dying.
#[derive(Default, Copy, Clone)]
pub struct PlayerSurvived;

//-------------------------------------------------------------------------------------------------------------------
