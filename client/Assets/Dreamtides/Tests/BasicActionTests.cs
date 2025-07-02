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
using Dreamtides.UnityInternal;

namespace Dreamtides.Tests
{
  public class BasicActionTests : IntegrationTest
  {
    public static readonly GameViewResolution[] MobileAndDesktop = new GameViewResolution[]
    {
      GameViewResolution.Resolution16x9,
      GameViewResolution.ResolutionIPhone12,
    };

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
        .AddCardToHand(DisplayPlayer.User, CardName.TestVanillaCharacter)
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
      AssertIsOnscreen(card._battlefieldCardImage, $"Battlefield card image should be visible");
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
        .AddCardToHand(DisplayPlayer.User, CardName.TestDissolve)
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

    [UnityTest]
    public IEnumerator TestSelectCardOnStackPrevent([ValueSource("MobileAndDesktop")] GameViewResolution resolution)
    {
      yield return Connect(resolution: resolution);
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .SetEnergy(DisplayPlayer.User, 99)
        .SetEnergy(DisplayPlayer.Enemy, 99)
        .Build()
      );

      yield return PerformAddCardAction(TestBattle.New()
         .AddCardToHand(DisplayPlayer.User, CardName.TestVanillaCharacter)
         .Build()
       );
      var characterCard = Registry.LayoutService.GetCard(CurrentCardId);
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestCounterspell)
        .Build()
      );
      var userPrevent1 = Registry.LayoutService.GetCard(CurrentCardId);
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestCounterspell)
        .Build()
      );
      var userPrevent2 = Registry.LayoutService.GetCard(CurrentCardId);
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.Enemy, CardName.TestDrawOne)
        .Build()
      );
      var enemyCard1 = CurrentCardId;
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.Enemy, CardName.TestDrawOne)
        .Build()
      );
      var enemyCard2 = CurrentCardId;

      yield return TestDragInputProvider.DragTo(
        Registry,
        characterCard,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.DefaultStack, 1);
      AssertPrimaryActionButtonIsVisible();

      yield return PerformOpponentAction(new BattleAction
      {
        BattleActionClass = new BattleActionClass
        {
          PlayCardFromHand = long.Parse(enemyCard1)
        },
      });
      yield return WaitForCount(Registry.Layout.DefaultStack, 2);
      AssertPrimaryActionButtonIsVisible();

      yield return TestDragInputProvider.DragTo(
        Registry,
        userPrevent1,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.DefaultStack, 3);
      AssertPrimaryActionButtonIsVisible();

      yield return PerformOpponentAction(new BattleAction
      {
        BattleActionClass = new BattleActionClass
        {
          PlayCardFromHand = long.Parse(enemyCard2)
        },
      });
      yield return WaitForCount(Registry.Layout.DefaultStack, 4);
      AssertPrimaryActionButtonIsVisible();

      yield return TestDragInputProvider.DragTo(
        Registry,
        userPrevent2,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.DefaultStack, 5);
      foreach (var displayable in Registry.Layout.DefaultStack.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertIsOnscreen(card._costBackground, $"Energy Cost of {card.Id}");
      }

      AssertPrimaryActionButtonIsVisible();
      AssertArrowBetween(userPrevent1, Registry.LayoutService.GetCard(enemyCard1), "userPrevent1 should target enemyCard1");

      yield return TestClickInputProvider.ClickOn(Registry, Registry.LayoutService.GetCard(enemyCard2));
      yield return WaitForCount(Registry.Layout.DefaultStack, 0);

      AssertCountIs(Registry.Layout.UserVoid, 2);
      AssertCountIs(Registry.Layout.EnemyVoid, 2);
      AssertCountIs(Registry.Layout.UserBattlefield, 1);

      yield return EndTest();
    }

    [UnityTest]
    public IEnumerator TestSelectTargetsOnBattlefieldRespond()
    {
      yield return Connect(resolution: GameViewResolution.ResolutionIPhoneSE);
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .SetEnergy(DisplayPlayer.User, 99)
        .SetEnergy(DisplayPlayer.Enemy, 99)
        .AddCardToHand(DisplayPlayer.Enemy, CardName.TestDrawOne)
        .AddCardsToBattlefield(DisplayPlayer.Enemy, 8)
        .Build()
      );
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestDissolve)
        .Build()
      );
      var userDissolve = Registry.LayoutService.GetCard(CurrentCardId);

      yield return TestDragInputProvider.DragTo(
        Registry,
        userDissolve,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.TargetingEnemyStack, 1);
      var target = Registry.Layout.EnemyBattlefield.Objects.Last().GetComponent<Card>();
      yield return TestClickInputProvider.ClickOn(Registry, target);

      AssertArrowBetween(userDissolve, target, "userDissolve should target the selected enemy");

      yield return EndTest();
    }

    private void AssertArrowBetween(Component source, Component target, string message)
    {
      Assert.That(Registry.ArrowService._arrows.Count, Is.EqualTo(1), "One arrow should be visible");
      var arrow = Registry.ArrowService._arrows[0];
      Assert.That(arrow.Source.gameObject,
        Is.EqualTo(source.gameObject),
        $"Arrow should originate from {source.name}");
      Assert.That(arrow.Target.gameObject,
        Is.EqualTo(target.gameObject),
        $"Arrow should target {target.name}. {message}");
    }
  }
}