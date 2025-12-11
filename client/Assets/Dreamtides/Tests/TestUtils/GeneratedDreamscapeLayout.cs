// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: Registry
// Generated at: 2025-12-10 15:56:32

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
  public class GeneratedDreamscapeLayout
  {
    public static DreamscapeLayout Create(List<GameObject> createdObjects)
    {
      var layoutGo = new GameObject("Registry");
      createdObjects.Add(layoutGo);
      var layout = layoutGo.AddComponent<DreamscapeLayout>();

      var questDeckGo = new GameObject("QuestDeck");
      createdObjects.Add(questDeckGo);
      questDeckGo.transform.localPosition = new Vector3(0.209f, -0.488f, 1f);
      questDeckGo.transform.localScale = new Vector3(0.03f, 0.03f, 0.03f);
      var questDeck = questDeckGo.AddComponent<RenderAsChildObjectLayout>();
      questDeck._yMultiplier = 0.5f;
      layout._questDeck = questDeck;

      var questDeckBrowserPortraitGo = new GameObject("QuestDeckBrowserPortrait");
      createdObjects.Add(questDeckBrowserPortraitGo);
      var questDeckBrowserPortraitGoRect = questDeckBrowserPortraitGo.AddComponent<RectTransform>();
      var questDeckBrowserPortrait =
        questDeckBrowserPortraitGo.AddComponent<QuestDeckBrowserObjectLayout>();
      questDeckBrowserPortrait._scrollbarFadeDuration = 0.3f;
      questDeckBrowserPortrait._cardWidth = 35f;
      questDeckBrowserPortrait._cardHeight = 52f;
      questDeckBrowserPortrait._cardSpacing = 5f;
      questDeckBrowserPortrait._worldSpaceDepth = 1f;
      questDeckBrowserPortrait._cardScale = 0.036f;
      questDeckBrowserPortrait._disableInterfaceOnOpen = true;

      var contentGo = new GameObject("Content");
      createdObjects.Add(contentGo);
      var contentGoRect = contentGo.AddComponent<RectTransform>();
      contentGoRect.anchorMin = new Vector2(0f, 1f);
      contentGoRect.anchorMax = new Vector2(1f, 1f);
      contentGoRect.pivot = new Vector2(0f, 1f);
      contentGoRect.sizeDelta = new Vector2(0f, 300f);
      questDeckBrowserPortrait._content = (RectTransform)contentGo.transform;

      var scrollbarCanvasGroupGo = new GameObject("CloseBrowserButton");
      createdObjects.Add(scrollbarCanvasGroupGo);
      var scrollbarCanvasGroupGoRect = scrollbarCanvasGroupGo.AddComponent<RectTransform>();
      scrollbarCanvasGroupGoRect.anchorMin = new Vector2(1f, 0.5f);
      scrollbarCanvasGroupGoRect.anchorMax = new Vector2(1f, 0.5f);
      scrollbarCanvasGroupGoRect.pivot = new Vector2(1f, 0.5f);
      scrollbarCanvasGroupGoRect.anchoredPosition = new Vector2(-6f, -3.9f);
      scrollbarCanvasGroupGoRect.sizeDelta = new Vector2(22f, 22f);
      var canvasGroup = scrollbarCanvasGroupGo.AddComponent<CanvasGroup>();
      questDeckBrowserPortrait._scrollbarCanvasGroup = canvasGroup;
      questDeckBrowserPortrait._closeButton =
        scrollbarCanvasGroupGo.GetComponent<CloseBrowserButton>();

      var worldSpaceParentGo = new GameObject("QuestDeckBrowserWorldSpace");
      createdObjects.Add(worldSpaceParentGo);
      worldSpaceParentGo.transform.localPosition = new Vector3(0f, 0f, 1f);
      questDeckBrowserPortrait._worldSpaceParent = worldSpaceParentGo.transform;

      var backgroundOverlayGo = new GameObject("WorldSpaceBackground");
      createdObjects.Add(backgroundOverlayGo);
      backgroundOverlayGo.transform.SetParent(worldSpaceParentGo.transform, false);
      backgroundOverlayGo.transform.localScale = new Vector3(100f, 100f, 1f);
      var backgroundOverlay1 = backgroundOverlayGo.AddComponent<BackgroundOverlay>();
      var spriteRenderer = backgroundOverlayGo.AddComponent<SpriteRenderer>();
      backgroundOverlay1._overlay = spriteRenderer;
      questDeckBrowserPortrait._backgroundOverlay = backgroundOverlay1;

      var canvasBackgroundOverlayGo = new GameObject("Background");
      createdObjects.Add(canvasBackgroundOverlayGo);
      var canvasBackgroundOverlayGoRect = canvasBackgroundOverlayGo.AddComponent<RectTransform>();
      canvasBackgroundOverlayGoRect.localRotation = Quaternion.Euler(0f, 180f, 180f);
      var image = canvasBackgroundOverlayGo.AddComponent<Image>();
      questDeckBrowserPortrait._canvasBackgroundOverlay = image;
      layout._questDeckBrowserPortrait = questDeckBrowserPortrait;

      var questDeckBrowserLandscapeGo = new GameObject("QuestDeckBrowserLandscape");
      createdObjects.Add(questDeckBrowserLandscapeGo);
      var questDeckBrowserLandscapeGoRect =
        questDeckBrowserLandscapeGo.AddComponent<RectTransform>();
      var questDeckBrowserLandscape =
        questDeckBrowserLandscapeGo.AddComponent<QuestDeckBrowserObjectLayout>();
      questDeckBrowserLandscape._scrollbarFadeDuration = 0.5f;
      questDeckBrowserLandscape._cardWidth = 100f;
      questDeckBrowserLandscape._cardHeight = 160f;
      questDeckBrowserLandscape._cardSpacing = 15f;
      questDeckBrowserLandscape._worldSpaceDepth = 1f;
      questDeckBrowserLandscape._cardScale = 0.125f;
      questDeckBrowserLandscape._backgroundOverlayOpacity = 0.85f;

      var content1Go = new GameObject("Content");
      createdObjects.Add(content1Go);
      var content1GoRect = content1Go.AddComponent<RectTransform>();
      content1GoRect.anchorMin = new Vector2(0f, 1f);
      content1GoRect.anchorMax = new Vector2(1f, 1f);
      content1GoRect.pivot = new Vector2(0f, 1f);
      content1GoRect.anchoredPosition = new Vector2(0f, -6.103516E-05f);
      content1GoRect.sizeDelta = new Vector2(-20f, 300f);
      questDeckBrowserLandscape._content = (RectTransform)content1Go.transform;

      var scrollbarCanvasGroup1Go = new GameObject("Scrollbar Vertical");
      createdObjects.Add(scrollbarCanvasGroup1Go);
      var scrollbarCanvasGroup1GoRect = scrollbarCanvasGroup1Go.AddComponent<RectTransform>();
      scrollbarCanvasGroup1GoRect.anchorMin = new Vector2(1f, 0f);
      scrollbarCanvasGroup1GoRect.anchorMax = new Vector2(1f, 1f);
      scrollbarCanvasGroup1GoRect.pivot = new Vector2(1f, 1f);
      scrollbarCanvasGroup1GoRect.sizeDelta = new Vector2(20f, -17f);
      var canvasGroup1 = scrollbarCanvasGroup1Go.AddComponent<CanvasGroup>();
      questDeckBrowserLandscape._scrollbarCanvasGroup = canvasGroup1;

      var closeButtonGo = new GameObject("CloseBrowserButton");
      createdObjects.Add(closeButtonGo);
      var closeButtonGoRect = closeButtonGo.AddComponent<RectTransform>();
      closeButtonGoRect.anchorMin = new Vector2(1f, 0.5f);
      closeButtonGoRect.anchorMax = new Vector2(1f, 0.5f);
      closeButtonGoRect.pivot = new Vector2(1f, 0.5f);
      closeButtonGoRect.anchoredPosition = new Vector2(-22f, -10f);
      closeButtonGoRect.sizeDelta = new Vector2(22f, 22f);
      var closeBrowserButton = closeButtonGo.AddComponent<CloseBrowserButton>();
      questDeckBrowserLandscape._closeButton = closeBrowserButton;
      questDeckBrowserLandscape._worldSpaceParent = worldSpaceParentGo.transform;
      questDeckBrowserLandscape._backgroundOverlay = backgroundOverlay1;
      layout._questDeckBrowserLandscape = questDeckBrowserLandscape;

      var essenceTotalGo = new GameObject("EssenceDisplay");
      createdObjects.Add(essenceTotalGo);
      var essenceTotalGoRect = essenceTotalGo.AddComponent<RectTransform>();
      essenceTotalGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      essenceTotalGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      essenceTotalGoRect.anchoredPosition = new Vector2(28.4f, 14.8f);
      essenceTotalGoRect.sizeDelta = new Vector2(500f, 128f);
      essenceTotalGoRect.localScale = new Vector3(0.1579779f, 0.1579779f, 0.1579779f);
      var essenceTotal1 = essenceTotalGo.AddComponent<EssenceTotal>();

      var textGo = new GameObject("EssenceText");
      createdObjects.Add(textGo);
      textGo.transform.SetParent(essenceTotalGo.transform, false);
      var textGoRect = textGo.AddComponent<RectTransform>();
      textGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      textGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      textGoRect.anchoredPosition = new Vector2(-52.3f, 0f);
      textGoRect.sizeDelta = new Vector2(275.6726f, 100f);
      var textMeshProUGUI = textGo.AddComponent<TextMeshProUGUI>();
      essenceTotal1._text = textMeshProUGUI;

      var onChangeGo = new GameObject("EssenceTotalChangeEffect");
      createdObjects.Add(onChangeGo);
      onChangeGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var timedEffect = onChangeGo.AddComponent<TimedEffect>();
      essenceTotal1._onChange = timedEffect;
      layout._essenceTotal = essenceTotal1;

      var draftPickLayoutGo = new GameObject("DraftPickObjectLayout");
      createdObjects.Add(draftPickLayoutGo);
      draftPickLayoutGo.transform.localPosition = new Vector3(0f, 0f, 2.75f);
      draftPickLayoutGo.transform.localScale = new Vector3(0.25f, 0.25f, 0.25f);
      var draftPickLayout = draftPickLayoutGo.AddComponent<SitePickObjectLayout>();
      draftPickLayout._horizontalSpacing = 0.7f;
      draftPickLayout._verticalSpacing = 1.05f;
      draftPickLayout._cardWidth = 2.5f;
      draftPickLayout._cardHeight = 4f;
      draftPickLayout._landscapeScaleOverride = 0.4f;
      draftPickLayout._landscapeHorizontalSpacingOverride = 1.2f;
      layout._draftPickLayout = draftPickLayout;

      var destroyedQuestCardsGo = new GameObject("DestroyedQuestCards");
      createdObjects.Add(destroyedQuestCardsGo);
      destroyedQuestCardsGo.transform.localPosition = new Vector3(0f, 0f, 5f);
      destroyedQuestCardsGo.transform.localScale = new Vector3(0f, 0f, 0f);
      var destroyedQuestCards = destroyedQuestCardsGo.AddComponent<PileObjectLayout>();
      destroyedQuestCards._yMultiplier = 0f;
      layout._destroyedQuestCards = destroyedQuestCards;

      var aboveQuestDeckGo = new GameObject("AboveQuestDeck");
      createdObjects.Add(aboveQuestDeckGo);
      aboveQuestDeckGo.transform.SetParent(questDeckGo.transform, false);
      aboveQuestDeckGo.transform.localPosition = new Vector3(0f, 4.5f, 0f);
      aboveQuestDeckGo.transform.localScale = new Vector3(0.03f, 0.03f, 0.03f);
      layout._aboveQuestDeck = aboveQuestDeckGo.transform;

      var shopLayoutGo = new GameObject("ShopObjectLayout");
      createdObjects.Add(shopLayoutGo);
      shopLayoutGo.transform.localPosition = new Vector3(-0.2069296f, -0.6909008f, 2.5f);
      shopLayoutGo.transform.localScale = new Vector3(0.15f, 0.15f, 0.15f);
      var shopLayout = shopLayoutGo.AddComponent<SitePickObjectLayout>();
      shopLayout._horizontalSpacing = 0.45f;
      shopLayout._verticalSpacing = 0.7f;
      shopLayout._cardWidth = 2.5f;
      shopLayout._cardHeight = 4f;
      shopLayout._forceTwoRows = true;
      shopLayout._preserveLayoutOnRemoval = true;

      var closeSiteButtonGo = new GameObject("CloseSiteButton");
      createdObjects.Add(closeSiteButtonGo);
      var closeSiteButtonGoRect = closeSiteButtonGo.AddComponent<RectTransform>();
      closeSiteButtonGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      closeSiteButtonGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      closeSiteButtonGoRect.anchoredPosition = new Vector2(342.2555f, 68.90001f);
      closeSiteButtonGoRect.sizeDelta = new Vector2(22f, 22f);
      shopLayout._closeSiteButton = (RectTransform)closeSiteButtonGo.transform;
      layout._shopLayout = shopLayout;

      var dreamsignDisplayGo = new GameObject("DreamsignDisplay");
      createdObjects.Add(dreamsignDisplayGo);
      dreamsignDisplayGo.transform.SetParent(questDeckGo.transform, false);
      dreamsignDisplayGo.transform.localPosition = new Vector3(
        -3.166668f,
        -0.2333552f,
        2.384186E-05f
      );
      dreamsignDisplayGo.transform.localScale = new Vector3(0.6666667f, 0.6666668f, 0.6666668f);
      var dreamsignDisplay = dreamsignDisplayGo.AddComponent<DreamsignDisplayLayout>();
      dreamsignDisplay._horizontalSpacing = 3f;
      dreamsignDisplay._verticalSpacing = 3f;
      dreamsignDisplay._cardWidth = 2.5f;
      dreamsignDisplay._cardHeight = 3.5f;
      layout._dreamsignDisplay = dreamsignDisplay;

      var journeyChoiceDisplayGo = new GameObject("JourneyObjectLayout");
      createdObjects.Add(journeyChoiceDisplayGo);
      journeyChoiceDisplayGo.transform.localPosition = new Vector3(-0.2069296f, -0.6909008f, 2.5f);
      journeyChoiceDisplayGo.transform.localScale = new Vector3(0.15f, 0.15f, 0.15f);
      var journeyChoiceDisplay = journeyChoiceDisplayGo.AddComponent<SitePickObjectLayout>();
      journeyChoiceDisplay._horizontalSpacing = 0.45f;
      journeyChoiceDisplay._verticalSpacing = 0.7f;
      journeyChoiceDisplay._cardWidth = 2.5f;
      journeyChoiceDisplay._cardHeight = 4f;
      journeyChoiceDisplay._forceTwoRows = true;
      journeyChoiceDisplay._preserveLayoutOnRemoval = true;
      journeyChoiceDisplay._closeSiteButton = (RectTransform)closeSiteButtonGo.transform;
      layout._journeyChoiceDisplay = journeyChoiceDisplay;

      var temptingOfferDisplayGo = new GameObject("TemptingOfferObjectLayout");
      createdObjects.Add(temptingOfferDisplayGo);
      temptingOfferDisplayGo.transform.localPosition = new Vector3(0f, 0f, 2.5f);
      temptingOfferDisplayGo.transform.localScale = new Vector3(0.125f, 0.125f, 0.125f);
      var temptingOfferDisplay = temptingOfferDisplayGo.AddComponent<TemptingOfferObjectLayout>();
      temptingOfferDisplay._horizontalSpacing = 0.5f;
      temptingOfferDisplay._verticalSpacing = 0.75f;
      temptingOfferDisplay._cardWidth = 2.5f;
      temptingOfferDisplay._cardHeight = 2.5f;
      temptingOfferDisplay._landscapeScaleOverride = 0.175f;
      temptingOfferDisplay._landscapeHorizontalSpacingOverride = 0.63f;
      temptingOfferDisplay._preserveLayoutOnRemoval = true;
      temptingOfferDisplay._acceptButtonScale = 0.3f;
      temptingOfferDisplay._landscapeVerticalSpacingOverride = 1f;
      temptingOfferDisplay._closeSiteButton = (RectTransform)closeSiteButtonGo.transform;

      var acceptButtonPrefabGo = new GameObject("DisplayableButton");
      createdObjects.Add(acceptButtonPrefabGo);
      var displayableButton = acceptButtonPrefabGo.AddComponent<DisplayableButton>();
      var spriteRenderer1 = acceptButtonPrefabGo.AddComponent<SpriteRenderer>();
      displayableButton._background = spriteRenderer1;

      var text1Go = new GameObject("ActionButtonText");
      createdObjects.Add(text1Go);
      text1Go.transform.SetParent(acceptButtonPrefabGo.transform, false);
      var text1GoRect = text1Go.AddComponent<RectTransform>();
      text1GoRect.anchorMin = new Vector2(0.5f, 0.5f);
      text1GoRect.anchorMax = new Vector2(0.5f, 0.5f);
      text1GoRect.anchoredPosition = new Vector2(-0.012f, -0.007f);
      text1GoRect.sizeDelta = new Vector2(7.416f, 2.536f);
      var textMeshPro = text1Go.AddComponent<TextMeshPro>();
      displayableButton._text = textMeshPro;
      var boxCollider = acceptButtonPrefabGo.AddComponent<BoxCollider>();
      displayableButton._collider = boxCollider;
      temptingOfferDisplay._acceptButtonPrefab = displayableButton;
      layout._temptingOfferDisplay = temptingOfferDisplay;

      var questEffectPositionGo = new GameObject("QuestEffectPosition");
      createdObjects.Add(questEffectPositionGo);
      questEffectPositionGo.transform.localScale = new Vector3(0.2f, 0.2f, 0.2f);
      var questEffectPosition = questEffectPositionGo.AddComponent<CenteredLineObjectLayout>();
      questEffectPosition._horizontalSpacing = 3.5f;
      questEffectPosition._cardWidth = 2.5f;
      questEffectPosition._minScale = 0.01f;
      questEffectPosition._maxScale = 0.2f;
      layout._questEffectPosition = questEffectPosition;

      var startBattleLayoutGo = new GameObject("StartBattleObjectLayout");
      createdObjects.Add(startBattleLayoutGo);
      startBattleLayoutGo.transform.localPosition = new Vector3(0f, 0.5f, 2.75f);
      var startBattleLayout = startBattleLayoutGo.AddComponent<StartBattleObjectLayout>();
      startBattleLayout._internalGameContext = GameContext.Interface;
      startBattleLayout._cardInwardOffsetPortrait = 1.15f;
      startBattleLayout._cardInwardOffsetLandscape = 0.7f;
      startBattleLayout._cardScalePortrait = 0.25f;
      startBattleLayout._cardScaleLandscape = 0.45f;
      startBattleLayout._buttonVerticalOffsetPortrait = -1.25f;
      startBattleLayout._buttonVerticalOffsetLandscape = -1.5f;
      startBattleLayout._buttonScalePortrait = 0.1f;
      startBattleLayout._buttonScaleLandscape = 0.1f;

      var vsTextGo = new GameObject("VsText");
      createdObjects.Add(vsTextGo);
      vsTextGo.transform.SetParent(startBattleLayoutGo.transform, false);
      var vsTextGoRect = vsTextGo.AddComponent<RectTransform>();
      vsTextGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      vsTextGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      vsTextGoRect.sizeDelta = new Vector2(20f, 5f);
      var textMeshPro1 = vsTextGo.AddComponent<TextMeshPro>();
      startBattleLayout._vsText = textMeshPro1;
      startBattleLayout._buttonPrefab = displayableButton;
      layout._startBattleLayout = startBattleLayout;

      var essenceTotalWorldPositionGo = new GameObject("EssenceTotalWorldPosition");
      createdObjects.Add(essenceTotalWorldPositionGo);
      essenceTotalWorldPositionGo.transform.localPosition = new Vector3(-0.215f, -0.488f, 1f);
      var staticDisplayable = essenceTotalWorldPositionGo.AddComponent<StaticDisplayable>();
      layout._essenceTotalWorldPosition = staticDisplayable;

      return layout;
    }
  }
}
