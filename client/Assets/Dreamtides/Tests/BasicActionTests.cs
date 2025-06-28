#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.Schema;
using Dreamtides.Utils;
using Dreamtides.Components;
using UnityEngine;
using System.Linq;

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

    [UnityTest]
    public IEnumerator TestSelectTargetDissolveEnemy()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .AddCardsToBattlefield(DisplayPlayer.Enemy, 8)
        .Build()
      );
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.Immolate)
        .Build()
      );

      var card = Registry.LayoutService.GetCard(CurrentCardId);
      AssertCountIs(Registry.Layout.UserHand, 1);
      AssertCountIs(Registry.Layout.UserVoid, 0);
      foreach (var enemy in Registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.EqualTo(Color.white),
            "Enemy card outline should be white before targeting");
      }

      yield return TestDragInputProvider.DragTo(
        Registry,
        card,
        Registry.Layout.TargetingEnemyStack);
      yield return WaitForCount(Registry.Layout.TargetingEnemyStack, 1);

      AssertCountIs(Registry.Layout.UserHand, 0);
      AssertTextIsInInterface("Select an enemy character", "Target prompt message not found");
      foreach (var enemy in Registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.Not.EqualTo(Color.white),
            "Enemy card outline should not be white during targeting");
      }

      var target = Registry.Layout.EnemyBattlefield.Objects.Last().GetComponent<Card>();
      yield return TestClickInputProvider.ClickOn(Registry, target);
      yield return WaitForCount(Registry.Layout.UserVoid, 1);

      foreach (var enemy in Registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.EqualTo(Color.white),
            "Enemy card outline should be white after selecting");
      }
      AssertLayoutContains(Registry.Layout.UserVoid, card, "Card should be in user void");
      AssertLayoutContains(Registry.Layout.EnemyVoid, target, "Target should be in enemy void");

      yield return EndTest();
    }
  }
}