#nullable enable

namespace Dreamtides.Layout
{
  using Dreamtides.Buttons;
  using Dreamtides.Components;
  using Dreamtides.Utils;
  using UnityEngine;

  public class GameLayout : MonoBehaviour
  {
    [SerializeField] Camera? _mainCamera;
    public Camera MainCamera => Check(_mainCamera);

    [SerializeField] AudioSource? _mainAudioSource;
    public AudioSource MainAudioSource => Check(_mainAudioSource);

    [SerializeField] AudioSource? _musicAudioSource;
    public AudioSource MusicAudioSource => Check(_musicAudioSource);

    [SerializeField] UserHandLayout? _userHand;
    public UserHandLayout UserHand => Check(_userHand);

    [SerializeField] ObjectLayout? _enemyHand;
    public ObjectLayout EnemyHand => Check(_enemyHand);

    [SerializeField] ObjectLayout? _userDeck;
    public ObjectLayout UserDeck => Check(_userDeck);

    [SerializeField] ObjectLayout? _enemyDeck;
    public ObjectLayout EnemyDeck => Check(_enemyDeck);

    [SerializeField] ObjectLayout? _userBattlefield;
    public ObjectLayout UserBattlefield => Check(_userBattlefield);

    [SerializeField] ObjectLayout? _enemyBattlefield;
    public ObjectLayout EnemyBattlefield => Check(_enemyBattlefield);

    [SerializeField] ObjectLayout? _userVoid;
    public ObjectLayout UserVoid => Check(_userVoid);

    [SerializeField] ObjectLayout? _enemyVoid;
    public ObjectLayout EnemyVoid => Check(_enemyVoid);

    [SerializeField] PlayerStatusDisplay? _userStatusDisplay;
    public PlayerStatusDisplay UserStatusDisplay => Check(_userStatusDisplay);

    [SerializeField] PlayerStatusDisplay? _enemyStatusDisplay;
    public PlayerStatusDisplay EnemyStatusDisplay => Check(_enemyStatusDisplay);

    [SerializeField] ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField] ObjectLayout? _drawnCardsPosition;
    public ObjectLayout DrawnCardsPosition => Check(_drawnCardsPosition);

    [SerializeField] ObjectLayout? _defaultStack;
    public ObjectLayout DefaultStack => Check(_defaultStack);

    [SerializeField] ObjectLayout? _targetingUserStack;
    public ObjectLayout TargetingUserStack => Check(_targetingUserStack);

    [SerializeField] ObjectLayout? _targetingEnemyStack;
    public ObjectLayout TargetingEnemyStack => Check(_targetingEnemyStack);

    [SerializeField] ObjectLayout? _targetingBothStack;
    public ObjectLayout TargetingBothStack => Check(_targetingBothStack);

    [SerializeField] SpriteRenderer? _battlefieldOverlay;
    public SpriteRenderer BattlefieldOverlay => Check(_battlefieldOverlay);

    [SerializeField] GameMessage? _gameMessage;
    public GameMessage GameMessage => Check(_gameMessage);

    [SerializeField] Transform _infoZoomLeft = null!;
    public Transform InfoZoomLeft => Check(_infoZoomLeft);

    [SerializeField] Transform _infoZoomRight = null!;
    public Transform InfoZoomRight => Check(_infoZoomRight);

    [SerializeField] Transform _supplementalCardInfoLeft = null!;
    public Transform SupplementalCardInfoLeft => Check(_supplementalCardInfoLeft);

    [SerializeField] Transform _supplementalCardInfoRight = null!;
    public Transform SupplementalCardInfoRight => Check(_supplementalCardInfoRight);

    [SerializeField] CardBrowser _browser = null!;
    public CardBrowser Browser => Check(_browser);

    [SerializeField] BrowserBackground _browserBackground = null!;
    public BrowserBackground BrowserBackground => Check(_browserBackground);

    [SerializeField] ObjectLayout? _userDreamwell;
    public ObjectLayout UserDreamwell => Check(_userDreamwell);

    [SerializeField] ObjectLayout? _enemyDreamwell;
    public ObjectLayout EnemyDreamwell => Check(_enemyDreamwell);

    [SerializeField] ObjectLayout? _dreamwellActivation;
    public ObjectLayout DreamwellActivation => Check(_dreamwellActivation);

    [SerializeField] Transform? _dreamwellDisplay;
    public Transform DreamwellDisplay => Check(_dreamwellDisplay);

    [SerializeField] CardOrderSelector? _cardOrderSelector;
    public CardOrderSelector CardOrderSelector => Check(_cardOrderSelector);

    [SerializeField] ObjectLayout? _cardOrderSelectorVoid;
    public ObjectLayout CardOrderSelectorVoid => Check(_cardOrderSelectorVoid);

    [SerializeField] ObjectLayout? _gameModifiersDisplay;
    public ObjectLayout GameModifiersDisplay => Check(_gameModifiersDisplay);

    [SerializeField] ObjectLayout? _onScreenStorage;
    public ObjectLayout OnScreenStorage => Check(_onScreenStorage);

    [SerializeField] ActionButton? _primaryActionButton;
    public ActionButton PrimaryActionButton => Check(_primaryActionButton);

    [SerializeField] ActionButton? _secondaryActionButton;
    public ActionButton SecondaryActionButton => Check(_secondaryActionButton);

    [SerializeField] ActionButton? _incrementActionButton;
    public ActionButton IncrementActionButton => Check(_incrementActionButton);

    [SerializeField] ActionButton? _decrementActionButton;
    public ActionButton DecrementActionButton => Check(_decrementActionButton);

    [SerializeField] ControlledButton? _undoButton;
    public ControlledButton UndoButton => Check(_undoButton);

    [SerializeField] ControlledButton? _devButton;
    public ControlledButton DevButton => Check(_devButton);

    [SerializeField] EnemyMessage? _enemyMessage;
    public EnemyMessage EnemyMessage => Check(_enemyMessage);

    [SerializeField] GameObject? _thinkingIndicator;
    public GameObject ThinkingIndicator => Check(_thinkingIndicator);

    [SerializeField] CloseBrowserButton? _closeBrowserButton;
    public CloseBrowserButton CloseBrowserButton => Check(_closeBrowserButton);

    [SerializeField] ObjectLayout? _aboveUserVoid;
    public ObjectLayout AboveUserVoid => Check(_aboveUserVoid);

    [SerializeField] ObjectLayout? _aboveEnemyVoid;
    public ObjectLayout AboveEnemyVoid => Check(_aboveEnemyVoid);

    T Check<T>(T? value) where T : Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}