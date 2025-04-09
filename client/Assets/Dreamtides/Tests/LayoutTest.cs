using System.Collections;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;

namespace Dreamtides.Tests
{
  public class AllTests
  {
    [OneTimeSetUp]
    public void OneTimeSetUp()
    {
      Registry.IsTest = true;
    }

    [OneTimeTearDown]
    public void OneTimeTearDown()
    {
      Registry.IsTest = false;
    }

    [UnityTest]
    public IEnumerator MasterTest()
    {
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      Screen.SetResolution(1170, 2532, fullscreen: false);
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