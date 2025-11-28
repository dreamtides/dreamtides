#nullable enable

using System.Collections;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class SceneElementScreenPositionTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtTopLeftAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-10.26f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtTopCenterAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtTopRightAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(10.26f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleLeftAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleLeft;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-10.26f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleCenterAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleRightAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(10.26f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleLeftHalfAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleLeftHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-5.13f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleRightHalfAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleRightHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(5.13f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleTopHalfAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleTopHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 2.89f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtMiddleBottomHalfAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleBottomHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -2.89f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtBottomLeftAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomLeft;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-10.26f, -5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtBottomCenterAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdatePositionsObjectAtBottomRightAnchor()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(10.26f, -5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateAppliesPositiveXOffset()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._xOffset = 100f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(1.07f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateAppliesNegativeXOffset()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopRight;
        e._xOffset = -200f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(8.13f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateAppliesPositiveYOffset()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomCenter;
        e._yOffset = 150f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -4.17f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateAppliesNegativeYOffset()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopCenter;
        e._yOffset = -100f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 4.70f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateAppliesBothOffsets()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._xOffset = 50f;
        e._yOffset = -75f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0.53f, -0.80f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesDifferentDistanceFromCamera()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 5f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 0f, 5f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesLargerDistanceFromCamera()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._distanceFromCamera = 20f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-20.53f, 11.55f, 20f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithDifferentScreenResolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(8.88f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithWideScreenResolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(13.79f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesLandscapeAnchorWhenInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._landscapeAnchor = SceneElementScreenAnchor.BottomRight;
        e._useLandscapeAnchor = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(13.79f, -5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesLandscapeXOffsetWhenInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._xOffset = 50f;
        e._landscapeXOffset = -100f;
        e._useLandscapeXOffset = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-1.07f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesLandscapeYOffsetWhenInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._yOffset = 75f;
        e._landscapeYOffset = -150f;
        e._useLandscapeYOffset = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -1.60f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesLandscapeDistanceWhenInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopCenter;
        e._distanceFromCamera = 10f;
        e._landscapeDistanceFromCamera = 15f;
        e._useLandscapeDistanceFromCamera = true;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 8.66f, 15f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateIgnoresLandscapeAnchorWhenPortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._landscapeAnchor = SceneElementScreenAnchor.BottomRight;
        e._useLandscapeAnchor = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-2.67f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateIgnoresLandscapeOffsetsWhenPortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._xOffset = 50f;
        e._yOffset = 75f;
        e._landscapeXOffset = -200f;
        e._landscapeYOffset = -300f;
        e._useLandscapeXOffset = true;
        e._useLandscapeYOffset = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0.23f, 0.34f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateUsesAllLandscapeSettingsInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._landscapeAnchor = SceneElementScreenAnchor.BottomRight;
        e._landscapeXOffset = -50f;
        e._landscapeYOffset = 100f;
        e._landscapeDistanceFromCamera = 15f;
        e._useLandscapeAnchor = true;
        e._useLandscapeXOffset = true;
        e._useLandscapeYOffset = true;
        e._useLandscapeDistanceFromCamera = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(20.09f, -7.46f, 15f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateRespectsFullScreenRectWhenIgnoringSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._ignoreSafeArea = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-10.26f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateRespectsSafeAreaWhenNotIgnoring()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._ignoreSafeArea = false;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-8.21f, 4.62f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateRespectsSafeAreaBottomRight()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomRight;
        e._ignoreSafeArea = false;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(8.21f, -4.62f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateRespectsSafeAreaMiddleCenter()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.2f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.8f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._ignoreSafeArea = false;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateRespectsSafeAreaWithOffsets()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopRight;
        e._xOffset = -100f;
        e._yOffset = -50f;
        e._ignoreSafeArea = false;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(7.14f, 4.08f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateDeactivatesObjectWhenGameModeDoesNotMatch()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._gameMode = GameMode.Battle;
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });
      element.gameObject.SetActive(true);

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      Assert.That(element.gameObject.activeSelf, Is.False);
    }

    [UnityTest]
    public IEnumerator OnUpdateActivatesObjectWhenGameModeMatches()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._gameMode = GameMode.Quest;
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });
      element.gameObject.SetActive(false);

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      Assert.That(element.gameObject.activeSelf, Is.True);
    }

    [UnityTest]
    public IEnumerator OnUpdateDoesNotRepositionWhenGameModeDoesNotMatch()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._gameMode = GameMode.Battle;
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });
      element.transform.position = new Vector3(100f, 200f, 300f);

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(100f, 200f, 300f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithIPhoneSEResolution()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhoneSE);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithIPadPro12Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPadPro12);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithSamsungNote20Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionSamsungNote20);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopRight;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(2.69f, 5.77f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithPixel5Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionPixel5);
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleLeft;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-2.66f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateCombinesSafeAreaWithLandscapeSettings()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution21x9,
        safeAreaMinimumAnchor: new Vector2(0.05f, 0.05f),
        safeAreaMaximumAnchor: new Vector2(0.95f, 0.95f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.TopLeft;
        e._landscapeAnchor = SceneElementScreenAnchor.TopRight;
        e._landscapeXOffset = -75f;
        e._useLandscapeAnchor = true;
        e._useLandscapeXOffset = true;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(11.81f, 5.20f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithMiddleLeftHalfInSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleLeftHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(-4.11f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithMiddleRightHalfInSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.1f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.9f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleRightHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(4.11f, 0f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithMiddleTopHalfInSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.2f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.8f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleTopHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 1.73f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithMiddleBottomHalfInSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.1f, 0.2f),
        safeAreaMaximumAnchor: new Vector2(0.9f, 0.8f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleBottomHalf;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, -1.73f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateWorksWithAsymmetricSafeArea()
    {
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        safeAreaMinimumAnchor: new Vector2(0.2f, 0.05f),
        safeAreaMaximumAnchor: new Vector2(0.95f, 0.85f)
      );
      yield return Initialize(viewport);
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(1.54f, -0.58f, 10f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateHandlesZeroDistance()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._distanceFromCamera = 0f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(0f, 0f, 0f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateHandlesVeryLargeDistance()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.BottomRight;
        e._distanceFromCamera = 100f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(102.64f, -57.74f, 100f), element.transform.position);
    }

    [UnityTest]
    public IEnumerator OnUpdateHandlesVeryLargeOffsets()
    {
      yield return Initialize();
      var element = CreateSceneElement<SceneElementScreenPosition>(e =>
      {
        e._anchor = SceneElementScreenAnchor.MiddleCenter;
        e._xOffset = 500f;
        e._yOffset = -400f;
        e._distanceFromCamera = 10f;
      });

      element.OnUpdate(GameMode.Quest, TestConfiguration);

      AssertVector3Equal(new Vector3(5.35f, -4.28f, 10f), element.transform.position);
    }
  }
}
