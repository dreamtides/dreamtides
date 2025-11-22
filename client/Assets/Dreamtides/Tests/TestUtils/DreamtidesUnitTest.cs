#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using NUnit.Framework;
using UnityEngine;

namespace Dreamtides.Tests.TestUtils
{
  public abstract class DreamtidesUnitTest
  {
    readonly List<GameObject> _createdObjects = new();

    Registry? _registry;
    GameLayout? _portraitLayout;
    GameLayout? _landscapeLayout;
    GameMode _gameMode = GameMode.Quest;
    TestConfiguration? _testConfiguration;

    protected Registry Registry =>
      _registry ?? throw new InvalidOperationException("Registry not initialized");

    protected IEnumerator Initialize(
      GameViewResolution resolution = GameViewResolution.Resolution16x9
    )
    {
      _registry = CreateGameObject().AddComponent<Registry>();
      _portraitLayout = CreateGameObject().AddComponent<GameLayout>();
      _landscapeLayout = CreateGameObject().AddComponent<GameLayout>();
      _registry._portraitLayout = _portraitLayout;
      _registry._landscapeLayout = _landscapeLayout;
      _testConfiguration = new TestConfiguration(Guid.NewGuid());
      var fakeViewport = new FakeViewport(resolution.AsVector(), _registry.transform, 60f);
      return _registry.RunAwake(_gameMode, _testConfiguration, fakeViewport);
    }

    [TearDown]
    public void TearDown()
    {
      foreach (var createdObject in _createdObjects)
      {
        if (createdObject)
        {
          UnityEngine.Object.DestroyImmediate(createdObject);
        }
      }
    }

    public static void AssertVector3Equal(
      Vector3 expected,
      Vector3 actual,
      float tolerance = 0.0001f
    )
    {
      Assert.That(actual.x, Is.EqualTo(expected.x).Within(tolerance));
      Assert.That(actual.y, Is.EqualTo(expected.y).Within(tolerance));
      Assert.That(actual.z, Is.EqualTo(expected.z).Within(tolerance));
    }

    protected GameObject CreateGameObject()
    {
      var gameObject = new GameObject();
      _createdObjects.Add(gameObject);
      return gameObject;
    }

    protected T CreateSceneObject<T>(Action<T>? configure = null)
      where T : Displayable
    {
      var result = CreateGameObject().AddComponent<T>();
      configure?.Invoke(result);
      result.Initialize(Registry, _gameMode, _testConfiguration, fromRegistry: true);
      result.StartFromRegistry();
      return result;
    }
  }
}
