#nullable enable

using System.Collections;
using Dreamtides.Layout;
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
    public IEnumerator TwoCardsAreHorizontallySpaced()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(card1.transform.position.x, Is.LessThan(card2.transform.position.x));
      Assert.That(card1.transform.position.x, Is.LessThan(0f));
      Assert.That(card2.transform.position.x, Is.GreaterThan(0f));
    }

    [UnityTest]
    public IEnumerator TwoCardsHaveCorrectScale()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScaleLandscape;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card1.transform.localScale
      );
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card2.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator TwoCardsHaveCorrectRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(layout.transform.eulerAngles, card1.transform.eulerAngles);
      AssertVector3Equal(layout.transform.eulerAngles, card2.transform.eulerAngles);
    }

    // [UnityTest]
    public IEnumerator TwoCardsInPortraitAreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    StartBattleObjectLayout GetStartBattleLayout()
    {
      var layout = Registry.DreamscapeLayout.StartBattleLayout;
      layout.GameContext = GameContext.Interface;
      return layout;
    }
  }
}
