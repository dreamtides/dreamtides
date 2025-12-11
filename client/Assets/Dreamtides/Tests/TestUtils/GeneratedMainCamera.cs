// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: MainCamera
// Generated at: 2025-12-11 06:45:15

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
  public class GeneratedMainCamera
  {
    public GameCamera GameCamera { get; private set; } = null!;

    public static GeneratedMainCamera Create(List<GameObject> createdObjects)
    {
      var result = new GeneratedMainCamera();

      var mainCameraGo = new GameObject("MainCamera");
      createdObjects.Add(mainCameraGo);
      mainCameraGo.transform.localPosition = new Vector3(56.7766f, 61.53397f, 7.263145f);
      mainCameraGo.transform.localRotation = Quaternion.Euler(50f, 250f, 2.656472E-06f);
      var mainCamera = mainCameraGo.AddComponent<GameCamera>();
      result.GameCamera = mainCameraGo.GetComponent<GameCamera>();

      var musicAudioSourceGo = new GameObject("MusicAudioSource");
      createdObjects.Add(musicAudioSourceGo);
      musicAudioSourceGo.transform.SetParent(mainCameraGo.transform, false);

      var postProcessingGo = new GameObject("PostProcessing");
      createdObjects.Add(postProcessingGo);
      postProcessingGo.transform.SetParent(mainCameraGo.transform, false);

      var drawnCardsPositionGo = new GameObject("DrawnCardsPosition");
      createdObjects.Add(drawnCardsPositionGo);
      drawnCardsPositionGo.transform.SetParent(mainCameraGo.transform, false);
      drawnCardsPositionGo.transform.localPosition = new Vector3(1.29f, 0.65f, 10.21f);
      var drawnCardsPosition = drawnCardsPositionGo.AddComponent<PileObjectLayout>();

      var portraitMusicAudioSourceGo = new GameObject("PortraitMusicAudioSource");
      createdObjects.Add(portraitMusicAudioSourceGo);
      portraitMusicAudioSourceGo.transform.SetParent(mainCameraGo.transform, false);

      var gameMessageGo = new GameObject("GameMessage");
      createdObjects.Add(gameMessageGo);
      gameMessageGo.transform.SetParent(mainCameraGo.transform, false);
      gameMessageGo.transform.localPosition = new Vector3(0f, 0.88f, 27.73f);
      gameMessageGo.transform.localRotation = Quaternion.Euler(90f, 180f, 0f);

      var topGo = new GameObject("Top");
      createdObjects.Add(topGo);
      topGo.transform.SetParent(gameMessageGo.transform, false);
      topGo.transform.localPosition = new Vector3(0f, 5f, -8f);

      var userJudgmentGo = new GameObject("UserJudgment");
      createdObjects.Add(userJudgmentGo);
      userJudgmentGo.transform.SetParent(gameMessageGo.transform, false);
      var userJudgmentGoRect = userJudgmentGo.AddComponent<RectTransform>();
      userJudgmentGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      userJudgmentGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      userJudgmentGoRect.anchoredPosition = new Vector2(0f, 10.43f);
      userJudgmentGoRect.sizeDelta = new Vector2(10f, 10f);
      userJudgmentGoRect.localRotation = Quaternion.Euler(80.00002f, 180f, 0f);

      var magicCircle25Go = new GameObject("Magic circle 25");
      createdObjects.Add(magicCircle25Go);
      magicCircle25Go.transform.SetParent(userJudgmentGo.transform, false);
      magicCircle25Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      magicCircle25Go.transform.localScale = new Vector3(1.25f, 1.25f, 1.25f);

      var godRayGo = new GameObject("GodRay");
      createdObjects.Add(godRayGo);
      godRayGo.transform.SetParent(magicCircle25Go.transform, false);

      var flashGo = new GameObject("Flash");
      createdObjects.Add(flashGo);
      flashGo.transform.SetParent(magicCircle25Go.transform, false);
      flashGo.transform.localPosition = new Vector3(0f, 1f, 0f);

      var circleGo = new GameObject("Circle");
      createdObjects.Add(circleGo);
      circleGo.transform.SetParent(magicCircle25Go.transform, false);

      var endGoldSparksGo = new GameObject("EndGoldSparks");
      createdObjects.Add(endGoldSparksGo);
      endGoldSparksGo.transform.SetParent(magicCircle25Go.transform, false);

      var endWhiteSparksGo = new GameObject("EndWhiteSparks");
      createdObjects.Add(endWhiteSparksGo);
      endWhiteSparksGo.transform.SetParent(magicCircle25Go.transform, false);

      var endBlackSparksGo = new GameObject("EndBlackSparks");
      createdObjects.Add(endBlackSparksGo);
      endBlackSparksGo.transform.SetParent(magicCircle25Go.transform, false);

      var enemyJudgmentGo = new GameObject("EnemyJudgment");
      createdObjects.Add(enemyJudgmentGo);
      enemyJudgmentGo.transform.SetParent(gameMessageGo.transform, false);
      var enemyJudgmentGoRect = enemyJudgmentGo.AddComponent<RectTransform>();
      enemyJudgmentGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      enemyJudgmentGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      enemyJudgmentGoRect.anchoredPosition = new Vector2(0f, 10.43f);
      enemyJudgmentGoRect.sizeDelta = new Vector2(10f, 10f);
      enemyJudgmentGoRect.localRotation = Quaternion.Euler(80.00002f, 180f, 0f);

      var magicCircle29Go = new GameObject("Magic circle 29");
      createdObjects.Add(magicCircle29Go);
      magicCircle29Go.transform.SetParent(enemyJudgmentGo.transform, false);
      magicCircle29Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      magicCircle29Go.transform.localScale = new Vector3(1.25f, 1.25f, 1.25f);

      var quadGo = new GameObject("Quad");
      createdObjects.Add(quadGo);
      quadGo.transform.SetParent(magicCircle29Go.transform, false);

      var leavesGo = new GameObject("Leaves");
      createdObjects.Add(leavesGo);
      leavesGo.transform.SetParent(magicCircle29Go.transform, false);

      var petalsGo = new GameObject("Petals");
      createdObjects.Add(petalsGo);
      petalsGo.transform.SetParent(magicCircle29Go.transform, false);

      var sparksGo = new GameObject("Sparks");
      createdObjects.Add(sparksGo);
      sparksGo.transform.SetParent(magicCircle29Go.transform, false);

      var endWhiteSparks1Go = new GameObject("EndWhiteSparks");
      createdObjects.Add(endWhiteSparks1Go);
      endWhiteSparks1Go.transform.SetParent(magicCircle29Go.transform, false);

      var endBlackSparks1Go = new GameObject("EndBlackSparks");
      createdObjects.Add(endBlackSparks1Go);
      endBlackSparks1Go.transform.SetParent(magicCircle29Go.transform, false);

      var victoryGo = new GameObject("Victory");
      createdObjects.Add(victoryGo);
      victoryGo.transform.SetParent(gameMessageGo.transform, false);
      var victoryGoRect = victoryGo.AddComponent<RectTransform>();
      victoryGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      victoryGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      victoryGoRect.anchoredPosition = new Vector2(0f, 10.43f);
      victoryGoRect.sizeDelta = new Vector2(10f, 10f);
      victoryGoRect.localRotation = Quaternion.Euler(80.00002f, 180f, 0f);

      var magicCircle33Go = new GameObject("Magic circle 33");
      createdObjects.Add(magicCircle33Go);
      magicCircle33Go.transform.SetParent(victoryGo.transform, false);
      magicCircle33Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      magicCircle33Go.transform.localScale = new Vector3(1.25f, 1.25f, 1.25f);

      var darknessGo = new GameObject("Darkness");
      createdObjects.Add(darknessGo);
      darknessGo.transform.SetParent(magicCircle33Go.transform, false);
      darknessGo.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var middleFlowGo = new GameObject("MiddleFlow");
      createdObjects.Add(middleFlowGo);
      middleFlowGo.transform.SetParent(magicCircle33Go.transform, false);
      middleFlowGo.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var sideFlowGo = new GameObject("SideFlow");
      createdObjects.Add(sideFlowGo);
      sideFlowGo.transform.SetParent(magicCircle33Go.transform, false);
      sideFlowGo.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var smokeGo = new GameObject("Smoke");
      createdObjects.Add(smokeGo);
      smokeGo.transform.SetParent(magicCircle33Go.transform, false);
      smokeGo.transform.localPosition = new Vector3(0f, 0.5f, 0f);

      var defeatGo = new GameObject("Defeat");
      createdObjects.Add(defeatGo);
      defeatGo.transform.SetParent(gameMessageGo.transform, false);
      var defeatGoRect = defeatGo.AddComponent<RectTransform>();
      defeatGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      defeatGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      defeatGoRect.anchoredPosition = new Vector2(0f, 10.43f);
      defeatGoRect.sizeDelta = new Vector2(10f, 10f);
      defeatGoRect.localRotation = Quaternion.Euler(80.00002f, 180f, 0f);

      var magicCircle331Go = new GameObject("Magic circle 33");
      createdObjects.Add(magicCircle331Go);
      magicCircle331Go.transform.SetParent(defeatGo.transform, false);
      magicCircle331Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      magicCircle331Go.transform.localScale = new Vector3(1.25f, 1.25f, 1.25f);

      var darkness1Go = new GameObject("Darkness");
      createdObjects.Add(darkness1Go);
      darkness1Go.transform.SetParent(magicCircle331Go.transform, false);
      darkness1Go.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var middleFlow1Go = new GameObject("MiddleFlow");
      createdObjects.Add(middleFlow1Go);
      middleFlow1Go.transform.SetParent(magicCircle331Go.transform, false);
      middleFlow1Go.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var sideFlow1Go = new GameObject("SideFlow");
      createdObjects.Add(sideFlow1Go);
      sideFlow1Go.transform.SetParent(magicCircle331Go.transform, false);
      sideFlow1Go.transform.localPosition = new Vector3(0f, 0.05f, 0f);

      var smoke1Go = new GameObject("Smoke");
      createdObjects.Add(smoke1Go);
      smoke1Go.transform.SetParent(magicCircle331Go.transform, false);
      smoke1Go.transform.localPosition = new Vector3(0f, 0.5f, 0f);

      var questDeckObjectLayoutGo = new GameObject("QuestDeckObjectLayout");
      createdObjects.Add(questDeckObjectLayoutGo);
      questDeckObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      questDeckObjectLayoutGo.transform.localPosition = new Vector3(0.209f, -0.488f, 1f);
      questDeckObjectLayoutGo.transform.localScale = new Vector3(0.03f, 0.03f, 0.03f);
      var questDeckObjectLayout = questDeckObjectLayoutGo.AddComponent<QuestDeckObjectLayout>();
      questDeckObjectLayout._yMultiplier = 0.5f;

      var backgroundGo = new GameObject("Background");
      createdObjects.Add(backgroundGo);
      backgroundGo.transform.SetParent(questDeckObjectLayoutGo.transform, false);
      backgroundGo.transform.localPosition = new Vector3(0f, -0.39f, 0.32f);
      backgroundGo.transform.localScale = new Vector3(0.3512196f, 0.4547648f, 0.3f);

      var pointLightGo = new GameObject("Point Light");
      createdObjects.Add(pointLightGo);
      pointLightGo.transform.SetParent(questDeckObjectLayoutGo.transform, false);
      pointLightGo.transform.localPosition = new Vector3(0f, 0f, -5f);

      var aboveQuestDeckGo = new GameObject("AboveQuestDeck");
      createdObjects.Add(aboveQuestDeckGo);
      aboveQuestDeckGo.transform.SetParent(questDeckObjectLayoutGo.transform, false);
      aboveQuestDeckGo.transform.localPosition = new Vector3(0f, 4.5f, 0f);
      aboveQuestDeckGo.transform.localScale = new Vector3(0.03f, 0.03f, 0.03f);

      var dreamsignDisplayGo = new GameObject("DreamsignDisplay");
      createdObjects.Add(dreamsignDisplayGo);
      dreamsignDisplayGo.transform.SetParent(questDeckObjectLayoutGo.transform, false);
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

      var questDeckButtonGo = new GameObject("QuestDeckButton");
      createdObjects.Add(questDeckButtonGo);
      questDeckButtonGo.transform.SetParent(questDeckObjectLayoutGo.transform, false);

      var essenceTotalWorldPositionGo = new GameObject("EssenceTotalWorldPosition");
      createdObjects.Add(essenceTotalWorldPositionGo);
      essenceTotalWorldPositionGo.transform.SetParent(mainCameraGo.transform, false);
      essenceTotalWorldPositionGo.transform.localPosition = new Vector3(-0.215f, -0.488f, 1f);

      var essenceTotalChangeEffectGo = new GameObject("EssenceTotalChangeEffect");
      createdObjects.Add(essenceTotalChangeEffectGo);
      essenceTotalChangeEffectGo.transform.SetParent(essenceTotalWorldPositionGo.transform, false);
      essenceTotalChangeEffectGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);

      var glowGo = new GameObject("Glow");
      createdObjects.Add(glowGo);
      glowGo.transform.SetParent(essenceTotalChangeEffectGo.transform, false);

      var flash1Go = new GameObject("Flash");
      createdObjects.Add(flash1Go);
      flash1Go.transform.SetParent(essenceTotalChangeEffectGo.transform, false);

      var dustGo = new GameObject("Dust");
      createdObjects.Add(dustGo);
      dustGo.transform.SetParent(essenceTotalChangeEffectGo.transform, false);

      var shockWaveGo = new GameObject("ShockWave");
      createdObjects.Add(shockWaveGo);
      shockWaveGo.transform.SetParent(essenceTotalChangeEffectGo.transform, false);

      var smoke2Go = new GameObject("Smoke");
      createdObjects.Add(smoke2Go);
      smoke2Go.transform.SetParent(essenceTotalChangeEffectGo.transform, false);

      var draftPickObjectLayoutGo = new GameObject("DraftPickObjectLayout");
      createdObjects.Add(draftPickObjectLayoutGo);
      draftPickObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      draftPickObjectLayoutGo.transform.localPosition = new Vector3(0f, 0f, 2.75f);
      draftPickObjectLayoutGo.transform.localScale = new Vector3(0.25f, 0.25f, 0.25f);
      var draftPickObjectLayout = draftPickObjectLayoutGo.AddComponent<SitePickObjectLayout>();
      draftPickObjectLayout._horizontalSpacing = 0.7f;
      draftPickObjectLayout._verticalSpacing = 1.05f;
      draftPickObjectLayout._cardWidth = 2.5f;
      draftPickObjectLayout._cardHeight = 4f;
      draftPickObjectLayout._landscapeScaleOverride = 0.4f;
      draftPickObjectLayout._landscapeHorizontalSpacingOverride = 1.2f;

      var destroyedQuestCardsGo = new GameObject("DestroyedQuestCards");
      createdObjects.Add(destroyedQuestCardsGo);
      destroyedQuestCardsGo.transform.SetParent(mainCameraGo.transform, false);
      destroyedQuestCardsGo.transform.localPosition = new Vector3(0f, 0f, 5f);
      destroyedQuestCardsGo.transform.localScale = new Vector3(0f, 0f, 0f);
      var destroyedQuestCards = destroyedQuestCardsGo.AddComponent<PileObjectLayout>();
      destroyedQuestCards._yMultiplier = 0f;

      var questEffectPositionGo = new GameObject("QuestEffectPosition");
      createdObjects.Add(questEffectPositionGo);
      questEffectPositionGo.transform.SetParent(mainCameraGo.transform, false);
      questEffectPositionGo.transform.localScale = new Vector3(0.2f, 0.2f, 0.2f);
      var questEffectPosition = questEffectPositionGo.AddComponent<CenteredLineObjectLayout>();
      questEffectPosition._horizontalSpacing = 3.5f;
      questEffectPosition._cardWidth = 2.5f;
      questEffectPosition._minScale = 0.01f;
      questEffectPosition._maxScale = 0.2f;

      var shopObjectLayoutGo = new GameObject("ShopObjectLayout");
      createdObjects.Add(shopObjectLayoutGo);
      shopObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      shopObjectLayoutGo.transform.localPosition = new Vector3(-0.2069296f, -0.6909008f, 2.5f);
      shopObjectLayoutGo.transform.localScale = new Vector3(0.15f, 0.15f, 0.15f);
      var shopObjectLayout = shopObjectLayoutGo.AddComponent<SitePickObjectLayout>();
      shopObjectLayout._horizontalSpacing = 0.45f;
      shopObjectLayout._verticalSpacing = 0.7f;
      shopObjectLayout._cardWidth = 2.5f;
      shopObjectLayout._cardHeight = 4f;
      shopObjectLayout._forceTwoRows = true;
      shopObjectLayout._preserveLayoutOnRemoval = true;

      var temptingOfferObjectLayoutGo = new GameObject("TemptingOfferObjectLayout");
      createdObjects.Add(temptingOfferObjectLayoutGo);
      temptingOfferObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      temptingOfferObjectLayoutGo.transform.localPosition = new Vector3(0f, 0f, 2.5f);
      temptingOfferObjectLayoutGo.transform.localScale = new Vector3(0.125f, 0.125f, 0.125f);
      var temptingOfferObjectLayout =
        temptingOfferObjectLayoutGo.AddComponent<TemptingOfferObjectLayout>();
      temptingOfferObjectLayout._horizontalSpacing = 0.5f;
      temptingOfferObjectLayout._verticalSpacing = 0.75f;
      temptingOfferObjectLayout._cardWidth = 2.5f;
      temptingOfferObjectLayout._cardHeight = 2.5f;
      temptingOfferObjectLayout._landscapeScaleOverride = 0.175f;
      temptingOfferObjectLayout._landscapeHorizontalSpacingOverride = 0.63f;
      temptingOfferObjectLayout._preserveLayoutOnRemoval = true;
      temptingOfferObjectLayout._acceptButtonScale = 0.3f;
      temptingOfferObjectLayout._landscapeVerticalSpacingOverride = 1f;

      var journeyObjectLayoutGo = new GameObject("JourneyObjectLayout");
      createdObjects.Add(journeyObjectLayoutGo);
      journeyObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      journeyObjectLayoutGo.transform.localPosition = new Vector3(-0.2069296f, -0.6909008f, 2.5f);
      journeyObjectLayoutGo.transform.localScale = new Vector3(0.15f, 0.15f, 0.15f);
      var journeyObjectLayout = journeyObjectLayoutGo.AddComponent<SitePickObjectLayout>();
      journeyObjectLayout._horizontalSpacing = 0.45f;
      journeyObjectLayout._verticalSpacing = 0.7f;
      journeyObjectLayout._cardWidth = 2.5f;
      journeyObjectLayout._cardHeight = 4f;
      journeyObjectLayout._forceTwoRows = true;
      journeyObjectLayout._preserveLayoutOnRemoval = true;

      var questDeckBrowserWorldSpaceGo = new GameObject("QuestDeckBrowserWorldSpace");
      createdObjects.Add(questDeckBrowserWorldSpaceGo);
      questDeckBrowserWorldSpaceGo.transform.SetParent(mainCameraGo.transform, false);
      questDeckBrowserWorldSpaceGo.transform.localPosition = new Vector3(0f, 0f, 1f);

      var worldSpaceBackgroundGo = new GameObject("WorldSpaceBackground");
      createdObjects.Add(worldSpaceBackgroundGo);
      worldSpaceBackgroundGo.transform.SetParent(questDeckBrowserWorldSpaceGo.transform, false);
      worldSpaceBackgroundGo.transform.localScale = new Vector3(100f, 100f, 1f);

      var startBattleObjectLayoutGo = new GameObject("StartBattleObjectLayout");
      createdObjects.Add(startBattleObjectLayoutGo);
      startBattleObjectLayoutGo.transform.SetParent(mainCameraGo.transform, false);
      startBattleObjectLayoutGo.transform.localPosition = new Vector3(0f, 0.5f, 2.75f);
      var startBattleObjectLayout =
        startBattleObjectLayoutGo.AddComponent<StartBattleObjectLayout>();
      startBattleObjectLayout._internalGameContext = GameContext.Interface;
      startBattleObjectLayout._cardInwardOffsetPortrait = 1.15f;
      startBattleObjectLayout._cardInwardOffsetLandscape = 0.7f;
      startBattleObjectLayout._cardScalePortrait = 0.25f;
      startBattleObjectLayout._cardScaleLandscape = 0.45f;
      startBattleObjectLayout._buttonVerticalOffsetPortrait = -1.5f;
      startBattleObjectLayout._buttonVerticalOffsetLandscape = -1.5f;
      startBattleObjectLayout._buttonScalePortrait = 0.1f;
      startBattleObjectLayout._buttonScaleLandscape = 0.1f;
      startBattleObjectLayout._dreamsignScalePortrait = 0.1f;
      startBattleObjectLayout._dreamsignScaleLandscape = 0.15f;
      startBattleObjectLayout._dreamsignVerticalSpacingPortrait = 0.3f;
      startBattleObjectLayout._dreamsignVerticalSpacingLandscape = 0.45f;
      startBattleObjectLayout._dreamsignColumnSpacingPortrait = 0.3f;
      startBattleObjectLayout._dreamsignColumnSpacingLandscape = 0.45f;
      startBattleObjectLayout._dreamsignOutwardOffsetPortrait = 0.4f;
      startBattleObjectLayout._dreamsignOutwardOffsetLandscape = 0.3f;
      startBattleObjectLayout._dreamsignVerticalOffsetPortrait = -0.75f;
      startBattleObjectLayout._dreamsignVerticalOffsetLandscape = -0.1f;

      var vsTextGo = new GameObject("VsText");
      createdObjects.Add(vsTextGo);
      vsTextGo.transform.SetParent(startBattleObjectLayoutGo.transform, false);
      var vsTextGoRect = vsTextGo.AddComponent<RectTransform>();
      vsTextGoRect.anchorMin = new Vector2(0.5f, 0.5f);
      vsTextGoRect.anchorMax = new Vector2(0.5f, 0.5f);
      vsTextGoRect.sizeDelta = new Vector2(20f, 5f);

      return result;
    }
  }
}
