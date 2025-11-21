#nullable enable

using System.Collections;
using Dreamtides.Components;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Components
{
  [TestFixture]
  public class FakeCameraTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator WorldToViewportPointMatchesCameraForForwardPoints()
    {
      yield return Initialize();
      var resolution = new Vector2(1920f, 1080f);
      var cameraComponent = CreateConfiguredCamera(
        resolution,
        new Vector3(1f, 2f, 3f),
        Quaternion.Euler(15f, 25f, 5f),
        55f
      );
      var fakeCamera = new FakeCamera(resolution, cameraComponent.transform, 55f);
      var localPoints = new[]
      {
        new Vector3(0f, 0f, 12f),
        new Vector3(1.5f, -0.5f, 18f),
        new Vector3(-2f, 3f, 40f),
      };

      foreach (var localPoint in localPoints)
      {
        var worldPoint = cameraComponent.transform.TransformPoint(localPoint);
        var expected = cameraComponent.WorldToViewportPoint(worldPoint);
        var actual = fakeCamera.WorldToViewportPoint(worldPoint);
        AssertVector3Equal(expected, actual, 0.0001f);
      }
    }

    [UnityTest]
    public IEnumerator WorldToViewportPointMatchesCameraForBehindPoints()
    {
      yield return Initialize();
      var resolution = new Vector2(2560f, 1440f);
      var cameraComponent = CreateConfiguredCamera(
        resolution,
        new Vector3(-3f, 1f, -4f),
        Quaternion.Euler(0f, 120f, 10f),
        70f
      );
      var fakeCamera = new FakeCamera(resolution, cameraComponent.transform, 70f);
      var localPoints = new[]
      {
        new Vector3(0f, 0f, -5f),
        new Vector3(-1f, 2f, -12f),
        new Vector3(3f, -4f, -25f),
      };

      foreach (var localPoint in localPoints)
      {
        var worldPoint = cameraComponent.transform.TransformPoint(localPoint);
        var expected = cameraComponent.WorldToViewportPoint(worldPoint);
        var actual = fakeCamera.WorldToViewportPoint(worldPoint);
        AssertVector3Equal(expected, actual, 0.0001f);
      }
    }

    Camera CreateConfiguredCamera(
      Vector2 resolution,
      Vector3 position,
      Quaternion rotation,
      float fieldOfView
    )
    {
      var result = CreateGameObject().AddComponent<Camera>();
      result.transform.position = position;
      result.transform.rotation = rotation;
      result.fieldOfView = fieldOfView;
      result.aspect = resolution.x / resolution.y;
      result.nearClipPlane = 0.3f;
      result.farClipPlane = 1000f;
      return result;
    }
  }
}
