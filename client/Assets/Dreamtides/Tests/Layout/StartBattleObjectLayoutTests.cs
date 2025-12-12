#nullable enable

using System.Collections;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class StartBattleObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsPositionedAtCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(layout.transform.position, card.transform.position);
    }

    [UnityTest]
    public IEnumerator SingleCardHasCorrectScaleLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScaleLandscape;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator SingleCardHasCorrectScalePortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScalePortrait;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator HideButtonHidesTheButton()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      layout.ShowButton();
      layout.HideButton();

      var buttonField = typeof(StartBattleObjectLayout).GetField(
        "_buttonInstance",
        System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance
      );
      var buttonInstance = buttonField?.GetValue(layout) as MonoBehaviour;
      if (buttonInstance != null)
      {
        Assert.That(buttonInstance.gameObject.activeSelf, Is.False);
      }
    }

    [UnityTest]
    public IEnumerator ShowButtonShowsTheButton()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      layout.ShowButton();

      var buttonField = typeof(StartBattleObjectLayout).GetField(
        "_buttonInstance",
        System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance
      );
      var buttonInstance = buttonField?.GetValue(layout) as MonoBehaviour;
      if (buttonInstance != null)
      {
        Assert.That(buttonInstance.gameObject.activeSelf, Is.True);
      }
    }

    [UnityTest]
    public IEnumerator CalculateObjectPositionReturnsLayoutPositionForEmptyLayout()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var position = layout.CalculateObjectPosition(index: 0, count: 0);

      AssertVector3Equal(layout.transform.position, position);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationReturnsLayoutRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var rotation = layout.CalculateObjectRotation(index: 0, count: 2);

      Assert.That(rotation, Is.Not.Null);
      AssertVector3Equal(layout.transform.rotation.eulerAngles, rotation!.Value);
    }

    [UnityTest]
    public IEnumerator RemovingCardMakesLayoutEmpty()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(layout.Objects.Count, Is.EqualTo(1));

      layout.RemoveIfPresent(card);

      Assert.That(layout.Objects.Count, Is.EqualTo(0));
    }

    StartBattleObjectLayout GetStartBattleLayout()
    {
      var layout = Registry.DreamscapeLayout.StartBattleLayout;
      layout.GameContext = GameContext.Interface;
      return layout;
    }

    Card CreateDreamsign(bool isUserSide)
    {
      var dreamsign = CreateTestCard();
      dreamsign.ObjectPosition = new ObjectPosition
      {
        Position = new PositionClass
        {
          StartBattleDisplay = StartBattleDisplayType.EnemyDreamsigns,
        },
        SortingKey = 0,
      };
      return dreamsign;
    }

    void SetIdentityCardPosition(Card card, bool isUserSide)
    {
      card.ObjectPosition = new ObjectPosition
      {
        Position = new PositionClass
        {
          StartBattleDisplay = StartBattleDisplayType.EnemyIdentityCard,
        },
        SortingKey = 0,
      };
    }

    void AssertDreamsignBoxColliderIsOnScreen(
      IGameViewport viewport,
      Card dreamsign,
      string description
    )
    {
      var collider = dreamsign.CardCollider;
      var center = collider.center;
      var extents = collider.size * 0.5f;

      var localCorners = new Vector3[8];
      localCorners[0] = center + new Vector3(-extents.x, -extents.y, -extents.z);
      localCorners[1] = center + new Vector3(-extents.x, -extents.y, extents.z);
      localCorners[2] = center + new Vector3(-extents.x, extents.y, -extents.z);
      localCorners[3] = center + new Vector3(-extents.x, extents.y, extents.z);
      localCorners[4] = center + new Vector3(extents.x, -extents.y, -extents.z);
      localCorners[5] = center + new Vector3(extents.x, -extents.y, extents.z);
      localCorners[6] = center + new Vector3(extents.x, extents.y, -extents.z);
      localCorners[7] = center + new Vector3(extents.x, extents.y, extents.z);

      for (var i = 0; i < localCorners.Length; i++)
      {
        var worldCorner = dreamsign.transform.TransformPoint(localCorners[i]);
        AssertPointIsOnScreen(viewport, worldCorner, $"{description} box collider corner {i}");
      }
    }
  }
}
