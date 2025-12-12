// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: Registry
// Generated at: 2025-12-12 13:52:36

#nullable enable

using System.Collections.Generic;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using TMPro;
using Unity.Cinemachine;
using UnityEngine;
using UnityEngine.UI;

namespace Dreamtides.Tests.TestUtils
{
  using Dreamtides.Services;
  using Dreamtides.TestFakes;

  public class GeneratedRegistry
  {
    public Registry Registry { get; private set; } = null!;
    public FakeSoundService FakeSoundService { get; private set; } = null!;
    public FakeActionService FakeActionService { get; private set; } = null!;

    public static GeneratedRegistry Create(
      List<GameObject> createdObjects,
      GeneratedCanvas canvas,
      GeneratedMainCamera mainCamera,
      BattleLayout portraitLayout,
      BattleLayout landscapeLayout,
      DreamscapeLayout? dreamscapeLayout = null
    )
    {
      var result = new GeneratedRegistry();

      var registryGo = new GameObject("Registry");
      createdObjects.Add(registryGo);
      var registryComponent = registryGo.AddComponent<Registry>();
      result.Registry = registryComponent;

      registryComponent._canvas = canvas.Canvas;
      registryComponent._cameraAdjuster = mainCamera.GameCamera;
      registryComponent._portraitLayout = portraitLayout;
      registryComponent._landscapeLayout = landscapeLayout;
      registryComponent._dreamscapeLayout = dreamscapeLayout;

      var mainAudioSource = registryGo.AddComponent<AudioSource>();
      registryComponent._mainAudioSource = mainAudioSource;
      var musicAudioSource = registryGo.AddComponent<AudioSource>();
      registryComponent._musicAudioSource = musicAudioSource;

      var canvasSafeArea = canvas.Canvas.GetComponentInChildren<RectTransform>();
      registryComponent._canvasSafeArea = canvasSafeArea;

      var cardService = registryGo.AddComponent<CardService>();
      registryComponent._cardService = cardService;
      var actionService = registryGo.AddComponent<ActionServiceImpl>();
      registryComponent._actionService = actionService;
      var inputService = registryGo.AddComponent<InputService>();
      registryComponent._inputService = inputService;
      var documentService = registryGo.AddComponent<DocumentService>();
      registryComponent._documentService = documentService;
      var capabilitiesService = registryGo.AddComponent<CapabilitiesService>();
      registryComponent._capabilitiesService = capabilitiesService;
      var soundService = registryGo.AddComponent<SoundServiceImpl>();
      registryComponent._soundService = soundService;
      var cardAnimationService = registryGo.AddComponent<CardAnimationService>();
      registryComponent._cardAnimationService = cardAnimationService;
      var settingsService = registryGo.AddComponent<SettingsService>();
      registryComponent._settingsService = settingsService;
      var assetService = registryGo.AddComponent<AssetService>();
      registryComponent._assetService = assetService;
      var assetPoolService = registryGo.AddComponent<AssetPoolService>();
      registryComponent._assetPoolService = assetPoolService;
      var effectService = registryGo.AddComponent<EffectService>();
      registryComponent._effectService = effectService;
      var environmentService = registryGo.AddComponent<EnvironmentService>();
      registryComponent._environmentService = environmentService;
      var judgmentService = registryGo.AddComponent<JudgmentService>();
      registryComponent._judgmentService = judgmentService;
      var dreamwellActivationService = registryGo.AddComponent<DreamwellActivationService>();
      registryComponent._dreamwellActivationService = dreamwellActivationService;
      var arrowService = registryGo.AddComponent<ArrowService>();
      registryComponent._arrowService = arrowService;
      var cardEffectPreviewService = registryGo.AddComponent<CardEffectPreviewService>();
      registryComponent._cardEffectPreviewService = cardEffectPreviewService;
      var testHelperService = registryGo.AddComponent<TestHelperService>();
      registryComponent._testHelperService = testHelperService;
      var loggingService = registryGo.AddComponent<LoggingServiceImpl>();
      registryComponent._loggingService = loggingService;
      var userHandHoverService = registryGo.AddComponent<UserHandHoverService>();
      registryComponent._userHandHoverService = userHandHoverService;

      return result;
    }
  }
}
