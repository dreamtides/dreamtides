#nullable enable

using System.Collections;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Registry : MonoBehaviour
  {
    public static TestConfiguration? TestConfiguration { get; set; }

    [SerializeField]
    GameLayout? _portraitLayout;

    [SerializeField]
    GameLayout? _landscapeLayout;
    bool _isLandscape = false;

    public bool IsLandscape => _isLandscape;

    public bool IsMobileDevice => UnityEngine.Device.Application.isMobilePlatform;

    public GameLayout Layout => IsLandscape ? Check(_landscapeLayout) : Check(_portraitLayout);

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
    ControlledButton? _bottomRightButton;
    public ControlledButton BottomRightButton => Check(_bottomRightButton);

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
      var mode = (GameMode)PlayerPrefs.GetInt(PlayerPrefKeys.SelectedPlayMode, (int)GameMode.Quest);
      var testConfiguration = TestConfiguration;

      if (testConfiguration != null)
      {
        // Note: Test screen resolution is not correct on Awake() frame
        Debug.Log($"Starting integration test {testConfiguration.IntegrationTestId}");
      }
      else
      {
        Debug.Log($"Starting Dreamtides with game mode {mode}");
      }

      StartCoroutine(DelayedAwake(mode, testConfiguration));
    }

    IEnumerator DelayedAwake(GameMode mode, TestConfiguration? testConfiguration)
    {
      yield return new WaitForEndOfFrame();
      yield return new WaitForEndOfFrame();
      yield return new WaitForEndOfFrame();
      yield return new WaitForEndOfFrame();
      yield return new WaitForEndOfFrame();

      RunAwake(mode, testConfiguration);
    }

    void RunAwake(GameMode mode, TestConfiguration? testConfiguration)
    {
      var width = UnityEngine.Device.Screen.width;
      var height = UnityEngine.Device.Screen.height;
      _isLandscape = width > height;
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
      ToggleGameObjectForMode(mode);

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this, mode, testConfiguration);
      }
    }

    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");

    void ToggleGameObjectForMode(GameMode mode)
    {
      var battleMode = mode == GameMode.Battle;
      if (battleMode)
      {
        var battleCamera = ComponentUtils.Get<GameCamera>(Layout.MainCamera);
        battleCamera.AdjustFieldOfView();
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
        button.gameObject.SetActive(false);
      }
    }
  }
}
