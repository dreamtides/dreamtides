#nullable enable

using System;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class Registry : MonoBehaviour
  {
    [SerializeField] Camera? _mainCamera;
    public Camera MainCamera => Check(_mainCamera);

    [SerializeField] AudioSource? _mainAudioSource;
    public AudioSource MainAudioSource => Check(_mainAudioSource);

    [SerializeField] ObjectLayout? _userHand;
    public ObjectLayout UserHand => Check(_userHand);

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

    [SerializeField] ObjectLayout? _userAvatar;
    public ObjectLayout UserAvatar => Check(_userAvatar);

    [SerializeField] ObjectLayout? _enemyAvatar;
    public ObjectLayout EnemyAvatar => Check(_enemyAvatar);

    [SerializeField] ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField] ObjectLayout? _drawnCardsPosition;
    public ObjectLayout DrawnCardsPosition => Check(_drawnCardsPosition);

    [SerializeField] ObjectLayout? _stack;
    public ObjectLayout Stack => Check(_stack);

    [SerializeField] ObjectLayout? _selectingTargetsEnemy;
    public ObjectLayout SelectingTargetsEnemy => Check(_selectingTargetsEnemy);

    [SerializeField] ObjectLayout? _selectingTargetsUser;
    public ObjectLayout SelectingTargetsUser => Check(_selectingTargetsUser);

    [SerializeField] SpriteRenderer? _battlefieldOverlay;
    public SpriteRenderer BattlefieldOverlay => Check(_battlefieldOverlay);

    [SerializeField] GameMessage? _gameMessage;
    public GameMessage GameMessage => Check(_gameMessage);

    [SerializeField] LayoutService? _layoutService;
    public LayoutService LayoutService => Check(_layoutService);

    [SerializeField] ActionService? _actionService;
    public ActionService ActionService => Check(_actionService);

    [SerializeField] InputService? _inputService;
    public InputService InputService => Check(_inputService);

    [SerializeField] DocumentService? _documentService;
    public DocumentService DocumentService => Check(_documentService);

    [SerializeField] CapabilitiesService? _capabilitiesService;
    public CapabilitiesService CapabilitiesService => Check(_capabilitiesService);

    [SerializeField] SoundService? _soundService;
    public SoundService SoundService => Check(_soundService);

    [SerializeField] CardService? _cardService;
    public CardService CardService => Check(_cardService);

    [SerializeField] SettingsService? _settingsService;
    public SettingsService SettingsService => Check(_settingsService);

    [SerializeField] AssetService? _assetService;
    public AssetService AssetService => Check(_assetService);

    [SerializeField] AssetPoolService? _assetPoolService;
    public AssetPoolService AssetPoolService => Check(_assetPoolService);

    [SerializeField] EffectService? _effectService;
    public EffectService EffectService => Check(_effectService);

    [SerializeField] MusicService? _musicService;
    public MusicService MusicService => Check(_musicService);

    void Start()
    {
      Application.targetFrameRate = 60;

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this);
      }
    }

    T Check<T>(T? value) where T : UnityEngine.Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}