using System.Collections;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;
namespace Dreamtides.Tests
{
  public class AllTests
  {
    [SetUp]
    public void SetUp()
    {
      Registry.IsTest = true;
    }

    [TearDown]
    public void TearDown()
    {
      Registry.IsTest = false;
    }

    [UnityTest]
    public IEnumerator Test16x9()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(1920, 1080)); // Most popular 16:9 resolution
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }

    [UnityTest]
    public IEnumerator Test16x10()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(2560, 1600)); // Most popular 16:10 resolution
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }


    [UnityTest]
    public IEnumerator Test4x3()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(1600, 1200));
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }


    [UnityTest]
    public IEnumerator Test21x19()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(3440, 1440)); // Most popular 21:9 resolution
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }

    [UnityTest]
    public IEnumerator Test5x4()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(1280, 1024));
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }

    [UnityTest]
    public IEnumerator Test32x9()
    {
      GameViewUtils.SetGameViewResolution(new Vector2(5120, 1440));
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteBoundsVisible(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }

    void AssertSpriteBoundsVisible(Registry registry, SpriteRenderer sprite, string message)
    {
      var bounds = sprite.bounds;
      var corners = new Vector3[4]
      {
        new Vector3(bounds.min.x, bounds.min.y, bounds.center.z),
        new Vector3(bounds.max.x, bounds.min.y, bounds.center.z),
        new Vector3(bounds.max.x, bounds.max.y, bounds.center.z),
        new Vector3(bounds.min.x, bounds.max.y, bounds.center.z)
      };

      foreach (var corner in corners)
      {
        var viewportPos = registry.Layout.MainCamera.WorldToViewportPoint(corner);
        Assert.That(viewportPos.x >= 0 && viewportPos.x <= 1 &&
                    viewportPos.y >= 0 && viewportPos.y <= 1 &&
                    viewportPos.z > 0,
                    $"{message}: Corner at world position {corner} is outside viewport: {viewportPos}");
      }
    }

    private IEnumerator WaitForSceneLoad()
    {
      while (SceneManager.GetActiveScene().buildIndex > 0)
      {
        yield return null;
      }
    }
  }
}
