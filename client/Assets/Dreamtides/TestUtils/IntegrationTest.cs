using System.Collections;
using NUnit.Framework;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.UnityInternal;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.SceneManagement;
using Dreamtides.Layout;
using UnityEngine.UIElements;
using Dreamtides.Schema;
using System.Runtime.CompilerServices;

#nullable enable

namespace Dreamtides.TestUtils
{
  public abstract class IntegrationTest
  {
    public const float TimeoutSeconds = 5.0f;

    Registry? _registry;
    protected Registry Registry => Errors.CheckNotNull(_registry);

    string? _cardId;
    protected string CurrentCardId => Errors.CheckNotNull(_cardId, "CardId not set");

    protected IEnumerator Connect(
      [CallerMemberName] string? testName = null,
      GameViewResolution resolution = GameViewResolution.Resolution16x9)
    {
      Registry.TestConfiguration = new TestConfiguration(Guid.NewGuid());
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      yield return new WaitForSeconds(0.1f);
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Assert.AreEqual(GameViewUtils.GetResolution(resolution), new Vector2(Screen.width, Screen.height),
          $"Resolution {resolution} not set");
      yield return new WaitUntil(() => registry.ActionService.Connected,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () => throw new TimeoutException("Timeout waiting for registry.ActionService.Connected"));
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
      yield return new WaitUntil(() => Registry.ActionService.LastResponseReceived == requestId,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () => throw new TimeoutException("Timeout waiting for ActionService.LastResponseReceived == requestId"));
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
    }

    /// <summary>
    /// Performs an action as the opponent player.
    /// </summary>
    protected IEnumerator PerformOpponentAction(BattleAction action)
    {
      return PerformAction(new GameAction
      {
        GameActionClass = new GameActionClass
        {
          DebugAction = new DebugAction
          {
            DebugActionClass = new DebugActionClass
            {
              PerformOpponentAction = action,
            },
          },
        },
      });
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
      var existingCardIds = Registry.LayoutService.GetCardIds().ToHashSet();
      yield return PerformAction(action);
      var newCardIds = Registry.LayoutService.GetCardIds().Except(existingCardIds).ToList();

      if (newCardIds.Count == 0)
      {
        throw new InvalidOperationException("No new card was added after performing the action");
      }

      if (newCardIds.Count > 1)
      {
        throw new InvalidOperationException("Multiple new cards were added after performing the action :" +
            $"{string.Join(", ", newCardIds)}");
      }

      _cardId = newCardIds[0];
    }

    protected IEnumerator EndTest()
    {
      Registry.TestConfiguration = null;
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
      yield return new WaitForSeconds(0.5f);
    }

    protected IEnumerator WaitForIdle() => Registry.TestHelperService.WaitForIdle(TimeoutSeconds);

    protected IEnumerator WaitForCount(ObjectLayout layout, int count)
    {
      yield return new WaitUntil(() => layout.Objects.Count == count,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () => throw new TimeoutException($"Timeout waiting for layout.Objects.Count == {count}"));
      yield return Registry.TestHelperService.WaitForIdle(TimeoutSeconds);
    }

    protected IEnumerator WaitForSceneLoad()
    {
      yield return new WaitUntil(() => SceneManager.GetActiveScene().buildIndex == 0,
        TimeSpan.FromSeconds(TimeoutSeconds),
        () => throw new TimeoutException("Timeout waiting for scene to load (buildIndex == 0)"));
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
      Assert.That(component.gameObject.activeInHierarchy, Is.True, $"{message}. Component is not active in hierarchy");
      if (component is MonoBehaviour monoBehaviour)
      {
        Assert.That(monoBehaviour.enabled, Is.True, $"{message}. Component is not enabled");
      }
    }

    protected void AssertLayoutContains(ObjectLayout objectLayout, Displayable displayable, string? message = null)
    {
      Assert.That(objectLayout.Objects.Any(obj => obj.GetComponent<Displayable>() == displayable),
          Is.True,
          $"{message}. {displayable.name} not found in layout {objectLayout.name}");
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

        var errorMessage = message ?? $"BoxCollider corner at {corner} is outside viewport: {viewportPos}";

        Assert.That(viewportPos.z > 0, errorMessage);
        Assert.That(viewportPos.x >= 0 && viewportPos.x <= 1, errorMessage);
        Assert.That(viewportPos.y >= 0 && viewportPos.y <= 1, errorMessage);
      }
    }

    protected void AssertSpriteIsOnScreen(SpriteRenderer sprite, string message)
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
        var viewportPos = Registry.Layout.MainCamera.WorldToViewportPoint(corner);
        Assert.That(viewportPos.x >= -0.01f && viewportPos.x <= 1.01f &&
                    viewportPos.y >= -0.01f && viewportPos.y <= 1.01f &&
                    viewportPos.z >= -0.01f,
                    $"{message}: Corner at world position {corner} is outside viewport: {viewportPos}");
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
      Assert.That(primaryButton._text.text, Is.EqualTo(text), $"Primary button text is not '{text}'");
    }

    protected BoxCollider GetBoxCollider(Component component)
    {
      return Errors.CheckNotNull(component.GetComponentInChildren<BoxCollider>(),
          $"No BoxCollider found on {component.gameObject}");
    }
  }
}
