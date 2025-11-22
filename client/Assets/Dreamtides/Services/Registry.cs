#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class Registry : MonoBehaviour
  {
    public static TestConfiguration? TestConfiguration { get; set; }

    [SerializeField]
    internal GameLayout? _portraitLayout;

    [SerializeField]
    internal GameLayout? _landscapeLayout;

    bool _isLandscape = false;

    [SerializeField]
    GameMode _currentGameMode;

    public bool IsLandscape => _isLandscape;

    public bool IsMobileDevice => UnityEngine.Device.Application.isMobilePlatform;

    public GameLayout Layout => IsLandscape ? Check(_landscapeLayout) : Check(_portraitLayout);

    [SerializeField]
    Camera _mainCamera = null!;
    public Camera MainCamera => Check(_mainCamera);

    public IGameViewport GameViewport { get; private set; } = null!;

    [SerializeField]
    GameCamera _cameraAdjuster = null!;

    [SerializeField]
    Canvas _canvas = null!;
    public Canvas Canvas => Check(_canvas);

    [SerializeField]
    RectTransform _canvasSafeArea = null!;
    public RectTransform CanvasSafeArea => Check(_canvasSafeArea);

    [SerializeField]
    AudioSource _mainAudioSource = null!;
    public AudioSource MainAudioSource => Check(_mainAudioSource);

    [SerializeField]
    AudioSource _musicAudioSource = null!;
    public AudioSource MusicAudioSource => Check(_musicAudioSource);

    [SerializeField]
    DreamscapeLayout? _dreamscapeLayout;
    public DreamscapeLayout DreamscapeLayout => Check(_dreamscapeLayout);

    [SerializeField]
    CardService? _cardService;
    public CardService CardService => Check(_cardService);

    [SerializeField]
    ActionService? _actionService;
    public ActionService ActionService => Check(_actionService);

    [SerializeField]
    InputService? _inputService;
    public InputService InputService => Check(_inputService);

    [SerializeField]
    DocumentService? _documentService;
    public DocumentService DocumentService => Check(_documentService);

    [SerializeField]
    CapabilitiesService? _capabilitiesService;
    public CapabilitiesService CapabilitiesService => Check(_capabilitiesService);

    [SerializeField]
    SoundService? _soundService;
    public SoundService SoundService => Check(_soundService);

    [SerializeField]
    CardAnimationService? _cardAnimationService;
    public CardAnimationService CardAnimationService => Check(_cardAnimationService);

    [SerializeField]
    SettingsService? _settingsService;
    public SettingsService SettingsService => Check(_settingsService);

    [SerializeField]
    AssetService? _assetService;
    public AssetService AssetService => Check(_assetService);

    [SerializeField]
    AssetPoolService? _assetPoolService;
    public AssetPoolService AssetPoolService => Check(_assetPoolService);

    [SerializeField]
    EffectService? _effectService;
    public EffectService EffectService => Check(_effectService);

    [SerializeField]
    MusicService? _musicService;
    public MusicService MusicService => Check(_musicService);

    [SerializeField]
    EnvironmentService? _environmentService;
    public EnvironmentService EnvironmentService => Check(_environmentService);

    [SerializeField]
    JudgmentService? _judgmentService;
    public JudgmentService JudgmentService => Check(_judgmentService);

    [SerializeField]
    DreamwellActivationService? _dreamwellActivationService;
    public DreamwellActivationService DreamwellActivationService =>
      Check(_dreamwellActivationService);

    [SerializeField]
    ArrowService? _arrowService;
    public ArrowService ArrowService => Check(_arrowService);

    [SerializeField]
    CardEffectPreviewService? _cardEffectPreviewService;
    public CardEffectPreviewService CardEffectPreviewService => Check(_cardEffectPreviewService);

    [SerializeField]
    TestHelperService? _testHelperService;
    public TestHelperService TestHelperService => Check(_testHelperService);

    [SerializeField]
    LoggingService? _loggingService;
    public LoggingService LoggingService => Check(_loggingService);

    [SerializeField]
    StudioService? _studioService;
    public StudioService StudioService => Check(_studioService);

    [SerializeField]
    UserHandHoverService? _userHandHoverService;
    public UserHandHoverService UserHandHoverService => Check(_userHandHoverService);

    [SerializeField]
    IdleReconnectService? _idleReconnectService;
    public IdleReconnectService IdleReconnectService => Check(_idleReconnectService);

    [SerializeField]
    DreamscapeService _dreamscapeService = null!;
    public DreamscapeService DreamscapeService => Check(_dreamscapeService);

    [SerializeField]
    PrototypeQuest? _prototypeQuest;
    public PrototypeQuest PrototypeQuest => Check(_prototypeQuest);

    void Awake()
    {
      StartCoroutine(RunAwake());
    }

    public IEnumerator RunAwake(
      GameMode? mode = null,
      TestConfiguration? testConfiguration = null,
      IGameViewport? gameViewport = null
    )
    {
      _currentGameMode =
        mode ?? (GameMode)PlayerPrefs.GetInt(PlayerPrefKeys.SelectedPlayMode, (int)GameMode.Quest);
      // Need to use this fully-qualified UnityEngine.Device.Screen API to have
      // it work in Device Simulator.
      var width = gameViewport?.ScreenWidth ?? UnityEngine.Device.Screen.width;
      var height = gameViewport?.ScreenHeight ?? UnityEngine.Device.Screen.height;
      _isLandscape = width > height;
      GameViewport = gameViewport ?? new RealViewport(this);

      if (testConfiguration == null)
      {
        Debug.Log($"Starting Dreamtides with game mode {_currentGameMode}");
      }

      yield return new WaitForEndOfFrame();
      yield return InitializeAll(_currentGameMode, testConfiguration, gameViewport);
    }

    IEnumerator InitializeAll(
      GameMode mode,
      TestConfiguration? testConfiguration,
      IGameViewport? gameViewport
    )
    {
      if (_isLandscape)
      {
        Check(_portraitLayout).gameObject.SetActive(false);
        Check(_landscapeLayout).gameObject.SetActive(true);
      }
      else
      {
        Check(_portraitLayout).gameObject.SetActive(true);
        Check(_landscapeLayout).gameObject.SetActive(false);
      }

      Application.targetFrameRate = 60;

      foreach (
        var element in FindObjectsByType<Displayable>(
          FindObjectsInactive.Include,
          FindObjectsSortMode.None
        )
      )
      {
        if (element.gameObject.scene == gameObject.scene)
        {
          element.Initialize(this, mode, testConfiguration, fromRegistry: true);
        }
      }

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this, mode, testConfiguration);
      }

      foreach (
        var element in FindObjectsByType<SceneElement>(
          FindObjectsInactive.Exclude,
          FindObjectsSortMode.None
        )
      )
      {
        if (element.gameObject.scene == gameObject.scene)
        {
          element.Initialize(this, mode, testConfiguration);
        }
      }

      if (testConfiguration == null)
      {
        TempToggleGameObjectsForMode(mode);
      }

      yield return new WaitForEndOfFrame();

      var startCoroutines = new List<Coroutine>();
      foreach (
        var element in FindObjectsByType<Displayable>(
          FindObjectsInactive.Exclude,
          FindObjectsSortMode.None
        )
      )
      {
        if (element.gameObject.scene == gameObject.scene)
        {
          var routine = element.StartFromRegistry();
          if (routine != null)
          {
            startCoroutines.Add(StartCoroutine(routine));
          }
        }
      }
      foreach (var coroutine in startCoroutines)
      {
        yield return coroutine;
      }
    }

    [SerializeField]
    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");

    void TempToggleGameObjectsForMode(GameMode mode)
    {
      // In the future we need to run all this logic after connecting to the
      // rules engine, this is just a temporary solution for prototyping.

      var battleMode = mode == GameMode.Battle;
      if (battleMode)
      {
        MainCamera.transform.position = Layout.CameraPosition.position;
        MainCamera.transform.rotation = Layout.CameraPosition.rotation;
        _cameraAdjuster.AdjustFieldOfView(Layout.BattleCameraBounds);
      }

      Layout.UserStatusDisplay.TotalSpark.gameObject.SetActive(battleMode);
      Layout.UserStatusDisplay.gameObject.SetActive(battleMode);
      Layout.EnemyStatusDisplay.TotalSpark.gameObject.SetActive(battleMode);
      Layout.EnemyStatusDisplay.gameObject.SetActive(battleMode);
      Layout.PrimaryActionButton.gameObject.SetActive(battleMode);
      Layout.SecondaryActionButton.gameObject.SetActive(battleMode);
      Layout.IncrementActionButton.gameObject.SetActive(battleMode);
      Layout.DecrementActionButton.gameObject.SetActive(battleMode);
      foreach (var button in Layout.GetComponentsInChildren<CardBrowserButton>())
      {
        button.gameObject.SetActive(battleMode);
      }
    }
  }
}
