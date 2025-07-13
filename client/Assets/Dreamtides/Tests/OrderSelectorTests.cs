#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.Schema;
using Dreamtides.Components;
using Dreamtides.Utils;

namespace Dreamtides.Tests
{
  public class OrderSelectorTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestActivatedAbility()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .SetEnergy(DisplayPlayer.User, 50)
        .RemovePlayerHands()
        .Build()
      );

      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestForeseeTwo)
        .Build()
      );
      var foreseeTwo = Registry.LayoutService.GetCard(CurrentCardId);
      yield return TestDragInputProvider.DragTo(
        Registry,
        foreseeTwo,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.CardOrderSelector, 2);

      var drag = Registry.Layout.CardOrderSelector.Objects[0];
      yield return TestDragInputProvider.DragTo(Registry, drag, Registry.Layout.CardOrderSelectorVoid);
      AssertCountIs(Registry.Layout.CardOrderSelectorVoid, 1,
          "Card order selector void should now have 1 card");
      AssertCountIs(Registry.Layout.CardOrderSelector, 1,
          "Card order selector should now have 1 card");

      yield return EndTest();
    }

    [UnityTest]
    public IEnumerator TestForeseeOneDrawACardKeepsCardOnTop()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .SetEnergy(DisplayPlayer.User, 50)
        .RemovePlayerHands()
        .Build()
      );

      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestForeseeOneDrawACard)
        .Build()
      );
      var foreseeCard = Registry.LayoutService.GetCard(CurrentCardId);

      var initialHandCount = Registry.Layout.UserHand.Objects.Count;

      yield return TestDragInputProvider.DragTo(
        Registry,
        foreseeCard,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.CardOrderSelector, 1);

      var seenCard = Registry.Layout.CardOrderSelector.Objects[0];
      var seenCardComponent = ComponentUtils.Get<Card>(seenCard);
      var seenCardId = seenCardComponent.Id;

      yield return TestDragInputProvider.DragTo(Registry, seenCard, Registry.Layout.CardOrderSelector);

      yield return WaitForCount(Registry.Layout.CardOrderSelector, 0);
      yield return WaitForCount(Registry.Layout.UserHand, initialHandCount + 1);

      var drawnCard = Registry.Layout.UserHand.Objects[Registry.Layout.UserHand.Objects.Count - 1];
      var drawnCardComponent = ComponentUtils.Get<Card>(drawnCard);
      var drawnCardId = drawnCardComponent.Id;

      Assert.That(drawnCardId, Is.EqualTo(seenCardId),
          "The drawn card should be the same as the card that was seen and put back on top");

      yield return EndTest();
    }

    [UnityTest]
    public IEnumerator TestForeseeOneDrawACardDiscardsToVoid()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .SetEnergy(DisplayPlayer.User, 50)
        .RemovePlayerHands()
        .Build()
      );

      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestForeseeOneDrawACard)
        .Build()
      );
      var foreseeCard = Registry.LayoutService.GetCard(CurrentCardId);

      var initialHandCount = Registry.Layout.UserHand.Objects.Count;
      var initialVoidCount = Registry.Layout.UserVoid.Objects.Count;

      yield return TestDragInputProvider.DragTo(
        Registry,
        foreseeCard,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.CardOrderSelector, 1);

      var seenCard = Registry.Layout.CardOrderSelector.Objects[0];
      var seenCardComponent = ComponentUtils.Get<Card>(seenCard);
      var seenCardId = seenCardComponent.Id;

      yield return TestDragInputProvider.DragTo(Registry, seenCard, Registry.Layout.CardOrderSelectorVoid);

      yield return WaitForCount(Registry.Layout.CardOrderSelector, 0);
      yield return WaitForCount(Registry.Layout.UserHand, initialHandCount + 1);
      yield return WaitForCount(Registry.Layout.UserVoid, initialVoidCount + 1);

      var drawnCard = Registry.Layout.UserHand.Objects[Registry.Layout.UserHand.Objects.Count - 1];
      var drawnCardComponent = ComponentUtils.Get<Card>(drawnCard);
      var drawnCardId = drawnCardComponent.Id;

      var discardedCard = Registry.Layout.UserVoid.Objects[Registry.Layout.UserVoid.Objects.Count - 1];
      var discardedCardComponent = ComponentUtils.Get<Card>(discardedCard);
      var discardedCardId = discardedCardComponent.Id;

      Assert.That(discardedCardId, Is.EqualTo(seenCardId),
          "The discarded card should be the same as the card that was seen and put into void");
      Assert.That(drawnCardId, Is.Not.EqualTo(seenCardId),
          "The drawn card should be different from the card that was discarded");

      yield return EndTest();
    }
  }
}