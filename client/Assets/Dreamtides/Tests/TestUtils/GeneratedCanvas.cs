// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: Canvas
// Generated at: 2025-12-10 06:28:55

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
  public class GeneratedCanvas
  {
    public Canvas Canvas { get; private set; } = null!;
    public Dictionary<string, GameObject> Objects { get; } = new Dictionary<string, GameObject>();

    public static GeneratedCanvas Create(List<GameObject> createdObjects)
    {
      var result = new GeneratedCanvas();

      var canvasGo = new GameObject("Canvas");
      createdObjects.Add(canvasGo);
      var canvasGoRect = canvasGo.AddComponent<RectTransform>();
      canvasGoRect.anchorMin = new Vector2(0f, 0f);
      canvasGoRect.anchorMax = new Vector2(0f, 0f);
      canvasGoRect.anchoredPosition = new Vector2(960f, 540f);
      canvasGoRect.sizeDelta = new Vector2(711.1111f, 400f);
      canvasGoRect.localScale = new Vector3(2.7f, 2.7f, 2.7f);
      var canvas = canvasGo.AddComponent<Canvas>();
      var canvasCanvasScaler = canvasGo.AddComponent<CanvasScaler>();
      var canvasGraphicRaycaster = canvasGo.AddComponent<GraphicRaycaster>();
      result.Canvas = canvas;

      var safeAreaGo = new GameObject("SafeArea");
      createdObjects.Add(safeAreaGo);
      safeAreaGo.transform.SetParent(canvasGo.transform, false);
      var safeAreaGoRect = safeAreaGo.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea", safeAreaGo);

      var topLeftGo = new GameObject("TopLeft");
      createdObjects.Add(topLeftGo);
      topLeftGo.transform.SetParent(safeAreaGo.transform, false);
      var topLeftGoRect = topLeftGo.AddComponent<RectTransform>();
      topLeftGoRect.anchorMin = new Vector2(0f, 1f);
      topLeftGoRect.anchorMax = new Vector2(0f, 1f);
      topLeftGoRect.pivot = new Vector2(0f, 1f);
      topLeftGoRect.sizeDelta = new Vector2(50f, 70f);
      result.Objects.Add("SafeArea/TopLeft", topLeftGo);

      var menuButtonGo = new GameObject("MenuButton");
      createdObjects.Add(menuButtonGo);
      menuButtonGo.transform.SetParent(topLeftGo.transform, false);
      var menuButtonGoRect = menuButtonGo.AddComponent<RectTransform>();
      menuButtonGoRect.anchorMin = new Vector2(1f, 1f);
      menuButtonGoRect.anchorMax = new Vector2(1f, 1f);
      menuButtonGoRect.pivot = new Vector2(1f, 1f);
      menuButtonGoRect.anchoredPosition = new Vector2(-15.39998f, -5.3f);
      menuButtonGoRect.sizeDelta = new Vector2(25f, 25f);
      result.Objects.Add("SafeArea/TopLeft/MenuButton", menuButtonGo);

      var textTMPGo = new GameObject("Text (TMP)");
      createdObjects.Add(textTMPGo);
      textTMPGo.transform.SetParent(menuButtonGo.transform, false);
      var textTMPGoRect = textTMPGo.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/TopLeft/MenuButton/Text (TMP)", textTMPGo);

      var devButtonGo = new GameObject("DevButton");
      createdObjects.Add(devButtonGo);
      devButtonGo.transform.SetParent(topLeftGo.transform, false);
      var devButtonGoRect = devButtonGo.AddComponent<RectTransform>();
      devButtonGoRect.anchorMin = new Vector2(0f, 1f);
      devButtonGoRect.anchorMax = new Vector2(0f, 1f);
      devButtonGoRect.pivot = new Vector2(0f, 1f);
      devButtonGoRect.anchoredPosition = new Vector2(8.6f, -35.7f);
      devButtonGoRect.sizeDelta = new Vector2(28.6507f, 12.8774f);
      result.Objects.Add("SafeArea/TopLeft/DevButton", devButtonGo);

      var textTMP1Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP1Go);
      textTMP1Go.transform.SetParent(devButtonGo.transform, false);
      var textTMP1GoRect = textTMP1Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/TopLeft/DevButton/Text (TMP)", textTMP1Go);

      var tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGo = new GameObject(
        "TMP SubMeshUI [fa-solid-900 SDF Material + LiberationSans SDF Atlas]"
      );
      createdObjects.Add(tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGo);
      tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGo.transform.SetParent(
        textTMP1Go.transform,
        false
      );
      var tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGoRect =
        tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGo.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/TopLeft/DevButton/Text (TMP)/TMP SubMeshUI [fa-solid-900 SDF Material + LiberationSans SDF Atlas]",
        tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlasGo
      );

      var tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGo = new GameObject(
        "TMP SubMeshUI [fa-solid-900 SDF Material + font_awsome_all Atlas]"
      );
      createdObjects.Add(tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGo);
      tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGo.transform.SetParent(
        textTMP1Go.transform,
        false
      );
      var tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGoRect =
        tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGo.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/TopLeft/DevButton/Text (TMP)/TMP SubMeshUI [fa-solid-900 SDF Material + font_awsome_all Atlas]",
        tMPSubMeshUIFaSolid900SDFMaterialFontAwsomeAllAtlasGo
      );

      var infoZoomLandscapeLeftGo = new GameObject("InfoZoomLandscapeLeft");
      createdObjects.Add(infoZoomLandscapeLeftGo);
      infoZoomLandscapeLeftGo.transform.SetParent(topLeftGo.transform, false);
      var infoZoomLandscapeLeftGoRect = infoZoomLandscapeLeftGo.AddComponent<RectTransform>();
      infoZoomLandscapeLeftGoRect.anchorMin = new Vector2(0f, 1f);
      infoZoomLandscapeLeftGoRect.anchorMax = new Vector2(0f, 1f);
      infoZoomLandscapeLeftGoRect.pivot = new Vector2(0f, 1f);
      infoZoomLandscapeLeftGoRect.anchoredPosition = new Vector2(20f, -55f);
      result.Objects.Add("SafeArea/TopLeft/InfoZoomLandscapeLeft", infoZoomLandscapeLeftGo);

      var infoZoomPortraitLeftGo = new GameObject("InfoZoomPortraitLeft");
      createdObjects.Add(infoZoomPortraitLeftGo);
      infoZoomPortraitLeftGo.transform.SetParent(topLeftGo.transform, false);
      var infoZoomPortraitLeftGoRect = infoZoomPortraitLeftGo.AddComponent<RectTransform>();
      infoZoomPortraitLeftGoRect.anchorMin = new Vector2(0f, 1f);
      infoZoomPortraitLeftGoRect.anchorMax = new Vector2(0f, 1f);
      infoZoomPortraitLeftGoRect.pivot = new Vector2(0f, 1f);
      infoZoomPortraitLeftGoRect.anchoredPosition = new Vector2(10f, -60f);
      result.Objects.Add("SafeArea/TopLeft/InfoZoomPortraitLeft", infoZoomPortraitLeftGo);

      var siteButtonsGo = new GameObject("SiteButtons");
      createdObjects.Add(siteButtonsGo);
      siteButtonsGo.transform.SetParent(topLeftGo.transform, false);
      var siteButtonsGoRect = siteButtonsGo.AddComponent<RectTransform>();
      siteButtonsGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      siteButtonsGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      siteButtonsGoRect.anchoredPosition = new Vector2(37.71719f, -118.2385f);
      siteButtonsGoRect.sizeDelta = new Vector2(20.22116f, 20.22116f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons", siteButtonsGo);

      var siteButtonGo = new GameObject("SiteButton");
      createdObjects.Add(siteButtonGo);
      siteButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var siteButtonGoRect = siteButtonGo.AddComponent<RectTransform>();
      siteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      siteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      siteButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/SiteButton", siteButtonGo);

      var textTMP2Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP2Go);
      textTMP2Go.transform.SetParent(siteButtonGo.transform, false);
      var textTMP2GoRect = textTMP2Go.AddComponent<RectTransform>();
      textTMP2GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP2GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      textTMP2GoRect.localScale = new Vector3(0.16f, 0.16f, 0.16f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/SiteButton/Text (TMP)", textTMP2Go);

      var shopSiteButtonGo = new GameObject("ShopSiteButton");
      createdObjects.Add(shopSiteButtonGo);
      shopSiteButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var shopSiteButtonGoRect = shopSiteButtonGo.AddComponent<RectTransform>();
      shopSiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      shopSiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      shopSiteButtonGoRect.anchoredPosition = new Vector2(-42.49607f, 53.39653f);
      shopSiteButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      shopSiteButtonGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/ShopSiteButton", shopSiteButtonGo);

      var textTMP3Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP3Go);
      textTMP3Go.transform.SetParent(shopSiteButtonGo.transform, false);
      var textTMP3GoRect = textTMP3Go.AddComponent<RectTransform>();
      textTMP3GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP3GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/ShopSiteButton/Text (TMP)", textTMP3Go);

      var journeySiteButtonGo = new GameObject("JourneySiteButton");
      createdObjects.Add(journeySiteButtonGo);
      journeySiteButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var journeySiteButtonGoRect = journeySiteButtonGo.AddComponent<RectTransform>();
      journeySiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      journeySiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      journeySiteButtonGoRect.anchoredPosition = new Vector2(106.8089f, 0.2685242f);
      journeySiteButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      journeySiteButtonGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/JourneySiteButton", journeySiteButtonGo);

      var textTMP4Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP4Go);
      textTMP4Go.transform.SetParent(journeySiteButtonGo.transform, false);
      var textTMP4GoRect = textTMP4Go.AddComponent<RectTransform>();
      textTMP4GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP4GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/JourneySiteButton/Text (TMP)", textTMP4Go);

      var essenceSiteButtonGo = new GameObject("EssenceSiteButton");
      createdObjects.Add(essenceSiteButtonGo);
      essenceSiteButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var essenceSiteButtonGoRect = essenceSiteButtonGo.AddComponent<RectTransform>();
      essenceSiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      essenceSiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      essenceSiteButtonGoRect.anchoredPosition = new Vector2(26.38232f, -51.8168f);
      essenceSiteButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      essenceSiteButtonGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/EssenceSiteButton", essenceSiteButtonGo);

      var textTMP5Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP5Go);
      textTMP5Go.transform.SetParent(essenceSiteButtonGo.transform, false);
      var textTMP5GoRect = textTMP5Go.AddComponent<RectTransform>();
      textTMP5GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP5GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/EssenceSiteButton/Text (TMP)", textTMP5Go);

      var draft2SiteButtonGo = new GameObject("Draft2SiteButton");
      createdObjects.Add(draft2SiteButtonGo);
      draft2SiteButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var draft2SiteButtonGoRect = draft2SiteButtonGo.AddComponent<RectTransform>();
      draft2SiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      draft2SiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      draft2SiteButtonGoRect.anchoredPosition = new Vector2(-18.48341f, -80.56876f);
      draft2SiteButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      draft2SiteButtonGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/Draft2SiteButton", draft2SiteButtonGo);

      var textTMP6Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP6Go);
      textTMP6Go.transform.SetParent(draft2SiteButtonGo.transform, false);
      var textTMP6GoRect = textTMP6Go.AddComponent<RectTransform>();
      textTMP6GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP6GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/Draft2SiteButton/Text (TMP)", textTMP6Go);

      var battleButtonGo = new GameObject("BattleButton");
      createdObjects.Add(battleButtonGo);
      battleButtonGo.transform.SetParent(siteButtonsGo.transform, false);
      var battleButtonGoRect = battleButtonGo.AddComponent<RectTransform>();
      battleButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      battleButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      battleButtonGoRect.anchoredPosition = new Vector2(80.25277f, 65.71873f);
      battleButtonGoRect.sizeDelta = new Vector2(128f, 128f);
      battleButtonGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/BattleButton", battleButtonGo);

      var textTMP7Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP7Go);
      textTMP7Go.transform.SetParent(battleButtonGo.transform, false);
      var textTMP7GoRect = textTMP7Go.AddComponent<RectTransform>();
      textTMP7GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textTMP7GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      result.Objects.Add("SafeArea/TopLeft/SiteButtons/BattleButton/Text (TMP)", textTMP7Go);

      var topCenterGo = new GameObject("TopCenter");
      createdObjects.Add(topCenterGo);
      topCenterGo.transform.SetParent(safeAreaGo.transform, false);
      var topCenterGoRect = topCenterGo.AddComponent<RectTransform>();
      topCenterGoRect.anchorMin = new Vector2(0.5f, 1f);
      topCenterGoRect.anchorMax = new Vector2(0.5f, 1f);
      topCenterGoRect.pivot = new Vector2(0.5f, 1f);
      topCenterGoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add("SafeArea/TopCenter", topCenterGo);

      var topRightGo = new GameObject("TopRight");
      createdObjects.Add(topRightGo);
      topRightGo.transform.SetParent(safeAreaGo.transform, false);
      var topRightGoRect = topRightGo.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/TopRight", topRightGo);

      var undoButtonGo = new GameObject("UndoButton");
      createdObjects.Add(undoButtonGo);
      undoButtonGo.transform.SetParent(topRightGo.transform, false);
      var undoButtonGoRect = undoButtonGo.AddComponent<RectTransform>();
      undoButtonGoRect.anchorMin = new Vector2(1f, 1f);
      undoButtonGoRect.anchorMax = new Vector2(1f, 1f);
      undoButtonGoRect.pivot = new Vector2(1f, 1f);
      undoButtonGoRect.anchoredPosition = new Vector2(-15.3999f, -5.300049f);
      undoButtonGoRect.sizeDelta = new Vector2(25f, 25f);
      result.Objects.Add("SafeArea/TopRight/UndoButton", undoButtonGo);

      var textTMP8Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP8Go);
      textTMP8Go.transform.SetParent(undoButtonGo.transform, false);
      var textTMP8GoRect = textTMP8Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/TopRight/UndoButton/Text (TMP)", textTMP8Go);

      var bugButtonGo = new GameObject("BugButton");
      createdObjects.Add(bugButtonGo);
      bugButtonGo.transform.SetParent(topRightGo.transform, false);
      var bugButtonGoRect = bugButtonGo.AddComponent<RectTransform>();
      bugButtonGoRect.anchorMin = new Vector2(1f, 1f);
      bugButtonGoRect.anchorMax = new Vector2(1f, 1f);
      bugButtonGoRect.pivot = new Vector2(1f, 1f);
      bugButtonGoRect.anchoredPosition = new Vector2(-13f, -35f);
      bugButtonGoRect.sizeDelta = new Vector2(28.6507f, 12.8774f);
      result.Objects.Add("SafeArea/TopRight/BugButton", bugButtonGo);

      var textTMP9Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP9Go);
      textTMP9Go.transform.SetParent(bugButtonGo.transform, false);
      var textTMP9GoRect = textTMP9Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/TopRight/BugButton/Text (TMP)", textTMP9Go);

      var tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1Go = new GameObject(
        "TMP SubMeshUI [fa-solid-900 SDF Material + LiberationSans SDF Atlas]"
      );
      createdObjects.Add(tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1Go);
      tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1Go.transform.SetParent(
        textTMP9Go.transform,
        false
      );
      var tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1GoRect =
        tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/TopRight/BugButton/Text (TMP)/TMP SubMeshUI [fa-solid-900 SDF Material + LiberationSans SDF Atlas]",
        tMPSubMeshUIFaSolid900SDFMaterialLiberationSansSDFAtlas1Go
      );

      var gameModifierDisplayPositionGo = new GameObject("GameModifierDisplayPosition");
      createdObjects.Add(gameModifierDisplayPositionGo);
      gameModifierDisplayPositionGo.transform.SetParent(topRightGo.transform, false);
      var gameModifierDisplayPositionGoRect =
        gameModifierDisplayPositionGo.AddComponent<RectTransform>();
      gameModifierDisplayPositionGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      gameModifierDisplayPositionGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      gameModifierDisplayPositionGoRect.anchoredPosition = new Vector2(-2.5f, -30f);
      result.Objects.Add(
        "SafeArea/TopRight/GameModifierDisplayPosition",
        gameModifierDisplayPositionGo
      );

      var gameModifierDisplayLandscapeGo = new GameObject("GameModifierDisplayLandscape");
      createdObjects.Add(gameModifierDisplayLandscapeGo);
      gameModifierDisplayLandscapeGo.transform.SetParent(
        gameModifierDisplayPositionGo.transform,
        false
      );
      var gameModifierDisplayLandscapeGoRect =
        gameModifierDisplayLandscapeGo.AddComponent<RectTransform>();
      gameModifierDisplayLandscapeGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      gameModifierDisplayLandscapeGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      gameModifierDisplayLandscapeGoRect.anchoredPosition = new Vector2(0f, -5f);
      gameModifierDisplayLandscapeGoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add(
        "SafeArea/TopRight/GameModifierDisplayPosition/GameModifierDisplayLandscape",
        gameModifierDisplayLandscapeGo
      );

      var infoZoomLandscapeRightGo = new GameObject("InfoZoomLandscapeRight");
      createdObjects.Add(infoZoomLandscapeRightGo);
      infoZoomLandscapeRightGo.transform.SetParent(topRightGo.transform, false);
      var infoZoomLandscapeRightGoRect = infoZoomLandscapeRightGo.AddComponent<RectTransform>();
      infoZoomLandscapeRightGoRect.anchorMin = new Vector2(1f, 1f);
      infoZoomLandscapeRightGoRect.anchorMax = new Vector2(1f, 1f);
      infoZoomLandscapeRightGoRect.pivot = new Vector2(1f, 1f);
      infoZoomLandscapeRightGoRect.anchoredPosition = new Vector2(-20f, -55f);
      result.Objects.Add("SafeArea/TopRight/InfoZoomLandscapeRight", infoZoomLandscapeRightGo);

      var infoZoomPortraitRightGo = new GameObject("InfoZoomPortraitRight");
      createdObjects.Add(infoZoomPortraitRightGo);
      infoZoomPortraitRightGo.transform.SetParent(topRightGo.transform, false);
      var infoZoomPortraitRightGoRect = infoZoomPortraitRightGo.AddComponent<RectTransform>();
      infoZoomPortraitRightGoRect.anchorMin = new Vector2(1f, 1f);
      infoZoomPortraitRightGoRect.anchorMax = new Vector2(1f, 1f);
      infoZoomPortraitRightGoRect.pivot = new Vector2(1f, 1f);
      infoZoomPortraitRightGoRect.anchoredPosition = new Vector2(-10f, -60f);
      result.Objects.Add("SafeArea/TopRight/InfoZoomPortraitRight", infoZoomPortraitRightGo);

      var speechBubbleGo = new GameObject("SpeechBubble");
      createdObjects.Add(speechBubbleGo);
      speechBubbleGo.transform.SetParent(safeAreaGo.transform, false);
      var speechBubbleGoRect = speechBubbleGo.AddComponent<RectTransform>();
      speechBubbleGoRect.anchorMin = new Vector2(1f, 1f);
      speechBubbleGoRect.anchorMax = new Vector2(1f, 1f);
      speechBubbleGoRect.pivot = new Vector2(1f, 1f);
      speechBubbleGoRect.anchoredPosition = new Vector2(-24.5f, -60.69995f);
      result.Objects.Add("SafeArea/SpeechBubble", speechBubbleGo);

      var backgroundGo = new GameObject("Background");
      createdObjects.Add(backgroundGo);
      backgroundGo.transform.SetParent(speechBubbleGo.transform, false);
      var backgroundGoRect = backgroundGo.AddComponent<RectTransform>();
      backgroundGoRect.anchorMin = new Vector2(1f, 1f);
      backgroundGoRect.anchorMax = new Vector2(1f, 1f);
      backgroundGoRect.pivot = new Vector2(1f, 1f);
      backgroundGoRect.anchoredPosition = new Vector2(-103.7f, -16f);
      backgroundGoRect.sizeDelta = new Vector2(36f, 46f);
      backgroundGoRect.localScale = new Vector3(0.25f, 0.25f, 0.25f);
      result.Objects.Add("SafeArea/SpeechBubble/Background", backgroundGo);

      var speechTextGo = new GameObject("SpeechText");
      createdObjects.Add(speechTextGo);
      speechTextGo.transform.SetParent(backgroundGo.transform, false);
      var speechTextGoRect = speechTextGo.AddComponent<RectTransform>();
      speechTextGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      speechTextGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      speechTextGoRect.anchoredPosition = new Vector2(11.59807f, 10.22645f);
      speechTextGoRect.sizeDelta = new Vector2(90.60387f, 0.46905f);
      result.Objects.Add("SafeArea/SpeechBubble/Background/SpeechText", speechTextGo);

      var bottomLeftGo = new GameObject("BottomLeft");
      createdObjects.Add(bottomLeftGo);
      bottomLeftGo.transform.SetParent(safeAreaGo.transform, false);
      var bottomLeftGoRect = bottomLeftGo.AddComponent<RectTransform>();
      bottomLeftGoRect.anchorMin = new Vector2(0f, 0f);
      bottomLeftGoRect.anchorMax = new Vector2(0f, 0f);
      bottomLeftGoRect.pivot = new Vector2(0f, 0f);
      result.Objects.Add("SafeArea/BottomLeft", bottomLeftGo);

      var essenceDisplayGo = new GameObject("EssenceDisplay");
      createdObjects.Add(essenceDisplayGo);
      essenceDisplayGo.transform.SetParent(bottomLeftGo.transform, false);
      var essenceDisplayGoRect = essenceDisplayGo.AddComponent<RectTransform>();
      essenceDisplayGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      essenceDisplayGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      essenceDisplayGoRect.anchoredPosition = new Vector2(28.4f, 14.8f);
      essenceDisplayGoRect.sizeDelta = new Vector2(500f, 128f);
      essenceDisplayGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/BottomLeft/EssenceDisplay", essenceDisplayGo);

      var essenceTextGo = new GameObject("EssenceText");
      createdObjects.Add(essenceTextGo);
      essenceTextGo.transform.SetParent(essenceDisplayGo.transform, false);
      var essenceTextGoRect = essenceTextGo.AddComponent<RectTransform>();
      essenceTextGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      essenceTextGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      essenceTextGoRect.anchoredPosition = new Vector2(-52.3f, 0f);
      essenceTextGoRect.sizeDelta = new Vector2(275.6726f, 100f);
      result.Objects.Add("SafeArea/BottomLeft/EssenceDisplay/EssenceText", essenceTextGo);

      var essenceSymbolGo = new GameObject("EssenceSymbol");
      createdObjects.Add(essenceSymbolGo);
      essenceSymbolGo.transform.SetParent(essenceDisplayGo.transform, false);
      var essenceSymbolGoRect = essenceSymbolGo.AddComponent<RectTransform>();
      essenceSymbolGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      essenceSymbolGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      essenceSymbolGoRect.anchoredPosition = new Vector2(196.2942f, 0f);
      essenceSymbolGoRect.sizeDelta = new Vector2(209.8115f, 100f);
      result.Objects.Add("SafeArea/BottomLeft/EssenceDisplay/EssenceSymbol", essenceSymbolGo);

      var spaceCameraFarGo = new GameObject("SpaceCameraFar");
      createdObjects.Add(spaceCameraFarGo);
      spaceCameraFarGo.transform.SetParent(bottomLeftGo.transform, false);
      var spaceCameraFarGoRect = spaceCameraFarGo.AddComponent<RectTransform>();
      spaceCameraFarGoRect.anchorMin = new Vector2(0f, 1f);
      spaceCameraFarGoRect.anchorMax = new Vector2(0f, 1f);
      spaceCameraFarGoRect.pivot = new Vector2(0f, 1f);
      spaceCameraFarGoRect.anchoredPosition = new Vector2(10.3f, 42.4f);
      spaceCameraFarGoRect.sizeDelta = new Vector2(88f, 88f);
      spaceCameraFarGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/BottomLeft/SpaceCameraFar", spaceCameraFarGo);

      var textTMP10Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP10Go);
      textTMP10Go.transform.SetParent(spaceCameraFarGo.transform, false);
      var textTMP10GoRect = textTMP10Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/BottomLeft/SpaceCameraFar/Text (TMP)", textTMP10Go);

      var spaceCameraNearGo = new GameObject("SpaceCameraNear");
      createdObjects.Add(spaceCameraNearGo);
      spaceCameraNearGo.transform.SetParent(bottomLeftGo.transform, false);
      var spaceCameraNearGoRect = spaceCameraNearGo.AddComponent<RectTransform>();
      spaceCameraNearGoRect.anchorMin = new Vector2(0f, 1f);
      spaceCameraNearGoRect.anchorMax = new Vector2(0f, 1f);
      spaceCameraNearGoRect.pivot = new Vector2(0f, 1f);
      spaceCameraNearGoRect.anchoredPosition = new Vector2(125.5f, 366f);
      spaceCameraNearGoRect.sizeDelta = new Vector2(88f, 88f);
      spaceCameraNearGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/BottomLeft/SpaceCameraNear", spaceCameraNearGo);

      var textTMP11Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP11Go);
      textTMP11Go.transform.SetParent(spaceCameraNearGo.transform, false);
      var textTMP11GoRect = textTMP11Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/BottomLeft/SpaceCameraNear/Text (TMP)", textTMP11Go);

      var mapCameraGo = new GameObject("MapCamera");
      createdObjects.Add(mapCameraGo);
      mapCameraGo.transform.SetParent(bottomLeftGo.transform, false);
      var mapCameraGoRect = mapCameraGo.AddComponent<RectTransform>();
      mapCameraGoRect.anchorMin = new Vector2(0f, 1f);
      mapCameraGoRect.anchorMax = new Vector2(0f, 1f);
      mapCameraGoRect.pivot = new Vector2(0f, 1f);
      mapCameraGoRect.anchoredPosition = new Vector2(41.3f, 363.7f);
      mapCameraGoRect.sizeDelta = new Vector2(88f, 88f);
      mapCameraGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      result.Objects.Add("SafeArea/BottomLeft/MapCamera", mapCameraGo);

      var textTMP12Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP12Go);
      textTMP12Go.transform.SetParent(mapCameraGo.transform, false);
      var textTMP12GoRect = textTMP12Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/BottomLeft/MapCamera/Text (TMP)", textTMP12Go);

      var bottomRightGo = new GameObject("BottomRight");
      createdObjects.Add(bottomRightGo);
      bottomRightGo.transform.SetParent(safeAreaGo.transform, false);
      var bottomRightGoRect = bottomRightGo.AddComponent<RectTransform>();
      bottomRightGoRect.anchorMin = new Vector2(1f, 0f);
      bottomRightGoRect.anchorMax = new Vector2(1f, 0f);
      bottomRightGoRect.pivot = new Vector2(1f, 0f);
      bottomRightGoRect.sizeDelta = new Vector2(50f, 70f);
      result.Objects.Add("SafeArea/BottomRight", bottomRightGo);

      var questDeckPositionGo = new GameObject("QuestDeckPosition");
      createdObjects.Add(questDeckPositionGo);
      questDeckPositionGo.transform.SetParent(bottomRightGo.transform, false);
      var questDeckPositionGoRect = questDeckPositionGo.AddComponent<RectTransform>();
      questDeckPositionGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      questDeckPositionGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      questDeckPositionGoRect.anchoredPosition = new Vector2(0f, -5f);
      result.Objects.Add("SafeArea/BottomRight/QuestDeckPosition", questDeckPositionGo);

      var bottomCenterGo = new GameObject("BottomCenter");
      createdObjects.Add(bottomCenterGo);
      bottomCenterGo.transform.SetParent(safeAreaGo.transform, false);
      var bottomCenterGoRect = bottomCenterGo.AddComponent<RectTransform>();
      bottomCenterGoRect.anchorMin = new Vector2(0.5f, 0f);
      bottomCenterGoRect.anchorMax = new Vector2(0.5f, 0f);
      bottomCenterGoRect.pivot = new Vector2(0.5f, 0f);
      bottomCenterGoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add("SafeArea/BottomCenter", bottomCenterGo);

      var cardBrowserLeftGo = new GameObject("CardBrowserLeft");
      createdObjects.Add(cardBrowserLeftGo);
      cardBrowserLeftGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserLeftGoRect = cardBrowserLeftGo.AddComponent<RectTransform>();
      cardBrowserLeftGoRect.anchorMin = new Vector2(0f, 0.6f);
      cardBrowserLeftGoRect.anchorMax = new Vector2(0f, 0.6f);
      cardBrowserLeftGoRect.pivot = new Vector2(0f, 0.6f);
      result.Objects.Add("SafeArea/CardBrowserLeft", cardBrowserLeftGo);

      var cardBrowserLeftLandscapeGo = new GameObject("CardBrowserLeftLandscape");
      createdObjects.Add(cardBrowserLeftLandscapeGo);
      cardBrowserLeftLandscapeGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserLeftLandscapeGoRect = cardBrowserLeftLandscapeGo.AddComponent<RectTransform>();
      cardBrowserLeftLandscapeGoRect.anchorMin = new Vector2(0f, 0.6f);
      cardBrowserLeftLandscapeGoRect.anchorMax = new Vector2(0f, 0.6f);
      cardBrowserLeftLandscapeGoRect.pivot = new Vector2(0f, 0.6f);
      result.Objects.Add("SafeArea/CardBrowserLeftLandscape", cardBrowserLeftLandscapeGo);

      var cardBrowserRightGo = new GameObject("CardBrowserRight");
      createdObjects.Add(cardBrowserRightGo);
      cardBrowserRightGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserRightGoRect = cardBrowserRightGo.AddComponent<RectTransform>();
      cardBrowserRightGoRect.anchorMin = new Vector2(1f, 0.6f);
      cardBrowserRightGoRect.anchorMax = new Vector2(1f, 0.6f);
      cardBrowserRightGoRect.pivot = new Vector2(1f, 0.6f);
      result.Objects.Add("SafeArea/CardBrowserRight", cardBrowserRightGo);

      var closeBrowserButtonGo = new GameObject("CloseBrowserButton");
      createdObjects.Add(closeBrowserButtonGo);
      closeBrowserButtonGo.transform.SetParent(cardBrowserRightGo.transform, false);
      var closeBrowserButtonGoRect = closeBrowserButtonGo.AddComponent<RectTransform>();
      closeBrowserButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      closeBrowserButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      closeBrowserButtonGoRect.anchoredPosition = new Vector2(-15f, 20f);
      closeBrowserButtonGoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add("SafeArea/CardBrowserRight/CloseBrowserButton", closeBrowserButtonGo);

      var textTMP13Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP13Go);
      textTMP13Go.transform.SetParent(closeBrowserButtonGo.transform, false);
      var textTMP13GoRect = textTMP13Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/CardBrowserRight/CloseBrowserButton/Text (TMP)", textTMP13Go);

      var closeSiteButtonGo = new GameObject("CloseSiteButton");
      createdObjects.Add(closeSiteButtonGo);
      closeSiteButtonGo.transform.SetParent(safeAreaGo.transform, false);
      var closeSiteButtonGoRect = closeSiteButtonGo.AddComponent<RectTransform>();
      closeSiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      closeSiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      closeSiteButtonGoRect.anchoredPosition = new Vector2(342.2555f, 68.90001f);
      closeSiteButtonGoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add("SafeArea/CloseSiteButton", closeSiteButtonGo);

      var textTMP14Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP14Go);
      textTMP14Go.transform.SetParent(closeSiteButtonGo.transform, false);
      var textTMP14GoRect = textTMP14Go.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/CloseSiteButton/Text (TMP)", textTMP14Go);

      var cardBrowserRightLandscapeGo = new GameObject("CardBrowserRightLandscape");
      createdObjects.Add(cardBrowserRightLandscapeGo);
      cardBrowserRightLandscapeGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserRightLandscapeGoRect =
        cardBrowserRightLandscapeGo.AddComponent<RectTransform>();
      cardBrowserRightLandscapeGoRect.anchorMin = new Vector2(1f, 0.9f);
      cardBrowserRightLandscapeGoRect.anchorMax = new Vector2(1f, 0.9f);
      cardBrowserRightLandscapeGoRect.pivot = new Vector2(1f, 0.9f);
      cardBrowserRightLandscapeGoRect.anchoredPosition = new Vector2(0f, -120f);
      result.Objects.Add("SafeArea/CardBrowserRightLandscape", cardBrowserRightLandscapeGo);

      var closeBrowserButtonLandscapeGo = new GameObject("CloseBrowserButtonLandscape");
      createdObjects.Add(closeBrowserButtonLandscapeGo);
      closeBrowserButtonLandscapeGo.transform.SetParent(
        cardBrowserRightLandscapeGo.transform,
        false
      );
      var closeBrowserButtonLandscapeGoRect =
        closeBrowserButtonLandscapeGo.AddComponent<RectTransform>();
      closeBrowserButtonLandscapeGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      closeBrowserButtonLandscapeGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      closeBrowserButtonLandscapeGoRect.anchoredPosition = new Vector2(-75f, 132f);
      closeBrowserButtonLandscapeGoRect.sizeDelta = new Vector2(40f, 40f);
      result.Objects.Add(
        "SafeArea/CardBrowserRightLandscape/CloseBrowserButtonLandscape",
        closeBrowserButtonLandscapeGo
      );

      var textTMP15Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP15Go);
      textTMP15Go.transform.SetParent(closeBrowserButtonLandscapeGo.transform, false);
      var textTMP15GoRect = textTMP15Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/CardBrowserRightLandscape/CloseBrowserButtonLandscape/Text (TMP)",
        textTMP15Go
      );

      var cardBrowserScrollbarPortraitGo = new GameObject("CardBrowserScrollbarPortrait");
      createdObjects.Add(cardBrowserScrollbarPortraitGo);
      cardBrowserScrollbarPortraitGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserScrollbarPortraitGoRect =
        cardBrowserScrollbarPortraitGo.AddComponent<RectTransform>();
      cardBrowserScrollbarPortraitGoRect.anchorMin = new Vector2(0f, 0.5f);
      cardBrowserScrollbarPortraitGoRect.anchorMax = new Vector2(1f, 0.5f);
      cardBrowserScrollbarPortraitGoRect.anchoredPosition = new Vector2(0f, -30f);
      cardBrowserScrollbarPortraitGoRect.sizeDelta = new Vector2(-24f, 20f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarPortrait", cardBrowserScrollbarPortraitGo);

      var barGo = new GameObject("Bar");
      createdObjects.Add(barGo);
      barGo.transform.SetParent(cardBrowserScrollbarPortraitGo.transform, false);
      var barGoRect = barGo.AddComponent<RectTransform>();
      barGoRect.anchorMin = new Vector2(0f, 0.5f);
      barGoRect.anchorMax = new Vector2(1f, 0.5f);
      barGoRect.sizeDelta = new Vector2(0f, 4f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarPortrait/Bar", barGo);

      var barHitSlopGo = new GameObject("BarHitSlop");
      createdObjects.Add(barHitSlopGo);
      barHitSlopGo.transform.SetParent(cardBrowserScrollbarPortraitGo.transform, false);
      var barHitSlopGoRect = barHitSlopGo.AddComponent<RectTransform>();
      barHitSlopGoRect.anchorMin = new Vector2(0f, 0.5f);
      barHitSlopGoRect.anchorMax = new Vector2(1f, 0.5f);
      barHitSlopGoRect.sizeDelta = new Vector2(0f, 24f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarPortrait/BarHitSlop", barHitSlopGo);

      var slidingAreaGo = new GameObject("Sliding Area");
      createdObjects.Add(slidingAreaGo);
      slidingAreaGo.transform.SetParent(cardBrowserScrollbarPortraitGo.transform, false);
      var slidingAreaGoRect = slidingAreaGo.AddComponent<RectTransform>();
      slidingAreaGoRect.sizeDelta = new Vector2(-20f, -20f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarPortrait/Sliding Area", slidingAreaGo);

      var handleGo = new GameObject("Handle");
      createdObjects.Add(handleGo);
      handleGo.transform.SetParent(slidingAreaGo.transform, false);
      var handleGoRect = handleGo.AddComponent<RectTransform>();
      handleGoRect.anchorMin = new Vector2(0f, 0f);
      handleGoRect.anchorMax = new Vector2(0.2f, 1f);
      handleGoRect.sizeDelta = new Vector2(20f, 12f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarPortrait/Sliding Area/Handle", handleGo);

      var arrowsGo = new GameObject("Arrows");
      createdObjects.Add(arrowsGo);
      arrowsGo.transform.SetParent(handleGo.transform, false);
      var arrowsGoRect = arrowsGo.AddComponent<RectTransform>();
      arrowsGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      arrowsGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      arrowsGoRect.sizeDelta = new Vector2(16f, 16f);
      result.Objects.Add(
        "SafeArea/CardBrowserScrollbarPortrait/Sliding Area/Handle/Arrows",
        arrowsGo
      );

      var cardBrowserScrollbarLandscapeGo = new GameObject("CardBrowserScrollbarLandscape");
      createdObjects.Add(cardBrowserScrollbarLandscapeGo);
      cardBrowserScrollbarLandscapeGo.transform.SetParent(safeAreaGo.transform, false);
      var cardBrowserScrollbarLandscapeGoRect =
        cardBrowserScrollbarLandscapeGo.AddComponent<RectTransform>();
      cardBrowserScrollbarLandscapeGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      cardBrowserScrollbarLandscapeGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      cardBrowserScrollbarLandscapeGoRect.anchoredPosition = new Vector2(0f, -95f);
      cardBrowserScrollbarLandscapeGoRect.sizeDelta = new Vector2(300f, 20f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarLandscape", cardBrowserScrollbarLandscapeGo);

      var bar1Go = new GameObject("Bar");
      createdObjects.Add(bar1Go);
      bar1Go.transform.SetParent(cardBrowserScrollbarLandscapeGo.transform, false);
      var bar1GoRect = bar1Go.AddComponent<RectTransform>();
      bar1GoRect.anchorMin = new Vector2(0f, 0.5f);
      bar1GoRect.anchorMax = new Vector2(1f, 0.5f);
      bar1GoRect.sizeDelta = new Vector2(0f, 4f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarLandscape/Bar", bar1Go);

      var barHitSlop1Go = new GameObject("BarHitSlop");
      createdObjects.Add(barHitSlop1Go);
      barHitSlop1Go.transform.SetParent(cardBrowserScrollbarLandscapeGo.transform, false);
      var barHitSlop1GoRect = barHitSlop1Go.AddComponent<RectTransform>();
      barHitSlop1GoRect.anchorMin = new Vector2(0f, 0.5f);
      barHitSlop1GoRect.anchorMax = new Vector2(1f, 0.5f);
      barHitSlop1GoRect.sizeDelta = new Vector2(0f, 24f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarLandscape/BarHitSlop", barHitSlop1Go);

      var slidingArea1Go = new GameObject("Sliding Area");
      createdObjects.Add(slidingArea1Go);
      slidingArea1Go.transform.SetParent(cardBrowserScrollbarLandscapeGo.transform, false);
      var slidingArea1GoRect = slidingArea1Go.AddComponent<RectTransform>();
      slidingArea1GoRect.sizeDelta = new Vector2(-20f, -20f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarLandscape/Sliding Area", slidingArea1Go);

      var handle1Go = new GameObject("Handle");
      createdObjects.Add(handle1Go);
      handle1Go.transform.SetParent(slidingArea1Go.transform, false);
      var handle1GoRect = handle1Go.AddComponent<RectTransform>();
      handle1GoRect.anchorMin = new Vector2(0f, 0f);
      handle1GoRect.anchorMax = new Vector2(0.2f, 1f);
      handle1GoRect.sizeDelta = new Vector2(20f, 12f);
      result.Objects.Add("SafeArea/CardBrowserScrollbarLandscape/Sliding Area/Handle", handle1Go);

      var arrows1Go = new GameObject("Arrows");
      createdObjects.Add(arrows1Go);
      arrows1Go.transform.SetParent(handle1Go.transform, false);
      var arrows1GoRect = arrows1Go.AddComponent<RectTransform>();
      arrows1GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      arrows1GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      arrows1GoRect.sizeDelta = new Vector2(16f, 16f);
      result.Objects.Add(
        "SafeArea/CardBrowserScrollbarLandscape/Sliding Area/Handle/Arrows",
        arrows1Go
      );

      var userHandScrollbarGo = new GameObject("UserHandScrollbar");
      createdObjects.Add(userHandScrollbarGo);
      userHandScrollbarGo.transform.SetParent(safeAreaGo.transform, false);
      var userHandScrollbarGoRect = userHandScrollbarGo.AddComponent<RectTransform>();
      userHandScrollbarGoRect.anchorMin = new Vector2(0.5f, 0f);
      userHandScrollbarGoRect.anchorMax = new Vector2(0.5f, 0f);
      userHandScrollbarGoRect.pivot = new Vector2(0.5f, 0f);
      userHandScrollbarGoRect.anchoredPosition = new Vector2(0f, 85f);
      userHandScrollbarGoRect.sizeDelta = new Vector2(160f, 20f);
      result.Objects.Add("SafeArea/UserHandScrollbar", userHandScrollbarGo);

      var bar2Go = new GameObject("Bar");
      createdObjects.Add(bar2Go);
      bar2Go.transform.SetParent(userHandScrollbarGo.transform, false);
      var bar2GoRect = bar2Go.AddComponent<RectTransform>();
      bar2GoRect.anchorMin = new Vector2(0f, 0.5f);
      bar2GoRect.anchorMax = new Vector2(1f, 0.5f);
      bar2GoRect.sizeDelta = new Vector2(0f, 4f);
      result.Objects.Add("SafeArea/UserHandScrollbar/Bar", bar2Go);

      var barHitSlop2Go = new GameObject("BarHitSlop");
      createdObjects.Add(barHitSlop2Go);
      barHitSlop2Go.transform.SetParent(userHandScrollbarGo.transform, false);
      var barHitSlop2GoRect = barHitSlop2Go.AddComponent<RectTransform>();
      barHitSlop2GoRect.anchorMin = new Vector2(0f, 0.5f);
      barHitSlop2GoRect.anchorMax = new Vector2(1f, 0.5f);
      barHitSlop2GoRect.sizeDelta = new Vector2(0f, 24f);
      result.Objects.Add("SafeArea/UserHandScrollbar/BarHitSlop", barHitSlop2Go);

      var slidingArea2Go = new GameObject("Sliding Area");
      createdObjects.Add(slidingArea2Go);
      slidingArea2Go.transform.SetParent(userHandScrollbarGo.transform, false);
      var slidingArea2GoRect = slidingArea2Go.AddComponent<RectTransform>();
      slidingArea2GoRect.sizeDelta = new Vector2(-20f, -20f);
      result.Objects.Add("SafeArea/UserHandScrollbar/Sliding Area", slidingArea2Go);

      var handle2Go = new GameObject("Handle");
      createdObjects.Add(handle2Go);
      handle2Go.transform.SetParent(slidingArea2Go.transform, false);
      var handle2GoRect = handle2Go.AddComponent<RectTransform>();
      handle2GoRect.anchorMin = new Vector2(0f, 0f);
      handle2GoRect.anchorMax = new Vector2(0.2f, 1f);
      handle2GoRect.sizeDelta = new Vector2(20f, 12f);
      result.Objects.Add("SafeArea/UserHandScrollbar/Sliding Area/Handle", handle2Go);

      var arrows2Go = new GameObject("Arrows");
      createdObjects.Add(arrows2Go);
      arrows2Go.transform.SetParent(handle2Go.transform, false);
      var arrows2GoRect = arrows2Go.AddComponent<RectTransform>();
      arrows2GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      arrows2GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      arrows2GoRect.sizeDelta = new Vector2(16f, 16f);
      result.Objects.Add("SafeArea/UserHandScrollbar/Sliding Area/Handle/Arrows", arrows2Go);

      var actionButtonsPortraitGo = new GameObject("ActionButtonsPortrait");
      createdObjects.Add(actionButtonsPortraitGo);
      actionButtonsPortraitGo.transform.SetParent(safeAreaGo.transform, false);
      var actionButtonsPortraitGoRect = actionButtonsPortraitGo.AddComponent<RectTransform>();
      actionButtonsPortraitGoRect.anchorMin = new Vector2(0f, 0f);
      actionButtonsPortraitGoRect.anchorMax = new Vector2(0f, 0f);
      actionButtonsPortraitGoRect.pivot = new Vector2(0f, 0f);
      actionButtonsPortraitGoRect.anchoredPosition = new Vector2(35f, 122f);
      result.Objects.Add("SafeArea/ActionButtonsPortrait", actionButtonsPortraitGo);

      var actionButtonsLandscapeGo = new GameObject("ActionButtonsLandscape");
      createdObjects.Add(actionButtonsLandscapeGo);
      actionButtonsLandscapeGo.transform.SetParent(safeAreaGo.transform, false);
      var actionButtonsLandscapeGoRect = actionButtonsLandscapeGo.AddComponent<RectTransform>();
      actionButtonsLandscapeGoRect.anchorMin = new Vector2(1f, 0f);
      actionButtonsLandscapeGoRect.anchorMax = new Vector2(1f, 0f);
      actionButtonsLandscapeGoRect.pivot = new Vector2(1f, 0f);
      actionButtonsLandscapeGoRect.anchoredPosition = new Vector2(-50f, 100f);
      result.Objects.Add("SafeArea/ActionButtonsLandscape", actionButtonsLandscapeGo);

      var handLeftGo = new GameObject("HandLeft");
      createdObjects.Add(handLeftGo);
      handLeftGo.transform.SetParent(safeAreaGo.transform, false);
      var handLeftGoRect = handLeftGo.AddComponent<RectTransform>();
      handLeftGoRect.anchorMin = new Vector2(0f, 0f);
      handLeftGoRect.anchorMax = new Vector2(0f, 0f);
      handLeftGoRect.pivot = new Vector2(0f, 0f);
      handLeftGoRect.anchoredPosition = new Vector2(40f, 45f);
      result.Objects.Add("SafeArea/HandLeft", handLeftGo);

      var handLeftRow2Go = new GameObject("HandLeftRow2");
      createdObjects.Add(handLeftRow2Go);
      handLeftRow2Go.transform.SetParent(handLeftGo.transform, false);
      var handLeftRow2GoRect = handLeftRow2Go.AddComponent<RectTransform>();
      handLeftRow2GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      handLeftRow2GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      handLeftRow2GoRect.anchoredPosition = new Vector2(5f, 0f);
      handLeftRow2GoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add("SafeArea/HandLeft/HandLeftRow2", handLeftRow2Go);

      var scrollableHandLeftGo = new GameObject("ScrollableHandLeft");
      createdObjects.Add(scrollableHandLeftGo);
      scrollableHandLeftGo.transform.SetParent(handLeftGo.transform, false);
      var scrollableHandLeftGoRect = scrollableHandLeftGo.AddComponent<RectTransform>();
      scrollableHandLeftGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      scrollableHandLeftGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      scrollableHandLeftGoRect.anchoredPosition = new Vector2(10f, 0f);
      scrollableHandLeftGoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add("SafeArea/HandLeft/ScrollableHandLeft", scrollableHandLeftGo);

      var hannerInnerLeftGo = new GameObject("HannerInnerLeft");
      createdObjects.Add(hannerInnerLeftGo);
      hannerInnerLeftGo.transform.SetParent(handLeftGo.transform, false);
      var hannerInnerLeftGoRect = hannerInnerLeftGo.AddComponent<RectTransform>();
      hannerInnerLeftGoRect.anchorMin = new Vector2(0f, 0f);
      hannerInnerLeftGoRect.anchorMax = new Vector2(0f, 0f);
      hannerInnerLeftGoRect.pivot = new Vector2(0f, 0f);
      hannerInnerLeftGoRect.anchoredPosition = new Vector2(40f, 12f);
      result.Objects.Add("SafeArea/HandLeft/HannerInnerLeft", hannerInnerLeftGo);

      var handRightGo = new GameObject("HandRight");
      createdObjects.Add(handRightGo);
      handRightGo.transform.SetParent(safeAreaGo.transform, false);
      var handRightGoRect = handRightGo.AddComponent<RectTransform>();
      handRightGoRect.anchorMin = new Vector2(1f, 0f);
      handRightGoRect.anchorMax = new Vector2(1f, 0f);
      handRightGoRect.pivot = new Vector2(1f, 0f);
      handRightGoRect.anchoredPosition = new Vector2(-40f, 45f);
      result.Objects.Add("SafeArea/HandRight", handRightGo);

      var handRightRow2Go = new GameObject("HandRightRow2");
      createdObjects.Add(handRightRow2Go);
      handRightRow2Go.transform.SetParent(handRightGo.transform, false);
      var handRightRow2GoRect = handRightRow2Go.AddComponent<RectTransform>();
      handRightRow2GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      handRightRow2GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      handRightRow2GoRect.anchoredPosition = new Vector2(-5f, 0f);
      handRightRow2GoRect.sizeDelta = new Vector2(100f, 100f);
      result.Objects.Add("SafeArea/HandRight/HandRightRow2", handRightRow2Go);

      var handInnerRightGo = new GameObject("HandInnerRight");
      createdObjects.Add(handInnerRightGo);
      handInnerRightGo.transform.SetParent(handRightGo.transform, false);
      var handInnerRightGoRect = handInnerRightGo.AddComponent<RectTransform>();
      handInnerRightGoRect.anchorMin = new Vector2(1f, 0f);
      handInnerRightGoRect.anchorMax = new Vector2(1f, 0f);
      handInnerRightGoRect.pivot = new Vector2(1f, 0f);
      handInnerRightGoRect.anchoredPosition = new Vector2(-40f, 12f);
      result.Objects.Add("SafeArea/HandRight/HandInnerRight", handInnerRightGo);

      var questDeckBrowserPortraitGo = new GameObject("QuestDeckBrowserPortrait");
      createdObjects.Add(questDeckBrowserPortraitGo);
      questDeckBrowserPortraitGo.transform.SetParent(safeAreaGo.transform, false);
      var questDeckBrowserPortraitGoRect = questDeckBrowserPortraitGo.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/QuestDeckBrowserPortrait", questDeckBrowserPortraitGo);

      var controlBarGo = new GameObject("ControlBar");
      createdObjects.Add(controlBarGo);
      controlBarGo.transform.SetParent(questDeckBrowserPortraitGo.transform, false);
      var controlBarGoRect = controlBarGo.AddComponent<RectTransform>();
      controlBarGoRect.anchorMin = new Vector2(0f, 1f);
      controlBarGoRect.anchorMax = new Vector2(1f, 1f);
      controlBarGoRect.pivot = new Vector2(0.5f, 1f);
      controlBarGoRect.sizeDelta = new Vector2(0f, 36f);
      result.Objects.Add("SafeArea/QuestDeckBrowserPortrait/ControlBar", controlBarGo);

      var background1Go = new GameObject("Background");
      createdObjects.Add(background1Go);
      background1Go.transform.SetParent(controlBarGo.transform, false);
      var background1GoRect = background1Go.AddComponent<RectTransform>();
      background1GoRect.localRotation = Quaternion.Euler(0f, 180f, 180f);
      result.Objects.Add("SafeArea/QuestDeckBrowserPortrait/ControlBar/Background", background1Go);

      var closeBrowserButton1Go = new GameObject("CloseBrowserButton");
      createdObjects.Add(closeBrowserButton1Go);
      closeBrowserButton1Go.transform.SetParent(controlBarGo.transform, false);
      var closeBrowserButton1GoRect = closeBrowserButton1Go.AddComponent<RectTransform>();
      closeBrowserButton1GoRect.anchorMin = new Vector2(1f, 0.5f);
      closeBrowserButton1GoRect.anchorMax = new Vector2(1f, 0.5f);
      closeBrowserButton1GoRect.pivot = new Vector2(1f, 0.5f);
      closeBrowserButton1GoRect.anchoredPosition = new Vector2(-6f, -3.9f);
      closeBrowserButton1GoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserPortrait/ControlBar/CloseBrowserButton",
        closeBrowserButton1Go
      );

      var textTMP16Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP16Go);
      textTMP16Go.transform.SetParent(closeBrowserButton1Go.transform, false);
      var textTMP16GoRect = textTMP16Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserPortrait/ControlBar/CloseBrowserButton/Text (TMP)",
        textTMP16Go
      );

      var filterButtonGo = new GameObject("FilterButton");
      createdObjects.Add(filterButtonGo);
      filterButtonGo.transform.SetParent(controlBarGo.transform, false);
      var filterButtonGoRect = filterButtonGo.AddComponent<RectTransform>();
      filterButtonGoRect.anchorMin = new Vector2(0f, 0.5f);
      filterButtonGoRect.anchorMax = new Vector2(0f, 0.5f);
      filterButtonGoRect.pivot = new Vector2(1f, 0.5f);
      filterButtonGoRect.anchoredPosition = new Vector2(30f, -3.900002f);
      filterButtonGoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserPortrait/ControlBar/FilterButton",
        filterButtonGo
      );

      var textTMP17Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP17Go);
      textTMP17Go.transform.SetParent(filterButtonGo.transform, false);
      var textTMP17GoRect = textTMP17Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserPortrait/ControlBar/FilterButton/Text (TMP)",
        textTMP17Go
      );

      var scrollViewGo = new GameObject("ScrollView");
      createdObjects.Add(scrollViewGo);
      scrollViewGo.transform.SetParent(questDeckBrowserPortraitGo.transform, false);
      var scrollViewGoRect = scrollViewGo.AddComponent<RectTransform>();
      scrollViewGoRect.anchoredPosition = new Vector2(0f, -18f);
      scrollViewGoRect.sizeDelta = new Vector2(0f, -36f);
      result.Objects.Add("SafeArea/QuestDeckBrowserPortrait/ScrollView", scrollViewGo);

      var viewportGo = new GameObject("Viewport");
      createdObjects.Add(viewportGo);
      viewportGo.transform.SetParent(scrollViewGo.transform, false);
      var viewportGoRect = viewportGo.AddComponent<RectTransform>();
      viewportGoRect.pivot = new Vector2(0f, 1f);
      result.Objects.Add("SafeArea/QuestDeckBrowserPortrait/ScrollView/Viewport", viewportGo);

      var contentGo = new GameObject("Content");
      createdObjects.Add(contentGo);
      contentGo.transform.SetParent(viewportGo.transform, false);
      var contentGoRect = contentGo.AddComponent<RectTransform>();
      contentGoRect.anchorMin = new Vector2(0f, 1f);
      contentGoRect.anchorMax = new Vector2(1f, 1f);
      contentGoRect.pivot = new Vector2(0f, 1f);
      contentGoRect.sizeDelta = new Vector2(0f, 300f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserPortrait/ScrollView/Viewport/Content",
        contentGo
      );

      var questDeckBrowserLandscapeGo = new GameObject("QuestDeckBrowserLandscape");
      createdObjects.Add(questDeckBrowserLandscapeGo);
      questDeckBrowserLandscapeGo.transform.SetParent(safeAreaGo.transform, false);
      var questDeckBrowserLandscapeGoRect =
        questDeckBrowserLandscapeGo.AddComponent<RectTransform>();
      result.Objects.Add("SafeArea/QuestDeckBrowserLandscape", questDeckBrowserLandscapeGo);

      var controlBar1Go = new GameObject("ControlBar");
      createdObjects.Add(controlBar1Go);
      controlBar1Go.transform.SetParent(questDeckBrowserLandscapeGo.transform, false);
      var controlBar1GoRect = controlBar1Go.AddComponent<RectTransform>();
      controlBar1GoRect.anchorMin = new Vector2(0f, 1f);
      controlBar1GoRect.anchorMax = new Vector2(1f, 1f);
      controlBar1GoRect.pivot = new Vector2(0.5f, 1f);
      controlBar1GoRect.sizeDelta = new Vector2(-96f, 64f);
      result.Objects.Add("SafeArea/QuestDeckBrowserLandscape/ControlBar", controlBar1Go);

      var closeBrowserButton2Go = new GameObject("CloseBrowserButton");
      createdObjects.Add(closeBrowserButton2Go);
      closeBrowserButton2Go.transform.SetParent(controlBar1Go.transform, false);
      var closeBrowserButton2GoRect = closeBrowserButton2Go.AddComponent<RectTransform>();
      closeBrowserButton2GoRect.anchorMin = new Vector2(1f, 0.5f);
      closeBrowserButton2GoRect.anchorMax = new Vector2(1f, 0.5f);
      closeBrowserButton2GoRect.pivot = new Vector2(1f, 0.5f);
      closeBrowserButton2GoRect.anchoredPosition = new Vector2(-22f, -10f);
      closeBrowserButton2GoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ControlBar/CloseBrowserButton",
        closeBrowserButton2Go
      );

      var textTMP18Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP18Go);
      textTMP18Go.transform.SetParent(closeBrowserButton2Go.transform, false);
      var textTMP18GoRect = textTMP18Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ControlBar/CloseBrowserButton/Text (TMP)",
        textTMP18Go
      );

      var filterButton1Go = new GameObject("FilterButton");
      createdObjects.Add(filterButton1Go);
      filterButton1Go.transform.SetParent(controlBar1Go.transform, false);
      var filterButton1GoRect = filterButton1Go.AddComponent<RectTransform>();
      filterButton1GoRect.anchorMin = new Vector2(1f, 0.5f);
      filterButton1GoRect.anchorMax = new Vector2(1f, 0.5f);
      filterButton1GoRect.pivot = new Vector2(1f, 0.5f);
      filterButton1GoRect.anchoredPosition = new Vector2(-55f, -10f);
      filterButton1GoRect.sizeDelta = new Vector2(22f, 22f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ControlBar/FilterButton",
        filterButton1Go
      );

      var textTMP19Go = new GameObject("Text (TMP)");
      createdObjects.Add(textTMP19Go);
      textTMP19Go.transform.SetParent(filterButton1Go.transform, false);
      var textTMP19GoRect = textTMP19Go.AddComponent<RectTransform>();
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ControlBar/FilterButton/Text (TMP)",
        textTMP19Go
      );

      var scrollView1Go = new GameObject("ScrollView");
      createdObjects.Add(scrollView1Go);
      scrollView1Go.transform.SetParent(questDeckBrowserLandscapeGo.transform, false);
      var scrollView1GoRect = scrollView1Go.AddComponent<RectTransform>();
      scrollView1GoRect.anchoredPosition = new Vector2(0f, -32f);
      scrollView1GoRect.sizeDelta = new Vector2(-144f, -64f);
      result.Objects.Add("SafeArea/QuestDeckBrowserLandscape/ScrollView", scrollView1Go);

      var viewport1Go = new GameObject("Viewport");
      createdObjects.Add(viewport1Go);
      viewport1Go.transform.SetParent(scrollView1Go.transform, false);
      var viewport1GoRect = viewport1Go.AddComponent<RectTransform>();
      viewport1GoRect.pivot = new Vector2(0f, 1f);
      viewport1GoRect.sizeDelta = new Vector2(-17f, 0f);
      result.Objects.Add("SafeArea/QuestDeckBrowserLandscape/ScrollView/Viewport", viewport1Go);

      var content1Go = new GameObject("Content");
      createdObjects.Add(content1Go);
      content1Go.transform.SetParent(viewport1Go.transform, false);
      var content1GoRect = content1Go.AddComponent<RectTransform>();
      content1GoRect.anchorMin = new Vector2(0f, 1f);
      content1GoRect.anchorMax = new Vector2(1f, 1f);
      content1GoRect.pivot = new Vector2(0f, 1f);
      content1GoRect.anchoredPosition = new Vector2(0f, -6.103516E-05f);
      content1GoRect.sizeDelta = new Vector2(-20f, 300f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Viewport/Content",
        content1Go
      );

      var scrollbarVerticalGo = new GameObject("Scrollbar Vertical");
      createdObjects.Add(scrollbarVerticalGo);
      scrollbarVerticalGo.transform.SetParent(scrollView1Go.transform, false);
      var scrollbarVerticalGoRect = scrollbarVerticalGo.AddComponent<RectTransform>();
      scrollbarVerticalGoRect.anchorMin = new Vector2(1f, 0f);
      scrollbarVerticalGoRect.anchorMax = new Vector2(1f, 1f);
      scrollbarVerticalGoRect.pivot = new Vector2(1f, 1f);
      scrollbarVerticalGoRect.sizeDelta = new Vector2(20f, -17f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Scrollbar Vertical",
        scrollbarVerticalGo
      );

      var bar3Go = new GameObject("Bar");
      createdObjects.Add(bar3Go);
      bar3Go.transform.SetParent(scrollbarVerticalGo.transform, false);
      var bar3GoRect = bar3Go.AddComponent<RectTransform>();
      bar3GoRect.anchorMin = new Vector2(0.5f, 0f);
      bar3GoRect.anchorMax = new Vector2(0.5f, 1f);
      bar3GoRect.sizeDelta = new Vector2(4f, 0f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Scrollbar Vertical/Bar",
        bar3Go
      );

      var barHitSlop3Go = new GameObject("BarHitSlop");
      createdObjects.Add(barHitSlop3Go);
      barHitSlop3Go.transform.SetParent(scrollbarVerticalGo.transform, false);
      var barHitSlop3GoRect = barHitSlop3Go.AddComponent<RectTransform>();
      barHitSlop3GoRect.anchorMin = new Vector2(0.5f, 0f);
      barHitSlop3GoRect.anchorMax = new Vector2(0.5f, 1f);
      barHitSlop3GoRect.sizeDelta = new Vector2(24f, 0f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Scrollbar Vertical/BarHitSlop",
        barHitSlop3Go
      );

      var slidingArea3Go = new GameObject("Sliding Area");
      createdObjects.Add(slidingArea3Go);
      slidingArea3Go.transform.SetParent(scrollbarVerticalGo.transform, false);
      var slidingArea3GoRect = slidingArea3Go.AddComponent<RectTransform>();
      slidingArea3GoRect.sizeDelta = new Vector2(-20f, -20f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Scrollbar Vertical/Sliding Area",
        slidingArea3Go
      );

      var handle3Go = new GameObject("Handle");
      createdObjects.Add(handle3Go);
      handle3Go.transform.SetParent(slidingArea3Go.transform, false);
      var handle3GoRect = handle3Go.AddComponent<RectTransform>();
      handle3GoRect.sizeDelta = new Vector2(20f, 20f);
      result.Objects.Add(
        "SafeArea/QuestDeckBrowserLandscape/ScrollView/Scrollbar Vertical/Sliding Area/Handle",
        handle3Go
      );

      return result;
    }
  }
}
