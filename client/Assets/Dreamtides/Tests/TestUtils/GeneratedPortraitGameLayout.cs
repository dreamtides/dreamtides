// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: PortraitLayout
// Generated at: 2025-12-08 07:44:50

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
  public class GeneratedPortraitGameLayout
  {
    public static GameLayout Create(List<GameObject> createdObjects)
    {
      var layoutGo = new GameObject("PortraitLayout");
      createdObjects.Add(layoutGo);
      var layout = layoutGo.AddComponent<GameLayout>();

      var cameraPositionGo = new GameObject("PortraitBattleCamera");
      createdObjects.Add(cameraPositionGo);
      cameraPositionGo.transform.SetParent(layoutGo.transform, false);
      cameraPositionGo.transform.localPosition = new Vector3(0f, 30f, -25f);
      cameraPositionGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      layout._cameraPosition = cameraPositionGo.transform;
      var cinemachineCamera = cameraPositionGo.AddComponent<CinemachineCamera>();
      layout._battleCamera = cinemachineCamera;

      var battleCameraBoundsGo = new GameObject("Bounds");
      createdObjects.Add(battleCameraBoundsGo);
      battleCameraBoundsGo.transform.SetParent(layoutGo.transform, false);
      var battleCameraBounds1 = battleCameraBoundsGo.AddComponent<BattleCameraBounds>();

      var bottomLeftAnchorGo = new GameObject("BottomLeft");
      createdObjects.Add(bottomLeftAnchorGo);
      bottomLeftAnchorGo.transform.SetParent(battleCameraBoundsGo.transform, false);
      bottomLeftAnchorGo.transform.localPosition = new Vector3(-7.5f, 0.5f, -27f);
      bottomLeftAnchorGo.transform.localScale = new Vector3(0.1f, 0.1f, 0.1f);
      battleCameraBounds1._bottomLeftAnchor = bottomLeftAnchorGo.transform;

      var topLeftAnchorGo = new GameObject("TopLeft");
      createdObjects.Add(topLeftAnchorGo);
      topLeftAnchorGo.transform.SetParent(battleCameraBoundsGo.transform, false);
      topLeftAnchorGo.transform.localPosition = new Vector3(-8.8f, 1.93f, -0.5f);
      topLeftAnchorGo.transform.localScale = new Vector3(0.1f, 0.1f, 0.1f);
      battleCameraBounds1._topLeftAnchor = topLeftAnchorGo.transform;
      layout._battleCameraBounds = battleCameraBounds1;

      var userHandGo = new GameObject("UserHand");
      createdObjects.Add(userHandGo);
      userHandGo.transform.SetParent(layoutGo.transform, false);
      userHandGo.transform.localPosition = new Vector3(0f, 30f, -25f);
      userHandGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var userHand = userHandGo.AddComponent<UserHandLayout>();
      userHand._useSecondLayoutAfter = 5;
      userHand._useBrowserAfter = 10;

      var layout1Go = new GameObject("UserHandRow1");
      createdObjects.Add(layout1Go);
      layout1Go.transform.SetParent(userHandGo.transform, false);
      layout1Go.transform.localPosition = new Vector3(0f, -5f, 12f);
      var layout1 = layout1Go.AddComponent<CurveObjectLayout>();
      layout1._zRotationMultiplier = 1f;
      layout1._portraitLayout = true;
      var layout1StaticGameContext = layout1Go.AddComponent<StaticGameContext>();
      layout1StaticGameContext._startingContext = GameContext.Hand;

      var controlPoint1Go = new GameObject("Control1");
      createdObjects.Add(controlPoint1Go);
      controlPoint1Go.transform.SetParent(layout1Go.transform, false);
      controlPoint1Go.transform.localPosition = new Vector3(-1.815559f, -0.25f, 0f);
      layout1._controlPoint1 = controlPoint1Go.transform;

      var controlPoint2Go = new GameObject("Control2");
      createdObjects.Add(controlPoint2Go);
      controlPoint2Go.transform.SetParent(layout1Go.transform, false);
      controlPoint2Go.transform.localPosition = new Vector3(-1f, 0f, 0f);
      layout1._controlPoint2 = controlPoint2Go.transform;

      var controlPoint3Go = new GameObject("Control3");
      createdObjects.Add(controlPoint3Go);
      controlPoint3Go.transform.SetParent(layout1Go.transform, false);
      controlPoint3Go.transform.localPosition = new Vector3(1f, 0f, 0f);
      layout1._controlPoint3 = controlPoint3Go.transform;

      var controlPoint4Go = new GameObject("Control4");
      createdObjects.Add(controlPoint4Go);
      controlPoint4Go.transform.SetParent(layout1Go.transform, false);
      controlPoint4Go.transform.localPosition = new Vector3(1.815559f, -0.25f, 0f);
      layout1._controlPoint4 = controlPoint4Go.transform;
      userHand._layout1 = layout1;

      var layout2Go = new GameObject("UserHandRow2");
      createdObjects.Add(layout2Go);
      layout2Go.transform.SetParent(userHandGo.transform, false);
      layout2Go.transform.localPosition = new Vector3(0f, -6f, 11.5f);
      var layout2 = layout2Go.AddComponent<CurveObjectLayout>();
      layout2._zRotationMultiplier = 1f;
      layout2._portraitLayout = true;
      var layout2StaticGameContext = layout2Go.AddComponent<StaticGameContext>();
      layout2StaticGameContext._startingContext = GameContext.Hand;

      var controlPoint11Go = new GameObject("Control1");
      createdObjects.Add(controlPoint11Go);
      controlPoint11Go.transform.SetParent(layout2Go.transform, false);
      controlPoint11Go.transform.localPosition = new Vector3(-1.75f, -0.25f, 0f);
      layout2._controlPoint1 = controlPoint11Go.transform;

      var controlPoint21Go = new GameObject("Control2");
      createdObjects.Add(controlPoint21Go);
      controlPoint21Go.transform.SetParent(layout2Go.transform, false);
      controlPoint21Go.transform.localPosition = new Vector3(-1f, 0f, 0f);
      layout2._controlPoint2 = controlPoint21Go.transform;

      var controlPoint31Go = new GameObject("Control3");
      createdObjects.Add(controlPoint31Go);
      controlPoint31Go.transform.SetParent(layout2Go.transform, false);
      controlPoint31Go.transform.localPosition = new Vector3(1f, 0f, 0f);
      layout2._controlPoint3 = controlPoint31Go.transform;

      var controlPoint41Go = new GameObject("Control4");
      createdObjects.Add(controlPoint41Go);
      controlPoint41Go.transform.SetParent(layout2Go.transform, false);
      controlPoint41Go.transform.localPosition = new Vector3(1.75f, -0.25f, 0f);
      layout2._controlPoint4 = controlPoint41Go.transform;
      userHand._layout2 = layout2;

      var scrollableHandGo = new GameObject("ScrollableHand");
      createdObjects.Add(scrollableHandGo);
      scrollableHandGo.transform.SetParent(userHandGo.transform, false);
      scrollableHandGo.transform.localPosition = new Vector3(1f, -7f, 16f);
      var scrollableHand = scrollableHandGo.AddComponent<StaticGameContext>();
      scrollableHand._startingContext = GameContext.Hand;
      var scrollableHandScrollableUserHandLayout =
        scrollableHandGo.AddComponent<ScrollableUserHandLayout>();
      scrollableHandScrollableUserHandLayout._offset = 2f;
      scrollableHandScrollableUserHandLayout._cardWidth = 1.7f;

      var scrollbarGo = new GameObject("UserHandScrollbar");
      createdObjects.Add(scrollbarGo);
      scrollbarGo.transform.localPosition = new Vector3(0f, -115f, 0f);
      var scrollbar1 = scrollbarGo.AddComponent<Scrollbar>();
      scrollableHandScrollableUserHandLayout._scrollbar = scrollbar1;

      var leftEdgeGo = new GameObject("Left");
      createdObjects.Add(leftEdgeGo);
      leftEdgeGo.transform.SetParent(scrollableHandGo.transform, false);
      scrollableHandScrollableUserHandLayout._leftEdge = leftEdgeGo.transform;

      var rightEdgeGo = new GameObject("Right");
      createdObjects.Add(rightEdgeGo);
      rightEdgeGo.transform.SetParent(scrollableHandGo.transform, false);
      scrollableHandScrollableUserHandLayout._rightEdge = rightEdgeGo.transform;
      userHand._scrollableHand = scrollableHandScrollableUserHandLayout;
      layout._userHand = userHand;

      var enemyHandGo = new GameObject("EnemyHand");
      createdObjects.Add(enemyHandGo);
      enemyHandGo.transform.SetParent(layoutGo.transform, false);
      enemyHandGo.transform.localPosition = new Vector3(0f, 2.5f, 0f);
      enemyHandGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var enemyHand = enemyHandGo.AddComponent<StaticGameContext>();
      enemyHand._startingContext = GameContext.Hand;
      var enemyHandEnemyHandLayout = enemyHandGo.AddComponent<EnemyHandLayout>();

      var controlPoint12Go = new GameObject("Control1");
      createdObjects.Add(controlPoint12Go);
      controlPoint12Go.transform.SetParent(enemyHandGo.transform, false);
      controlPoint12Go.transform.localPosition = new Vector3(-6f, 1f, 0f);
      enemyHandEnemyHandLayout._controlPoint1 = controlPoint12Go.transform;

      var controlPoint22Go = new GameObject("Control2");
      createdObjects.Add(controlPoint22Go);
      controlPoint22Go.transform.SetParent(enemyHandGo.transform, false);
      controlPoint22Go.transform.localPosition = new Vector3(-2f, 0f, 0f);
      enemyHandEnemyHandLayout._controlPoint2 = controlPoint22Go.transform;

      var controlPoint32Go = new GameObject("Control3");
      createdObjects.Add(controlPoint32Go);
      controlPoint32Go.transform.SetParent(enemyHandGo.transform, false);
      controlPoint32Go.transform.localPosition = new Vector3(2f, 0f, 0f);
      enemyHandEnemyHandLayout._controlPoint3 = controlPoint32Go.transform;

      var controlPoint42Go = new GameObject("Control4");
      createdObjects.Add(controlPoint42Go);
      controlPoint42Go.transform.SetParent(enemyHandGo.transform, false);
      controlPoint42Go.transform.localPosition = new Vector3(6f, 1f, 0f);
      enemyHandEnemyHandLayout._controlPoint4 = controlPoint42Go.transform;
      layout._enemyHand = enemyHandEnemyHandLayout;

      var userDeckGo = new GameObject("UserDeck");
      createdObjects.Add(userDeckGo);
      userDeckGo.transform.SetParent(layoutGo.transform, false);
      userDeckGo.transform.localPosition = new Vector3(6f, 0f, -22.5f);
      userDeckGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var userDeck = userDeckGo.AddComponent<StaticGameContext>();
      userDeck._startingContext = GameContext.Deck;
      var userDeckPileObjectLayout = userDeckGo.AddComponent<PileObjectLayout>();

      var displayEffectPositionGo = new GameObject("EffectPosition");
      createdObjects.Add(displayEffectPositionGo);
      displayEffectPositionGo.transform.SetParent(userDeckGo.transform, false);
      displayEffectPositionGo.transform.localPosition = new Vector3(-0.25f, -0.39f, -2f);
      userDeckPileObjectLayout._displayEffectPosition = displayEffectPositionGo.transform;
      layout._userDeck = userDeckPileObjectLayout;

      var enemyDeckGo = new GameObject("EnemyDeck");
      createdObjects.Add(enemyDeckGo);
      enemyDeckGo.transform.SetParent(layoutGo.transform, false);
      enemyDeckGo.transform.localPosition = new Vector3(-6f, 0f, -4f);
      enemyDeckGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var enemyDeck = enemyDeckGo.AddComponent<StaticGameContext>();
      enemyDeck._startingContext = GameContext.Deck;
      var enemyDeckPileObjectLayout = enemyDeckGo.AddComponent<PileObjectLayout>();

      var displayEffectPosition1Go = new GameObject("EffectPosition");
      createdObjects.Add(displayEffectPosition1Go);
      displayEffectPosition1Go.transform.SetParent(enemyDeckGo.transform, false);
      displayEffectPosition1Go.transform.localPosition = new Vector3(0.3f, -1.1f, -2f);
      enemyDeckPileObjectLayout._displayEffectPosition = displayEffectPosition1Go.transform;
      layout._enemyDeck = enemyDeckPileObjectLayout;

      var userBattlefieldGo = new GameObject("UserBattlefield");
      createdObjects.Add(userBattlefieldGo);
      userBattlefieldGo.transform.SetParent(layoutGo.transform, false);
      userBattlefieldGo.transform.localPosition = new Vector3(0.5f, 0f, -16f);
      userBattlefieldGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var userBattlefield = userBattlefieldGo.AddComponent<RectangularObjectLayout>();
      userBattlefield._width = 15f;
      userBattlefield._height = 6f;
      userBattlefield._itemHorizontalSpacing = 0.5f;
      userBattlefield._itemVerticalSpacing = 0.5f;
      userBattlefield._itemWidth = 2.5f;
      userBattlefield._itemHeight = 3f;
      userBattlefield._rowCount = 4;
      var userBattlefieldStaticGameContext = userBattlefieldGo.AddComponent<StaticGameContext>();
      userBattlefieldStaticGameContext._startingContext = GameContext.Battlefield;
      layout._userBattlefield = userBattlefield;

      var enemyBattlefieldGo = new GameObject("EnemyBattlefield");
      createdObjects.Add(enemyBattlefieldGo);
      enemyBattlefieldGo.transform.SetParent(layoutGo.transform, false);
      enemyBattlefieldGo.transform.localPosition = new Vector3(0.5f, 0.25f, -8.99f);
      enemyBattlefieldGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var enemyBattlefield = enemyBattlefieldGo.AddComponent<RectangularObjectLayout>();
      enemyBattlefield._width = 15f;
      enemyBattlefield._height = 6f;
      enemyBattlefield._itemHorizontalSpacing = 0.5f;
      enemyBattlefield._itemVerticalSpacing = 0.5f;
      enemyBattlefield._itemWidth = 2.5f;
      enemyBattlefield._itemHeight = 3f;
      enemyBattlefield._rowCount = 4;
      var enemyBattlefieldStaticGameContext = enemyBattlefieldGo.AddComponent<StaticGameContext>();
      enemyBattlefieldStaticGameContext._startingContext = GameContext.Battlefield;
      layout._enemyBattlefield = enemyBattlefield;

      var userVoidGo = new GameObject("UserVoid");
      createdObjects.Add(userVoidGo);
      userVoidGo.transform.SetParent(layoutGo.transform, false);
      userVoidGo.transform.localPosition = new Vector3(3f, 0f, -23f);
      userVoidGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var userVoid = userVoidGo.AddComponent<StaticGameContext>();
      userVoid._startingContext = GameContext.DiscardPile;
      var userVoidPileObjectLayout = userVoidGo.AddComponent<PileObjectLayout>();
      layout._userVoid = userVoidPileObjectLayout;

      var enemyVoidGo = new GameObject("EnemyVoid");
      createdObjects.Add(enemyVoidGo);
      enemyVoidGo.transform.SetParent(layoutGo.transform, false);
      enemyVoidGo.transform.localPosition = new Vector3(-3f, 0f, -4.5f);
      enemyVoidGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var enemyVoid = enemyVoidGo.AddComponent<StaticGameContext>();
      enemyVoid._startingContext = GameContext.DiscardPile;
      var enemyVoidPileObjectLayout = enemyVoidGo.AddComponent<PileObjectLayout>();
      layout._enemyVoid = enemyVoidPileObjectLayout;

      var userStatusDisplayGo = new GameObject("UserStatus");
      createdObjects.Add(userStatusDisplayGo);
      userStatusDisplayGo.transform.SetParent(layoutGo.transform, false);
      userStatusDisplayGo.transform.localPosition = new Vector3(0f, 0.3f, -22.9f);
      userStatusDisplayGo.transform.localScale = new Vector3(0.9f, 0.9f, 0.9f);
      var userStatusDisplay = userStatusDisplayGo.AddComponent<StaticGameContext>();
      userStatusDisplay._startingContext = GameContext.PlayerStatus;
      var userStatusDisplayPlayerStatusDisplay =
        userStatusDisplayGo.AddComponent<PlayerStatusDisplay>();
      userStatusDisplayPlayerStatusDisplay._studioType = StudioType.UserStatus;

      var energyGo = new GameObject("Energy");
      createdObjects.Add(energyGo);
      energyGo.transform.SetParent(userStatusDisplayGo.transform, false);
      energyGo.transform.localPosition = new Vector3(0f, 0f, -0.265f);
      energyGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var battlefieldNumber = energyGo.AddComponent<BattlefieldNumber>();

      var textGo = new GameObject("Text (TMP)");
      createdObjects.Add(textGo);
      textGo.transform.SetParent(energyGo.transform, false);
      textGo.transform.localPosition = new Vector3(-0.0169f, -0.447f, -0.01f);
      var textMeshPro = textGo.AddComponent<TextMeshPro>();
      battlefieldNumber._text = textMeshPro;

      var onChangeGo = new GameObject("Hit 2");
      createdObjects.Add(onChangeGo);
      onChangeGo.transform.SetParent(energyGo.transform, false);
      onChangeGo.transform.localPosition = new Vector3(-0.72f, -0.8800001f, -0.4999999f);
      onChangeGo.transform.localScale = new Vector3(2f, 3.200001f, 1f);
      var timedEffect = onChangeGo.AddComponent<TimedEffect>();
      battlefieldNumber._onChange = timedEffect;
      userStatusDisplayPlayerStatusDisplay._energy = battlefieldNumber;

      var scoreGo = new GameObject("Score");
      createdObjects.Add(scoreGo);
      scoreGo.transform.SetParent(userStatusDisplayGo.transform, false);
      scoreGo.transform.localPosition = new Vector3(0f, 0.01f, -1.637f);
      var battlefieldNumber1 = scoreGo.AddComponent<BattlefieldNumber>();

      var text1Go = new GameObject("ScoreText");
      createdObjects.Add(text1Go);
      text1Go.transform.SetParent(scoreGo.transform, false);
      text1Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var textMeshPro1 = text1Go.AddComponent<TextMeshPro>();
      battlefieldNumber1._text = textMeshPro1;

      var onChange1Go = new GameObject("Hit 5");
      createdObjects.Add(onChange1Go);
      onChange1Go.transform.SetParent(scoreGo.transform, false);
      onChange1Go.transform.localPosition = new Vector3(0.65f, 0.4f, -0.94f);
      onChange1Go.transform.localRotation = Quaternion.Euler(0f, 0f, 0f);
      onChange1Go.transform.localScale = new Vector3(2f, 1f, 3.200002f);
      var timedEffect1 = onChange1Go.AddComponent<TimedEffect>();
      battlefieldNumber1._onChange = timedEffect1;
      userStatusDisplayPlayerStatusDisplay._score = battlefieldNumber1;

      var totalSparkGo = new GameObject("UserSparkTotal");
      createdObjects.Add(totalSparkGo);
      totalSparkGo.transform.SetParent(layoutGo.transform, false);
      totalSparkGo.transform.localPosition = new Vector3(-6.87f, 1.45f, -15.01f);
      totalSparkGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      totalSparkGo.transform.localScale = new Vector3(2f, 2f, 2f);
      var battlefieldNumber2 = totalSparkGo.AddComponent<BattlefieldNumber>();

      var text2Go = new GameObject("SparkText");
      createdObjects.Add(text2Go);
      text2Go.transform.SetParent(totalSparkGo.transform, false);
      text2Go.transform.localPosition = new Vector3(0f, 0f, -0.1f);
      var textMeshPro2 = text2Go.AddComponent<TextMeshPro>();
      battlefieldNumber2._text = textMeshPro2;

      var onChange2Go = new GameObject("Hit 1");
      createdObjects.Add(onChange2Go);
      onChange2Go.transform.SetParent(totalSparkGo.transform, false);
      onChange2Go.transform.localPosition = new Vector3(0f, 0f, -0.25f);
      var timedEffect2 = onChange2Go.AddComponent<TimedEffect>();
      battlefieldNumber2._onChange = timedEffect2;
      userStatusDisplayPlayerStatusDisplay._totalSpark = battlefieldNumber2;

      var characterImageGo = new GameObject("Image");
      createdObjects.Add(characterImageGo);
      characterImageGo.transform.SetParent(userStatusDisplayGo.transform, false);
      characterImageGo.transform.localPosition = new Vector3(0f, 0.1f, 0.84f);
      characterImageGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      characterImageGo.transform.localScale = new Vector3(2f, 2f, 2f);
      var meshRenderer = characterImageGo.AddComponent<MeshRenderer>();
      userStatusDisplayPlayerStatusDisplay._characterImage = meshRenderer;
      layout._userStatusDisplay = userStatusDisplayPlayerStatusDisplay;

      var enemyStatusDisplayGo = new GameObject("EnemyStatus");
      createdObjects.Add(enemyStatusDisplayGo);
      enemyStatusDisplayGo.transform.SetParent(layoutGo.transform, false);
      enemyStatusDisplayGo.transform.localPosition = new Vector3(0f, 0.75f, -4.5f);
      enemyStatusDisplayGo.transform.localRotation = Quaternion.Euler(350f, 0f, 0f);
      enemyStatusDisplayGo.transform.localScale = new Vector3(0.9f, 0.8999999f, 0.8999999f);
      var enemyStatusDisplay = enemyStatusDisplayGo.AddComponent<StaticGameContext>();
      enemyStatusDisplay._startingContext = GameContext.PlayerStatus;
      var enemyStatusDisplayPlayerStatusDisplay =
        enemyStatusDisplayGo.AddComponent<PlayerStatusDisplay>();
      enemyStatusDisplayPlayerStatusDisplay._studioType = StudioType.EnemyStatus;

      var energy1Go = new GameObject("Energy");
      createdObjects.Add(energy1Go);
      energy1Go.transform.SetParent(enemyStatusDisplayGo.transform, false);
      energy1Go.transform.localPosition = new Vector3(0f, 0f, -0.265f);
      energy1Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var battlefieldNumber3 = energy1Go.AddComponent<BattlefieldNumber>();

      var text3Go = new GameObject("Text (TMP)");
      createdObjects.Add(text3Go);
      text3Go.transform.SetParent(energy1Go.transform, false);
      text3Go.transform.localPosition = new Vector3(-0.0169f, -0.447f, -0.01f);
      var textMeshPro3 = text3Go.AddComponent<TextMeshPro>();
      battlefieldNumber3._text = textMeshPro3;

      var onChange3Go = new GameObject("Hit 2");
      createdObjects.Add(onChange3Go);
      onChange3Go.transform.SetParent(energy1Go.transform, false);
      onChange3Go.transform.localPosition = new Vector3(-0.72f, -0.8800001f, -0.4999999f);
      onChange3Go.transform.localScale = new Vector3(2f, 3.200001f, 1f);
      var timedEffect3 = onChange3Go.AddComponent<TimedEffect>();
      battlefieldNumber3._onChange = timedEffect3;
      enemyStatusDisplayPlayerStatusDisplay._energy = battlefieldNumber3;

      var score1Go = new GameObject("Score");
      createdObjects.Add(score1Go);
      score1Go.transform.SetParent(enemyStatusDisplayGo.transform, false);
      score1Go.transform.localPosition = new Vector3(0f, 0.01f, -1.637f);
      var battlefieldNumber4 = score1Go.AddComponent<BattlefieldNumber>();

      var text4Go = new GameObject("ScoreText");
      createdObjects.Add(text4Go);
      text4Go.transform.SetParent(score1Go.transform, false);
      text4Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var textMeshPro4 = text4Go.AddComponent<TextMeshPro>();
      battlefieldNumber4._text = textMeshPro4;

      var onChange4Go = new GameObject("Hit 5");
      createdObjects.Add(onChange4Go);
      onChange4Go.transform.SetParent(score1Go.transform, false);
      onChange4Go.transform.localPosition = new Vector3(0.65f, 0.4f, -0.94f);
      onChange4Go.transform.localRotation = Quaternion.Euler(0f, 0f, 0f);
      onChange4Go.transform.localScale = new Vector3(2f, 1f, 3.200002f);
      var timedEffect4 = onChange4Go.AddComponent<TimedEffect>();
      battlefieldNumber4._onChange = timedEffect4;
      enemyStatusDisplayPlayerStatusDisplay._score = battlefieldNumber4;

      var totalSpark1Go = new GameObject("EnemySparkTotal");
      createdObjects.Add(totalSpark1Go);
      totalSpark1Go.transform.SetParent(layoutGo.transform, false);
      totalSpark1Go.transform.localPosition = new Vector3(-6.87f, 1.13f, -12.78f);
      totalSpark1Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      totalSpark1Go.transform.localScale = new Vector3(2f, 2f, 2f);
      var battlefieldNumber5 = totalSpark1Go.AddComponent<BattlefieldNumber>();

      var text5Go = new GameObject("SparkText");
      createdObjects.Add(text5Go);
      text5Go.transform.SetParent(totalSpark1Go.transform, false);
      text5Go.transform.localPosition = new Vector3(0f, 0f, -0.1f);
      var textMeshPro5 = text5Go.AddComponent<TextMeshPro>();
      battlefieldNumber5._text = textMeshPro5;

      var onChange5Go = new GameObject("Hit 1");
      createdObjects.Add(onChange5Go);
      onChange5Go.transform.SetParent(totalSpark1Go.transform, false);
      onChange5Go.transform.localPosition = new Vector3(0f, 0f, -0.25f);
      var timedEffect5 = onChange5Go.AddComponent<TimedEffect>();
      battlefieldNumber5._onChange = timedEffect5;
      enemyStatusDisplayPlayerStatusDisplay._totalSpark = battlefieldNumber5;

      var characterImage1Go = new GameObject("Image");
      createdObjects.Add(characterImage1Go);
      characterImage1Go.transform.SetParent(enemyStatusDisplayGo.transform, false);
      characterImage1Go.transform.localPosition = new Vector3(0f, 0.1f, 0.84f);
      characterImage1Go.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      characterImage1Go.transform.localScale = new Vector3(2f, 2f, 2f);
      var meshRenderer1 = characterImage1Go.AddComponent<MeshRenderer>();
      enemyStatusDisplayPlayerStatusDisplay._characterImage = meshRenderer1;
      layout._enemyStatusDisplay = enemyStatusDisplayPlayerStatusDisplay;

      var offscreenGo = new GameObject("Offscreen");
      createdObjects.Add(offscreenGo);
      offscreenGo.transform.SetParent(layoutGo.transform, false);
      offscreenGo.transform.localPosition = new Vector3(100f, 100f, 100f);
      var offscreen = offscreenGo.AddComponent<PileObjectLayout>();
      var offscreenStaticGameContext = offscreenGo.AddComponent<StaticGameContext>();
      offscreenStaticGameContext._startingContext = GameContext.Hidden;
      layout._offscreen = offscreen;

      var drawnCardsPositionGo = new GameObject("DrawnCardsPosition");
      createdObjects.Add(drawnCardsPositionGo);
      drawnCardsPositionGo.transform.SetParent(cameraPositionGo.transform, false);
      drawnCardsPositionGo.transform.localPosition = new Vector3(1.29f, 0.65f, 10.21f);
      var drawnCardsPosition = drawnCardsPositionGo.AddComponent<PileObjectLayout>();
      var drawnCardsPositionStaticGameContext =
        drawnCardsPositionGo.AddComponent<StaticGameContext>();
      drawnCardsPositionStaticGameContext._startingContext = GameContext.DrawnCards;
      layout._drawnCardsPosition = drawnCardsPosition;

      var defaultStackGo = new GameObject("DefaultStack");
      createdObjects.Add(defaultStackGo);
      defaultStackGo.transform.localPosition = new Vector3(1.5f, 18.5f, -20f);
      defaultStackGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var defaultStack = defaultStackGo.AddComponent<StaticGameContext>();
      defaultStack._startingContext = GameContext.Stack;
      var defaultStackStackingObjectLayout = defaultStackGo.AddComponent<StackingObjectLayout>();
      defaultStackStackingObjectLayout._offset = 1.5f;
      defaultStackStackingObjectLayout._shrinkOffset = 0.75f;
      defaultStackStackingObjectLayout._shrinkOffsetThreshold = 4;
      layout._defaultStack = defaultStackStackingObjectLayout;

      var targetingUserStackGo = new GameObject("TargetingUser");
      createdObjects.Add(targetingUserStackGo);
      targetingUserStackGo.transform.localPosition = new Vector3(3f, 10f, -12f);
      targetingUserStackGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      targetingUserStackGo.transform.localScale = new Vector3(1.5f, 1.5f, 1.5f);
      var targetingUserStack = targetingUserStackGo.AddComponent<StaticGameContext>();
      targetingUserStack._startingContext = GameContext.Stack;
      var targetingUserStackStackingObjectLayout =
        targetingUserStackGo.AddComponent<StackingObjectLayout>();
      targetingUserStackStackingObjectLayout._offset = 1.5f;
      targetingUserStackStackingObjectLayout._shrinkOffset = 0.75f;
      targetingUserStackStackingObjectLayout._shrinkOffsetThreshold = 4;
      layout._targetingUserStack = targetingUserStackStackingObjectLayout;

      var targetingEnemyStackGo = new GameObject("TargetingEnemy");
      createdObjects.Add(targetingEnemyStackGo);
      targetingEnemyStackGo.transform.localPosition = new Vector3(3f, 10f, -21f);
      targetingEnemyStackGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      targetingEnemyStackGo.transform.localScale = new Vector3(1.5f, 1.5f, 1.5f);
      var targetingEnemyStack = targetingEnemyStackGo.AddComponent<StaticGameContext>();
      targetingEnemyStack._startingContext = GameContext.Stack;
      var targetingEnemyStackStackingObjectLayout =
        targetingEnemyStackGo.AddComponent<StackingObjectLayout>();
      targetingEnemyStackStackingObjectLayout._offset = 1.5f;
      targetingEnemyStackStackingObjectLayout._shrinkOffset = 0.75f;
      targetingEnemyStackStackingObjectLayout._shrinkOffsetThreshold = 4;
      layout._targetingEnemyStack = targetingEnemyStackStackingObjectLayout;

      var targetingBothStackGo = new GameObject("TargetingBoth");
      createdObjects.Add(targetingBothStackGo);
      targetingBothStackGo.transform.localPosition = new Vector3(4.5f, 2f, -22.5f);
      targetingBothStackGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var targetingBothStack = targetingBothStackGo.AddComponent<StaticGameContext>();
      targetingBothStack._startingContext = GameContext.Stack;
      var targetingBothStackStackingObjectLayout =
        targetingBothStackGo.AddComponent<StackingObjectLayout>();
      targetingBothStackStackingObjectLayout._offset = 1f;
      targetingBothStackStackingObjectLayout._shrinkOffset = 0.75f;
      targetingBothStackStackingObjectLayout._shrinkOffsetThreshold = 4;
      layout._targetingBothStack = targetingBothStackStackingObjectLayout;

      var battlefieldOverlayGo = new GameObject("BrowserBackground");
      createdObjects.Add(battlefieldOverlayGo);
      battlefieldOverlayGo.transform.SetParent(layoutGo.transform, false);
      battlefieldOverlayGo.transform.localPosition = new Vector3(0f, 15f, -15f);
      battlefieldOverlayGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      battlefieldOverlayGo.transform.localScale = new Vector3(100f, 100f, 1f);
      var spriteRenderer = battlefieldOverlayGo.AddComponent<SpriteRenderer>();
      layout._battlefieldOverlay = spriteRenderer;

      var gameMessageGo = new GameObject("GameMessage");
      createdObjects.Add(gameMessageGo);
      gameMessageGo.transform.SetParent(cameraPositionGo.transform, false);
      gameMessageGo.transform.localPosition = new Vector3(0f, 0.88f, 27.73f);
      gameMessageGo.transform.localRotation = Quaternion.Euler(90f, 180f, 0f);
      var gameMessage1 = gameMessageGo.AddComponent<GameMessage>();

      var topGo = new GameObject("Top");
      createdObjects.Add(topGo);
      topGo.transform.SetParent(gameMessageGo.transform, false);
      topGo.transform.localPosition = new Vector3(0f, 5f, -8f);
      gameMessage1._top = topGo.transform;
      layout._gameMessage = gameMessage1;

      var infoZoomLeftGo = new GameObject("InfoZoomLeftContainer");
      createdObjects.Add(infoZoomLeftGo);
      infoZoomLeftGo.transform.localPosition = new Vector3(1.25f, 0f, -1.5f);
      infoZoomLeftGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      layout._infoZoomLeft = infoZoomLeftGo.transform;

      var infoZoomRightGo = new GameObject("InfoZoomRightContainer");
      createdObjects.Add(infoZoomRightGo);
      infoZoomRightGo.transform.localPosition = new Vector3(-1.25f, 0f, -1.5f);
      infoZoomRightGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      layout._infoZoomRight = infoZoomRightGo.transform;

      var supplementalCardInfoLeftGo = new GameObject("LeftContainerSupplemental");
      createdObjects.Add(supplementalCardInfoLeftGo);
      supplementalCardInfoLeftGo.transform.SetParent(infoZoomLeftGo.transform, false);
      supplementalCardInfoLeftGo.transform.localPosition = new Vector3(1.3f, 1.4f, 0f);
      supplementalCardInfoLeftGo.transform.localRotation = Quaternion.Euler(285f, 0f, 0f);
      layout._supplementalCardInfoLeft = supplementalCardInfoLeftGo.transform;

      var supplementalCardInfoRightGo = new GameObject("RightContainerSupplemental");
      createdObjects.Add(supplementalCardInfoRightGo);
      supplementalCardInfoRightGo.transform.SetParent(infoZoomRightGo.transform, false);
      supplementalCardInfoRightGo.transform.localPosition = new Vector3(-1.3f, 1.4f, 0f);
      supplementalCardInfoRightGo.transform.localRotation = Quaternion.Euler(285f, 0f, 0f);
      layout._supplementalCardInfoRight = supplementalCardInfoRightGo.transform;

      var browserGo = new GameObject("CardBrowser");
      createdObjects.Add(browserGo);
      browserGo.transform.SetParent(layoutGo.transform, false);
      browserGo.transform.localPosition = new Vector3(0f, 8f, -20f);
      browserGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var browser = browserGo.AddComponent<StaticGameContext>();
      browser._startingContext = GameContext.Browser;
      var browserCardBrowser = browserGo.AddComponent<CardBrowser>();

      var leftEdge1Go = new GameObject("CardBrowserLeft");
      createdObjects.Add(leftEdge1Go);
      leftEdge1Go.transform.SetParent(browserGo.transform, false);
      leftEdge1Go.transform.localPosition = new Vector3(-6f, 0f, 0f);
      browserCardBrowser._leftEdge = leftEdge1Go.transform;

      var rightEdge1Go = new GameObject("CardBrowserRight");
      createdObjects.Add(rightEdge1Go);
      rightEdge1Go.transform.SetParent(browserGo.transform, false);
      rightEdge1Go.transform.localPosition = new Vector3(6f, 0f, 0f);
      browserCardBrowser._rightEdge = rightEdge1Go.transform;

      var scrollbar2Go = new GameObject("CardBrowserScrollbarPortrait");
      createdObjects.Add(scrollbar2Go);
      scrollbar2Go.transform.localPosition = new Vector3(0f, -30f, 0f);
      var scrollbar3 = scrollbar2Go.AddComponent<Scrollbar>();
      browserCardBrowser._scrollbar = scrollbar3;

      var closeButtonGo = new GameObject("CloseBrowserButton");
      createdObjects.Add(closeButtonGo);
      closeButtonGo.transform.localPosition = new Vector3(-15f, 20f, 0f);
      var closeBrowserButton = closeButtonGo.AddComponent<CloseBrowserButton>();
      browserCardBrowser._closeButton = closeBrowserButton;

      var largeCardPositionGo = new GameObject("LargeCardPosition");
      createdObjects.Add(largeCardPositionGo);
      largeCardPositionGo.transform.SetParent(browserGo.transform, false);
      largeCardPositionGo.transform.localPosition = new Vector3(0f, 1.5f, -15f);
      browserCardBrowser._largeCardPosition = largeCardPositionGo.transform;

      var twoCardsPositionGo = new GameObject("TwoCardsPosition");
      createdObjects.Add(twoCardsPositionGo);
      twoCardsPositionGo.transform.SetParent(browserGo.transform, false);
      twoCardsPositionGo.transform.localPosition = new Vector3(0f, 1.5f, -12.25f);
      browserCardBrowser._twoCardsPosition = twoCardsPositionGo.transform;
      layout._browser = browserCardBrowser;
      var backgroundOverlay = battlefieldOverlayGo.AddComponent<BackgroundOverlay>();
      backgroundOverlay._overlay = spriteRenderer;
      layout._browserBackground = backgroundOverlay;

      var userDreamwellGo = new GameObject("UserDreamwell");
      createdObjects.Add(userDreamwellGo);
      userDreamwellGo.transform.SetParent(layoutGo.transform, false);
      userDreamwellGo.transform.localPosition = new Vector3(0f, 5f, -35f);
      userDreamwellGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var userDreamwell = userDreamwellGo.AddComponent<PileObjectLayout>();
      var userDreamwellStaticGameContext = userDreamwellGo.AddComponent<StaticGameContext>();
      userDreamwellStaticGameContext._startingContext = GameContext.Hidden;
      layout._userDreamwell = userDreamwell;

      var enemyDreamwellGo = new GameObject("EnemyDreamwell");
      createdObjects.Add(enemyDreamwellGo);
      enemyDreamwellGo.transform.SetParent(layoutGo.transform, false);
      enemyDreamwellGo.transform.localPosition = new Vector3(0f, 8f, 3f);
      enemyDreamwellGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var enemyDreamwell = enemyDreamwellGo.AddComponent<PileObjectLayout>();
      var enemyDreamwellStaticGameContext = enemyDreamwellGo.AddComponent<StaticGameContext>();
      enemyDreamwellStaticGameContext._startingContext = GameContext.Hidden;
      layout._enemyDreamwell = enemyDreamwell;

      var dreamwellActivationGo = new GameObject("DreamwellActivation");
      createdObjects.Add(dreamwellActivationGo);
      dreamwellActivationGo.transform.SetParent(layoutGo.transform, false);
      dreamwellActivationGo.transform.localPosition = new Vector3(0f, 5f, -15f);
      dreamwellActivationGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      var dreamwellActivation = dreamwellActivationGo.AddComponent<PileObjectLayout>();
      var dreamwellActivationStaticGameContext =
        dreamwellActivationGo.AddComponent<StaticGameContext>();
      dreamwellActivationStaticGameContext._startingContext = GameContext.CardActivation;
      layout._dreamwellActivation = dreamwellActivation;

      var dreamwellDisplayGo = new GameObject("DreamwellDisplay");
      createdObjects.Add(dreamwellDisplayGo);
      dreamwellDisplayGo.transform.SetParent(layoutGo.transform, false);
      dreamwellDisplayGo.transform.localPosition = new Vector3(0f, 18f, -20f);
      dreamwellDisplayGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      layout._dreamwellDisplay = dreamwellDisplayGo.transform;

      var cardOrderSelectorGo = new GameObject("CardOrderSelector");
      createdObjects.Add(cardOrderSelectorGo);
      cardOrderSelectorGo.transform.SetParent(layoutGo.transform, false);
      cardOrderSelectorGo.transform.localPosition = new Vector3(0f, 8f, -20f);
      cardOrderSelectorGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      var cardOrderSelector = cardOrderSelectorGo.AddComponent<StaticGameContext>();
      cardOrderSelector._startingContext = GameContext.Browser;
      var cardOrderSelectorCardOrderSelector =
        cardOrderSelectorGo.AddComponent<CardOrderSelector>();
      cardOrderSelectorCardOrderSelector._initialSpacing = -0.5f;
      cardOrderSelectorCardOrderSelector._initialOffset = 4.1f;

      var leftEdge2Go = new GameObject("Left");
      createdObjects.Add(leftEdge2Go);
      leftEdge2Go.transform.SetParent(cardOrderSelectorGo.transform, false);
      leftEdge2Go.transform.localPosition = new Vector3(-6f, 0f, 0f);
      cardOrderSelectorCardOrderSelector._leftEdge = leftEdge2Go.transform;

      var rightEdge2Go = new GameObject("Right");
      createdObjects.Add(rightEdge2Go);
      rightEdge2Go.transform.SetParent(cardOrderSelectorGo.transform, false);
      rightEdge2Go.transform.localPosition = new Vector3(6f, 0f, 0f);
      cardOrderSelectorCardOrderSelector._rightEdge = rightEdge2Go.transform;

      var cardOrderSelectorVoidGo = new GameObject("VoidLayout");
      createdObjects.Add(cardOrderSelectorVoidGo);
      cardOrderSelectorVoidGo.transform.SetParent(rightEdge2Go.transform, false);
      cardOrderSelectorVoidGo.transform.localPosition = new Vector3(-2f, -0.24f, -0.07f);
      var cardOrderSelectorVoid = cardOrderSelectorVoidGo.AddComponent<PileObjectLayout>();
      var cardOrderSelectorVoidStaticGameContext =
        cardOrderSelectorVoidGo.AddComponent<StaticGameContext>();
      cardOrderSelectorVoidStaticGameContext._startingContext = GameContext.Browser;
      cardOrderSelectorCardOrderSelector._cardOrderSelectorVoid = cardOrderSelectorVoid;
      layout._cardOrderSelector = cardOrderSelectorCardOrderSelector;
      layout._cardOrderSelectorVoid = cardOrderSelectorVoid;

      var gameModifiersDisplayGo = new GameObject("GameModifiersDisplay");
      createdObjects.Add(gameModifiersDisplayGo);
      gameModifiersDisplayGo.transform.SetParent(layoutGo.transform, false);
      gameModifiersDisplayGo.transform.localPosition = new Vector3(5.4f, 5f, -6.34f);
      gameModifiersDisplayGo.transform.localRotation = Quaternion.Euler(74.99998f, 0f, 0f);
      gameModifiersDisplayGo.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var gameModifiersDisplay = gameModifiersDisplayGo.AddComponent<CenteredObjectLayout>();
      gameModifiersDisplay._width = 5f;
      gameModifiersDisplay._initialSpacing = -0.5f;
      gameModifiersDisplay._cardSize = 2.5f;
      var gameModifiersDisplayStaticGameContext =
        gameModifiersDisplayGo.AddComponent<StaticGameContext>();
      gameModifiersDisplayStaticGameContext._startingContext = GameContext.GameModifiers;
      layout._gameModifiersDisplay = gameModifiersDisplay;

      var onScreenStorageGo = new GameObject("OnScreenStorage");
      createdObjects.Add(onScreenStorageGo);
      onScreenStorageGo.transform.SetParent(layoutGo.transform, false);
      onScreenStorageGo.transform.localPosition = new Vector3(-5.53f, 5f, -21.67f);
      onScreenStorageGo.transform.localRotation = Quaternion.Euler(90f, 0f, 0f);
      onScreenStorageGo.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var onScreenStorage = onScreenStorageGo.AddComponent<PileObjectLayout>();
      var onScreenStorageStaticGameContext = onScreenStorageGo.AddComponent<StaticGameContext>();
      onScreenStorageStaticGameContext._startingContext = GameContext.OnScreenStorage;
      layout._onScreenStorage = onScreenStorage;

      var primaryActionButtonGo = new GameObject("PrimaryActionButton");
      createdObjects.Add(primaryActionButtonGo);
      primaryActionButtonGo.transform.localScale = new Vector3(0.75f, 0.75f, 0.75f);
      var primaryActionButton = primaryActionButtonGo.AddComponent<StaticGameContext>();
      primaryActionButton._startingContext = GameContext.PrimaryActionButton;
      var actionButton = primaryActionButtonGo.AddComponent<ActionButton>();

      var backgroundGo = new GameObject("Background");
      createdObjects.Add(backgroundGo);
      backgroundGo.transform.SetParent(primaryActionButtonGo.transform, false);
      backgroundGo.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var spriteRenderer1 = backgroundGo.AddComponent<SpriteRenderer>();
      actionButton._background = spriteRenderer1;

      var text6Go = new GameObject("Text (TMP)");
      createdObjects.Add(text6Go);
      text6Go.transform.SetParent(primaryActionButtonGo.transform, false);
      text6Go.transform.localPosition = new Vector3(0.0112f, -0.0085f, 0f);
      var textMeshPro6 = text6Go.AddComponent<TextMeshPro>();
      actionButton._text = textMeshPro6;
      var boxCollider = primaryActionButtonGo.AddComponent<BoxCollider>();
      actionButton._collider = boxCollider;
      layout._primaryActionButton = actionButton;

      var secondaryActionButtonGo = new GameObject("SecondaryActionButton");
      createdObjects.Add(secondaryActionButtonGo);
      secondaryActionButtonGo.transform.localPosition = new Vector3(-0.055f, 1.21f, 0.324f);
      secondaryActionButtonGo.transform.localScale = new Vector3(0.75f, 0.75f, 0.75f);
      var secondaryActionButton = secondaryActionButtonGo.AddComponent<StaticGameContext>();
      secondaryActionButton._startingContext = GameContext.PrimaryActionButton;
      var actionButton1 = secondaryActionButtonGo.AddComponent<ActionButton>();

      var background1Go = new GameObject("Background");
      createdObjects.Add(background1Go);
      background1Go.transform.SetParent(secondaryActionButtonGo.transform, false);
      background1Go.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var spriteRenderer2 = background1Go.AddComponent<SpriteRenderer>();
      actionButton1._background = spriteRenderer2;

      var text7Go = new GameObject("Text (TMP)");
      createdObjects.Add(text7Go);
      text7Go.transform.SetParent(secondaryActionButtonGo.transform, false);
      text7Go.transform.localPosition = new Vector3(0.0112f, -0.0085f, 0f);
      var textMeshPro7 = text7Go.AddComponent<TextMeshPro>();
      actionButton1._text = textMeshPro7;
      var boxCollider1 = secondaryActionButtonGo.AddComponent<BoxCollider>();
      actionButton1._collider = boxCollider1;
      layout._secondaryActionButton = actionButton1;

      var incrementActionButtonGo = new GameObject("IncrementButton");
      createdObjects.Add(incrementActionButtonGo);
      incrementActionButtonGo.transform.localPosition = new Vector3(-0.9f, 1.2f, 0f);
      incrementActionButtonGo.transform.localScale = new Vector3(0.75f, 0.75f, 0.75f);
      var incrementActionButton = incrementActionButtonGo.AddComponent<StaticGameContext>();
      incrementActionButton._startingContext = GameContext.PrimaryActionButton;
      var actionButton2 = incrementActionButtonGo.AddComponent<ActionButton>();

      var background2Go = new GameObject("Background");
      createdObjects.Add(background2Go);
      background2Go.transform.SetParent(incrementActionButtonGo.transform, false);
      background2Go.transform.localScale = new Vector3(0.25f, 0.25f, 0.25f);
      var spriteRenderer3 = background2Go.AddComponent<SpriteRenderer>();
      actionButton2._background = spriteRenderer3;

      var text8Go = new GameObject("Text (TMP)");
      createdObjects.Add(text8Go);
      text8Go.transform.SetParent(incrementActionButtonGo.transform, false);
      text8Go.transform.localPosition = new Vector3(0.0112f, -0.0085f, 0f);
      var textMeshPro8 = text8Go.AddComponent<TextMeshPro>();
      actionButton2._text = textMeshPro8;
      var boxCollider2 = incrementActionButtonGo.AddComponent<BoxCollider>();
      actionButton2._collider = boxCollider2;
      layout._incrementActionButton = actionButton2;

      var decrementActionButtonGo = new GameObject("DecrementButton");
      createdObjects.Add(decrementActionButtonGo);
      decrementActionButtonGo.transform.localPosition = new Vector3(0.9f, 1.2f, 0f);
      decrementActionButtonGo.transform.localScale = new Vector3(0.75f, 0.75f, 0.75f);
      var decrementActionButton = decrementActionButtonGo.AddComponent<StaticGameContext>();
      decrementActionButton._startingContext = GameContext.PrimaryActionButton;
      var actionButton3 = decrementActionButtonGo.AddComponent<ActionButton>();

      var background3Go = new GameObject("Background");
      createdObjects.Add(background3Go);
      background3Go.transform.SetParent(decrementActionButtonGo.transform, false);
      background3Go.transform.localScale = new Vector3(0.25f, 0.25f, 0.25f);
      var spriteRenderer4 = background3Go.AddComponent<SpriteRenderer>();
      actionButton3._background = spriteRenderer4;

      var text9Go = new GameObject("Text (TMP)");
      createdObjects.Add(text9Go);
      text9Go.transform.SetParent(decrementActionButtonGo.transform, false);
      text9Go.transform.localPosition = new Vector3(0.0112f, -0.0085f, 0f);
      var textMeshPro9 = text9Go.AddComponent<TextMeshPro>();
      actionButton3._text = textMeshPro9;
      var boxCollider3 = decrementActionButtonGo.AddComponent<BoxCollider>();
      actionButton3._collider = boxCollider3;
      layout._decrementActionButton = actionButton3;

      var undoButtonGo = new GameObject("UndoButton");
      createdObjects.Add(undoButtonGo);
      undoButtonGo.transform.localPosition = new Vector3(340.1556f, 194.7f, 0f);
      var canvasButton = undoButtonGo.AddComponent<CanvasButton>();
      var canvasGroup = undoButtonGo.AddComponent<CanvasGroup>();
      canvasButton._canvasGroup = canvasGroup;

      var text10Go = new GameObject("Text (TMP)");
      createdObjects.Add(text10Go);
      text10Go.transform.SetParent(undoButtonGo.transform, false);
      text10Go.transform.localPosition = new Vector3(-12.5f, -12.5f, 0f);
      var textMeshProUGUI = text10Go.AddComponent<TextMeshProUGUI>();
      canvasButton._text = textMeshProUGUI;
      layout._undoButton = canvasButton;

      var devButtonGo = new GameObject("DevButton");
      createdObjects.Add(devButtonGo);
      devButtonGo.transform.localPosition = new Vector3(8.6f, -35.7f, 0f);
      var canvasButton1 = devButtonGo.AddComponent<CanvasButton>();
      var canvasGroup1 = devButtonGo.AddComponent<CanvasGroup>();
      canvasButton1._canvasGroup = canvasGroup1;

      var text11Go = new GameObject("Text (TMP)");
      createdObjects.Add(text11Go);
      text11Go.transform.SetParent(devButtonGo.transform, false);
      text11Go.transform.localPosition = new Vector3(14.32535f, -6.4387f, 0f);
      var textMeshProUGUI1 = text11Go.AddComponent<TextMeshProUGUI>();
      canvasButton1._text = textMeshProUGUI1;
      layout._devButton = canvasButton1;

      var enemyMessageGo = new GameObject("EnemyMessage");
      createdObjects.Add(enemyMessageGo);
      enemyMessageGo.transform.SetParent(enemyStatusDisplayGo.transform, false);
      enemyMessageGo.transform.localPosition = new Vector3(-1.69f, 6f, -4.280001f);
      enemyMessageGo.transform.localRotation = Quaternion.Euler(75.00003f, 0f, 0f);
      var enemyMessage1 = enemyMessageGo.AddComponent<EnemyMessage>();
      var spriteRenderer5 = enemyMessageGo.AddComponent<SpriteRenderer>();
      enemyMessage1._background = spriteRenderer5;

      var messageTextGo = new GameObject("Text (TMP)");
      createdObjects.Add(messageTextGo);
      messageTextGo.transform.SetParent(enemyMessageGo.transform, false);
      messageTextGo.transform.localPosition = new Vector3(0.0165f, -0.14f, 0f);
      var textMeshPro10 = messageTextGo.AddComponent<TextMeshPro>();
      enemyMessage1._messageText = textMeshPro10;
      layout._enemyMessage = enemyMessage1;

      var thinkingIndicatorGo = new GameObject("ThinkingIndicator");
      createdObjects.Add(thinkingIndicatorGo);
      thinkingIndicatorGo.transform.SetParent(layoutGo.transform, false);
      thinkingIndicatorGo.transform.localPosition = new Vector3(0f, 5f, -3f);
      thinkingIndicatorGo.transform.localScale = new Vector3(0.2f, 0.2f, 0.2f);
      layout._thinkingIndicator = thinkingIndicatorGo;
      layout._closeBrowserButton = closeBrowserButton;

      var aboveUserVoidGo = new GameObject("AboveUserVoid");
      createdObjects.Add(aboveUserVoidGo);
      aboveUserVoidGo.transform.SetParent(layoutGo.transform, false);
      aboveUserVoidGo.transform.localPosition = new Vector3(3f, 1.5f, -23f);
      aboveUserVoidGo.transform.localRotation = Quaternion.Euler(75.00003f, 0f, 0f);
      var aboveUserVoid = aboveUserVoidGo.AddComponent<StackingObjectLayout>();
      aboveUserVoid._offset = 1.5f;
      aboveUserVoid._shrinkOffset = 0.75f;
      aboveUserVoid._shrinkOffsetThreshold = 8;
      aboveUserVoid._stackRight = true;
      var aboveUserVoidStaticGameContext = aboveUserVoidGo.AddComponent<StaticGameContext>();
      aboveUserVoidStaticGameContext._startingContext = GameContext.OnScreenStorage;
      layout._aboveUserVoid = aboveUserVoid;

      var aboveEnemyVoidGo = new GameObject("AboveEnemyVoid");
      createdObjects.Add(aboveEnemyVoidGo);
      aboveEnemyVoidGo.transform.SetParent(layoutGo.transform, false);
      aboveEnemyVoidGo.transform.localPosition = new Vector3(-4f, 1.5f, -4.5f);
      aboveEnemyVoidGo.transform.localRotation = Quaternion.Euler(75.00003f, 0f, 0f);
      var aboveEnemyVoid = aboveEnemyVoidGo.AddComponent<StackingObjectLayout>();
      aboveEnemyVoid._offset = 1.5f;
      aboveEnemyVoid._shrinkOffset = 0.75f;
      aboveEnemyVoid._shrinkOffsetThreshold = 8;
      aboveEnemyVoid._stackRight = true;
      var aboveEnemyVoidStaticGameContext = aboveEnemyVoidGo.AddComponent<StaticGameContext>();
      aboveEnemyVoidStaticGameContext._startingContext = GameContext.OnScreenStorage;
      layout._aboveEnemyVoid = aboveEnemyVoid;

      return layout;
    }
  }
}
