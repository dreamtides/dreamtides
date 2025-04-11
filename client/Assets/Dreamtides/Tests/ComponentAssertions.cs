#nullable enable

using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Utils;
using NUnit.Framework;
using UnityEngine;

namespace Dreamtides.Tests
{
  public static class ComponentAssertions
  {
    public static void AssertEmpty(ObjectLayout objectLayout)
    {
      Assert.That(objectLayout.Objects.Count, Is.EqualTo(0));
    }

    public static void AssertNotEmpty(ObjectLayout objectLayout)
    {
      Assert.That(objectLayout.Objects.Count, Is.GreaterThan(0));
    }

    public static void AssertCountIs(ObjectLayout objectLayout, int count)
    {
      Assert.That(objectLayout.Objects.Count, Is.EqualTo(count));
    }

    public static void AssertActive(Component component, string? message = null)
    {
      Assert.That(component.gameObject.activeSelf, Is.True, $"{message}: Component is not active");
      Assert.That(component.gameObject.activeInHierarchy, Is.True, $"{message}: Component is not active in hierarchy");
      if (component is MonoBehaviour monoBehaviour)
      {
        Assert.That(monoBehaviour.enabled, Is.True, $"{message}: Component is not enabled");
      }
    }

    public static void AssertBoxColliderIsOnScreen(Registry registry, BoxCollider collider, string? message = null)
    {
      var bounds = collider.bounds;
      var corners = new Vector3[8];

      corners[0] = new Vector3(bounds.min.x, bounds.min.y, bounds.min.z);
      corners[1] = new Vector3(bounds.min.x, bounds.min.y, bounds.max.z);
      corners[2] = new Vector3(bounds.min.x, bounds.max.y, bounds.min.z);
      corners[3] = new Vector3(bounds.min.x, bounds.max.y, bounds.max.z);
      corners[4] = new Vector3(bounds.max.x, bounds.min.y, bounds.min.z);
      corners[5] = new Vector3(bounds.max.x, bounds.min.y, bounds.max.z);
      corners[6] = new Vector3(bounds.max.x, bounds.max.y, bounds.min.z);
      corners[7] = new Vector3(bounds.max.x, bounds.max.y, bounds.max.z);

      foreach (var corner in corners)
      {
        var viewportPos = registry.Layout.MainCamera.WorldToViewportPoint(corner);

        var errorMessage = message ?? $"BoxCollider corner at {corner} is outside viewport: {viewportPos}";

        Assert.That(viewportPos.z > 0, errorMessage);
        Assert.That(viewportPos.x >= 0 && viewportPos.x <= 1, errorMessage);
        Assert.That(viewportPos.y >= 0 && viewportPos.y <= 1, errorMessage);
      }
    }

    public static void AssertSpriteIsOnScreen(Registry registry, SpriteRenderer sprite, string message)
    {
      // Assert.That(sprite.isVisible, $"{message}: Sprite is not visible");
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
        Assert.That(viewportPos.x >= -0.01f && viewportPos.x <= 1.01f &&
                    viewportPos.y >= -0.01f && viewportPos.y <= 1.01f &&
                    viewportPos.z >= -0.01f,
                    $"{message}: Corner at world position {corner} is outside viewport: {viewportPos}");
      }
    }
  }
}