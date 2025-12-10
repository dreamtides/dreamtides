// AUTO-GENERATED CODE - DO NOT EDIT
// Generated from: Sites
// Generated at: 2025-12-10 06:28:57

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
  public class GeneratedSites
  {
    public static List<DreamscapeSite> Create(List<GameObject> createdObjects)
    {
      var result = new List<DreamscapeSite>();

      var draftSiteGo = new GameObject("DraftSite");
      createdObjects.Add(draftSiteGo);
      draftSiteGo.transform.localPosition = new Vector3(-12.07f, -0.7134895f, 3.41f);
      draftSiteGo.transform.localRotation = Quaternion.Euler(0.8484083f, 2.150115f, 358.6395f);
      var draftSite = draftSiteGo.AddComponent<DreamscapeSite>();
      draftSite._draftSite = true;
      draftSite._siteId = "117b9377-e0ab-4304-9f74-6d5b4fc5c778";
      draftSite._isActive = false;
      draftSite._buttonLabel = "\\ufc42";
      draftSite._debugClickAction = "FocusDraftCamera";

      var targetScreenLeftCameraGo = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCameraGo);
      targetScreenLeftCameraGo.transform.SetParent(draftSiteGo.transform, false);
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
      var cinemachineCamera = targetScreenLeftCameraGo.AddComponent<CinemachineCamera>();
      draftSite._targetScreenLeftCamera = cinemachineCamera;

      var targetScreenRightCameraGo = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCameraGo);
      targetScreenRightCameraGo.transform.SetParent(draftSiteGo.transform, false);
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
      var cinemachineCamera1 = targetScreenRightCameraGo.AddComponent<CinemachineCamera>();
      draftSite._targetScreenRightCamera = cinemachineCamera1;

      var targetScreenTopCameraGo = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCameraGo);
      targetScreenTopCameraGo.transform.SetParent(draftSiteGo.transform, false);
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
      var cinemachineCamera2 = targetScreenTopCameraGo.AddComponent<CinemachineCamera>();
      draftSite._targetScreenTopCamera = cinemachineCamera2;

      var targetDraftSiteCameraGo = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCameraGo);
      targetDraftSiteCameraGo.transform.SetParent(draftSiteGo.transform, false);
      targetDraftSiteCameraGo.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCameraGo.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera3 = targetDraftSiteCameraGo.AddComponent<CinemachineCamera>();
      draftSite._targetDraftSiteCamera = cinemachineCamera3;

      var siteCharacterGo = new GameObject("Character");
      createdObjects.Add(siteCharacterGo);
      siteCharacterGo.transform.SetParent(draftSiteGo.transform, false);
      draftSite._siteCharacter = siteCharacterGo;

      var characterOwnedObjectsGo = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjectsGo);
      characterOwnedObjectsGo.transform.SetParent(draftSiteGo.transform, false);
      characterOwnedObjectsGo.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjectsGo.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects = characterOwnedObjectsGo.AddComponent<PileObjectLayout>();
      characterOwnedObjects._singleElementY = 0.1f;
      characterOwnedObjects._yMultiplier = 0f;
      draftSite._characterOwnedObjects = characterOwnedObjects;

      var siteDeckLayoutGo = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayoutGo);
      siteDeckLayoutGo.transform.SetParent(draftSiteGo.transform, false);
      siteDeckLayoutGo.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayoutGo.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout = siteDeckLayoutGo.AddComponent<PileObjectLayout>();
      siteDeckLayout._yMultiplier = 0.25f;
      draftSite._siteDeckLayout = siteDeckLayout;

      var characterSpeechPositionGo = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPositionGo);
      characterSpeechPositionGo.transform.SetParent(draftSiteGo.transform, false);
      characterSpeechPositionGo.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      draftSite._characterSpeechPosition = characterSpeechPositionGo.transform;
      var mecanimAnimator = siteCharacterGo.AddComponent<MecanimAnimator>();
      var animator = siteCharacterGo.AddComponent<Animator>();
      mecanimAnimator._animator = animator;
      draftSite._characterAnimator = mecanimAnimator;
      result.Add(draftSite);

      var shopSiteGo = new GameObject("ShopSite");
      createdObjects.Add(shopSiteGo);
      shopSiteGo.transform.localPosition = new Vector3(-17.18f, -1.59f, 20.07f);
      shopSiteGo.transform.localRotation = Quaternion.Euler(0.848409f, 41.51424f, 358.6395f);
      var shopSite = shopSiteGo.AddComponent<DreamscapeSite>();
      shopSite._siteId = "4ce46579-7d0c-455a-a404-628894cff331";
      shopSite._isActive = false;
      shopSite._buttonLabel = "\\ufd09";
      shopSite._debugClickAction = "FocusShopCamera";

      var targetScreenLeftCamera1Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera1Go);
      targetScreenLeftCamera1Go.transform.SetParent(shopSiteGo.transform, false);
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
      shopSite._targetScreenLeftCamera = cinemachineCamera4;

      var targetScreenRightCamera1Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera1Go);
      targetScreenRightCamera1Go.transform.SetParent(shopSiteGo.transform, false);
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
      shopSite._targetScreenRightCamera = cinemachineCamera5;

      var targetScreenTopCamera1Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera1Go);
      targetScreenTopCamera1Go.transform.SetParent(shopSiteGo.transform, false);
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
      shopSite._targetScreenTopCamera = cinemachineCamera6;

      var targetDraftSiteCamera1Go = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCamera1Go);
      targetDraftSiteCamera1Go.transform.SetParent(shopSiteGo.transform, false);
      targetDraftSiteCamera1Go.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCamera1Go.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera7 = targetDraftSiteCamera1Go.AddComponent<CinemachineCamera>();
      shopSite._targetDraftSiteCamera = cinemachineCamera7;

      var siteCharacter1Go = new GameObject("Character");
      createdObjects.Add(siteCharacter1Go);
      siteCharacter1Go.transform.SetParent(shopSiteGo.transform, false);
      shopSite._siteCharacter = siteCharacter1Go;

      var characterOwnedObjects1Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects1Go);
      characterOwnedObjects1Go.transform.SetParent(shopSiteGo.transform, false);
      characterOwnedObjects1Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects1Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects1 = characterOwnedObjects1Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects1._singleElementY = 0.1f;
      characterOwnedObjects1._yMultiplier = 0f;
      shopSite._characterOwnedObjects = characterOwnedObjects1;

      var siteDeckLayout1Go = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayout1Go);
      siteDeckLayout1Go.transform.SetParent(shopSiteGo.transform, false);
      siteDeckLayout1Go.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayout1Go.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout1 = siteDeckLayout1Go.AddComponent<PileObjectLayout>();
      siteDeckLayout1._yMultiplier = 0.25f;
      shopSite._siteDeckLayout = siteDeckLayout1;

      var characterSpeechPosition1Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition1Go);
      characterSpeechPosition1Go.transform.SetParent(shopSiteGo.transform, false);
      characterSpeechPosition1Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      shopSite._characterSpeechPosition = characterSpeechPosition1Go.transform;
      var mecanimAnimator1 = siteCharacter1Go.AddComponent<MecanimAnimator>();
      var animator1 = siteCharacter1Go.AddComponent<Animator>();
      mecanimAnimator1._animator = animator1;
      shopSite._characterAnimator = mecanimAnimator1;
      result.Add(shopSite);

      var temptingOfferSiteGo = new GameObject("TemptingOfferSite");
      createdObjects.Add(temptingOfferSiteGo);
      temptingOfferSiteGo.transform.localPosition = new Vector3(4.743646f, -0.775f, 4.335932f);
      temptingOfferSiteGo.transform.localRotation = Quaternion.Euler(
        0.8484089f,
        71.72803f,
        358.6395f
      );
      var temptingOfferSite = temptingOfferSiteGo.AddComponent<DreamscapeSite>();
      temptingOfferSite._landscapeCameraTargetSide = LandscapeCameraTargetSide.Right;
      temptingOfferSite._siteId = "2d9b1d2c-6637-4930-b9fc-a70fa901d662";
      temptingOfferSite._isActive = false;
      temptingOfferSite._buttonLabel = "\\ufaf3";
      temptingOfferSite._debugClickAction = "FocusEventCamera";

      var targetScreenLeftCamera2Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera2Go);
      targetScreenLeftCamera2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
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
      var cinemachineCamera8 = targetScreenLeftCamera2Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenLeftCamera = cinemachineCamera8;

      var targetScreenRightCamera2Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera2Go);
      targetScreenRightCamera2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
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
      var cinemachineCamera9 = targetScreenRightCamera2Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenRightCamera = cinemachineCamera9;

      var targetScreenTopCamera2Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera2Go);
      targetScreenTopCamera2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
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
      var cinemachineCamera10 = targetScreenTopCamera2Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetScreenTopCamera = cinemachineCamera10;

      var targetDraftSiteCamera2Go = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCamera2Go);
      targetDraftSiteCamera2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      targetDraftSiteCamera2Go.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCamera2Go.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera11 = targetDraftSiteCamera2Go.AddComponent<CinemachineCamera>();
      temptingOfferSite._targetDraftSiteCamera = cinemachineCamera11;

      var siteCharacter2Go = new GameObject("Character");
      createdObjects.Add(siteCharacter2Go);
      siteCharacter2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      temptingOfferSite._siteCharacter = siteCharacter2Go;

      var characterOwnedObjects2Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects2Go);
      characterOwnedObjects2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      characterOwnedObjects2Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects2Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects2 = characterOwnedObjects2Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects2._singleElementY = 0.1f;
      characterOwnedObjects2._yMultiplier = 0f;
      temptingOfferSite._characterOwnedObjects = characterOwnedObjects2;

      var siteDeckLayout2Go = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayout2Go);
      siteDeckLayout2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      siteDeckLayout2Go.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayout2Go.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout2 = siteDeckLayout2Go.AddComponent<PileObjectLayout>();
      siteDeckLayout2._yMultiplier = 0.25f;
      temptingOfferSite._siteDeckLayout = siteDeckLayout2;

      var characterSpeechPosition2Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition2Go);
      characterSpeechPosition2Go.transform.SetParent(temptingOfferSiteGo.transform, false);
      characterSpeechPosition2Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      temptingOfferSite._characterSpeechPosition = characterSpeechPosition2Go.transform;
      var mecanimAnimator2 = siteCharacter2Go.AddComponent<MecanimAnimator>();
      var animator2 = siteCharacter2Go.AddComponent<Animator>();
      mecanimAnimator2._animator = animator2;
      temptingOfferSite._characterAnimator = mecanimAnimator2;
      result.Add(temptingOfferSite);

      var essenceSiteGo = new GameObject("EssenceSite");
      createdObjects.Add(essenceSiteGo);
      essenceSiteGo.transform.localPosition = new Vector3(-20.24711f, 15.14783f, -30.98448f);
      essenceSiteGo.transform.localRotation = Quaternion.Euler(0.8484085f, 24.99774f, 358.6395f);
      var essenceSite = essenceSiteGo.AddComponent<DreamscapeSite>();
      essenceSite._siteId = "f9049049-d923-4f36-9e99-d92650a21020";
      essenceSite._isActive = false;
      essenceSite._buttonLabel = "\\uf997";
      essenceSite._debugClickAction = "FocusEssenceCamera";

      var targetScreenLeftCamera3Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera3Go);
      targetScreenLeftCamera3Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenLeftCamera3Go.transform.localPosition = new Vector3(
        -0.3461801f,
        1.310676f,
        3.389937f
      );
      targetScreenLeftCamera3Go.transform.localRotation = Quaternion.Euler(
        10.34058f,
        206.0243f,
        359.1393f
      );
      var cinemachineCamera12 = targetScreenLeftCamera3Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenLeftCamera = cinemachineCamera12;

      var targetScreenRightCamera3Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera3Go);
      targetScreenRightCamera3Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenRightCamera3Go.transform.localPosition = new Vector3(
        -0.3056821f,
        1.507511f,
        3.579504f
      );
      targetScreenRightCamera3Go.transform.localRotation = Quaternion.Euler(
        10.58978f,
        137.8022f,
        358.3934f
      );
      var cinemachineCamera13 = targetScreenRightCamera3Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenRightCamera = cinemachineCamera13;

      var targetScreenTopCamera3Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera3Go);
      targetScreenTopCamera3Go.transform.SetParent(essenceSiteGo.transform, false);
      targetScreenTopCamera3Go.transform.localPosition = new Vector3(
        -0.1428564f,
        0.9890453f,
        2.744863f
      );
      targetScreenTopCamera3Go.transform.localRotation = Quaternion.Euler(
        6.642229f,
        176.1051f,
        358.5776f
      );
      var cinemachineCamera14 = targetScreenTopCamera3Go.AddComponent<CinemachineCamera>();
      essenceSite._targetScreenTopCamera = cinemachineCamera14;

      var targetDraftSiteCamera3Go = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCamera3Go);
      targetDraftSiteCamera3Go.transform.SetParent(essenceSiteGo.transform, false);
      targetDraftSiteCamera3Go.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCamera3Go.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera15 = targetDraftSiteCamera3Go.AddComponent<CinemachineCamera>();
      essenceSite._targetDraftSiteCamera = cinemachineCamera15;

      var siteCharacter3Go = new GameObject("Character");
      createdObjects.Add(siteCharacter3Go);
      siteCharacter3Go.transform.SetParent(essenceSiteGo.transform, false);
      essenceSite._siteCharacter = siteCharacter3Go;

      var characterOwnedObjects3Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects3Go);
      characterOwnedObjects3Go.transform.SetParent(essenceSiteGo.transform, false);
      characterOwnedObjects3Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects3Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects3 = characterOwnedObjects3Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects3._singleElementY = 0.1f;
      characterOwnedObjects3._yMultiplier = 0f;
      essenceSite._characterOwnedObjects = characterOwnedObjects3;

      var siteDeckLayout3Go = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayout3Go);
      siteDeckLayout3Go.transform.SetParent(essenceSiteGo.transform, false);
      siteDeckLayout3Go.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayout3Go.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout3 = siteDeckLayout3Go.AddComponent<PileObjectLayout>();
      siteDeckLayout3._yMultiplier = 0.25f;
      essenceSite._siteDeckLayout = siteDeckLayout3;

      var characterSpeechPosition3Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition3Go);
      characterSpeechPosition3Go.transform.SetParent(essenceSiteGo.transform, false);
      characterSpeechPosition3Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      essenceSite._characterSpeechPosition = characterSpeechPosition3Go.transform;
      var mecanimAnimator3 = siteCharacter3Go.AddComponent<MecanimAnimator>();
      var animator3 = siteCharacter3Go.AddComponent<Animator>();
      mecanimAnimator3._animator = animator3;
      essenceSite._characterAnimator = mecanimAnimator3;
      result.Add(essenceSite);

      var battleSiteGo = new GameObject("BattleSite");
      createdObjects.Add(battleSiteGo);
      battleSiteGo.transform.localPosition = new Vector3(5.699469f, 9.2694f, -39.36598f);
      battleSiteGo.transform.localRotation = Quaternion.Euler(0.8484105f, 303.7469f, 358.6395f);
      var battleSite = battleSiteGo.AddComponent<DreamscapeSite>();
      battleSite._siteId = "f9049049-d923-4f36-9e99-d92650a21020";
      battleSite._isActive = false;
      battleSite._buttonLabel = "\\ufd26";
      battleSite._debugClickAction = "FocusBattleCamera";

      var targetScreenLeftCamera4Go = new GameObject("TargetScreenLeftCamera");
      createdObjects.Add(targetScreenLeftCamera4Go);
      targetScreenLeftCamera4Go.transform.SetParent(battleSiteGo.transform, false);
      targetScreenLeftCamera4Go.transform.localPosition = new Vector3(
        -0.3461801f,
        1.310676f,
        3.389937f
      );
      targetScreenLeftCamera4Go.transform.localRotation = Quaternion.Euler(
        10.34058f,
        206.0243f,
        359.1393f
      );
      var cinemachineCamera16 = targetScreenLeftCamera4Go.AddComponent<CinemachineCamera>();
      battleSite._targetScreenLeftCamera = cinemachineCamera16;

      var targetScreenRightCamera4Go = new GameObject("TargetScreenRightCamera");
      createdObjects.Add(targetScreenRightCamera4Go);
      targetScreenRightCamera4Go.transform.SetParent(battleSiteGo.transform, false);
      targetScreenRightCamera4Go.transform.localPosition = new Vector3(
        -0.3056821f,
        1.507511f,
        3.579504f
      );
      targetScreenRightCamera4Go.transform.localRotation = Quaternion.Euler(
        10.58978f,
        137.8022f,
        358.3934f
      );
      var cinemachineCamera17 = targetScreenRightCamera4Go.AddComponent<CinemachineCamera>();
      battleSite._targetScreenRightCamera = cinemachineCamera17;

      var targetScreenTopCamera4Go = new GameObject("TargetScreenTopCamera");
      createdObjects.Add(targetScreenTopCamera4Go);
      targetScreenTopCamera4Go.transform.SetParent(battleSiteGo.transform, false);
      targetScreenTopCamera4Go.transform.localPosition = new Vector3(
        -0.1428564f,
        0.9890453f,
        2.744863f
      );
      targetScreenTopCamera4Go.transform.localRotation = Quaternion.Euler(
        6.642229f,
        176.1051f,
        358.5776f
      );
      var cinemachineCamera18 = targetScreenTopCamera4Go.AddComponent<CinemachineCamera>();
      battleSite._targetScreenTopCamera = cinemachineCamera18;

      var targetDraftSiteCamera4Go = new GameObject("TargetDraftSiteCamera");
      createdObjects.Add(targetDraftSiteCamera4Go);
      targetDraftSiteCamera4Go.transform.SetParent(battleSiteGo.transform, false);
      targetDraftSiteCamera4Go.transform.localPosition = new Vector3(0f, 4f, 6f);
      targetDraftSiteCamera4Go.transform.localRotation = Quaternion.Euler(20f, 180f, 0f);
      var cinemachineCamera19 = targetDraftSiteCamera4Go.AddComponent<CinemachineCamera>();
      battleSite._targetDraftSiteCamera = cinemachineCamera19;

      var siteCharacter4Go = new GameObject("Character");
      createdObjects.Add(siteCharacter4Go);
      siteCharacter4Go.transform.SetParent(battleSiteGo.transform, false);
      battleSite._siteCharacter = siteCharacter4Go;

      var characterOwnedObjects4Go = new GameObject("CharacterOwnedObjects");
      createdObjects.Add(characterOwnedObjects4Go);
      characterOwnedObjects4Go.transform.SetParent(battleSiteGo.transform, false);
      characterOwnedObjects4Go.transform.localPosition = new Vector3(0f, 1.25f, 0f);
      characterOwnedObjects4Go.transform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
      var characterOwnedObjects4 = characterOwnedObjects4Go.AddComponent<PileObjectLayout>();
      characterOwnedObjects4._singleElementY = 0.1f;
      characterOwnedObjects4._yMultiplier = 0f;
      battleSite._characterOwnedObjects = characterOwnedObjects4;

      var siteDeckLayout4Go = new GameObject("SiteDeck");
      createdObjects.Add(siteDeckLayout4Go);
      siteDeckLayout4Go.transform.SetParent(battleSiteGo.transform, false);
      siteDeckLayout4Go.transform.localRotation = Quaternion.Euler(90f, 90f, 0f);
      siteDeckLayout4Go.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);
      var siteDeckLayout4 = siteDeckLayout4Go.AddComponent<PileObjectLayout>();
      siteDeckLayout4._yMultiplier = 0.25f;
      battleSite._siteDeckLayout = siteDeckLayout4;

      var characterSpeechPosition4Go = new GameObject("CharacterSpeechBubblePosition");
      createdObjects.Add(characterSpeechPosition4Go);
      characterSpeechPosition4Go.transform.SetParent(battleSiteGo.transform, false);
      characterSpeechPosition4Go.transform.localPosition = new Vector3(-0.3f, 1.561f, 0f);
      battleSite._characterSpeechPosition = characterSpeechPosition4Go.transform;
      var mecanimAnimator4 = siteCharacter4Go.AddComponent<MecanimAnimator>();
      var animator4 = siteCharacter4Go.AddComponent<Animator>();
      mecanimAnimator4._animator = animator4;
      battleSite._characterAnimator = mecanimAnimator4;
      result.Add(battleSite);

      var battleSite1Go = new GameObject("BattleSite");
      createdObjects.Add(battleSite1Go);
      battleSite1Go.transform.localPosition = new Vector3(11.74f, 10f, -37.92f);
      battleSite1Go.transform.localRotation = Quaternion.Euler(0f, 120.3519f, 0f);
      var battleSite1 = battleSite1Go.AddComponent<DreamscapeBattleSite>();
      battleSite1._siteId = "a44011c7-7b18-4352-8814-f0fe3dcac934";
      battleSite1._buttonLabel = "\\ufd26";
      battleSite1._debugClickAction = "FocusBattleCamera";

      var portraitBattleLayoutAnchorGo = new GameObject("PortraitBattleAnchor");
      createdObjects.Add(portraitBattleLayoutAnchorGo);
      battleSite1._portraitBattleLayoutAnchor = portraitBattleLayoutAnchorGo.transform;

      var landscapeBattleLayoutAnchorGo = new GameObject("LandscapeBattleAnchor");
      createdObjects.Add(landscapeBattleLayoutAnchorGo);
      landscapeBattleLayoutAnchorGo.transform.localPosition = new Vector3(1f, 0f, -15f);
      battleSite1._landscapeBattleLayoutAnchor = landscapeBattleLayoutAnchorGo.transform;
      result.Add(battleSite1);

      return result;
    }
  }
}
