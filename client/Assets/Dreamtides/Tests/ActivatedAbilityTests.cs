#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.Schema;

namespace Dreamtides.Tests
{
  public class ActivatedAbilityTests : IntegrationTest
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
        .AddCardToHand(DisplayPlayer.User, CardName.TestActivatedAbilityDrawCard)
        .Build()
      );
      var activatedAbilityCharacter = Registry.LayoutService.GetCard(CurrentCardId);
      yield return TestDragInputProvider.DragTo(
        Registry,
        activatedAbilityCharacter,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.UserBattlefield, 1);
      yield return WaitForCount(Registry.Layout.UserHand, 1);
      Assert.That(Registry.Layout.UserStatusDisplay.Energy._text.text.Contains("50"),
          Is.True,
          "User energy should be 50");
      var abilityCard = Registry.Layout.UserHand.Objects[0];
      yield return TestDragInputProvider.DragTo(
        Registry,
        abilityCard,
        Registry.Layout.DefaultStack);

      yield return WaitForCount(Registry.Layout.UserHand, 1);
      var newCardInHand = Registry.Layout.UserHand.Objects[0];
      Assert.That(newCardInHand, Is.Not.EqualTo(abilityCard), "New card should be in hand");
      Assert.That(abilityCard == null, "Ability card should be destroyed");
      Assert.That(Registry.Layout.UserStatusDisplay.Energy._text.text.Contains("49"),
          Is.True,
          "User energy should be 49");

      yield return EndTest();
    }
  }
}