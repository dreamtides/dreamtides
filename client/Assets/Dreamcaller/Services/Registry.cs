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
    bool _isPortrait = false;
    [SerializeField] GameLayout? _portraitLayout;
    [SerializeField] GameLayout? _landscapeLayout;

    public bool IsPortrait => _isPortrait;

    public GameLayout Layout =>
        IsPortrait ? Check(_portraitLayout) : Check(_landscapeLayout);

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

      _isPortrait = Screen.width < Screen.height;
      if (_isPortrait)
      {
        Check(_portraitLayout).gameObject.SetActive(true);
        Check(_landscapeLayout).gameObject.SetActive(false);
      }
      else
      {
        Check(_portraitLayout).gameObject.SetActive(false);
        Check(_landscapeLayout).gameObject.SetActive(true);
      }

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this);
      }
    }

    T Check<T>(T? value) where T : UnityEngine.Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}