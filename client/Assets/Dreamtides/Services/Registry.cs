#nullable enable

using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Registry : MonoBehaviour
  {
    public static TestConfiguration? TestConfiguration { get; set; }

    [SerializeField] string? _debugTestScenarioOverride;
    [SerializeField] GameLayout? _portraitLayout;
    [SerializeField] GameLayout? _landscapeLayout;
    bool _isLandscape = false;

    public bool IsLandscape => _isLandscape;

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

    [SerializeField] ControlledButton? _bottomRightButton;
    public ControlledButton BottomRightButton => Check(_bottomRightButton);

    void Awake()
    {
      var testConfiguration = TestConfiguration;
      if (!string.IsNullOrEmpty(_debugTestScenarioOverride))
      {
        testConfiguration = new TestConfiguration(_debugTestScenarioOverride);
      }

      if (testConfiguration != null)
      {
        Debug.Log($"Running test scenario: {testConfiguration.TestScenario}");
      }
      else
      {
        Debug.Log($"Starting Dreamtides");
      }

      Application.targetFrameRate = 60;

      _isLandscape = Screen.width > Screen.height;
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

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this, testConfiguration);
      }
    }

    T Check<T>(T? value) where T : Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}