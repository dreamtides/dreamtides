#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Utils;
using Unity.Cinemachine;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class BattleLayout : MonoBehaviour
  {
    [SerializeField]
    internal Transform? _cameraPosition;
    public Transform CameraPosition => Check(_cameraPosition);

    [SerializeField]
    internal CinemachineCamera? _battleCamera;
    public CinemachineCamera BattleCamera => Check(_battleCamera);

    [SerializeField]
    internal GameObject? _contents;
    public GameObject Contents => Check(_contents);

    [SerializeField]
    internal BattleCameraBounds? _battleCameraBounds;
    public BattleCameraBounds BattleCameraBounds => Check(_battleCameraBounds);

    [SerializeField]
    internal UserHandLayout? _userHand;
    public UserHandLayout UserHand => Check(_userHand);

    [SerializeField]
    internal ObjectLayout? _enemyHand;
    public ObjectLayout EnemyHand => Check(_enemyHand);

    [SerializeField]
    internal ObjectLayout? _userDeck;
    public ObjectLayout UserDeck => Check(_userDeck);

    [SerializeField]
    internal ObjectLayout? _enemyDeck;
    public ObjectLayout EnemyDeck => Check(_enemyDeck);

    [SerializeField]
    internal ObjectLayout? _userBattlefield;
    public ObjectLayout UserBattlefield => Check(_userBattlefield);

    [SerializeField]
    internal ObjectLayout? _enemyBattlefield;
    public ObjectLayout EnemyBattlefield => Check(_enemyBattlefield);

    [SerializeField]
    internal ObjectLayout? _userVoid;
    public ObjectLayout UserVoid => Check(_userVoid);

    [SerializeField]
    internal ObjectLayout? _enemyVoid;
    public ObjectLayout EnemyVoid => Check(_enemyVoid);

    [SerializeField]
    internal PlayerStatusDisplay? _userStatusDisplay;
    public PlayerStatusDisplay UserStatusDisplay => Check(_userStatusDisplay);

    [SerializeField]
    internal PlayerStatusDisplay? _enemyStatusDisplay;
    public PlayerStatusDisplay EnemyStatusDisplay => Check(_enemyStatusDisplay);

    [SerializeField]
    internal ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField]
    internal ObjectLayout? _drawnCardsPosition;
    public ObjectLayout DrawnCardsPosition => Check(_drawnCardsPosition);

    [SerializeField]
    internal ObjectLayout? _defaultStack;
    public ObjectLayout DefaultStack => Check(_defaultStack);

    [SerializeField]
    internal ObjectLayout? _targetingUserStack;
    public ObjectLayout TargetingUserStack => Check(_targetingUserStack);

    [SerializeField]
    internal ObjectLayout? _targetingEnemyStack;
    public ObjectLayout TargetingEnemyStack => Check(_targetingEnemyStack);

    [SerializeField]
    internal ObjectLayout? _targetingBothStack;
    public ObjectLayout TargetingBothStack => Check(_targetingBothStack);

    [SerializeField]
    internal SpriteRenderer? _battlefieldOverlay;
    public SpriteRenderer BattlefieldOverlay => Check(_battlefieldOverlay);

    [SerializeField]
    internal GameMessage? _gameMessage;
    public GameMessage GameMessage => Check(_gameMessage);

    [SerializeField]
    internal Transform _infoZoomLeft = null!;
    public Transform InfoZoomLeft => Check(_infoZoomLeft);

    [SerializeField]
    internal Transform _infoZoomRight = null!;
    public Transform InfoZoomRight => Check(_infoZoomRight);

    [SerializeField]
    internal Transform _supplementalCardInfoLeft = null!;
    public Transform SupplementalCardInfoLeft => Check(_supplementalCardInfoLeft);

    [SerializeField]
    internal Transform _supplementalCardInfoRight = null!;
    public Transform SupplementalCardInfoRight => Check(_supplementalCardInfoRight);

    [SerializeField]
    internal CardBrowser _browser = null!;
    public CardBrowser Browser => Check(_browser);

    [SerializeField]
    internal BackgroundOverlay _browserBackground = null!;
    public BackgroundOverlay BrowserBackground => Check(_browserBackground);

    [SerializeField]
    internal ObjectLayout? _userDreamwell;
    public ObjectLayout UserDreamwell => Check(_userDreamwell);

    [SerializeField]
    internal ObjectLayout? _enemyDreamwell;
    public ObjectLayout EnemyDreamwell => Check(_enemyDreamwell);

    [SerializeField]
    internal ObjectLayout? _dreamwellActivation;
    public ObjectLayout DreamwellActivation => Check(_dreamwellActivation);

    [SerializeField]
    internal Transform? _dreamwellDisplay;
    public Transform DreamwellDisplay => Check(_dreamwellDisplay);

    [SerializeField]
    internal CardOrderSelector? _cardOrderSelector;
    public CardOrderSelector CardOrderSelector => Check(_cardOrderSelector);

    [SerializeField]
    internal ObjectLayout? _cardOrderSelectorVoid;
    public ObjectLayout CardOrderSelectorVoid => Check(_cardOrderSelectorVoid);

    [SerializeField]
    internal ObjectLayout? _gameModifiersDisplay;
    public ObjectLayout GameModifiersDisplay => Check(_gameModifiersDisplay);

    [SerializeField]
    internal ObjectLayout? _onScreenStorage;
    public ObjectLayout OnScreenStorage => Check(_onScreenStorage);

    [SerializeField]
    internal ActionButton? _primaryActionButton;
    public ActionButton PrimaryActionButton => Check(_primaryActionButton);

    [SerializeField]
    internal ActionButton? _secondaryActionButton;
    public ActionButton SecondaryActionButton => Check(_secondaryActionButton);

    [SerializeField]
    internal ActionButton? _incrementActionButton;
    public ActionButton IncrementActionButton => Check(_incrementActionButton);

    [SerializeField]
    internal ActionButton? _decrementActionButton;
    public ActionButton DecrementActionButton => Check(_decrementActionButton);

    [SerializeField]
    internal CanvasButton? _undoButton;
    public CanvasButton UndoButton => Check(_undoButton);

    [SerializeField]
    internal CanvasButton? _devButton;
    public CanvasButton DevButton => Check(_devButton);

    [SerializeField]
    internal EnemyMessage? _enemyMessage;
    public EnemyMessage EnemyMessage => Check(_enemyMessage);

    [SerializeField]
    internal GameObject? _thinkingIndicator;
    public GameObject ThinkingIndicator => Check(_thinkingIndicator);

    [SerializeField]
    internal CloseBrowserButton? _closeBrowserButton;
    public CloseBrowserButton CloseBrowserButton => Check(_closeBrowserButton);

    [SerializeField]
    internal ObjectLayout? _aboveUserVoid;
    public ObjectLayout AboveUserVoid => Check(_aboveUserVoid);

    [SerializeField]
    internal ObjectLayout? _aboveEnemyVoid;
    public ObjectLayout AboveEnemyVoid => Check(_aboveEnemyVoid);

    /// <summary>
    /// Current turn number from the rules engine.
    /// </summary>
    public long TurnNumber { get; set; }

    /// <summary>
    /// Returns the Y rotation to use for cards to be displayed in the battle.
    /// </summary>
    public float BattleYRotation() => BattleCamera.transform.eulerAngles.y;

    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
