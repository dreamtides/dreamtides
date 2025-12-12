// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: Sites
// Generated at: 2025-12-12 13:52:38

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
  using Dreamtides.Sites;

  public class GeneratedSites
  {
    public static List<AbstractDreamscapeSite> Create(List<GameObject> createdObjects)
    {
      var result = new List<AbstractDreamscapeSite>();

      var draftSiteGo = new GameObject("DraftSite");
      createdObjects.Add(draftSiteGo);
      draftSiteGo.transform.localPosition = new Vector3(-12.07f, -0.7134895f, 3.41f);
      draftSiteGo.transform.localRotation = Quaternion.Euler(0.8484083f, 2.150115f, 358.6395f);
      var draftSite = draftSiteGo.AddComponent<DraftSite>();
      draftSite._siteId = "117b9377-e0ab-4304-9f74-6d5b4fc5c778";
      draftSite._buttonLabel = "\\ufc42";
      draftSite._debugClickAction = "FocusDraftCamera";

      var targetDraftSiteCameraGo = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCameraGo);
      targetDraftSiteCameraGo.transform.SetParent(draftSiteGo.transform, false);
      targetDraftSiteCameraGo.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCameraGo.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera = targetDraftSiteCameraGo.AddComponent<CinemachineCamera>();
      draftSite._targetDraftSiteCamera = cinemachineCamera;

      var siteDeckLayoutGo = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayoutGo);
      siteDeckLayoutGo.transform.SetParent(draftSiteGo.transform, false);
      siteDeckLayoutGo.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayoutGo.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout = siteDeckLayoutGo.AddComponent<PileObjectLayout>();
      siteDeckLayout._yMultiplier = 0.25f;
      draftSite._siteDeckLayout = siteDeckLayout;
      result.Add(draftSite);

      var shopSiteGo = new GameObject("ShopSite");
      createdObjects.Add(shopSiteGo);
      shopSiteGo.transform.localPosition = new Vector3(-17.18f, -1.59f, 20.07f);
      shopSiteGo.transform.localRotation = Quaternion.Euler(0.848409f, 41.51424f, 358.6395f);
      var shopSite = shopSiteGo.AddComponent<CharacterSite>();
      shopSite._siteId = "4ce46579-7d0c-455a-a404-628894cff331";
      shopSite._buttonLabel = "\\ufd09";
      shopSite._debugClickAction = "FocusShopCamera";

      var targetScreenLeftCameraGo = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCameraGo);
      targetScreenLeftCameraGo.transform.SetParent(shopSiteGo.transform, false);
      targetScreenLeftCameraGo.transform.localPosition = new Vector3(
        -0.3461801f,
        1.310676f,
        3.389937f
      );
      targetScreenLeftCameraGo.transform.localRotation = Quaternion.Euler(
        10.34058f,
        206.0243f,
        359.1393f
      );
      var cinemachineCamera1 = targetScreenLeftCameraGo.AddComponent<CinemachineCamera>();
      shopSite._targetScreenLeftCamera = cinemachineCamera1;

      var targetScreenRightCameraGo = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCameraGo);
      targetScreenRightCameraGo.transform.SetParent(shopSiteGo.transform, false);
      targetScreenRightCameraGo.transform.localPosition = new Vector3(
        -0.3056821f,
        1.507511f,
        3.579504f
      );
      targetScreenRightCameraGo.transform.localRotation = Quaternion.Euler(
        10.58978f,
        137.8022f,
        358.3934f
      );
      var cinemachineCamera2 = targetScreenRightCameraGo.AddComponent<CinemachineCamera>();
      shopSite._targetScreenRightCamera = cinemachineCamera2;

      var targetScreenTopCameraGo = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCameraGo);
      targetScreenTopCameraGo.transform.SetParent(shopSiteGo.transform, false);
      targetScreenTopCameraGo.transform.localPosition = new Vector3(
        -0.1428564f,
        0.9890453f,
        2.744863f
      );
      targetScreenTopCameraGo.transform.localRotation = Quaternion.Euler(
        6.642229f,
        176.1051f,
        358.5776f
      );
      var cinemachineCamera3 = targetScreenTopCameraGo.AddComponent<CinemachineCamera>();
      shopSite._targetScreenTopCamera = cinemachineCamera3;

      var siteCharacterGo = new GameObject("Character");
      createdObjects.Add(siteCharacterGo);
      siteCharacterGo.transform.SetParent(shopSiteGo.transform, false);
      shopSite._siteCharacter = siteCharacterGo;

      var characterOwnedObjectsGo = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjectsGo);
      characterOwnedObjectsGo.transform.SetParent(shopSiteGo.transform, false);
      characterOwnedObjectsGo.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjectsGo.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects = characterOwnedObjectsGo.AddComponent<PileObjectLayout>();
      characterOwnedObjects._singleElementY = 0.1f;
      characterOwnedObjects._yMultiplier = 0f;
      shopSite._characterOwnedObjects = characterOwnedObjects;

      var characterSpeechPositionGo = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPositionGo);
      characterSpeechPositionGo.transform.SetParent(shopSiteGo.transform, false);
      characterSpeechPositionGo.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      shopSite._characterSpeechPosition = characterSpeechPositionGo.transform;
      var mecanimAnimator = siteCharacterGo.AddComponent<MecanimAnimator>();
      var animator = siteCharacterGo.AddComponent<Animator>();
      mecanimAnimator._animator = animator;
      shopSite._characterAnimator = mecanimAnimator;
      result.Add(shopSite);

      var temptingOfferSiteGo = new GameObject("TemptingOfferSite");
      createdObjects.Add(temptingOfferSiteGo);
      temptingOfferSiteGo.transform.localPosition = new Vector3(4.743646f, -0.775f, 4.335932f);
      temptingOfferSiteGo.transform.localRotation = Quaternion.Euler(
        0.8484089f,
        71.72803f,
        358.6395f
      );
      var temptingOfferSite = temptingOfferSiteGo.AddComponent<CharacterSite>();
      temptingOfferSite._siteId = "2d9b1d2c-6637-4930-b9fc-a70fa901d662";
      temptingOfferSite._buttonLabel = "\\ufaf3";
      temptingOfferSite._debugClickAction = "FocusEventCamera";
      temptingOfferSite._landscapeCameraTargetSide = LandscapeCameraTargetSide.Right;

      var targetScreenLeftCamera1Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera1Go);
      targetScreenLeftCamera1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      targetScreenLeftCamera1Go.transform.localPosition = new Vector3(
        -0.3461801f,
        1.310676f,
        3.389937f
      );
      targetScreenLeftCamera1Go.transform.localRotation = Quaternion.Euler(
        10.34058f,
        206.0243f,
        359.1393f
      );
      var cinemachineCamera4 = targetScreenLeftCamera1Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenLeftCamera = cinemachineCamera4;

      var targetScreenRightCamera1Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera1Go);
      targetScreenRightCamera1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      targetScreenRightCamera1Go.transform.localPosition = new Vector3(
        -0.3056821f,
        1.507511f,
        3.579504f
      );
      targetScreenRightCamera1Go.transform.localRotation = Quaternion.Euler(
        10.58978f,
        137.8022f,
        358.3934f
      );
      var cinemachineCamera5 = targetScreenRightCamera1Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenRightCamera = cinemachineCamera5;

      var targetScreenTopCamera1Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera1Go);
      targetScreenTopCamera1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      targetScreenTopCamera1Go.transform.localPosition = new Vector3(
        -0.1428564f,
        0.9890453f,
        2.744863f
      );
      targetScreenTopCamera1Go.transform.localRotation = Quaternion.Euler(
        6.642229f,
        176.1051f,
        358.5776f
      );
      var cinemachineCamera6 = targetScreenTopCamera1Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenTopCamera = cinemachineCamera6;

      var siteCharacter1Go = new GameObject("Character");
      createdObjects.Add(siteCharacter1Go);
      siteCharacter1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      temptingOfferSite._siteCharacter = siteCharacter1Go;

      var characterOwnedObjects1Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects1Go);
      characterOwnedObjects1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      characterOwnedObjects1Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects1Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects1 = characterOwnedObjects1Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects1._singleElementY = 0.1f;
      characterOwnedObjects1._yMultiplier = 0f;
      temptingOfferSite._characterOwnedObjects = characterOwnedObjects1;

      var characterSpeechPosition1Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition1Go);
      characterSpeechPosition1Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      characterSpeechPosition1Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      temptingOfferSite._characterSpeechPosition = characterSpeechPosition1Go.transform;
      var mecanimAnimator1 = siteCharacter1Go.AddComponent<MecanimAnimator>();
      var animator1 = siteCharacter1Go.AddComponent<Animator>();
      mecanimAnimator1._animator = animator1;
      temptingOfferSite._characterAnimator = mecanimAnimator1;
      result.Add(temptingOfferSite);

      var essenceSiteGo = new GameObject("EssenceSite");
      createdObjects.Add(essenceSiteGo);
      essenceSiteGo.transform.localPosition = new Vector3(-20.24711f, 15.14783f, -30.98448f);
      essenceSiteGo.transform.localRotation = Quaternion.Euler(0.8484085f, 24.99774f, 358.6395f);
      var essenceSite = essenceSiteGo.AddComponent<CharacterSite>();
      essenceSite._siteId = "3c7b7144-6aaf-4f2f-b67d-eaf27f633ccd";
      essenceSite._buttonLabel = "\\uf997";
      essenceSite._debugClickAction = "FocusEssenceCamera";

      var targetScreenLeftCamera2Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera2Go);
      targetScreenLeftCamera2Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenLeftCamera2Go.transform.localPosition = new Vector3(
        -0.3461801f,
        1.310676f,
        3.389937f
      );
      targetScreenLeftCamera2Go.transform.localRotation = Quaternion.Euler(
        10.34058f,
        206.0243f,
        359.1393f
      );
      var cinemachineCamera7 = targetScreenLeftCamera2Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenLeftCamera = cinemachineCamera7;

      var targetScreenRightCamera2Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera2Go);
      targetScreenRightCamera2Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenRightCamera2Go.transform.localPosition = new Vector3(
        -0.3056821f,
        1.507511f,
        3.579504f
      );
      targetScreenRightCamera2Go.transform.localRotation = Quaternion.Euler(
        10.58978f,
        137.8022f,
        358.3934f
      );
      var cinemachineCamera8 = targetScreenRightCamera2Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenRightCamera = cinemachineCamera8;

      var targetScreenTopCamera2Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera2Go);
      targetScreenTopCamera2Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenTopCamera2Go.transform.localPosition = new Vector3(
        -0.1428564f,
        0.9890453f,
        2.744863f
      );
      targetScreenTopCamera2Go.transform.localRotation = Quaternion.Euler(
        6.642229f,
        176.1051f,
        358.5776f
      );
      var cinemachineCamera9 = targetScreenTopCamera2Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenTopCamera = cinemachineCamera9;

      var siteCharacter2Go = new GameObject("Character");
      createdObjects.Add(siteCharacter2Go);
      siteCharacter2Go.transform.SetParent(essenceSiteGo.transform, false);
      essenceSite._siteCharacter = siteCharacter2Go;

      var characterOwnedObjects2Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects2Go);
      characterOwnedObjects2Go.transform.SetParent(essenceSiteGo.transform, false);
      characterOwnedObjects2Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects2Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects2 = characterOwnedObjects2Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects2._singleElementY = 0.1f;
      characterOwnedObjects2._yMultiplier = 0f;
      essenceSite._characterOwnedObjects = characterOwnedObjects2;

      var characterSpeechPosition2Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition2Go);
      characterSpeechPosition2Go.transform.SetParent(essenceSiteGo.transform, false);
      characterSpeechPosition2Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      essenceSite._characterSpeechPosition = characterSpeechPosition2Go.transform;
      var mecanimAnimator2 = siteCharacter2Go.AddComponent<MecanimAnimator>();
      var animator2 = siteCharacter2Go.AddComponent<Animator>();
      mecanimAnimator2._animator = animator2;
      essenceSite._characterAnimator = mecanimAnimator2;
      result.Add(essenceSite);

      var battleSiteGo = new GameObject("BattleSite");
      createdObjects.Add(battleSiteGo);
      battleSiteGo.transform.localPosition = new Vector3(11.74f, 10f, -37.92f);
      battleSiteGo.transform.localRotation = Quaternion.Euler(0f, 120.3519f, 0f);
      var battleSite = battleSiteGo.AddComponent<BattleSite>();
      battleSite._siteId = "5d64e4b5-b27b-493e-a2e5-b70c32cf7d77";
      battleSite._buttonLabel = "\\ufd26";
      battleSite._debugClickAction = "FocusBattleCamera";

      var portraitBattleLayoutAnchorGo = new GameObject("PortraitBattleAnchor");
      createdObjects.Add(portraitBattleLayoutAnchorGo);
      battleSite._portraitBattleLayoutAnchor = portraitBattleLayoutAnchorGo.transform;

      var landscapeBattleLayoutAnchorGo = new GameObject("LandscapeBattleAnchor");
      createdObjects.Add(landscapeBattleLayoutAnchorGo);
      landscapeBattleLayoutAnchorGo.transform.localPosition = new Vector3(1f, 0f, -15f);
      battleSite._landscapeBattleLayoutAnchor = landscapeBattleLayoutAnchorGo.transform;
      result.Add(battleSite);

      return result;
    }
  }
}
