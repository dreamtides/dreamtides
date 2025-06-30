#nullable enable

using System.Collections;
using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Registry : MonoBehaviour
  {
    public static TestConfiguration? TestConfiguration { get; set; }

    [SerializeField] GameLayout? _portraitLayout;
    [SerializeField] GameLayout? _landscapeLayout;
    bool _isLandscape = false;

    public bool IsLandscape => _isLandscape;

    public bool IsMobileDevice => UnityEngine.Device.Application.isMobilePlatform;

    public GameLayout Layout =>
        IsLandscape ? Check(_landscapeLayout) : Check(_portraitLayout);

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

    [SerializeField] EnvironmentService? _environmentService;
    public EnvironmentService EnvironmentService => Check(_environmentService);

    [SerializeField] JudgmentService? _judgmentService;
    public JudgmentService JudgmentService => Check(_judgmentService);

    [SerializeField] DreamwellActivationService? _dreamwellActivationService;
    public DreamwellActivationService DreamwellActivationService => Check(_dreamwellActivationService);

    [SerializeField] ArrowService? _arrowService;
    public ArrowService ArrowService => Check(_arrowService);

    [SerializeField] CardEffectPreviewService? _cardEffectPreviewService;
    public CardEffectPreviewService CardEffectPreviewService => Check(_cardEffectPreviewService);

    [SerializeField] TestHelperService? _testHelperService;
    public TestHelperService TestHelperService => Check(_testHelperService);

    [SerializeField] LoggingService? _loggingService;
    public LoggingService LoggingService => Check(_loggingService);

    [SerializeField] StudioService? _studioService;
    public StudioService StudioService => Check(_studioService);

    [SerializeField] ControlledButton? _bottomRightButton;
    public ControlledButton BottomRightButton => Check(_bottomRightButton);

    [SerializeField] UserHandHoverService? _userHandHoverService;
    public UserHandHoverService UserHandHoverService => Check(_userHandHoverService);

    void Awake()
    {
      var testConfiguration = TestConfiguration;

      if (testConfiguration != null)
      {
        // Screen resolution is not correct on Awake() frame in tests because of
        // the hacks we are using to set it, so we delay the Awake() call.
        Debug.Log($"Starting integration test {testConfiguration.IntegrationTestId}");
        StartCoroutine(DelayedAwake(testConfiguration));
      }
      else
      {
        Debug.Log($"Starting Dreamtides");
        RunAwake(testConfiguration);
      }
    }

    IEnumerator DelayedAwake(TestConfiguration testConfiguration)
    {
      yield return new WaitForSeconds(0.1f);
      RunAwake(testConfiguration);
    }

    void RunAwake(TestConfiguration? testConfiguration)
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

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this, testConfiguration);
      }
    }

    T Check<T>(T? value) where T : Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}