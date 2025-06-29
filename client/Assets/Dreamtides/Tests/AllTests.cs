using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;
using UnityEngine;
using System.Linq;
using Dreamtides.TestUtils;

namespace Dreamtides.Tests
{
  public class AllTests
  {
    public static readonly GameViewResolution[] Resolutions = new GameViewResolution[]
    {
      GameViewResolution.Resolution16x9,
      GameViewResolution.Resolution16x10,
      GameViewResolution.Resolution21x9,
      GameViewResolution.Resolution4x3,
      GameViewResolution.Resolution5x4,
      GameViewResolution.Resolution32x9,
      GameViewResolution.ResolutionIPhone12,
      GameViewResolution.ResolutionIPhoneSE,
      GameViewResolution.ResolutionIPadPro12,
      GameViewResolution.ResolutionIPodTouch6,
      GameViewResolution.ResolutionSamsungNote20,
      GameViewResolution.ResolutionSamsungZFold2,
      GameViewResolution.ResolutionPixel5,
    };

    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBasicLayout([ValueSource("Resolutions")] GameViewResolution resolution)
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(resolution, "basic", (r) =>
      {
        registry = r;
      });

      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.UserDeck), "User deck is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.EnemyDeck), "Enemy deck is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.UserVoid), "User void is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.EnemyVoid), "Enemy void is not on screen");

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        ComponentAssertions.AssertSpriteIsOnScreen(registry, card._costBackground, $"Energy Cost of {card.Id}");
      }

      yield return TestUtil.TearDownScenario(registry);
    }

    [UnityTest]
    public IEnumerator TestOpenUserVoidBrowser()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "basic", (r) =>
      {
        registry = r;
      });

      ComponentAssertions.AssertNotEmpty(registry.Layout.UserVoid);
      yield return TestClickInputProvider.ClickOn(registry,
          registry.Layout.UserVoid.GetComponentInChildren<CardBrowserButton>());
      ComponentAssertions.AssertEmpty(registry.Layout.UserVoid);
      ComponentAssertions.AssertNotEmpty(registry.Layout.Browser);
      ComponentAssertions.AssertActive(registry.Layout.Browser._closeButton);

      yield return TestUtil.TearDownScenario(registry);
    }

    [UnityTest]
    public IEnumerator TestPlayCharacter()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "basic", (r) =>
      {
        registry = r;
      });

      var card = GameObject.Find("[9] Moonlit Voyage").GetComponent<Card>();

      ComponentAssertions.AssertCountIs(registry.Layout.UserBattlefield, 7);
      ComponentAssertions.AssertCountIs(registry.Layout.UserHand, 5);

      yield return TestDragInputProvider.DragTo(
        registry,
        card,
        registry.Layout.DefaultStack);
      yield return new WaitUntil(() => registry.Layout.UserBattlefield.Objects.Count == 8);
      yield return registry.TestHelperService.WaitForIdle(5.0f);

      ComponentAssertions.AssertCountIs(registry.Layout.UserHand, 4);
      ComponentAssertions.AssertSpriteIsOnScreen(registry,
          card._battlefieldCardImage, $"Battlefield card image should be visible");
      ComponentAssertions.AssertActive(card._battlefieldCardFront, "Battlefield card front should be active");
      ComponentAssertions.AssertActive(card._battlefieldCardImage, "Battlefield card image should be active");
      Assert.That(card._cardImage.isVisible, Is.False, $"Card image should not be visible");

      yield return TestUtil.TearDownScenario(registry);
    }

    [UnityTest]
    public IEnumerator TestPlayCardWithTarget()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "play_card_with_targets", (r) =>
      {
        registry = r;
      });

      var card = GameObject.Find("[6] Beacon of Tomorrow").GetComponent<Card>();

      ComponentAssertions.AssertCountIs(registry.Layout.UserBattlefield, 7);
      ComponentAssertions.AssertCountIs(registry.Layout.UserHand, 5);
      ComponentAssertions.AssertCountIs(registry.Layout.UserVoid, 10);
      foreach (var enemy in registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.EqualTo(Color.white),
            "Enemy card outline should be white before targeting");
      }

      yield return TestDragInputProvider.DragTo(
        registry,
        card,
        registry.Layout.TargetingEnemyStack);
      yield return TestUtil.WaitForCount(registry, registry.Layout.TargetingEnemyStack, 1);

      ComponentAssertions.AssertCountIs(registry.Layout.UserHand, 4);
      ComponentAssertions.AssertTextIsInInterface(registry,
          "Choose an enemy character",
          "Target prompt message not found");
      foreach (var enemy in registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.Not.EqualTo(Color.white),
            "Enemy card outline should not be white during targeting");
      }

      var target = registry.Layout.EnemyBattlefield.Objects.Last().GetComponent<Card>();
      yield return TestClickInputProvider.ClickOn(registry, target);
      yield return TestUtil.WaitForCount(registry, registry.Layout.UserVoid, 11);

      foreach (var enemy in registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.EqualTo(Color.white),
            "Enemy card outline should be white after selecting");
      }
      ComponentAssertions.AssertLayoutContains(registry.Layout.UserVoid, card, "Card should be in user void");
      ComponentAssertions.AssertLayoutContains(registry.Layout.EnemyVoid, target, "Target should be in enemy void");

      yield return TestUtil.TearDownScenario(registry);
    }

    [UnityTest]
    public IEnumerator TestUserJudgmentPhase()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "user_judgment_phase", (r) =>
      {
        registry = r;
      });

      yield return TestClickInputProvider.ClickOn(registry, registry.Layout.PrimaryActionButton, 30.0f);
      Assert.That(registry.TestHelperService.DidObjectMove(registry.Layout.UserStatusDisplay.TotalSpark),
          Is.True,
          "User spark should have moved");
      Assert.That(registry.TestHelperService.DidObjectMove(registry.Layout.EnemyStatusDisplay.TotalSpark),
          Is.True,
          "Enemy spark should have moved");

      yield return TestUtil.TearDownScenario(registry);
    }

    [UnityTest]
    public IEnumerator TestRespondToEnemyCard()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "respond_to_enemy_card", (r) =>
      {
        registry = r;
      });

      yield return TestClickInputProvider.ClickOn(registry, registry.Layout.PrimaryActionButton);
      yield return TestUtil.WaitForCount(registry, registry.Layout.TargetingUserStack, 1);

      var card = GameObject.Find("[6] Beacon of Tomorrow").GetComponent<Card>();
      foreach (var enemy in registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.EqualTo(Color.white),
            "Enemy card outline should be white before targeting");
      }

      yield return TestDragInputProvider.DragTo(
        registry,
        card,
        registry.Layout.TargetingBothStack);
      yield return TestUtil.WaitForCount(registry, registry.Layout.TargetingBothStack, 2);
      foreach (var enemy in registry.Layout.EnemyBattlefield.Objects)
      {
        var enemyCard = ComponentUtils.Get<Card>(enemy);
        Assert.That(enemyCard._battlefieldOutline.color, Is.Not.EqualTo(Color.white),
            "Enemy card outline should not be white during targeting");
      }

      yield return TestUtil.TearDownScenario(registry);
    }


    static BoxCollider GetBoxCollider(Component component)
    {
      return Errors.CheckNotNull(component.GetComponentInChildren<BoxCollider>(),
          $"No BoxCollider found on {component.gameObject}");
    }
  }
}
