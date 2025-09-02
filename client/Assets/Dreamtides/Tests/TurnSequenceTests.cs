#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.Schema;

namespace Dreamtides.Tests
{
  public class TurnSequenceTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestEndTurnOpponentScoresPoints()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .AddCardsToBattlefield(DisplayPlayer.Enemy, 2, TestCards.TestVanillaCharacter)
        .Build()
      );

      AssertPrimaryButtonContainsText("End Turn");
      Assert.That(Registry.Layout.EnemyStatusDisplay.TotalSpark._text.text.Contains("10"),
          Is.True,
          "Enemy spark should be 10");
      yield return TestClickInputProvider.ClickOn(Registry, Registry.Layout.PrimaryActionButton);
      Assert.That(Registry.TestHelperService.DidObjectMove(Registry.Layout.UserStatusDisplay.TotalSpark),
          Is.True,
          "User spark should have moved");
      Assert.That(Registry.TestHelperService.DidObjectMove(Registry.Layout.EnemyStatusDisplay.TotalSpark),
          Is.True,
          "Enemy spark should have moved");
      Assert.That(Registry.Layout.EnemyStatusDisplay.Score._text.text.Contains("10"),
          Is.True,
          "Enemy score should be 10");

      yield return EndTest();
    }
  }
}