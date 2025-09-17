using System;
using System.Collections;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.UnityInternal;
using Dreamtides.Utils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.SceneManagement;
using UnityEngine.UIElements;

#nullable enable

namespace Dreamtides.TestUtils
{
  public abstract class IntegrationTest
  {
    public const float TimeoutSeconds = 30.0f;

    Registry? _registry;
    protected Registry Registry => Errors.CheckNotNull(_registry);

    string? _cardId;
    protected string CurrentCardId => Errors.CheckNotNull(_cardId, "CardId not set");

    protected IEnumerator Connect(
      [CallerMemberName] string? testName = null,
      GameViewResolution resolution = GameViewResolution.Resolution16x9
    )
    {
      Registry.TestConfiguration = new TestConfiguration(Guid.NewGuid());
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      yield return new WaitForSeconds(0.1f);
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Assert.AreEqual(
        GameViewUtils.GetResolution(resolution),
        new Vector2(Screen.width, Screen.height),
        $"Resolution {resolution} not set"
      );
      yield return new WaitUntil(
        () => registry.ActionService.Connected,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () =>
        {
          var screenshotPath = CaptureScreenshot($"Connect_Timeout_{DateTime.Now:yyyyMMdd_HHmmss}");
          Debug.Log($"Screenshot captured at: {screenshotPath}");
          throw new TimeoutException(
            $"Timeout waiting for registry.ActionService.Connected: {screenshotPath}"
          );
        }
      );
      yield return registry.TestHelperService.WaitForIdle(TimeoutSeconds);
      _registry = registry;
      Debug.Log($"{testName} started successfully");
    }

    /// <summary>
    /// Performs an action and waits for the response to be received.
    /// </summary>
    protected IEnumerator PerformAction(GameAction action)
    {
      var requestId = Guid.NewGuid();
      Registry.ActionService.PerformAction(action, requestId);
      yield return new WaitUntil(
        () => Registry.ActionService.LastResponseReceived == requestId,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () =>
        {
          var screenshotPath = CaptureScreenshot(
            $"PerformAction_Timeout_{DateTime.Now:yyyyMMdd_HHmmss}"
          );
          Debug.Log($"Screenshot captured at: {screenshotPath}");
          throw new TimeoutException(
            $"Timeout waiting for ActionService.LastResponseReceived == requestId: {screenshotPath}"
          );
        }
      );
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
    }

    /// <summary>
    /// Performs an action as the opponent player.
    /// </summary>
    protected IEnumerator PerformOpponentAction(BattleAction action)
    {
      return PerformAction(
        new GameAction
        {
          GameActionClass = new GameActionClass
          {
            DebugAction = new DebugAction
            {
              DebugActionClass = new DebugActionClass { PerformOpponentAction = action },
            },
          },
        }
      );
    }

    /// <summary>
    /// Performs an action that adds a new card to the battle and populates its
    /// ID in the "CurrentCardId" field.
    /// </summary>
    /// <remarks>
    /// This operates by storing the current set of CardIds owned by the
    /// LayoutService before performing the action, and then identifying the
    /// newly added item. Throws an exception if no new card is found after
    /// performing the action or if more than one new card is found.
    /// </remarks>
    protected IEnumerator PerformAddCardAction(GameAction action)
    {
      var existingCardIds = Registry.CardService.GetCardIds().ToHashSet();
      yield return PerformAction(action);
      var newCardIds = Registry.CardService.GetCardIds().Except(existingCardIds).ToList();

      if (newCardIds.Count == 0)
      {
        throw new InvalidOperationException("No new card was added after performing the action");
      }

      if (newCardIds.Count > 1)
      {
        throw new InvalidOperationException(
          "Multiple new cards were added after performing the action :"
            + $"{string.Join(", ", newCardIds)}"
        );
      }

      _cardId = newCardIds[0];
    }

    /// <summary>
    /// Ends the test.
    /// </summary>
    protected IEnumerator EndTest()
    {
      Registry.TestConfiguration = null;
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
      yield return new WaitForSeconds(0.5f);
    }

    /// <summary>
    /// Waits for the test to be idle.
    /// </summary>
    protected IEnumerator WaitForIdle() => Registry.TestHelperService.WaitForIdle(TimeoutSeconds);

    /// <summary>
    /// Waits for the number of objects in the layout to be equal to the
    /// specified count.
    /// </summary>
    protected IEnumerator WaitForCount(ObjectLayout layout, int count)
    {
      yield return new WaitUntil(
        () => layout.Objects.Count == count,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () =>
        {
          var screenshotPath = CaptureScreenshot(
            $"WaitForCount_Timeout_{DateTime.Now:yyyyMMdd_HHmmss}"
          );
          Debug.Log($"Screenshot captured at: {screenshotPath}");
          throw new TimeoutException(
            $"Timeout waiting for layout.Objects.Count == {count}: {screenshotPath}"
          );
        }
      );
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
    }

    /// <summary>
    /// Waits for the scene to load.
    /// </summary>
    protected IEnumerator WaitForSceneLoad()
    {
      yield return new WaitUntil(
        () => SceneManager.GetActiveScene().buildIndex == 0,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () =>
        {
          var screenshotPath = CaptureScreenshot(
            $"WaitForSceneLoad_Timeout_{DateTime.Now:yyyyMMdd_HHmmss}"
          );
          Debug.Log($"Screenshot captured at: {screenshotPath}");
          throw new TimeoutException(
            $"Timeout waiting for scene to load (buildIndex == 0): {screenshotPath}"
          );
        }
      );
    }

    protected void AssertEmpty(ObjectLayout objectLayout)
    {
      Assert.That(objectLayout.Objects.Count, Is.EqualTo(0));
    }

    protected void AssertNotEmpty(ObjectLayout objectLayout)
    {
      Assert.That(objectLayout.Objects.Count, Is.GreaterThan(0));
    }

    protected void AssertCountIs(ObjectLayout objectLayout, int count, string? message = null)
    {
      Assert.That(objectLayout.Objects.Count, Is.EqualTo(count), message);
    }

    protected void AssertActive(Component component, string? message = null)
    {
      Assert.That(component.gameObject.activeSelf, Is.True, $"{message}. Component is not active");
      Assert.That(
        component.gameObject.activeInHierarchy,
        Is.True,
        $"{message}. Component is not active in hierarchy"
      );
      if (component is MonoBehaviour monoBehaviour)
      {
        Assert.That(monoBehaviour.enabled, Is.True, $"{message}. Component is not enabled");
      }
    }

    protected void AssertLayoutContains(
      ObjectLayout objectLayout,
      Displayable displayable,
      string? message = null
    )
    {
      Assert.That(
        objectLayout.Objects.Any(obj => obj.GetComponent<Displayable>() == displayable),
        Is.True,
        $"{message}. {displayable.name} not found in layout {objectLayout.name}"
      );
    }

    protected void AssertBoxColliderIsOnScreen(BoxCollider collider, string? message = null)
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
        var viewportPos = Registry.Layout.MainCamera.WorldToViewportPoint(corner);

        var errorMessage =
          message ?? $"BoxCollider corner at {corner} is outside viewport: {viewportPos}";

        Assert.That(viewportPos.z > 0, errorMessage);
        Assert.That(viewportPos.x >= 0 && viewportPos.x <= 1, errorMessage);
        Assert.That(viewportPos.y >= 0 && viewportPos.y <= 1, errorMessage);
      }
    }

    /// <summary>
    /// Asserts that all four corners of a 'renderer' are visible on the screen
    /// </summary>
    protected void AssertIsOnscreen(
      Renderer sprite,
      string message,
      params GameObject[] ignoredObjects
    )
    {
      var bounds = sprite.bounds;
      var corners = new Vector3[4]
      {
        new Vector3(bounds.min.x, bounds.min.y, bounds.center.z),
        new Vector3(bounds.max.x, bounds.min.y, bounds.center.z),
        new Vector3(bounds.max.x, bounds.max.y, bounds.center.z),
        new Vector3(bounds.min.x, bounds.max.y, bounds.center.z),
      };

      var isOnscreen = true;
      var failureReason = "";

      foreach (var corner in corners)
      {
        var viewportPos = Registry.Layout.MainCamera.WorldToViewportPoint(corner);

        if (
          !(
            viewportPos.x >= -0.01f
            && viewportPos.x <= 1.01f
            && viewportPos.y >= -0.01f
            && viewportPos.y <= 1.01f
            && viewportPos.z >= -0.01f
          )
        )
        {
          isOnscreen = false;
          failureReason = $"Corner at world position {corner} is outside viewport: {viewportPos}";
          break;
        }
      }

      if (!isOnscreen)
      {
        var screenshotPath = CaptureScreenshot(
          $"AssertIsOnscreen_Failure_{DateTime.Now:yyyyMMdd_HHmmss}"
        );
        Debug.Log($"Screenshot captured at: {screenshotPath}");
        Assert.Fail($"{message}: {screenshotPath}\n{failureReason}");
      }
    }

    private string CaptureScreenshot(string filename)
    {
      var directory = Path.Combine(Application.persistentDataPath, "Screenshots");

      if (!Directory.Exists(directory))
      {
        Directory.CreateDirectory(directory);
      }

      var fullPath = Path.Combine(directory, $"{filename}.png");
      ScreenCapture.CaptureScreenshot(fullPath);

      return fullPath;
    }

    protected void AssertPrimaryActionButtonIsVisible()
    {
      AssertIsOnscreen(
        Registry.Layout.PrimaryActionButton._background,
        "Primary action button should be visible",
        Registry.Layout.PrimaryActionButton._text.gameObject
      );
      AssertIsTopmost(
        Registry.Layout.PrimaryActionButton._background,
        "Primary action button should be topmost",
        Registry.Layout.PrimaryActionButton._text.gameObject
      );
    }

    /// <summary>
    /// Asserts that a ray fired through the center of the renderer hits no other
    /// higher objects.
    /// </summary>
    protected void AssertIsTopmost(
      Renderer sprite,
      string message,
      params GameObject[] ignoredObjects
    )
    {
      var spriteBounds = sprite.bounds;
      var spriteCenter = spriteBounds.center;
      var colliderParent = sprite.GetComponentInParent<Collider>();
      var ofParent = colliderParent ? $" of {colliderParent.gameObject.name}" : "";

      var screenPoint = Registry.Layout.MainCamera.WorldToScreenPoint(spriteCenter);
      var ray = Registry.Layout.MainCamera.ScreenPointToRay(screenPoint);

      var hits = Physics.RaycastAll(ray);

      var sortingGroupParent = sprite.GetComponentInParent<SortingGroup>();
      var spriteSortingLayer = sortingGroupParent
        ? sortingGroupParent.sortingLayerID
        : sprite.sortingLayerID;
      var spriteSortingOrder = sortingGroupParent
        ? sortingGroupParent.sortingOrder
        : sprite.sortingOrder;
      var spriteSortingLayerValue = SortingLayer.GetLayerValueFromID(spriteSortingLayer);

      foreach (var hit in hits)
      {
        if (hit.collider == colliderParent)
        {
          // Ignore hits from this sprite's own collider.
          continue;
        }

        var sortingGroup = hit.collider.gameObject.GetComponentInChildren<SortingGroup>(true);

        if (sortingGroup != null)
        {
          if (
            sprite.transform.IsChildOf(sortingGroup.transform)
            || sortingGroup == sortingGroupParent
          )
          {
            continue;
          }

          if (
            ignoredObjects.Any(ignored =>
              sortingGroup.transform.IsChildOf(ignored.transform)
              || sortingGroup.gameObject == ignored
            )
          )
          {
            continue;
          }

          var groupSortingLayerValue = SortingLayer.GetLayerValueFromID(
            sortingGroup.sortingLayerID
          );

          if (groupSortingLayerValue > spriteSortingLayerValue)
          {
            var parent =
              sortingGroup.transform.parent != null
                ? $" of {sortingGroup.transform.parent.name}"
                : "";
            Assert.Fail(
              $"{message}: SortingGroup '{sortingGroup.name}'{parent} has a higher sorting layer "
                + $"(layer: {SortingLayer.IDToName(sortingGroup.sortingLayerID)}, value: {groupSortingLayerValue}) "
                + $"than sprite '{sprite.name}'{ofParent} (layer: {SortingLayer.IDToName(spriteSortingLayer)}, value: {spriteSortingLayerValue})"
            );
          }

          if (
            groupSortingLayerValue == spriteSortingLayerValue
            && sortingGroup.sortingOrder > spriteSortingOrder
          )
          {
            var parent =
              sortingGroup.transform.parent != null
                ? $" of {sortingGroup.transform.parent.name}"
                : "";
            Assert.Fail(
              $"{message}: SortingGroup '{sortingGroup.name}'{parent} has the same sorting layer "
                + $"but higher sorting order ({sortingGroup.sortingOrder}) than sprite '{sprite.name}{ofParent}' ({spriteSortingOrder})"
            );
          }
        }
        else
        {
          var renderers = hit.collider.gameObject.GetComponentsInChildren<Renderer>(true);

          foreach (var renderer in renderers)
          {
            if (renderer == sprite)
              continue;

            if (
              ignoredObjects.Any(ignored =>
                renderer.transform.IsChildOf(ignored.transform) || renderer.gameObject == ignored
              )
            )
            {
              continue;
            }

            var rendererSortingLayerValue = SortingLayer.GetLayerValueFromID(
              renderer.sortingLayerID
            );

            if (rendererSortingLayerValue > spriteSortingLayerValue)
            {
              var parent =
                renderer.transform.parent != null ? $" of {renderer.transform.parent.name}" : "";
              Assert.Fail(
                $"{message}: Renderer '{renderer.name}'{parent} has a higher sorting layer "
                  + $"(layer: {SortingLayer.IDToName(renderer.sortingLayerID)}, value: {rendererSortingLayerValue}) "
                  + $"than sprite '{sprite.name}{ofParent}' (layer: {SortingLayer.IDToName(spriteSortingLayer)}, value: {spriteSortingLayerValue})"
              );
            }

            if (
              rendererSortingLayerValue == spriteSortingLayerValue
              && renderer.sortingOrder > spriteSortingOrder
            )
            {
              var parent =
                renderer.transform.parent != null ? $" of {renderer.transform.parent.name}" : "";
              Assert.Fail(
                $"{message}: Renderer '{renderer.name}'{parent} has the same sorting layer "
                  + $"but higher sorting order ({renderer.sortingOrder}) than sprite '{sprite.name}{ofParent}' ({spriteSortingOrder})"
              );
            }
          }
        }
      }
    }

    protected void AssertTextIsInInterface(string text, string message)
    {
      var root = Registry.DocumentService.RootVisualElement;
      var textElements = root.Query<TextElement>().ToList();
      var found = textElements.Any(element => element.text.Contains(text));
      Assert.That(found, Is.True, $"{message}: Text '{text}' not found in any text elements");
    }

    protected void AssertPrimaryButtonContainsText(string text)
    {
      var primaryButton = Registry.Layout.PrimaryActionButton;
      Assert.That(
        primaryButton._text.text,
        Is.EqualTo(text),
        $"Primary button text is not '{text}'"
      );
    }

    protected BoxCollider GetBoxCollider(Component component)
    {
      return Errors.CheckNotNull(
        component.GetComponentInChildren<BoxCollider>(),
        $"No BoxCollider found on {component.gameObject}"
      );
    }
  }
}
