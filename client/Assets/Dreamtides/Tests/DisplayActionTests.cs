using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.Schema;
using System.Collections.Generic;
using System;
using System.Linq;
using UnityEngine;

namespace Dreamtides.Tests
{
  public class DisplayActionTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBrowseVoid()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New().AddCardsToVoid(DisplayPlayer.User, 4).Build());
      AssertNotEmpty(Registry.Layout.UserVoid);
      yield return TestClickInputProvider.ClickOn(Registry,
          Registry.Layout.UserVoid.GetComponentInChildren<CardBrowserButton>());
      AssertEmpty(Registry.Layout.UserVoid);
      AssertNotEmpty(Registry.Layout.Browser);
      AssertActive(Registry.Layout.Browser._closeButton);
      yield return EndTest();
    }
  }
}
