#nullable enable

using System;
using System.Collections;
using Dreamtides.Abu;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Abu
{
  [TestFixture]
  public class SettledProviderTests : DreamtidesUnitTest
  {
    /// <summary>
    /// When no action has been dispatched, the provider should report settled
    /// immediately (FakeActionService.IsProcessingCommands is always false).
    /// </summary>
    [UnityTest]
    public IEnumerator IsSettled_WhenNoActionDispatched_ReturnsTrue()
    {
      yield return Initialize();
      var provider = new DreamtidesSettledProvider(
        Registry.ActionService,
        animationsComplete: () => true
      );
      Assert.IsTrue(provider.IsSettled());
    }

    /// <summary>
    /// After dispatching an action, the provider should wait the configured
    /// number of settle frames before reporting settled, even when conditions
    /// (IsProcessingCommands=false, no tweens) are met immediately.
    /// </summary>
    [UnityTest]
    public IEnumerator SettleFrameCount_WaitsConfiguredFramesBeforeSettled()
    {
      yield return Initialize();
      var settleFrames = 3;
      var provider = new DreamtidesSettledProvider(
        Registry.ActionService,
        settleFrames: settleFrames,
        animationsComplete: () => true
      );

      provider.NotifyActionDispatched();

      // Each call to IsSettled() represents one frame of polling.
      // With FakeActionService (IsProcessingCommands=false) and animations
      // complete, conditions are met immediately but we need 3 settled frames.
      for (var i = 0; i < settleFrames - 1; i++)
      {
        Assert.IsFalse(
          provider.IsSettled(),
          $"Should not be settled on frame {i + 1} of {settleFrames}"
        );
        yield return null;
      }

      // On the Nth call, it should report settled
      Assert.IsTrue(provider.IsSettled(), "Should be settled after required frame count");
    }

    /// <summary>
    /// When conditions are never met (IsProcessingCommands stays true), the
    /// max timeout should cause the provider to report settled anyway.
    /// Uses a mock ActionService that always reports processing.
    /// </summary>
    [UnityTest]
    public IEnumerator MaxTimeout_ReportsSettledAfterTimeout()
    {
      yield return Initialize();
      var alwaysProcessing = CreateGameObject().AddComponent<AlwaysProcessingActionService>();

      // Use a very short timeout so the test completes quickly
      var provider = new DreamtidesSettledProvider(
        alwaysProcessing,
        settleFrames: 3,
        maxTimeoutSeconds: 0.1f,
        animationsComplete: () => true
      );

      provider.NotifyActionDispatched();

      // Should not be settled immediately (IsProcessingCommands blocks it)
      Assert.IsFalse(provider.IsSettled());

      // Wait long enough for the timeout
      var startTime = UnityEngine.Time.realtimeSinceStartup;
      var settled = false;
      while (UnityEngine.Time.realtimeSinceStartup - startTime < 1f)
      {
        if (provider.IsSettled())
        {
          settled = true;
          break;
        }
        yield return null;
      }

      Assert.IsTrue(settled, "Provider should have reported settled due to timeout");
    }

    /// <summary>
    /// If a new action is dispatched mid-way through the settle frame count,
    /// the counter should reset and require the full settle frame count again.
    /// </summary>
    [UnityTest]
    public IEnumerator SettleFrameCount_ResetsOnNewActionDispatch()
    {
      yield return Initialize();
      var settleFrames = 3;
      var provider = new DreamtidesSettledProvider(
        Registry.ActionService,
        settleFrames: settleFrames,
        animationsComplete: () => true
      );

      provider.NotifyActionDispatched();

      // Accumulate 2 settled frames
      Assert.IsFalse(provider.IsSettled());
      yield return null;
      Assert.IsFalse(provider.IsSettled());
      yield return null;

      // Dispatch a new action to reset
      provider.NotifyActionDispatched();

      // Now we need 3 more settled frames again
      Assert.IsFalse(provider.IsSettled());
      yield return null;
      Assert.IsFalse(provider.IsSettled());
      yield return null;
      Assert.IsTrue(provider.IsSettled());
    }

    /// <summary>
    /// When LastResponseIncremental is true, the provider should not report
    /// settled even if commands are done and animations are complete.
    /// </summary>
    [UnityTest]
    public IEnumerator IncrementalResponse_PreventsSettledUntilFinal()
    {
      yield return Initialize();
      var incrementalService = CreateGameObject().AddComponent<IncrementalResponseActionService>();

      var provider = new DreamtidesSettledProvider(
        incrementalService,
        settleFrames: 1,
        animationsComplete: () => true
      );

      provider.NotifyActionDispatched();

      // Should not settle while LastResponseIncremental is true
      Assert.IsFalse(provider.IsSettled());
      yield return null;
      Assert.IsFalse(provider.IsSettled());
      yield return null;

      // Simulate receiving a Final response
      incrementalService.SimulateFinalResponse();

      // Now should settle after 1 frame
      Assert.IsTrue(provider.IsSettled());
    }

    /// <summary>
    /// When animations are not complete, conditions are not met and the
    /// settled frame count should not advance.
    /// </summary>
    [UnityTest]
    public IEnumerator AnimationsBlocking_PreventsSettledUntilComplete()
    {
      yield return Initialize();
      var animationsPlaying = true;
      var provider = new DreamtidesSettledProvider(
        Registry.ActionService,
        settleFrames: 1,
        animationsComplete: () => !animationsPlaying
      );

      provider.NotifyActionDispatched();

      // Should not settle while animations play
      Assert.IsFalse(provider.IsSettled());
      yield return null;
      Assert.IsFalse(provider.IsSettled());
      yield return null;

      // Stop animations
      animationsPlaying = false;

      // Now should settle after 1 frame
      Assert.IsTrue(provider.IsSettled());
    }
  }

  /// <summary>
  /// Test helper: an ActionService that always reports IsProcessingCommands = true.
  /// Used to verify the max timeout behavior in settled provider tests.
  /// </summary>
  sealed class AlwaysProcessingActionService : ActionService
  {
    public override bool Connected { get; protected set; }
    public override float LastActionTime => 0f;
    public override bool IsProcessingCommands => true;
    public override bool LastResponseIncremental { get; protected set; }
    public override Guid? LastResponseReceived { get; protected set; }

    public override void PerformAction(GameAction? action, Guid? requestIdentifier = null) { }

    public override void Log(ClientLogRequest request) { }

    public override void TriggerReconnect() { }
  }

  /// <summary>
  /// Test helper: an ActionService that reports LastResponseIncremental = true.
  /// Used to verify that settled detection waits for a Final poll response.
  /// </summary>
  sealed class IncrementalResponseActionService : ActionService
  {
    public override bool Connected { get; protected set; }
    public override float LastActionTime => 0f;
    public override bool IsProcessingCommands => false;
    public override bool LastResponseIncremental { get; protected set; } = true;
    public override Guid? LastResponseReceived { get; protected set; }

    public void SimulateFinalResponse()
    {
      LastResponseIncremental = false;
    }

    public override void PerformAction(GameAction? action, Guid? requestIdentifier = null) { }

    public override void Log(ClientLogRequest request) { }

    public override void TriggerReconnect() { }
  }
}
