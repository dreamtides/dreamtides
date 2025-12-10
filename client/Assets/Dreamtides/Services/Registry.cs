#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Utils;
using Unity.Cinemachine;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;
using UnityEngine.SceneManagement;
#if UNITY_EDITOR
using UnityEditor;
#endif

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class Registry : MonoBehaviour
  {
    public static TestConfiguration? TestConfiguration { get; set; }

    [SerializeField]
    internal BattleLayout? _portraitLayout;

    [SerializeField]
    internal BattleLayout? _landscapeLayout;

    bool _isLandscape = false;
    TestConfiguration? _activeTestConfiguration;

    [SerializeField]
    internal GameMode _currentGameMode;

    [SerializeField]
    internal AssetReference? _additionalSceneReference;

    public bool IsLandscape => _isLandscape;

    public bool IsMobileDevice => UnityEngine.Device.Application.isMobilePlatform;

    public BattleLayout BattleLayout =>
      IsLandscape ? Check(_landscapeLayout) : Check(_portraitLayout);

    [SerializeField]
    internal Camera _mainCamera = null!;
    public Camera MainCamera => Check(_mainCamera);

    [SerializeField]
    internal CinemachineBrain _cinemachineBrain = null!;
    public CinemachineBrain CinemachineBrain => Check(_cinemachineBrain);

    public IGameViewport GameViewport { get; private set; } = null!;

    [SerializeField]
    internal GameCamera _cameraAdjuster = null!;

    [SerializeField]
    internal Canvas _canvas = null!;
    public Canvas Canvas => Check(_canvas);

    [SerializeField]
    internal RectTransform _canvasSafeArea = null!;
    public RectTransform CanvasSafeArea => Check(_canvasSafeArea);

    [SerializeField]
    internal AudioSource _mainAudioSource = null!;
    public AudioSource MainAudioSource => Check(_mainAudioSource);

    [SerializeField]
    internal AudioSource _musicAudioSource = null!;
    public AudioSource MusicAudioSource => Check(_musicAudioSource);

    [SerializeField]
    internal DreamscapeLayout? _dreamscapeLayout;
    public DreamscapeLayout DreamscapeLayout => Check(_dreamscapeLayout);

    [SerializeField]
    internal CardService? _cardService;
    public CardService CardService => Check(_cardService);

    [SerializeField]
    internal ActionService? _actionService;
    public ActionService ActionService => Check(_actionService);

    [SerializeField]
    internal InputService? _inputService;
    public InputService InputService => Check(_inputService);

    [SerializeField]
    internal DocumentService? _documentService;
    public DocumentService DocumentService => Check(_documentService);

    [SerializeField]
    internal CapabilitiesService? _capabilitiesService;
    public CapabilitiesService CapabilitiesService => Check(_capabilitiesService);

    [SerializeField]
    internal SoundService? _soundService;
    public SoundService SoundService => Check(_soundService);

    [SerializeField]
    internal CardAnimationService? _cardAnimationService;
    public CardAnimationService CardAnimationService => Check(_cardAnimationService);

    [SerializeField]
    internal SettingsService? _settingsService;
    public SettingsService SettingsService => Check(_settingsService);

    [SerializeField]
    internal AssetService? _assetService;
    public AssetService AssetService => Check(_assetService);

    [SerializeField]
    internal AssetPoolService? _assetPoolService;
    public AssetPoolService AssetPoolService => Check(_assetPoolService);

    [SerializeField]
    internal EffectService? _effectService;
    public EffectService EffectService => Check(_effectService);

    [SerializeField]
    internal MusicService? _musicService;
    public MusicService MusicService => Check(_musicService);

    [SerializeField]
    internal EnvironmentService? _environmentService;
    public EnvironmentService EnvironmentService => Check(_environmentService);

    [SerializeField]
    internal JudgmentService? _judgmentService;
    public JudgmentService JudgmentService => Check(_judgmentService);

    [SerializeField]
    internal DreamwellActivationService? _dreamwellActivationService;
    public DreamwellActivationService DreamwellActivationService =>
      Check(_dreamwellActivationService);

    [SerializeField]
    internal ArrowService? _arrowService;
    public ArrowService ArrowService => Check(_arrowService);

    [SerializeField]
    internal CardEffectPreviewService? _cardEffectPreviewService;
    public CardEffectPreviewService CardEffectPreviewService => Check(_cardEffectPreviewService);

    [SerializeField]
    internal TestHelperService? _testHelperService;
    public TestHelperService TestHelperService => Check(_testHelperService);

    [SerializeField]
    internal LoggingService? _loggingService;
    public LoggingService LoggingService => Check(_loggingService);

    [SerializeField]
    internal StudioService? _studioService;
    public StudioService StudioService => Check(_studioService);

    [SerializeField]
    internal UserHandHoverService? _userHandHoverService;
    public UserHandHoverService UserHandHoverService => Check(_userHandHoverService);

    [SerializeField]
    internal IdleReconnectService? _idleReconnectService;
    public IdleReconnectService IdleReconnectService => Check(_idleReconnectService);

    [SerializeField]
    internal DreamscapeService _dreamscapeService = null!;
    public DreamscapeService DreamscapeService => Check(_dreamscapeService);

    [SerializeField]
    internal PrototypeQuest? _prototypeQuest;
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
      _activeTestConfiguration = testConfiguration;
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

    public Coroutine InitializeDisplayablesInScene(Scene scene)
    {
      return StartCoroutine(InitializeDisplayablesRoutine(scene));
    }

    IEnumerator InitializeAll(
      GameMode mode,
      TestConfiguration? testConfiguration,
      IGameViewport? gameViewport
    )
    {
      var targetScenes = new List<Scene> { gameObject.scene };
      yield return LoadAdditionalSceneIfNeeded(targetScenes, mode);

      if (_isLandscape)
      {
        Check(_portraitLayout).Contents.SetActive(false);
        Check(_landscapeLayout).Contents.SetActive(true);
      }
      else
      {
        Check(_portraitLayout).Contents.SetActive(true);
        Check(_landscapeLayout).Contents.SetActive(false);
      }

      Application.targetFrameRate = 60;

      foreach (
        var element in FindObjectsByType<Displayable>(
          FindObjectsInactive.Include,
          FindObjectsSortMode.None
        )
      )
      {
        if (targetScenes.Contains(element.gameObject.scene))
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
        if (targetScenes.Contains(element.gameObject.scene))
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
        if (targetScenes.Contains(element.gameObject.scene))
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

    IEnumerator InitializeDisplayablesRoutine(Scene scene)
    {
      foreach (
        var element in FindObjectsByType<Displayable>(
          FindObjectsInactive.Include,
          FindObjectsSortMode.None
        )
      )
      {
        if (element.gameObject.scene == scene)
        {
          element.Initialize(this, _currentGameMode, _activeTestConfiguration, fromRegistry: true);
        }
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
        if (element.gameObject.scene == scene)
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

    IEnumerator LoadAdditionalSceneIfNeeded(List<Scene> targetScenes, GameMode mode)
    {
      if (mode == GameMode.Battle)
      {
        yield break;
      }

      if (_additionalSceneReference == null || !_additionalSceneReference.RuntimeKeyIsValid())
      {
        yield break;
      }

#if UNITY_EDITOR
      if (SceneAlreadyLoaded(targetScenes))
      {
        yield break;
      }
#endif

      var operation = Addressables.LoadSceneAsync(
        _additionalSceneReference,
        LoadSceneMode.Additive
      );
      if (!operation.IsValid())
      {
        yield break;
      }

      yield return operation;

      if (operation.Status != AsyncOperationStatus.Succeeded)
      {
        Addressables.Release(operation);
        yield break;
      }

      var scene = operation.Result.Scene;
      if (!scene.IsValid())
      {
        Addressables.UnloadSceneAsync(operation);
        yield break;
      }

      DisableEditingHelpers(scene);
      targetScenes.Add(scene);
    }

#if UNITY_EDITOR
    bool SceneAlreadyLoaded(List<Scene> targetScenes)
    {
      var path = AssetDatabase.GUIDToAssetPath(_additionalSceneReference!.AssetGUID);
      if (string.IsNullOrEmpty(path))
      {
        return false;
      }

      for (var i = 0; i < SceneManager.sceneCount; i += 1)
      {
        var scene = SceneManager.GetSceneAt(i);
        if (!scene.isLoaded)
        {
          continue;
        }

        if (scene.path != path)
        {
          continue;
        }

        DisableEditingHelpers(scene);
        targetScenes.Add(scene);
        return true;
      }

      return false;
    }
#endif

    void DisableEditingHelpers(Scene scene)
    {
      var roots = scene.GetRootGameObjects();
      foreach (var root in roots)
      {
        var cameras = root.GetComponentsInChildren<Camera>(true);
        foreach (var camera in cameras)
        {
          if (camera.CompareTag("MainCamera"))
          {
            camera.gameObject.SetActive(false);
          }
        }

        var lights = root.GetComponentsInChildren<Light>(true);
        foreach (var light in lights)
        {
          if (light.type == LightType.Directional)
          {
            light.gameObject.SetActive(false);
          }
        }
      }
    }

    [SerializeField]
    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");

    void TempToggleGameObjectsForMode(GameMode mode)
    {
      // In the future we need to run all this logic after connecting to the
      // rules engine, this is just a temporary solution for prototyping.
      if (mode == GameMode.Battle)
      {
        CinemachineBrain.enabled = false;
        MainCamera.transform.position = BattleLayout.CameraPosition.position;
        MainCamera.transform.rotation = BattleLayout.CameraPosition.rotation;
        _cameraAdjuster.AdjustFieldOfView(BattleLayout.BattleCameraBounds);
      }
      else
      {
        BattleLayout.Contents.SetActive(false);
      }
    }
  }
}
