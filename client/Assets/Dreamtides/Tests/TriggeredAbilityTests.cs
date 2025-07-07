#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.UnityInternal;

namespace Dreamtides.Tests
{
  public class TriggeredAbilityTests : IntegrationTest
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
    public IEnumerator TestTriggeredAbility()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .Build()
      );
      yield return EndTest();
    }
  }
}