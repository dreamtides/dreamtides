#nullable enable

using System.Collections;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using NUnit.Framework;
using UnityEngine.TestTools;

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
      yield return PerformAction(
        TestBattle.New().SetEnergy(DisplayPlayer.User, 50).RemovePlayerHands().Build()
      );

      yield return PerformAddCardAction(
        TestBattle.New().AddCardToHand(DisplayPlayer.User, TestCards.TestForeseeTwo).Build()
      );
      var foreseeTwo = Registry.CardService.GetCard(CurrentCardId);
      yield return TestDragInputProvider.DragTo(Registry, foreseeTwo, Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.CardOrderSelector, 2);

      var drag = Registry.Layout.CardOrderSelector.Objects[0];
      yield return TestDragInputProvider.DragTo(
        Registry,
        drag,
        Registry.Layout.CardOrderSelectorVoid
      );
      AssertCountIs(
        Registry.Layout.CardOrderSelectorVoid,
        1,
        "Card order selector void should now have 1 card"
      );
      AssertCountIs(
        Registry.Layout.CardOrderSelector,
        1,
        "Card order selector should now have 1 card"
      );

      yield return EndTest();
    }
  }
}
