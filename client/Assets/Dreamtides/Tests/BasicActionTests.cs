#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Tests
{
  public class BasicActionTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestPlayCharacter()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .AddCardsToHand(DisplayPlayer.User, 4)
        .AddCardsToBattlefield(DisplayPlayer.User, 7)
        .Build()
      );
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.MinstrelOfFallingLight)
        .Build()
      );

      var card = Registry.LayoutService.GetCard(CurrentCardId);
      AssertCountIs(Registry.Layout.UserBattlefield, 7);
      AssertCountIs(Registry.Layout.UserHand, 5);

      yield return TestDragInputProvider.DragTo(
        Registry,
        card,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.UserBattlefield, 8);

      AssertCountIs(Registry.Layout.UserHand, 4);
      AssertSpriteIsOnScreen(card._battlefieldCardImage, $"Battlefield card image should be visible");
      AssertActive(card._battlefieldCardFront, "Battlefield card front should be active");
      AssertActive(card._battlefieldCardImage, "Battlefield card image should be active");
      Assert.That(card._cardImage.isVisible, Is.False, $"Card image should not be visible");

      yield return EndTest();
    }
  }
}